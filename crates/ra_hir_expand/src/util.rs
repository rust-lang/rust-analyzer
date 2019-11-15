//! Internal utility functions.
//!
//! The `unquote funciton are mainly copy from enquote crate

use std::error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub(crate) enum QuoteError {
    NotEnoughChars,
    UnrecognizedQuote,
    UnexpectedEOF,
    IllegalChar,
    UnrecognizedEscape,
    InvalidUnicode,
}

impl fmt::Display for QuoteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl error::Error for QuoteError {
    fn description(&self) -> &str {
        match self {
            QuoteError::NotEnoughChars => "not enough chars",
            QuoteError::UnrecognizedQuote => "unrecognized quote character",
            QuoteError::UnexpectedEOF => "unexpected eof",
            QuoteError::IllegalChar => "illegal character",
            QuoteError::UnrecognizedEscape => "unrecognized escape sequence",
            QuoteError::InvalidUnicode => "invalid unicode code point",
        }
    }
}

/// Unquotes `s`.
pub(crate) fn unquote(s: &str) -> Result<String, QuoteError> {
    if s.chars().count() < 2 {
        return Err(QuoteError::NotEnoughChars);
    }

    let quote = s.chars().next().unwrap();

    if quote != '"' && quote != '\'' && quote != '`' {
        return Err(QuoteError::UnrecognizedQuote);
    }

    if s.chars().last().unwrap() != quote {
        return Err(QuoteError::UnexpectedEOF);
    }

    // removes quote characters
    // the sanity checks performed above ensure that the quotes will be ASCII and this will not
    // panic
    let s = &s[1..s.len() - 1];

    unescape(s, Some(quote))
}

/// Returns `s` after processing escapes such as `\n` and `\x00`.
pub(crate) fn unescape(s: &str, illegal: Option<char>) -> Result<String, QuoteError> {
    let mut chars = s.chars();
    let mut unescaped = String::new();
    loop {
        match chars.next() {
            None => break,
            Some(c) => unescaped.push(match c {
                _ if Some(c) == illegal => return Err(QuoteError::IllegalChar),
                '\\' => match chars.next() {
                    None => return Err(QuoteError::UnexpectedEOF),
                    Some(c) => match c {
                        _ if c == '\\' || c == '"' || c == '\'' || c == '`' => c,
                        'a' => '\x07',
                        'b' => '\x08',
                        'f' => '\x0c',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        'v' => '\x0b',
                        // octal
                        '0'..='9' => {
                            let octal = c.to_string() + &chars.by_ref().take(2).collect::<String>();
                            u8::from_str_radix(&octal, 8)
                                .map_err(|_| QuoteError::UnrecognizedEscape)?
                                as char
                        }
                        // hex
                        'x' => {
                            let hex: String = chars.by_ref().take(2).collect();
                            u8::from_str_radix(&hex, 16)
                                .map_err(|_| QuoteError::UnrecognizedEscape)?
                                as char
                        }
                        // unicode
                        'u' => decode_unicode(&chars.by_ref().take(4).collect::<String>())?,
                        'U' => decode_unicode(&chars.by_ref().take(8).collect::<String>())?,
                        _ => return Err(QuoteError::UnrecognizedEscape),
                    },
                },
                _ => c,
            }),
        }
    }

    Ok(unescaped)
}

fn decode_unicode(code_point: &str) -> Result<char, QuoteError> {
    match u32::from_str_radix(code_point, 16) {
        Err(_) => return Err(QuoteError::UnrecognizedEscape),
        Ok(n) => std::char::from_u32(n).ok_or(QuoteError::InvalidUnicode),
    }
}
