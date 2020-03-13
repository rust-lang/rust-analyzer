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
    let mut args = shelx::parse(cmd)?;
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

mod shelx {
    use anyhow::{bail, Result};

    pub(crate) fn parse(input: &str) -> Result<Vec<String>> {
        Lexer::new(input).tokenize().map(|tokens| {
            tokens
                .into_iter()
                .map(|token| match token {
                    Token::NotQuotedString(s) => s,
                    Token::QuotedString(s) => s,
                })
                .collect()
        })
    }

    /// Command line arguments parser.
    /// Note: the implementation is heavily inspired by `rustc_lexer` crate
    struct Lexer<'a> {
        chars: std::str::Chars<'a>,
    }

    /// Whitespace is thrown off for simplicity
    enum Token {
        NotQuotedString(String), // unescaped string
        QuotedString(String),    // unescaped string
    }

    impl Lexer<'_> {
        fn new(input: &str) -> Lexer<'_> {
            Lexer { chars: input.chars() }
        }

        fn bump(&mut self) -> Option<char> {
            self.chars.next()
        }
        fn nth(&self, n: usize) -> Option<char> {
            self.chars.clone().nth(n)
        }

        fn tokenize(&mut self) -> Result<Vec<Token>> {
            let mut acc = Vec::new();

            while let Some(token) = self.advance_token() {
                acc.push(token?);
            }

            Ok(acc)
        }

        fn advance_token(&mut self) -> Option<Result<Token>> {
            while matches!(self.nth(0), Some(char) if char.is_whitespace()) {
                self.bump();
            }

            Some(match self.nth(0)? {
                quote @ '"' | quote @ '\'' => self.quoted_string(quote),
                _ => self.non_quoted_string(),
            })
        }

        fn quoted_string(&mut self, quote: char) -> Result<Token> {
            assert_eq!(self.bump(), Some(quote));

            let mut acc = String::new();

            while let Some(char) = self.bump() {
                if char == quote {
                    return Ok(Token::QuotedString(acc));
                }
                if char != '\\' {
                    acc.push(char);
                    continue;
                }
                if let Some(escaped_char) = self.bump() {
                    acc.push(if escaped_char == quote {
                        quote
                    } else {
                        Self::unescape(escaped_char)?
                    });
                    continue;
                }
                bail!("Expected a character after `\\` escape but reached the end of input");
            }
            bail!("Expected the closing quote `{}` but reached the end of input", quote);
        }

        fn non_quoted_string(&mut self) -> Result<Token> {
            let mut acc = String::new();

            while let Some(char) = self.bump() {
                if char.is_whitespace() {
                    break;
                }
                if char != '\\' {
                    acc.push(char);
                    continue;
                }
                if let Some(escaped_char) = self.bump() {
                    acc.push(if escaped_char.is_whitespace() {
                        escaped_char
                    } else {
                        Self::unescape(escaped_char)?
                    });
                    continue;
                }
                bail!("Expected a character after `\\` escape but reached the end of input");
            }

            Ok(Token::NotQuotedString(acc))
        }

        fn unescape(char: char) -> Result<char> {
            Ok(match char {
                '\\' => '\\',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '0' => '\0',
                _ => bail!("Invalid escape `\\{}`", char),
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn assert_parses(input: &str, expected: &[&str]) {
            assert_eq!(parse(input).unwrap(), expected);
        }
        fn assert_parse_error(input: &str, err: &str) {
            assert_eq!(format!("{}", parse(input).unwrap_err()), err)
        }

        #[test]
        fn empty_string_results() {
            assert_parses("", &[]);
        }

        #[test]
        fn whitespace_only_string() {
            assert_parses("   \n \t \r ", &[]);
        }

        #[test]
        fn non_quoted_string_simple() {
            let str = "rust-analyzer:Турбо_пушка!";
            assert_parses(str, &[str]);
        }

        #[test]
        fn non_quoted_string_with_escapes() {
            assert_parses(r#"\n\t\r\0\ "#, &["\n\t\r\0 "]);
        }

        #[test]
        fn empty_quoted_string() {
            assert_parses("\"\"", &[""]);
        }

        #[test]
        fn quoted_string_simple() {
            assert_parses(
                "\"rust-analyzer \n \t \r \0 Турбо_пушка!\"",
                &["rust-analyzer \n \t \r \0 Турбо_пушка!"],
            );
            assert_parses(
                "'rust-analyzer \n \t \r \0 Турбо_пушка!'",
                &["rust-analyzer \n \t \r \0 Турбо_пушка!"],
            );
        }

        #[test]
        fn quoted_string_with_escapes() {
            assert_parses(r#""\n\t\r\0\" ""#, &["\n\t\r\0\" "]);
            assert_parses(r#"'\''"#, &["'"]);
        }

        #[test]
        fn multiple_strings() {
            assert_parses(r#" a   b  "c d"   "ef" \\ \t "#, &["a", "b", "c d", "ef", "\\", "\t"]);
        }

        #[test]
        fn joint_qouted_strigs() {
            assert_parses(r#" "a""""cd" "#, &["a", "", "cd"]);
        }

        #[test]
        fn error_unclosed_double_quoted_string() {
            let msg = "Expected the closing quote `\"` but reached the end of input";
            assert_parse_error("\"", &msg);
            assert_parse_error("\"abcd ef '", &msg);
        }

        #[test]
        fn error_unclosed_single_quoted_string() {
            let msg = "Expected the closing quote `'` but reached the end of input";
            assert_parse_error("'", &msg);
            assert_parse_error("'abcd ef \"", &msg);
        }

        #[test]
        fn error_invalid_escape() {
            let msg = "Expected a character after `\\` escape but reached the end of input";
            assert_parse_error("\\", msg);
            assert_parse_error("\\x", "Invalid escape `\\x`");
        }
    }
}
