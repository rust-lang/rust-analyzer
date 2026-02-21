//! An implementation of `loader::Handle`, based on `walkdir` and `notify`.
//!
//! The file watching bits here are untested and quite probably buggy. For this
//! reason, by default we don't watch files and rely on editor's file watching
//! capabilities.
//!
//! Hopefully, one day a reliable file watching/walking crate appears on
//! crates.io, and we can reduce this to trivial glue code.

use std::{
    fs,
    path::{Component, Path},
    sync::atomic::AtomicUsize,
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, Sender, bounded, select};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use paths::{AbsPath, AbsPathBuf, Utf8PathBuf};
use rayon::iter::{IndexedParallelIterator as _, IntoParallelIterator as _, ParallelIterator};
use rustc_hash::FxHashSet;
use vfs::loader::{self, LoadingProgress};
use walkdir::WalkDir;

/// Default debounce window for file system events in milliseconds.
/// Rapid file changes within this window are coalesced into a single reload.
const DEFAULT_DEBOUNCE_MS: u64 = 100;

#[derive(Debug)]
pub struct NotifyHandle {
    // Relative order of fields below is significant.
    sender: Sender<Message>,
    _thread: stdx::thread::JoinHandle,
}

#[derive(Debug)]
enum Message {
    Config(loader::Config),
    Invalidate(AbsPathBuf),
}

impl loader::Handle for NotifyHandle {
    fn spawn(sender: loader::Sender) -> NotifyHandle {
        let actor = NotifyActor::new(sender);
        // Bounded channel for config/invalidate messages - low volume
        let (sender, receiver) = bounded::<Message>(16);
        let thread = stdx::thread::Builder::new(stdx::thread::ThreadIntent::Worker, "VfsLoader")
            .spawn(move || actor.run(receiver))
            .expect("failed to spawn thread");
        NotifyHandle { sender, _thread: thread }
    }

    fn set_config(&mut self, config: loader::Config) {
        self.sender.send(Message::Config(config)).unwrap();
    }

    fn invalidate(&mut self, path: AbsPathBuf) {
        self.sender.send(Message::Invalidate(path)).unwrap();
    }

    fn load_sync(&mut self, path: &AbsPath) -> Option<Vec<u8>> {
        read(path)
    }
}

type NotifyEvent = notify::Result<notify::Event>;

struct NotifyActor {
    sender: loader::Sender,
    watched_file_entries: FxHashSet<AbsPathBuf>,
    watched_dir_entries: Vec<loader::Directories>,
    watcher: Option<(RecommendedWatcher, Receiver<NotifyEvent>)>,
    debounce: DebounceState,
}

#[derive(Debug)]
enum Event {
    Message(Message),
    NotifyEvent(NotifyEvent),
}

struct DebounceState {
    pending_paths: FxHashSet<AbsPathBuf>,
    first_event_time: Option<Instant>,
    debounce_duration: Duration,
}

impl DebounceState {
    fn new(debounce_ms: u64) -> Self {
        Self {
            pending_paths: FxHashSet::default(),
            first_event_time: None,
            debounce_duration: Duration::from_millis(debounce_ms),
        }
    }

    fn add_path(&mut self, path: AbsPathBuf) {
        if self.first_event_time.is_none() {
            self.first_event_time = Some(Instant::now());
        }
        self.pending_paths.insert(path);
    }

    fn should_flush(&self) -> bool {
        self.first_event_time
            .map(|start| start.elapsed() >= self.debounce_duration)
            .unwrap_or(false)
    }

    fn flush(&mut self) -> FxHashSet<AbsPathBuf> {
        self.first_event_time = None;
        std::mem::take(&mut self.pending_paths)
    }

    fn time_until_flush(&self) -> Option<Duration> {
        self.first_event_time.map(|start| {
            let elapsed = start.elapsed();
            if elapsed >= self.debounce_duration {
                Duration::ZERO
            } else {
                self.debounce_duration - elapsed
            }
        })
    }
}

impl NotifyActor {
    fn new(sender: loader::Sender) -> NotifyActor {
        NotifyActor {
            sender,
            watched_dir_entries: Vec::new(),
            watched_file_entries: FxHashSet::default(),
            watcher: None,
            debounce: DebounceState::new(DEFAULT_DEBOUNCE_MS),
        }
    }

