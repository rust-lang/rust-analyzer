//! Process management utilities to prevent processes from leaking on exit,
//! plus reading both stdout and stderr of a child without deadlocks.
//!
//! Every child process must be created via [`JodChild::spawn`],
//! [`JodChild::spawn_grouped`], or one of the helpers built on top of them
//! ([`output`], [`spawn_with_streaming_output`]). Funneling all spawning
//! through one place lets us uniformly tie the children's lifetime to
//! rust-analyzer's own, so that no processes are left behind when
//! rust-analyzer exits.
//!
//! <https://github.com/rust-lang/cargo/blob/905af549966f23a9288e9993a85d1249a5436556/crates/cargo-util/src/read2.rs>
//! <https://github.com/rust-lang/cargo/blob/58a961314437258065e23cb6316dfc121d96fb71/crates/cargo-util/src/process_builder.rs#L231>

use std::{
    io,
    process::{ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus, Output, Stdio},
};

use process_wrap::std::ChildWrapper;

/// 'Join on drop' handle to a child process: the child is killed when the
/// handle is dropped.
#[derive(Debug)]
pub struct JodChild {
    child: Box<dyn ChildWrapper>,
}

impl Drop for JodChild {
    fn drop(&mut self) {
        _ = self.child.kill();
        _ = self.child.wait();
    }
}

impl JodChild {
    /// Spawns `command` as a direct child process.
    pub fn spawn(command: &mut Command) -> io::Result<JodChild> {
        die_with_parent(command);
        #[allow(
            clippy::disallowed_methods,
            reason = "we are inside the module that implements the replacement"
        )]
        let child = command.spawn()?;
        let child = Box::new(child) as Box<dyn ChildWrapper>;
        Ok(JodChild { child })
    }

    /// Spawns `command` as the leader of a new process group (a new session on
    /// unix, a job object on windows), so that killing the child also kills
    /// the processes the child spawned itself.
    pub fn spawn_grouped(mut command: Command) -> io::Result<JodChild> {
        die_with_parent(&mut command);
        let mut command = process_wrap::std::CommandWrap::from(command);
        #[cfg(unix)]
        command.wrap(process_wrap::std::ProcessSession);
        #[cfg(windows)]
        command.wrap(process_wrap::std::JobObject);
        Ok(JodChild { child: command.spawn()? })
    }

    pub fn stdin(&mut self) -> &mut Option<ChildStdin> {
        self.child.stdin()
    }

    pub fn stdout(&mut self) -> &mut Option<ChildStdout> {
        self.child.stdout()
    }

    pub fn stderr(&mut self) -> &mut Option<ChildStderr> {
        self.child.stderr()
    }

    pub fn id(&self) -> u32 {
        self.child.id()
    }

    /// Kills the child process, and the process group it leads if spawned via
    /// [`JodChild::spawn_grouped`].
    pub fn kill(&mut self) -> io::Result<()> {
        self.child.kill()
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        self.child.wait()
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }

    /// Closes the child's stdin if piped, then waits for it to exit, capturing
    /// its remaining stdout and stderr.
    ///
    /// # Panics
    ///
    /// Panics if the child's stdout and stderr are not piped.
    pub fn wait_with_output(mut self) -> io::Result<Output> {
        drop(self.stdin().take());
        let stdout = self.stdout().take().expect("child stdout must be piped");
        let stderr = self.stderr().take().expect("child stderr must be piped");
        let (stdout, stderr) =
            streaming_output(stdout, stderr, &mut |_| (), &mut |_| (), &mut || ())?;
        let status = self.wait()?;
        Ok(Output { status, stdout, stderr })
    }
}

/// Equivalent of [`std::process::Command::output`], going through the spawn
/// chokepoint.
pub fn output(command: &mut Command) -> io::Result<Output> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::null());
    JodChild::spawn(command)?.wait_with_output()
}

/// Makes the OS kill all descendant processes when this process exits, no
/// matter how it exits (including crashes and being killed), so that no
/// spawned cargo or proc-macro server can be leaked.
///
/// On windows, this assigns the current process to a new job object with
/// `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`. Every process spawned from here on
/// (transitively, so including e.g. rustc spawned by cargo) inherits
/// membership in the job, and when this process dies the kernel closes the
/// deliberately leaked job handle, killing all remaining members. Failure to
/// set the job up is logged and otherwise ignored.
///
/// On linux, the equivalent is arranged per child via `PR_SET_PDEATHSIG` at
/// spawn time (see `die_with_parent`), so this function is a no-op.
///
/// On other platforms (macos), no kernel primitive for this exists, this
/// function is a no-op, and children are only cleaned up on graceful exit.
pub fn kill_descendants_on_exit() {
    #[cfg(windows)]
    windows_job::put_self_in_kill_on_close_job();
}

