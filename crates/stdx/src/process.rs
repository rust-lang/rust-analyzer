#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "we define `stdx::process`"
)]

//! Read both stdout and stderr of child without deadlocks.
//!
//! <https://github.com/rust-lang/cargo/blob/905af549966f23a9288e9993a85d1249a5436556/crates/cargo-util/src/read2.rs>
//! <https://github.com/rust-lang/cargo/blob/58a961314437258065e23cb6316dfc121d96fb71/crates/cargo-util/src/process_builder.rs#L231>

use std::{
    ffi::OsStr,
    fmt, io,
    path::Path,
    process::{
        ChildStderr, ChildStdin, ChildStdout, Command, CommandArgs, CommandEnvs, ExitStatus,
        Output, Stdio,
    },
    time::Duration,
};

const CHECK_CANCELLATION_EVERY: Duration = Duration::from_millis(100);

/// A [`std::process::Command`] wrapper that creates a [`JodChild`].
pub struct JodCommand {
    inner: std::process::Command,
    stdout_was_set: bool,
    stderr_was_set: bool,
}

impl fmt::Debug for JodCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("JodCommand").field(&self.inner).finish()
    }
}

impl From<Command> for JodCommand {
    fn from(inner: Command) -> Self {
        JodCommand { inner, stdout_was_set: false, stderr_was_set: false }
    }
}

impl JodCommand {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Self { inner: Command::new(program), stdout_was_set: false, stderr_was_set: false }
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.inner.arg(arg);
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.inner.args(args);
        self
    }

    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.inner.env(key, val);
        self
    }

    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.inner.envs(vars);
        self
    }

    pub fn env_remove<K: AsRef<OsStr>>(&mut self, key: K) -> &mut Self {
        self.inner.env_remove(key);
        self
    }

    pub fn env_clear(&mut self) -> &mut Self {
        self.inner.env_clear();
        self
    }

    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.inner.current_dir(dir);
        self
    }

    pub fn stdin<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.inner.stdin(cfg);
        self
    }

    pub fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.stdout_was_set = true;
        self.inner.stdout(cfg);
        self
    }

    pub fn stderr<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.stderr_was_set = true;
        self.inner.stderr(cfg);
        self
    }

    pub fn spawn(&mut self) -> io::Result<JodChild> {
        self.inner.spawn().map(JodChild)
    }

    pub fn output(&mut self) -> io::Result<Output> {
        self.stdin(Stdio::null());
        if !self.stdout_was_set {
            self.stdout(Stdio::piped());
        }
        if !self.stderr_was_set {
            self.stderr(Stdio::piped());
        }

        self.spawn()?.wait_with_output()
    }

    // `status()` is not provided, due to not being used and being less trivial
    // to implement correctly.

    pub fn get_program(&self) -> &OsStr {
        self.inner.get_program()
    }

    pub fn get_args(&self) -> CommandArgs<'_> {
        self.inner.get_args()
    }

    pub fn get_envs(&self) -> CommandEnvs<'_> {
        self.inner.get_envs()
    }

    pub fn get_current_dir(&self) -> Option<&Path> {
        self.inner.get_current_dir()
    }

    /// # Panics
    ///
    /// Panics if `cmd` is not configured to have `stdout` and `stderr` as `piped`.
    pub fn spawn_with_streaming_output(
        mut self,
        on_stdout_line: &mut dyn FnMut(&str),
        on_stderr_line: &mut dyn FnMut(&str),
    ) -> io::Result<Output> {
        self.stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::null());

        let mut child = self.spawn()?;
        let (stdout, stderr) = streaming_output(
            child.0.stdout.take().unwrap(),
            child.0.stderr.take().unwrap(),
            on_stdout_line,
            on_stderr_line,
            &mut || (),
        )?;
        let status = child.wait()?;
        Ok(Output { status, stdout, stderr })
    }

    pub fn into_inner(self) -> Command {
        self.inner
    }
}

/// A [`std::process::Child`] wrapper that will kill the child on drop, and also panics on [`crate::thread::Pool`]
/// cancellation while waiting for child.
#[cfg_attr(not(target_arch = "wasm32"), repr(transparent))]
#[derive(Debug)]
pub struct JodChild(std::process::Child);

impl JodChild {
    #[inline]
    pub fn stdin(&mut self) -> Option<&mut ChildStdin> {
        self.0.stdin.as_mut()
    }

    pub fn kill(&mut self) -> io::Result<()> {
        self.0.kill()
    }

    pub fn id(&self) -> u32 {
        self.0.id()
    }