    fn next_event_with_timeout(
        &self,
        receiver: &Receiver<Message>,
        timeout: Duration,
    ) -> Option<Event> {
        let Some((_, watcher_receiver)) = &self.watcher else {
            return receiver.recv_timeout(timeout).ok().map(Event::Message);
        };

        select! {
            recv(receiver) -> it => it.ok().map(Event::Message),
            recv(watcher_receiver) -> it => Some(Event::NotifyEvent(it.unwrap())),
            default(timeout) => None,
        }
    }

    fn next_event(&self, receiver: &Receiver<Message>) -> Option<Event> {
        let Some((_, watcher_receiver)) = &self.watcher else {
            return receiver.recv().ok().map(Event::Message);
        };

        select! {
            recv(receiver) -> it => it.ok().map(Event::Message),
            recv(watcher_receiver) -> it => Some(Event::NotifyEvent(it.unwrap())),
        }
    }

    fn run(mut self, inbox: Receiver<Message>) {
        while let Some(event) = self.next_event(&inbox) {
            tracing::debug!(?event, "vfs-notify event");
            match event {
                Event::Message(msg) => {
                    self.handle_message(msg);
                }
                Event::NotifyEvent(event) => {
                    if let Some(event) = log_notify_error(event)
                        && let EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) =
                            event.kind
                    {
                        for path in event.paths {
                            if let Ok(utf8_path) = Utf8PathBuf::from_path_buf(path)
                                && let Ok(abs_path) = AbsPathBuf::try_from(utf8_path) {
                                self.debounce.add_path(abs_path);
                            }
                        }

                        while !self.debounce.should_flush() {
                            let timeout =
                                self.debounce.time_until_flush().unwrap_or(Duration::ZERO);
                            if timeout.is_zero() {
                                break;
                            }
                            match self.next_event_with_timeout(&inbox, timeout) {
                                Some(Event::Message(msg)) => {
                                    self.handle_message(msg);
                                }
                                Some(Event::NotifyEvent(event)) => {
                                    if let Some(event) = log_notify_error(event)
                                        && let EventKind::Create(_)
                                        | EventKind::Modify(_)
                                        | EventKind::Remove(_) = event.kind
                                    {
                                        for path in event.paths {
                                            if let Ok(utf8_path) = Utf8PathBuf::from_path_buf(path)
                                                && let Ok(abs_path) =
                                                    AbsPathBuf::try_from(utf8_path)
                                            {
                                                self.debounce.add_path(abs_path);
                                            }
                                        }
                                    }
                                }
                                None => {}
                            }
                        }

                        let paths = self.debounce.flush();
                        let files: Vec<(AbsPathBuf, Option<Vec<u8>>)> = paths
                            .into_iter()
                            .filter_map(|path| -> Option<(AbsPathBuf, Option<Vec<u8>>)> {
                                let meta = fs::metadata(&path).ok()?;
                                if meta.file_type().is_dir()
                                    && self
                                        .watched_dir_entries
                                        .iter()
                                        .any(|dir| dir.contains_dir(&path))
                                {
                                    self.watch(path.as_ref());
                                    return None;
                                }

                                if !meta.file_type().is_file() {
                                    return None;
                                }

                                if !(self.watched_file_entries.contains(&path)
                                    || self
                                        .watched_dir_entries
                                        .iter()
                                        .any(|dir| dir.contains_file(&path)))
                                {
                                    return None;
                                }

                                let contents = read(&path);
                                Some((path, contents))
                            })
                            .collect();
                        if !files.is_empty() {
                            self.send(loader::Message::Changed { files });
                        }
                    }
                }
            }
        }
    }

    fn handle_message(&mut self, msg: Message) {
        match msg {
            Message::Config(config) => {
                self.watcher = None;
                if !config.watch.is_empty() {
                    // Bounded channel for filesystem events - can be high volume during bulk changes
                    let (watcher_sender, watcher_receiver) = bounded(64);
                    let watcher = log_notify_error(RecommendedWatcher::new(
                        move |event| {
                            _ = watcher_sender.send(event);
                        },
                        Config::default(),
                    ));
                    self.watcher = watcher.map(|it| (it, watcher_receiver));
                }

                let config_version = config.version;

                let n_total = config.load.len();
                self.watched_dir_entries.clear();
                self.watched_file_entries.clear();

                self.send(loader::Message::Progress {
                    n_total,
                    n_done: LoadingProgress::Started,
                    config_version,
                    dir: None,
                });

                // Bounded channels for parallel loading - capacity based on typical workspace size
                let (entry_tx, entry_rx) = bounded(32);
                let (watch_tx, watch_rx) = bounded(256);
                let processed = AtomicUsize::new(0);

                config.load.into_par_iter().enumerate().for_each(|(i, entry)| {
                    let do_watch = config.watch.contains(&i);
                    if do_watch {
                        _ = entry_tx.send(entry.clone());
                    }
                    let files = Self::load_entry(
                        |f| _ = watch_tx.send(f.to_owned()),
                        entry,
                        do_watch,
                        |file| {
                            self.send(loader::Message::Progress {
                                n_total,
                                n_done: LoadingProgress::Progress(
                                    processed.load(std::sync::atomic::Ordering::Relaxed),
                                ),
                                dir: Some(file),
                                config_version,
                            });
                        },
                    );
                    self.send(loader::Message::Loaded { files });
                    self.send(loader::Message::Progress {
                        n_total,
                        n_done: LoadingProgress::Progress(
                            processed.fetch_add(1, std::sync::atomic::Ordering::AcqRel) + 1,
                        ),
                        config_version,
                        dir: None,
                    });
                });

                drop(watch_tx);
                for path in watch_rx {
                    self.watch(&path);
                }

                drop(entry_tx);
                for entry in entry_rx {
                    match entry {
                        loader::Entry::Files(files) => self.watched_file_entries.extend(files),
                        loader::Entry::Directories(dir) => self.watched_dir_entries.push(dir),
                    }
                }

                self.send(loader::Message::Progress {
                    n_total,
                    n_done: LoadingProgress::Finished,
                    config_version,
                    dir: None,
                });
            }
            Message::Invalidate(path) => {
                let contents = read(path.as_path());
                let files = vec![(path, contents)];
                self.send(loader::Message::Changed { files });
            }
        }
    }

    fn load_entry(
        mut watch: impl FnMut(&Path),
        entry: loader::Entry,
        do_watch: bool,
        send_message: impl Fn(AbsPathBuf),
    ) -> Vec<(AbsPathBuf, Option<Vec<u8>>)> {
        match entry {
            loader::Entry::Files(files) => files
                .into_iter()
                .map(|file| {
                    if do_watch {
                        watch(file.as_ref());
                    }
                    let contents = read(file.as_path());
                    (file, contents)
                })
                .collect::<Vec<_>>(),
            loader::Entry::Directories(dirs) => {
                let mut res = Vec::new();

                for root in &dirs.include {
                    send_message(root.clone());
                    let walkdir =
                        WalkDir::new(root).follow_links(true).into_iter().filter_entry(|entry| {
                            if !entry.file_type().is_dir() {
                                return true;
                            }
                            let path = entry.path();

                            if path_might_be_cyclic(path) {
                                return false;
                            }

                            // We want to filter out subdirectories that are roots themselves, because they will be visited separately.
                            dirs.exclude.iter().all(|it| it != path)
                                && (root == path || dirs.include.iter().all(|it| it != path))
                        });

                    let files = walkdir.filter_map(|it| it.ok()).filter_map(|entry| {
                        let depth = entry.depth();
                        let is_dir = entry.file_type().is_dir();
                        let is_file = entry.file_type().is_file();
                        let abs_path = AbsPathBuf::try_from(
                            Utf8PathBuf::from_path_buf(entry.into_path()).ok()?,
                        )
                        .ok()?;
                        if depth < 2 && is_dir {
                            send_message(abs_path.clone());
                        }
                        if is_dir && do_watch {
                            watch(abs_path.as_ref());
                        }
                        if !is_file {
                            return None;
                        }
                        let ext = abs_path.extension().unwrap_or_default();
                        if dirs.extensions.iter().all(|it| it.as_str() != ext) {
                            return None;
                        }
                        Some(abs_path)
                    });

                    res.extend(files.map(|file| {
                        let contents = read(file.as_path());
                        (file, contents)
                    }));
                }
                res
            }
        }
    }

    fn watch(&mut self, path: &Path) {
        if let Some((watcher, _)) = &mut self.watcher {
            log_notify_error(watcher.watch(path, RecursiveMode::NonRecursive));
        }
    }

    #[track_caller]
    fn send(&self, msg: loader::Message) {
        self.sender.send(msg).unwrap();
    }
}

