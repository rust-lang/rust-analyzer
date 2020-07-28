//! Simple hand-written ungrammar lexer
use crate::error::{bail, Result};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum TokenKind {
    Node(String),
    Token(String),
    Eq,
    Star,
    Pipe,
    QMark,
    Colon,
    LParen,
    RParen,
}

#[derive(Debug)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) loc: Location,
}

#[derive(Copy, Clone, Default, Debug)]
pub(crate) struct Location {
    offset: usize,
}

pub(crate) fn tokenize(mut input: &str) -> Result<Vec<Token>> {
    let mut res = Vec::new();
    let mut loc = Location::default();
    while !input.is_empty() {
        let old_len = input.len();
        skip_ws(&mut input);
        if old_len == input.len() {
            match advance(&mut input) {
                Ok(kind) => {
                    res.push(Token { kind, loc });
                }
                Err(err) => return Err(err.with_location(loc)),
            }
        }
        loc.offset += old_len - input.len();
    }

    Ok(res)
}

fn skip_ws(input: &mut &str) {
    *input = input.trim_start_matches(|c: char| c.is_ascii_whitespace())
}
fn advance(input: &mut &str) -> Result<TokenKind> {
    let mut chars = input.chars();
    let c = chars.next().unwrap();
    let res = match c {
        '=' => TokenKind::Eq,
        '*' => TokenKind::Star,
        '?' => TokenKind::QMark,
        '(' => TokenKind::LParen,
        ')' => TokenKind::RParen,
        '|' => TokenKind::Pipe,
        ':' => TokenKind::Colon,
        '\'' => {
            let mut buf = String::new();
            loop {
                match chars.next() {
                    None => bail!("unclosed token literal"),
                    Some('\'') => break,
                    Some(c) => buf.push(c),
                }
            }
            TokenKind::Token(buf)
        }
        c if is_ident_char(c) => {
            let mut buf = String::new();
            buf.push(c);
            loop {
                match chars.clone().next() {
                    Some(c) if is_ident_char(c) => {
                        chars.next();
                        buf.push(c);
                    }
                    _ => break,
                }
            }
            TokenKind::Node(buf)
        }
        c => bail!("unexpected character: `{}`", c),
    };

    *input = chars.as_str();
    Ok(res)
}

fn is_ident_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '_')
}