    fn close_stdin(&mut self) {
        self.0.stdin = None;
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        self.close_stdin();

        if let Some(result) = self.0.try_wait().transpose() {
            return result;
        }

        loop {
            crate::thread::Pool::unwind_if_cancelled();

            match imp::wait_process_timeout(&mut self.0, CHECK_CANCELLATION_EVERY).transpose() {
                Some(result) => return result,
                None => {}
            }
        }
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.0.try_wait()
    }

    pub fn wait_with_output(mut self) -> io::Result<Output> {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        self.close_stdin();

        match (self.0.stdout.take(), self.0.stderr.take()) {
            (None, None) => {}
            (Some(out_pipe), None) => stdout = imp::read1(out_pipe)?,
            (None, Some(err_pipe)) => stderr = imp::read1(err_pipe)?,
            (Some(out_pipe), Some(err_pipe)) => {
                (stdout, stderr) = imp::read2(out_pipe, err_pipe, &mut |_, _, _| {})?;
            }
        }

        let status = self.wait()?;
        Ok(Output { status, stdout, stderr })
    }

    /// **Warning:** Waiting on the raw child can make thread pool cancellations wait for the process
    /// to finish!
    #[inline]
    pub fn inner_unchecked(&mut self) -> &mut std::process::Child {
        &mut self.0
    }

    #[must_use]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_inner(self) -> std::process::Child {
        // SAFETY: repr transparent, except on WASM
        unsafe { std::mem::transmute::<Self, std::process::Child>(self) }
    }
}

impl Drop for JodChild {
    fn drop(&mut self) {
        _ = self.0.kill();
        _ = self.0.wait();
    }
}

