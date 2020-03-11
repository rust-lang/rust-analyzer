//! A bad shell -- small cross platform module for writing glue code

use std::{
    cell::RefCell,
    env,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{bail, Context, Result};

pub mod fs2 {
    use std::{fs, path::Path};

    use anyhow::{Context, Result};

    pub fn read_dir<P: AsRef<Path>>(path: P) -> Result<fs::ReadDir> {
        let path = path.as_ref();
        fs::read_dir(path).with_context(|| format!("Failed to read {}", path.display()))
    }

    pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
        let path = path.as_ref();
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))
    }

    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
        let path = path.as_ref();
        fs::write(path, contents).with_context(|| format!("Failed to write {}", path.display()))
    }

    pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<u64> {
        let from = from.as_ref();
        let to = to.as_ref();
        fs::copy(from, to)
            .with_context(|| format!("Failed to copy {} to {}", from.display(), to.display()))
    }

    pub fn remove_file<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        fs::remove_file(path).with_context(|| format!("Failed to remove file {}", path.display()))
    }

    pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        fs::remove_dir_all(path).with_context(|| format!("Failed to remove dir {}", path.display()))
    }

    pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        fs::create_dir_all(path).with_context(|| format!("Failed to create dir {}", path.display()))
    }
}

macro_rules! _run {
    ($($expr:expr),*) => {
        run!($($expr),*; echo = true)
    };
    ($($expr:expr),* ; echo = $echo:expr) => {
        $crate::not_bash::run_process(format!($($expr),*), $echo)
    };
}
pub(crate) use _run as run;

pub struct Pushd {
    _p: (),
}

pub fn pushd(path: impl Into<PathBuf>) -> Pushd {
    Env::with(|env| env.pushd(path.into()));
    Pushd { _p: () }
}

impl Drop for Pushd {
    fn drop(&mut self) {
        Env::with(|env| env.popd())
    }
}

pub fn rm_rf(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(());
    }
    if path.is_file() {
        fs2::remove_file(path)
    } else {
        fs2::remove_dir_all(path)
    }
}

#[doc(hidden)]
pub fn run_process(cmd: String, echo: bool) -> Result<String> {
    run_process_inner(&cmd, echo).with_context(|| format!("process `{}` failed", cmd))
}

fn run_process_inner(cmd: &str, echo: bool) -> Result<String> {
    let mut args = shelx(cmd)?;
    let binary = args.remove(0);
    let current_dir = Env::with(|it| it.cwd().to_path_buf());

    if echo {
        println!("> {}", cmd)
    }

    let output = Command::new(binary)
        .args(args)
        .current_dir(current_dir)
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    if echo {
        print!("{}", stdout)
    }

    if !output.status.success() {
        bail!("{}", output.status)
    }

    Ok(stdout.trim().to_string())
}

struct Env {
    pushd_stack: Vec<PathBuf>,
}

impl Env {
    fn with<F: FnOnce(&mut Env) -> T, T>(f: F) -> T {
        thread_local! {
            static ENV: RefCell<Env> = RefCell::new(Env {
                pushd_stack: vec![env::current_dir().unwrap()]
            });
        }
        ENV.with(|it| f(&mut *it.borrow_mut()))
    }

    fn pushd(&mut self, dir: PathBuf) {
        let dir = self.cwd().join(dir);
        self.pushd_stack.push(dir);
        env::set_current_dir(self.cwd()).unwrap();
    }
    fn popd(&mut self) {
        self.pushd_stack.pop().unwrap();
        env::set_current_dir(self.cwd()).unwrap();
    }
    fn cwd(&self) -> &Path {
        self.pushd_stack.last().unwrap()
    }
}

use shelx_parser::parse_cmd as shelx;
mod shelx_parser {
    use anyhow::{bail, Result};

    const ESCAPE_CHAR: char = '\\';
    const QUOTE_CHARS: [char; 2] = ['"', '\''];

    pub(crate) fn parse_cmd(mut input: &str) -> Result<Vec<String>> {
        let mut acc = Vec::new();

        input = input.trim_start();
        while !input.is_empty() {
            let param = quoted_param(input).unwrap_or_else(|| non_quoted_param(input))?;

            input = input[param.len()..].trim_start();
            acc.push(param);
        }

        Ok(acc)
    }

    fn non_quoted_param(input: &str) -> Result<String> {
        assert_ne!(input.len(), 0);
        enum State {
            Begin,
            Char(char),
            EscapeChar,
        }
        let mut state = State::Begin;
        let mut acc = String::new();

        for char in input.chars() {
            match state {
                State::Char(char) => acc.push(char),
                _ => {}
            }
            state = match state {
                State::EscapeChar => State::Char(unescape(char)?),
                State::Begin | State::Char(_) => {
                    if char == ESCAPE_CHAR {
                        State::EscapeChar
                    } else if char.is_whitespace() {
                        return Ok(acc);
                    } else {
                        State::Char(char)
                    }
                }
            }
        }
        match state {
            State::EscapeChar => bail!(
                "Expected a character after `{}` escape but reached the end of input",
                ESCAPE_CHAR
            ),
            _ => Ok(acc),
        }
    }

    fn quoted_param(input: &str) -> Option<Result<String>> {
        // FIXME: change to str::strip_prefix once it is stable
        let quote = *QUOTE_CHARS.iter().find(|quote| input.starts_with(**quote))?;

        return Some(inner(input, quote));

        fn inner(input: &str, quote: char) -> Result<String> {
            enum State {
                Begin,
                Char(char),
                EscapeChar,
            }
            let mut state = State::Begin;
            let mut acc = String::new();

            for char in input[1..].chars() {
                match state {
                    State::Char(char) => acc.push(char),
                    _ => {}
                }
                state = match state {
                    State::Char(_) | State::Begin => {
                        if char == ESCAPE_CHAR {
                            State::EscapeChar
                        } else if char == quote {
                            return Ok(acc);
                        } else {
                            State::Char(char)
                        }
                    }
                    State::EscapeChar => {
                        State::Char(if char == quote { quote } else { unescape(char)? })
                    }
                }
            }
            match state {
                State::EscapeChar => bail!(
                    "Expected a character after `{}` escape but reached the end of input",
                    ESCAPE_CHAR
                ),
                _ => bail!("Expected the closing quote `{}` but reached the end of input", quote),
            }
        }
    }

    fn unescape(char: char) -> Result<char> {
        Ok(match char {
            ESCAPE_CHAR => ESCAPE_CHAR,
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '0' => '\0',
            _ => bail!("Invalid escape {}{}", ESCAPE_CHAR, char),
        })
    }
}
