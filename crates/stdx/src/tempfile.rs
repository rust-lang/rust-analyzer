//! A temporary named file that will be deleted on drop, and on operating systems that support that,
//! also when the process exits (including being killed).

use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};

pub struct NamedTempFile {
    _file: Option<File>,
    path: PathBuf,
    delete_on_drop: bool,
    dir_to_delete: Option<PathBuf>,
}

impl NamedTempFile {
    pub fn new(prefix: &str) -> io::Result<NamedTempFile> {
        imp::create(prefix)
    }

    /// Creates a new `NamedTempFile` that is a copy of an existing file, keeping its file
    /// name by placing the copy in a fresh temporary directory.
    ///
    /// Unlike [`NamedTempFile::new`], the returned path is guaranteed to stay linked in the
    /// filesystem until the value is dropped, so it can be handed to other processes. Some
    /// consumers also require the exact file name to be preserved (e.g. Cargo insists that
    /// a lockfile is named `Cargo.lock`), which the temporary directory provides.
    pub fn new_from_existing(prefix: &str, existing: &Path) -> io::Result<NamedTempFile> {
        let file_name = existing.file_name().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "existing file has no file name")
        })?;
        let dir = general_imp::create_dir(prefix)?;
        let path = dir.join(file_name);
        let copy = (|| {
            std::fs::copy(existing, &path)?;
            // The source may be read-only (e.g. a lockfile in a read-only toolchain
            // installation); make the copy writable so consumers can update it.
            let mut perms = std::fs::metadata(&path)?.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            perms.set_readonly(false);
            std::fs::set_permissions(&path, perms)
        })();
        if let Err(e) = copy {
            _ = std::fs::remove_dir_all(&dir);
            return Err(e);
        }
        Ok(NamedTempFile { _file: None, path, delete_on_drop: true, dir_to_delete: Some(dir) })
    }

    /// Creates a `NamedTempFile` from a path, without deleting it on drop.
    #[inline]
    pub fn from_path(path: PathBuf) -> NamedTempFile {
        NamedTempFile { _file: None, path, delete_on_drop: false, dir_to_delete: None }
    }

    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for NamedTempFile {
    fn drop(&mut self) {
        if self.delete_on_drop && std::fs::remove_file(&self.path).is_err() {
            tracing::info!("cannot remove temporary file {}", self.path.display());
        }
        if let Some(dir) = &self.dir_to_delete
            && std::fs::remove_dir(dir).is_err()
        {
            tracing::info!("cannot remove temporary directory {}", dir.display());
        }
    }
}

mod general_imp {
    use std::{
        fs::{File, OpenOptions},
        io::{self, ErrorKind},
        path::PathBuf,
        sync::atomic::{AtomicU32, Ordering},
    };

    static INTERNAL_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub(super) fn create(
        prefix: &str,
        mut options_callback: impl FnMut(&mut OpenOptions),
    ) -> io::Result<(File, PathBuf)> {
        let temp_dir = std::env::temp_dir().canonicalize()?;
        let pid = std::process::id();
        loop {
            let path = temp_dir.join(format!(
                "{prefix}{pid:x}-{:x}",
                INTERNAL_COUNTER.fetch_add(1, Ordering::AcqRel),
            ));
            let mut open_options = OpenOptions::new();
            // `create_new` requires the file to be opened with write or append access.
            open_options.write(true).create_new(true);
            options_callback(&mut open_options);
            match open_options.open(&path) {
                Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
                Err(e) => {
                    return Err(io::Error::new(
                        e.kind(),
                        format!("error creating directory {path:?}: {e}"),
                    ));
                }
                Ok(file) => {
                    return Ok((file, path));
                }
            }
        }
    }