pub fn streaming_output(
    out: ChildStdout,
    err: ChildStderr,
    on_stdout_line: &mut dyn FnMut(&str),
    on_stderr_line: &mut dyn FnMut(&str),
    on_eof: &mut dyn FnMut(),
) -> io::Result<(Vec<u8>, Vec<u8>)> {
    let (mut last_stdout_line, mut last_stderr_line) = (0, 0);
    imp::read2(out, err, &mut |is_out, data, eof| {
        // scope for new_lines
        let new_lines = {
            let last_line = if is_out { &mut last_stdout_line } else { &mut last_stderr_line };
            let mut new_data = &data[*last_line..];
            match new_data.iter().rposition(|&ch| ch == b'\n') {
                Some(new_last_line) => {
                    new_data = &new_data[..=new_last_line];
                    *last_line += new_last_line + 1;
                }
                None => {
                    if !eof {
                        // Not completed a line yet.
                        return;
                    }
                }
            }
            new_data
        };
        for line in String::from_utf8_lossy(new_lines).lines() {
            if is_out {
                on_stdout_line(line);
            } else {
                on_stderr_line(line);
            }
        }
        if eof {
            on_eof();
        }
    })
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
mod imp {
    use std::{
        io::{self, prelude::*},
        mem,
        os::unix::prelude::*,
        process::{Child, ChildStderr, ChildStdout, ExitStatus},
        time::Duration,
    };

    use crate::process::CHECK_CANCELLATION_EVERY;

    pub(crate) fn read1(mut pipe: impl AsRawFd + IntoRawFd + Read) -> io::Result<Vec<u8>> {
        unsafe {
            libc::fcntl(pipe.as_raw_fd(), libc::F_SETFL, libc::O_NONBLOCK);
        }

        let mut done = false;
        let mut data = Vec::new();

        let mut fds: [libc::pollfd; 1] = unsafe { mem::zeroed() };
        fds[0].fd = pipe.as_raw_fd();
        fds[0].events = libc::POLLIN;

        while !done {
            crate::thread::Pool::unwind_if_cancelled();

            // wait for either pipe to become readable using `select`
            let r = unsafe {
                libc::poll(fds.as_mut_ptr(), 1, CHECK_CANCELLATION_EVERY.as_millis() as _)
            };
            if r == -1 {
                let err = io::Error::last_os_error();
                if matches!(err.kind(), io::ErrorKind::Interrupted | io::ErrorKind::TimedOut) {
                    continue;
                }
                return Err(err);
            }

            // Read as much as we can from each pipe, ignoring EWOULDBLOCK or
            // EAGAIN. If we hit EOF, then this will happen because the underlying
            // reader will return Ok(0), in which case we'll see `Ok` ourselves. In
            // this case we flip the other fd back into blocking mode and read
            // whatever's leftover on that file descriptor.
            let handle = |res: io::Result<_>| match res {
                Ok(_) => Ok(true),
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        Ok(false)
                    } else {
                        Err(e)
                    }
                }
            };
            if fds[0].revents != 0 && handle(pipe.read_to_end(&mut data))? {
                done = true;
            }
        }
        Ok(data)
    }

    pub(crate) fn read2(
        mut out_pipe: ChildStdout,
        mut err_pipe: ChildStderr,
        data: &mut dyn FnMut(bool, &mut Vec<u8>, bool),
    ) -> io::Result<(Vec<u8>, Vec<u8>)> {
        unsafe {
            libc::fcntl(out_pipe.as_raw_fd(), libc::F_SETFL, libc::O_NONBLOCK);
            libc::fcntl(err_pipe.as_raw_fd(), libc::F_SETFL, libc::O_NONBLOCK);
        }

        let mut out_done = false;
        let mut err_done = false;
        let mut out = Vec::new();
        let mut err = Vec::new();

        let mut fds: [libc::pollfd; 2] = unsafe { mem::zeroed() };
        fds[0].fd = out_pipe.as_raw_fd();
        fds[0].events = libc::POLLIN;
        fds[1].fd = err_pipe.as_raw_fd();
        fds[1].events = libc::POLLIN;
        let mut nfds = 2;
        let mut errfd = 1;

        while nfds > 0 {
            crate::thread::Pool::unwind_if_cancelled();

            // wait for either pipe to become readable using `select`
            let r = unsafe {
                libc::poll(fds.as_mut_ptr(), nfds, CHECK_CANCELLATION_EVERY.as_millis() as _)
            };
            if r == -1 {
                let err = io::Error::last_os_error();
                if matches!(err.kind(), io::ErrorKind::Interrupted | io::ErrorKind::TimedOut) {
                    continue;
                }
                return Err(err);
            }

            // Read as much as we can from each pipe, ignoring EWOULDBLOCK or
            // EAGAIN. If we hit EOF, then this will happen because the underlying
            // reader will return Ok(0), in which case we'll see `Ok` ourselves. In
            // this case we flip the other fd back into blocking mode and read
            // whatever's leftover on that file descriptor.
            let handle = |res: io::Result<_>| match res {
                Ok(_) => Ok(true),
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        Ok(false)
                    } else {
                        Err(e)
                    }
                }
            };
            if !err_done && fds[errfd].revents != 0 && handle(err_pipe.read_to_end(&mut err))? {
                err_done = true;
                nfds -= 1;
            }
            data(false, &mut err, err_done);
            if !out_done && fds[0].revents != 0 && handle(out_pipe.read_to_end(&mut out))? {
                out_done = true;
                fds[0].fd = err_pipe.as_raw_fd();
                errfd = 0;
                nfds -= 1;
            }
            data(true, &mut out, out_done);
        }
        Ok((out, err))
    }

    pub(super) fn wait_process_timeout(
        child: &mut Child,
        dur: Duration,
    ) -> io::Result<Option<ExitStatus>> {
        // Unfortunately on Unix there is no easy way to wait for a process with a timeout.
        // There are convoluted ways with signals, and there is a Linux-specific way, but ugh.
        // So we just do the simple and stupid thing.
        std::thread::sleep(dur);
        child.try_wait()
    }
}

#[cfg(windows)]
mod imp {
    use std::{
        io::{self, Read},
        os::windows::prelude::*,
        process::{Child, ChildStderr, ChildStdout, ExitStatus},
        slice,
        time::Duration,
    };

    use miow::{
        Overlapped,
        iocp::{CompletionPort, CompletionStatus},
        pipe::NamedPipe,
    };
    use windows_sys::Win32::{
        Foundation::{ERROR_BROKEN_PIPE, WAIT_OBJECT_0, WAIT_TIMEOUT},
        System::Threading::WaitForSingleObject,
    };

    use crate::process::CHECK_CANCELLATION_EVERY;

