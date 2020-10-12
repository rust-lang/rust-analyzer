use std::{
    fs::FileType,
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::not_bash::{fs2, rm_rf, run};

pub struct PreCacheCmd;

impl PreCacheCmd {
    /// Cleans the `./target` dir after the build such that only
    /// dependencies are cached on CI.
    pub fn run(self) -> Result<()> {
        let slow_tests_cookie = Path::new("./target/.slow_tests_cookie");
        if !slow_tests_cookie.exists() {
            panic!("slow tests were skipped on CI!")
        }
        rm_rf(slow_tests_cookie)?;

        for dir in read_dir("./crates", FileType::is_dir)? {
            let crate_name = dir.file_name().unwrap().to_str().unwrap();
            run!("cargo clean -p {}", crate_name).unwrap();
        }

        Ok(())
    }
}
fn read_dir(path: impl AsRef<Path>, cond: impl Fn(&FileType) -> bool) -> Result<Vec<PathBuf>> {
    read_dir_impl(path.as_ref(), &cond)
}

fn read_dir_impl(path: &Path, cond: &dyn Fn(&FileType) -> bool) -> Result<Vec<PathBuf>> {
    let mut res = Vec::new();
    for entry in path.read_dir()? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if cond(&file_type) {
            res.push(entry.path())
        }
    }
    Ok(res)
}

//