/// Requests that the kernel deliver `SIGKILL` to `command`'s child when this
/// process dies, covering exits that never run cleanup code (crashes,
/// `SIGKILL`). Only direct children are covered; grandchildren like rustc are
/// killed on the graceful exit path only, via the process group of
/// [`JodChild::spawn_grouped`].
///
/// `PR_SET_PDEATHSIG` fires when the spawning *thread* exits, not the
/// process. That is sound for rust-analyzer: children are spawned either from
/// threads that block on the child (so the thread outlives it), or from
/// thread pool workers and the main loop thread, which live for the lifetime
/// of the process.
fn die_with_parent(_command: &mut Command) {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt;

        let parent = std::process::id();
        // SAFETY: `prctl`, `getppid` and `_exit` are async-signal-safe.
        unsafe {
            _command.pre_exec(move || {
                libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
                // If the parent died before the prctl above, the signal will
                // never be delivered; detect that (we were reparented) and bail.
                if libc::getppid() != parent as libc::pid_t {
                    libc::_exit(1);
                }
                Ok(())
            });
        }
    }
}

#[cfg(windows)]
mod windows_job {
    use std::{ffi::c_void, io, mem, ptr};

    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::{
            JobObjects::{
                AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
                JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
                SetInformationJobObject,
            },
            Threading::GetCurrentProcess,
        },
    };

    pub(super) fn put_self_in_kill_on_close_job() {
        // SAFETY: plain winapi calls with valid arguments. The job handle is
        // deliberately leaked on success so that it is only closed (killing
        // the job's members) when this process dies.
        unsafe {
            let job = CreateJobObjectW(ptr::null(), ptr::null());
            if job.is_null() {
                tracing::warn!(
                    "failed to create job object: {}, spawned processes may be leaked on abnormal exit",
                    io::Error::last_os_error()
                );
                return;
            }
            let mut info = mem::zeroed::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &raw const info as *const c_void,
                mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            ) == 0
            {
                tracing::warn!(
                    "failed to configure job object: {}, spawned processes may be leaked on abnormal exit",
                    io::Error::last_os_error()
                );
                CloseHandle(job);
                return;
            }
            if AssignProcessToJobObject(job, GetCurrentProcess()) == 0 {
                tracing::warn!(
                    "failed to assign self to job object: {}, spawned processes may be leaked on abnormal exit",
                    io::Error::last_os_error()
                );
                CloseHandle(job);
            }
        }
    }
}

pub fn streaming_output(
    out: ChildStdout,
    err: ChildStderr,
    on_stdout_line: &mut dyn FnMut(&str),
    on_stderr_line: &mut dyn FnMut(&str),
    on_eof: &mut dyn FnMut(),
) -> io::Result<(Vec<u8>, Vec<u8>)> {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    imp::read2(out, err, &mut |is_out, data, eof| {
        let idx = if eof {
            data.len()
        } else {
            match data.iter().rposition(|&b| b == b'\n') {
                Some(i) => i + 1,
                None => return,
            }
        };
        {
            // scope for new_lines
            let new_lines = {
                let dst = if is_out { &mut stdout } else { &mut stderr };
                let start = dst.len();
                let data = data.drain(..idx);
                dst.extend(data);
                &dst[start..]
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
        }
    })?;

    Ok((stdout, stderr))
}

/// Runs `cmd` to completion, invoking the callbacks on every line it writes
/// to stdout and stderr.
pub fn spawn_with_streaming_output(
    mut cmd: Command,
    on_stdout_line: &mut dyn FnMut(&str),
    on_stderr_line: &mut dyn FnMut(&str),
) -> io::Result<Output> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::null());

    let mut child = JodChild::spawn(&mut cmd)?;
    let (stdout, stderr) = streaming_output(
        child.stdout().take().unwrap(),
        child.stderr().take().unwrap(),
        on_stdout_line,
        on_stderr_line,
        &mut || (),
    )?;
    let status = child.wait()?;
    Ok(Output { status, stdout, stderr })
}

#[cfg(unix)]
mod imp {
    use std::{
        io::{self, prelude::*},
        mem,
        os::unix::prelude::*,
        process::{ChildStderr, ChildStdout},
    };

    pub(crate) fn read2(
        mut out_pipe: ChildStdout,
        mut err_pipe: ChildStderr,
        data: &mut dyn FnMut(bool, &mut Vec<u8>, bool),
    ) -> io::Result<()> {
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
            // wait for either pipe to become readable using `select`
            let r = unsafe { libc::poll(fds.as_mut_ptr(), nfds, -1) };
            if r == -1 {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::Interrupted {
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
        Ok(())
    }
}

#[cfg(windows)]
mod imp {
    use std::{
        io,
        os::windows::prelude::*,
        process::{ChildStderr, ChildStdout},
        slice,
    };

    use miow::{
        Overlapped,
        iocp::{CompletionPort, CompletionStatus},
        pipe::NamedPipe,
    };
    use windows_sys::Win32::Foundation::ERROR_BROKEN_PIPE;

    struct Pipe<'a> {
        dst: &'a mut Vec<u8>,
        overlapped: Overlapped,
        pipe: NamedPipe,
        done: bool,
    }

    pub(crate) fn read2(
        out_pipe: ChildStdout,
        err_pipe: ChildStderr,
        data: &mut dyn FnMut(bool, &mut Vec<u8>, bool),
    ) -> io::Result<()> {
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
                for status in port.get_many(&mut status, None)? {
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

            Ok(())
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
}