    struct Pipe<'a> {
        dst: &'a mut Vec<u8>,
        overlapped: Overlapped,
        pipe: NamedPipe,
        done: bool,
    }

    pub(super) fn read1(pipe: impl AsRawHandle + IntoRawHandle + Read) -> io::Result<Vec<u8>> {
        let mut data = Vec::new();

        let port = CompletionPort::new(1)?;
        port.add_handle(0, &pipe)?;

        unsafe {
            let mut pipe = Pipe::new(pipe, &mut data);

            pipe.read()?;

            while !pipe.done {
                crate::thread::Pool::unwind_if_cancelled();

                let get = port.get(Some(CHECK_CANCELLATION_EVERY));
                if let Err(err) = &get
                    && let io::ErrorKind::TimedOut = err.kind()
                {
                    continue;
                }
                let status = get?;
                pipe.complete(&status);
                pipe.read()?;
            }

            Ok(data)
        }
    }

    pub(super) fn read2(
        out_pipe: ChildStdout,
        err_pipe: ChildStderr,
        data: &mut dyn FnMut(bool, &mut Vec<u8>, bool),
    ) -> io::Result<(Vec<u8>, Vec<u8>)> {
        let mut out = Vec::new();
        let mut err = Vec::new();

        let port = CompletionPort::new(1)?;
        port.add_handle(0, &out_pipe)?;
        port.add_handle(1, &err_pipe)?;

        unsafe {
            let mut out_pipe = Pipe::new(out_pipe, &mut out);
            let mut err_pipe = Pipe::new(err_pipe, &mut err);

            out_pipe.read()?;
            err_pipe.read()?;

            let mut status = [CompletionStatus::zero(), CompletionStatus::zero()];

            while !out_pipe.done || !err_pipe.done {
                crate::thread::Pool::unwind_if_cancelled();

                let get_many = port.get_many(&mut status, Some(CHECK_CANCELLATION_EVERY));
                if let Err(err) = &get_many
                    && let io::ErrorKind::TimedOut = err.kind()
                {
                    continue;
                }
                for status in get_many? {
                    if status.token() == 0 {
                        out_pipe.complete(status);
                        data(true, out_pipe.dst, out_pipe.done);
                        out_pipe.read()?;
                    } else {
                        err_pipe.complete(status);
                        data(false, err_pipe.dst, err_pipe.done);
                        err_pipe.read()?;
                    }
                }
            }

            Ok((out, err))
        }
    }

    impl<'a> Pipe<'a> {
        unsafe fn new<P: IntoRawHandle>(p: P, dst: &'a mut Vec<u8>) -> Pipe<'a> {
            let pipe = unsafe { NamedPipe::from_raw_handle(p.into_raw_handle()) };
            Pipe { dst, pipe, overlapped: Overlapped::zero(), done: false }
        }

        unsafe fn read(&mut self) -> io::Result<()> {
            let dst = unsafe { slice_to_end(self.dst) };
            match unsafe { self.pipe.read_overlapped(dst, self.overlapped.raw()) } {
                Ok(_) => Ok(()),
                Err(e) => {
                    if e.raw_os_error() == Some(ERROR_BROKEN_PIPE as i32) {
                        self.done = true;
                        Ok(())
                    } else {
                        Err(e)
                    }
                }
            }
        }

        unsafe fn complete(&mut self, status: &CompletionStatus) {
            let prev = self.dst.len();
            unsafe { self.dst.set_len(prev + status.bytes_transferred() as usize) };
            if status.bytes_transferred() == 0 {
                self.done = true;
            }
        }
    }

    unsafe fn slice_to_end(v: &mut Vec<u8>) -> &mut [u8] {
        if v.capacity() == 0 {
            v.reserve(16);
        }
        if v.capacity() == v.len() {
            v.reserve(1);
        }
        let data = unsafe { v.as_mut_ptr().add(v.len()) };
        let len = v.capacity() - v.len();
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    pub(super) fn wait_process_timeout(
        child: &mut Child,
        dur: Duration,
    ) -> io::Result<Option<ExitStatus>> {
        let ms = dur.as_millis();
        let ms = if ms > u128::from(u32::MAX) { u32::MAX } else { ms as u32 };
        unsafe {
            match WaitForSingleObject(child.as_raw_handle() as *mut _, ms) {
                WAIT_OBJECT_0 => {}
                WAIT_TIMEOUT => return Ok(None),
                _ => return Err(io::Error::last_os_error()),
            }
        }
        child.try_wait()
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use std::{
        io,
        process::{Child, ChildStderr, ChildStdout, ExitStatus},
        time::Duration,
    };

    pub(super) fn read1<T>(_pipe: T) -> io::Result<Vec<u8>> {
        panic!("no processes on wasm")
    }

    pub(crate) fn read2(
        _out_pipe: ChildStdout,
        _err_pipe: ChildStderr,
        _data: &mut dyn FnMut(bool, &mut Vec<u8>, bool),
    ) -> io::Result<(Vec<u8>, Vec<u8>)> {
        panic!("no processes on wasm")
    }

    pub(super) fn wait_process_timeout(
        _child: &mut Child,
        _dur: Duration,
    ) -> io::Result<Option<ExitStatus>> {
        panic!("no processes on wasm")
    }
}
