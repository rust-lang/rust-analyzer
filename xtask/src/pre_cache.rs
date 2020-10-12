use std::path::Path;

use anyhow::Result;

use crate::not_bash::rm_rf;

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

        Ok(())
    }
}

//