fn read(path: &AbsPath) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}

fn log_notify_error<T>(res: notify::Result<T>) -> Option<T> {
    res.map_err(|err| tracing::warn!("notify error: {}", err)).ok()
}

/// Is `path` a symlink to a parent directory?
///
/// Including this path is guaranteed to cause an infinite loop. This
/// heuristic is not sufficient to catch all symlink cycles (it's
/// possible to construct cycle using two or more symlinks), but it
/// catches common cases.
fn path_might_be_cyclic(path: &Path) -> bool {
    let Ok(destination) = std::fs::read_link(path) else {
        return false;
    };

    // If the symlink is of the form "../..", it's a parent symlink.
    let is_relative_parent =
        destination.components().all(|c| matches!(c, Component::CurDir | Component::ParentDir));

    is_relative_parent || path.starts_with(destination)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    fn make_path(s: &str) -> AbsPathBuf {
        AbsPathBuf::assert_utf8(std::path::PathBuf::from(s))
    }

    #[test]
    fn debounce_state_new_is_empty() {
        let state = DebounceState::new(100);
        assert!(state.pending_paths.is_empty());
        assert!(state.first_event_time.is_none());
    }

    #[test]
    fn debounce_state_add_path_sets_first_event_time() {
        let mut state = DebounceState::new(100);
        let path = make_path("/test/file.rs");
        state.add_path(path);
        assert!(state.first_event_time.is_some());
        assert_eq!(state.pending_paths.len(), 1);
    }

    #[test]
    fn debounce_state_deduplicates_paths() {
        let mut state = DebounceState::new(100);
        let path = make_path("/test/file.rs");
        state.add_path(path.clone());
        state.add_path(path.clone());
        assert_eq!(state.pending_paths.len(), 1);
    }

    #[test]
    fn debounce_state_tracks_multiple_paths() {
        let mut state = DebounceState::new(100);
        let path1 = make_path("/test/file1.rs");
        let path2 = make_path("/test/file2.rs");
        state.add_path(path1);
        state.add_path(path2);
        assert_eq!(state.pending_paths.len(), 2);
    }

    #[test]
    fn debounce_state_should_flush_false_initially() {
        let mut state = DebounceState::new(100);
        let path = make_path("/test/file.rs");
        state.add_path(path);
        assert!(!state.should_flush());
    }

    #[test]
    fn debounce_state_should_flush_true_after_duration() {
        let mut state = DebounceState::new(10);
        let path = make_path("/test/file.rs");
        state.add_path(path);
        thread::sleep(Duration::from_millis(20));
        assert!(state.should_flush());
    }

    #[test]
    fn debounce_state_flush_returns_paths() {
        let mut state = DebounceState::new(100);
        let path1 = make_path("/test/file1.rs");
        let path2 = make_path("/test/file2.rs");
        state.add_path(path1.clone());
        state.add_path(path2.clone());

        let flushed = state.flush();
        assert_eq!(flushed.len(), 2);
        assert!(flushed.contains(&path1));
        assert!(flushed.contains(&path2));
    }

    #[test]
    fn debounce_state_flush_clears_state() {
        let mut state = DebounceState::new(100);
        let path = make_path("/test/file.rs");
        state.add_path(path);
        state.flush();
        assert!(state.pending_paths.is_empty());
        assert!(state.first_event_time.is_none());
    }

    #[test]
    fn debounce_state_time_until_flush_none_initially() {
        let state = DebounceState::new(100);
        assert!(state.time_until_flush().is_none());
    }

    #[test]
    fn debounce_state_time_until_flush_returns_remaining() {
        let mut state = DebounceState::new(100);
        let path = make_path("/test/file.rs");
        state.add_path(path);
        let remaining = state.time_until_flush().unwrap();
        assert!(remaining <= Duration::from_millis(100));
        assert!(remaining > Duration::from_millis(0));
    }

    #[test]
    fn debounce_state_time_until_flush_zero_after_duration() {
        let mut state = DebounceState::new(10);
        let path = make_path("/test/file.rs");
        state.add_path(path);
        thread::sleep(Duration::from_millis(20));
        let remaining = state.time_until_flush().unwrap();
        assert_eq!(remaining, Duration::ZERO);
    }
}