    /// Creates a fresh, uniquely named temporary directory.
    pub(super) fn create_dir(prefix: &str) -> io::Result<PathBuf> {
        let temp_dir = std::env::temp_dir().canonicalize()?;
        let pid = std::process::id();
        loop {
            let path = temp_dir.join(format!(
                "{prefix}{pid:x}-{:x}",
                INTERNAL_COUNTER.fetch_add(1, Ordering::AcqRel),
            ));
            match std::fs::create_dir(&path) {
                Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
                Err(e) => {
                    return Err(io::Error::new(
                        e.kind(),
                        format!("error creating directory {path:?}: {e}"),
                    ));
                }
                Ok(()) => return Ok(path),
            }
        }
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
))]
mod imp {
    use std::{
        ffi::CString,
        io,
        os::{
            fd::{AsRawFd, RawFd},
            unix::ffi::OsStrExt,
        },
    };

    use super::*;

    #[cfg(target_os = "linux")]
    fn path_after_unlink(fd: RawFd) -> PathBuf {
        PathBuf::from(format!("/proc/self/fd/{fd}"))
    }

    #[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    fn path_after_unlink(fd: RawFd) -> PathBuf {
        PathBuf::from(format!("/dev/fd/{fd}"))
    }

    pub(super) fn create(prefix: &str) -> io::Result<NamedTempFile> {
        let (file, mut path) = general_imp::create(prefix, |_| {})?;
        let mut delete_on_drop = true;
        if let Ok(original_path) = CString::new(path.as_os_str().as_bytes()) {
            // Unlinking the file will *not* remove it per the POSIX specification since it is open.
            // We cannot use `std::fs::remove_file()`, since, while currently using `unlink()`, it does
            // not guarantee it will use it.
            if unsafe { libc::unlink(original_path.as_ptr()) } == 0 {
                path = path_after_unlink(file.as_raw_fd());
                delete_on_drop = false;
            }
        }
        Ok(NamedTempFile { _file: Some(file), path, delete_on_drop, dir_to_delete: None })
    }
}

#[cfg(windows)]
mod imp {
    use std::os::windows::fs::OpenOptionsExt;

    use super::*;

    const FILE_ATTRIBUTE_TEMPORARY: u32 = 0x100;
    const FILE_FLAG_DELETE_ON_CLOSE: u32 = 0x04000000;

    pub(super) fn create(prefix: &str) -> io::Result<NamedTempFile> {
        let (file, path) = general_imp::create(prefix, |options| {
            options.attributes(FILE_ATTRIBUTE_TEMPORARY);
            options.custom_flags(FILE_FLAG_DELETE_ON_CLOSE);
        })?;
        Ok(NamedTempFile { _file: Some(file), path, delete_on_drop: false, dir_to_delete: None })
    }
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    windows,
)))]
mod imp {
    use super::*;

    pub(super) fn create(prefix: &str) -> io::Result<NamedTempFile> {
        let (file, path) = general_imp::create(prefix, |_| {})?;
        Ok(NamedTempFile { _file: Some(file), path, delete_on_drop: true, dir_to_delete: None })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_a_writable_file() {
        let temp = NamedTempFile::new("stdx-test-new").unwrap();
        std::fs::write(temp.path(), b"hello").unwrap();
    }

    #[test]
    fn new_from_existing_keeps_file_name_and_is_writable() {
        let source = NamedTempFile::new_from_existing("stdx-test-source", &{
            let dir = general_imp::create_dir("stdx-test-orig").unwrap();
            let path = dir.join("Cargo.lock");
            std::fs::write(&path, b"contents").unwrap();
            // Simulate a read-only source, like a lockfile in a read-only
            // toolchain installation.
            let mut perms = std::fs::metadata(&path).unwrap().permissions();
            perms.set_readonly(true);
            std::fs::set_permissions(&path, perms).unwrap();
            path
        })
        .unwrap();
        // The copy keeps the file name so that consumers which require an exact
        // name (Cargo insists on `Cargo.lock`) can use it.
        assert_eq!(source.path().file_name().unwrap(), "Cargo.lock");
        assert_eq!(std::fs::read(source.path()).unwrap(), b"contents");
        // The path is a real, linked file that other processes could open, and
        // the copy is writable even when the source was read-only.
        std::fs::write(source.path(), b"updated").unwrap();
    }
}
