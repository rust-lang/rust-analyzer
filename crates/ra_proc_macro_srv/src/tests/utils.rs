use crate::dylib;
pub use difference::Changeset as __Changeset;
use ra_tt::*;
use std::{fmt, str::FromStr};

fn nice_format_recur(tkn: &TokenTree, f: &mut fmt::Formatter, level: usize) -> fmt::Result {
    let align = std::iter::repeat("  ").take(level).collect::<String>();

    match tkn {
        TokenTree::Leaf(leaf) => match leaf {
            Leaf::Literal(lit) => write!(f, "{}LITERAL {}", align, lit.text)?,
            Leaf::Punct(punct) => write!(
                f,
                "{}PUNCH   {} [{}]",
                align,
                punct.char,
                if punct.spacing == Spacing::Alone { "alone" } else { "joint" }
            )?,
            Leaf::Ident(ident) => write!(f, "{}IDENT   {}", align, ident.text)?,
        },
        TokenTree::Subtree(subtree) => {
            let delim = match subtree.delimiter.map(|it| it.kind) {
                None => ("NODELIM"),
                Some(DelimiterKind::Parenthesis) => "()",
                Some(DelimiterKind::Brace) => "{}",
                Some(DelimiterKind::Bracket) => "[]",
            };
            if subtree.token_trees.is_empty() {
                write!(f, "{}SUBTREE {}", align, delim)?;
            } else {
                writeln!(f, "{}SUBTREE {}", align, delim)?;
                for (idx, child) in subtree.token_trees.iter().enumerate() {
                    nice_format_recur(child, f, level + 1)?;
                    if idx != subtree.token_trees.len() - 1 {
                        writeln!(f, "")?;
                    }
                }
            }
        }
    }

    Ok(())
}

struct NiceFormat<'a>(&'a TokenTree);
impl std::fmt::Display for NiceFormat<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        nice_format_recur(self.0, f, 0)
    }
}

fn nice_format(tkn: &TokenTree) -> String {
    format!("{}", NiceFormat(tkn))
}

macro_rules! assert_eq_text {
    ($left:expr, $right:expr) => {
        assert_eq_text!($left, $right,)
    };
    ($left:expr, $right:expr, $($tt:tt)*) => {{
        let left = $left.trim();
        let right = $right.trim();
        if left != right {
            let changeset = __Changeset::new(right, left, "\n");
            eprintln!("Diff:\n{}\n", changeset);
            eprintln!($($tt)*);
            panic!("text differs");
        }
    }};
}

macro_rules! assert_eq_file {
    ($left:expr, $right:literal) => {
        assert_eq_file!($left, $right,)
    };
    ($left:expr, $right:literal, $($tt:tt)*) => {{
        let right_raw = include_str!($right).replace("\r\n", "\n");
        assert_eq_text!($left,right_raw.trim(),$($tt)*);
    }};
}

mod fixtures {
    use cargo_metadata::{parse_messages, Message};
    use std::process::Command;

    // Use current project metadata to get the proc-macro dylib path
    pub fn dylib_path(crate_name: &str) -> std::path::PathBuf {
        let command = Command::new("cargo")
            .args(&["check", "--message-format", "json"])
            .output()
            .unwrap()
            .stdout;

        for message in parse_messages(command.as_slice()) {
            match message.unwrap() {
                Message::CompilerArtifact(artifact) => {
                    if artifact.target.kind.contains(&"proc-macro".to_string()) {
                        if artifact.package_id.repr.starts_with(crate_name) {
                            return artifact.filenames[0].clone();
                        }
                    }
                }
                _ => (), // Unknown message
            }
        }

        panic!("No proc-macro dylib for {} found!", crate_name);
    }
}

fn parse_string(code: &str) -> Option<crate::rustc_server::TokenStream> {
    Some(crate::rustc_server::TokenStream::from_str(code).unwrap())
}

pub fn expand(crate_name: &str, macro_name: &str, fixture: &str) -> String {
    let path = fixtures::dylib_path(crate_name);
    let expander = dylib::Expander::new(&path).unwrap();
    let fixture = parse_string(fixture).unwrap();

    let res = expander.expand(macro_name, &fixture.subtree, None).unwrap();
    nice_format(&res.into())
}
