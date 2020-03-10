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

    enum Cmd {
        Begin,
        Whitespace,
        /// Stands for `non-quoted parameter`
        NQParam(NQParam),
        /// Stands for `quoted parameter`
        QParam(QParam, char),
    }
    enum NQParam {
        Char(char),
        EscapeChar,
        End,
    }
    enum QParam {
        Begin,
        Char(char),
        EscapeChar,
        End,
    }

    const ESCAPE_CHAR: char = '\\';
    const QUOTE_CHARS: &'static [char] = &['"', '\''];
    // const QUOTE_CHARS: [char; 2] = ['"', '\''];

    pub(crate) fn parse_cmd(input: &str) -> Result<Vec<String>> {
        let mut acc = Vec::new();
        let mut param = String::new();
        let mut state = Cmd::Begin;
        for char in input.chars() {
            state = next_state(state, char)?;
            match state {
                Cmd::QParam(QParam::Char(char), ..) | Cmd::NQParam(NQParam::Char(char)) => {
                    param.push(char);
                }
                Cmd::QParam(QParam::End, ..) | Cmd::NQParam(NQParam::End) => {
                    acc.push(param);
                    param = String::new();
                }
                _ => {}
            }
        }

        match state {
            Cmd::NQParam(..) => {
                acc.push(param);
            }
            Cmd::QParam(QParam::Begin, quote) | Cmd::QParam(QParam::Char(..), quote) => {
                bail!("Expected closing quote `{}`", quote);
            }
            Cmd::QParam(QParam::EscapeChar, _) => {
                bail!("Expected closing escape char `{}`", ESCAPE_CHAR);
            }
            _ => {}
        }

        Ok(acc)
    }

    fn next_state(state: Cmd, char: char) -> Result<Cmd> {
        Ok(match state {
            Cmd::Begin
            | Cmd::Whitespace
            | Cmd::NQParam(NQParam::End)
            | Cmd::QParam(QParam::End, _) => {
                if char == ESCAPE_CHAR {
                    Cmd::NQParam(NQParam::EscapeChar)
                } else if QUOTE_CHARS.contains(&char) {
                    Cmd::QParam(QParam::Begin, char)
                } else if char.is_whitespace() {
                    Cmd::Whitespace
                } else {
                    Cmd::NQParam(NQParam::Char(char))
                }
            }
            Cmd::NQParam(NQParam::EscapeChar) => Cmd::NQParam(NQParam::Char(unescape(char)?)),
            Cmd::NQParam(NQParam::Char(_)) => Cmd::NQParam(if char.is_whitespace() {
                NQParam::End
            } else if char == ESCAPE_CHAR {
                NQParam::EscapeChar
            } else {
                NQParam::Char(char)
            }),
            Cmd::QParam(QParam::Begin, quote) | Cmd::QParam(QParam::Char(_), quote) => Cmd::QParam(
                if char == ESCAPE_CHAR {
                    QParam::EscapeChar
                } else if char == quote {
                    QParam::End
                } else {
                    QParam::Char(char)
                },
                quote,
            ),
            Cmd::QParam(QParam::EscapeChar, quote) => Cmd::QParam(
                QParam::Char(if char == quote { quote } else { unescape(char)? }),
                quote,
            ),
        })
    }

    fn unescape(char: char) -> Result<char> {
        Ok(match char {
            ESCAPE_CHAR => ESCAPE_CHAR,
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '0' => '\0',
            _ => bail!("invalid escape {}{}", ESCAPE_CHAR, char),
        })
    }
}
