//! FIXME: write short doc here

mod cmd;
pub mod install;
pub mod pre_commit;

pub mod codegen;
mod ast_src;

use anyhow::Context;
use std::{
    env, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::{cmd::run, codegen::Mode};

pub use anyhow::Result;

const TOOLCHAIN: &str = "stable";

pub fn project_root() -> PathBuf {
    Path::new(
        &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned()),
    )
    .ancestors()
    .nth(1)
    .unwrap()
    .to_path_buf()
}

pub fn run_rustfmt(mode: Mode) -> Result<()> {
    ensure_rustfmt()?;

    if mode == Mode::Verify {
        run(&format!("rustup run {} -- cargo fmt -- --check", TOOLCHAIN), ".")?;
    } else {
        run(&format!("rustup run {} -- cargo fmt", TOOLCHAIN), ".")?;
    }
    Ok(())
}

fn reformat(text: impl std::fmt::Display) -> Result<String> {
    ensure_rustfmt()?;
    let mut rustfmt = Command::new("rustup")
        .args(&["run", TOOLCHAIN, "--", "rustfmt", "--config-path"])
        .arg(project_root().join("rustfmt.toml"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    write!(rustfmt.stdin.take().unwrap(), "{}", text)?;
    let output = rustfmt.wait_with_output()?;
    let stdout = String::from_utf8(output.stdout)?;
    // TODO: update the preable: replace ra_tools with the relevant path
    let preamble = "Generated file, do not edit by hand, see `crate/ra_tools/src/codegen`";
    Ok(format!("//! {}\n\n{}", preamble, stdout))
}

fn ensure_rustfmt() -> Result<()> {
    match Command::new("rustup")
        .args(&["run", TOOLCHAIN, "--", "cargo", "fmt", "--version"])
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
    {
        Ok(status) if status.success() => return Ok(()),
        _ => (),
    };
    run(&format!("rustup toolchain install {}", TOOLCHAIN), ".")?;
    run(&format!("rustup component add rustfmt --toolchain {}", TOOLCHAIN), ".")
}

pub fn run_clippy() -> Result<()> {
    match Command::new("rustup")
        .args(&["run", TOOLCHAIN, "--", "cargo", "clippy", "--version"])
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
    {
        Ok(status) if status.success() => (),
        _ => install_clippy().context("install clippy")?,
    };

    let allowed_lints = [
        "clippy::collapsible_if",
        "clippy::map_clone", // FIXME: remove when Iterator::copied stabilizes (1.36.0)
        "clippy::needless_pass_by_value",
        "clippy::nonminimal_bool",
        "clippy::redundant_pattern_matching",
    ];
    run(
        &format!(
            "rustup run {} -- cargo clippy --all-features --all-targets -- -A {}",
            TOOLCHAIN,
            allowed_lints.join(" -A ")
        ),
        ".",
    )?;
    Ok(())
}

fn install_clippy() -> Result<()> {
    run(&format!("rustup toolchain install {}", TOOLCHAIN), ".")?;
    run(&format!("rustup component add clippy --toolchain {}", TOOLCHAIN), ".")
}

pub fn remove_test(test_path_relative_to_crates: &str) -> Result<()> {
    let (rm_test_path, test_dir, test_id) = parse_test_path(test_path_relative_to_crates)?;

    prompt_confirm(&format!("Going to remove file {:?}", &rm_test_path));

    fs::remove_file(rm_test_path)?;

    prompt_confirm(&format!("Going to decrease test ids in {:?}", &test_dir));

    update_test_ids(&test_dir, test_id, -1)?;

    Ok(())
}

fn prompt_confirm(text: &str) {
    println!("{}\nPress any key to continue...", &text);
    std::io::stdin().bytes().next().unwrap().unwrap();
}

pub fn insert_test(test_path_relative_to_crates: &str) -> Result<()> {
    let (new_test_path, test_dir, new_test_id) = parse_test_path(test_path_relative_to_crates)?;

    if new_test_path.exists() {
        anyhow::bail!("Test file {:?} already exists", new_test_path);
    }

    prompt_confirm(&format!("Going to increase tests ids in {:?}", &test_dir));

    update_test_ids(&test_dir, new_test_id, 1)?;

    // Its safe to create new file without confirmation here
    fs::File::create(&new_test_path)?;

    // Some IDEs can let you open the file by a link from the terminal
    println!("Created new test file at {:?}", &new_test_path);

    Ok(())
}

fn parse_test_path(test_path_relative_to_crates: &str) -> Result<(PathBuf, PathBuf, u32)> {
    let test_file_path = project_root().join("crates").join(test_path_relative_to_crates);

    let mut test_dir = test_file_path.clone();
    let test_id = parse_test_id(test_file_path.file_name().unwrap().to_str().unwrap())?;

    test_dir.pop();
    Ok((test_file_path, test_dir, test_id))
}

/// Update all test files ids by `diff` starting from first_id and higher.
fn update_test_ids(test_dir: &Path, first_id: u32, diff: i32) -> Result<()> {
    let test_files = test_dir
        .read_dir()?
        .map(Result::unwrap)
        .filter(|dir_entry| dir_entry.file_type().unwrap().is_file());

    for test_file in test_files {
        let mut test_name = test_file.file_name().into_string().unwrap();
        let test_id = parse_test_id(&test_name)?;

        if test_id >= first_id {
            test_name.replace_range(..4, &format!("{:04}", (test_id as i32) + diff));

            let test_path = test_file.path();
            fs::rename(&test_path, test_path.with_file_name(&test_name))?;
        }
    }
    Ok(())
}

fn parse_test_id(test_name: &str) -> Result<u32> {
    test_name
        .get(..4)
        .context("Test name should start with the 4 digit number")?
        .parse()
        .context("Failed to parse the test number")
}

pub fn run_fuzzer() -> Result<()> {
    match Command::new("cargo")
        .args(&["fuzz", "--help"])
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
    {
        Ok(status) if status.success() => (),
        _ => run("cargo install cargo-fuzz", ".")?,
    };

    run("rustup run nightly -- cargo fuzz run parser", "./crates/ra_syntax")
}

/// Cleans the `./target` dir after the build such that only
/// dependencies are cached on CI.
pub fn run_pre_cache() -> Result<()> {
    let slow_tests_cookie = Path::new("./target/.slow_tests_cookie");
    if !slow_tests_cookie.exists() {
        panic!("slow tests were skipped on CI!")
    }
    rm_rf(slow_tests_cookie)?;

    for entry in Path::new("./target/debug").read_dir()? {
        let entry = entry?;
        if entry.file_type().map(|it| it.is_file()).ok() == Some(true) {
            // Can't delete yourself on windows :-(
            if !entry.path().ends_with("xtask.exe") {
                rm_rf(&entry.path())?
            }
        }
    }

    fs::remove_file("./target/.rustc_info.json")?;
    let to_delete = ["ra_", "heavy_test"];
    for &dir in ["./target/debug/deps", "target/debug/.fingerprint"].iter() {
        for entry in Path::new(dir).read_dir()? {
            let entry = entry?;
            if to_delete.iter().any(|&it| entry.path().display().to_string().contains(it)) {
                rm_rf(&entry.path())?
            }
        }
    }

    Ok(())
}

fn rm_rf(path: &Path) -> Result<()> {
    if path.is_file() { fs::remove_file(path) } else { fs::remove_dir_all(path) }
        .with_context(|| format!("failed to remove {:?}", path))
}
