use std::sync::atomic::{AtomicBool, Ordering};

use tracing::warn;

use crate::global_state::GlobalState;

static IS_GIT_LOCKED: AtomicBool = AtomicBool::new(false);

/// Check if there is a git lock in the repo
pub(crate) fn has_git_lock(_global_state: &mut GlobalState) -> bool {
    IS_GIT_LOCKED.load(Ordering::Relaxed)
}

/// Do the IO to figure out if the lock exists and sets up the watcher
pub(crate) fn check_git_lock(global_state: &mut GlobalState) -> bool {
    if let Some(project_home) = global_state.config.get_primary_workspace_roots() {
        let main_lock = project_home.join(".git").join("index.lock");
        let locked = std::fs::metadata(&main_lock).is_ok();

        if locked {
            IS_GIT_LOCKED.store(true, Ordering::SeqCst);

            // setup watcher
            //let _ = std::thread::spawn(|| {
            use vfs_notify::notify::{
                recommended_watcher, RecommendedWatcher, RecursiveMode, Result, Watcher,
            };
            let mut watcher = recommended_watcher(|res| match res {
                Ok(event) => {
                    println!("event: {:?}", event);
                    remove_git_lock();
                }
                Err(e) => println!("watch error: {:?}", e),
            })
            .expect("well watcher setup failed ...");
            watcher
                .watch(main_lock.as_ref(), RecursiveMode::NonRecursive)
                .expect(&format!("unable to watch {}", main_lock));
            //});
        }
        locked
    } else {
        warn!("unable to find workspace_roots[0]");
        false
    }
}

pub(crate) fn remove_git_lock() {
    IS_GIT_LOCKED.store(false, Ordering::SeqCst);
}
