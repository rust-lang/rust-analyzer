//! Convert macro-by-example tokens which are specific to macro expansion into a
//! format that works for our parser.

use std::fmt;

use span::Edition;
use syntax::{SyntaxKind, SyntaxKind::*, T};
use tt::{RefTokenCursor, TokenCursor, TokenStream};

pub fn to_parser_input<S: Copy + fmt::Debug>(
    edition: Edition,
    mut cursor: RefTokenCursor<S>,
) -> parser::Input {
    let mut res = parser::Input::default();

    while let Some((t, spacing)) = cursor.next() {
        let kind = match t.kind {
            tt::TokenKind::Eq => SyntaxKind::EQ,
            tt::TokenKind::Lt => SyntaxKind::L_ANGLE,
            tt::TokenKind::Le => SyntaxKind::LTEQ,
            tt::TokenKind::EqEq => SyntaxKind::EQ2,
            tt::TokenKind::Ne => SyntaxKind::NEQ,
            tt::TokenKind::Ge => SyntaxKind::GTEQ,
            tt::TokenKind::Gt => SyntaxKind::R_ANGLE,
            tt::TokenKind::AndAnd => SyntaxKind::AMP2,
            tt::TokenKind::OrOr => SyntaxKind::PIPE2,
            tt::TokenKind::Not => SyntaxKind::BANG,
            tt::TokenKind::Tilde => SyntaxKind::TILDE,
            tt::TokenKind::BinOp(binop) => match binop {
                tt::BinOpToken::Plus => SyntaxKind::PLUS,
                tt::BinOpToken::Minus => SyntaxKind::MINUS,
                tt::BinOpToken::Star => SyntaxKind::STAR,
                tt::BinOpToken::Slash => SyntaxKind::SLASH,
                tt::BinOpToken::Percent => SyntaxKind::PERCENT,
                tt::BinOpToken::Caret => SyntaxKind::CARET,
                tt::BinOpToken::And => SyntaxKind::AMP,
                tt::BinOpToken::Or => SyntaxKind::PIPE,
                tt::BinOpToken::Shl => SyntaxKind::SHL,
                tt::BinOpToken::Shr => SyntaxKind::SHR,
            },
            tt::TokenKind::BinOpEq(binop) => match binop {
                tt::BinOpToken::Plus => SyntaxKind::PLUSEQ,
                tt::BinOpToken::Minus => SyntaxKind::MINUSEQ,
                tt::BinOpToken::Star => SyntaxKind::STAREQ,
                tt::BinOpToken::Slash => SyntaxKind::SLASHEQ,
                tt::BinOpToken::Percent => SyntaxKind::PERCENTEQ,
                tt::BinOpToken::Caret => SyntaxKind::CARETEQ,
                tt::BinOpToken::And => SyntaxKind::AMPEQ,
                tt::BinOpToken::Or => SyntaxKind::PIPEEQ,
                tt::BinOpToken::Shl => SyntaxKind::SHLEQ,
                tt::BinOpToken::Shr => SyntaxKind::SHREQ,
            },
            tt::TokenKind::At => SyntaxKind::AT,
            tt::TokenKind::Dot => SyntaxKind::DOT,
            tt::TokenKind::DotDot => SyntaxKind::DOT2,
            tt::TokenKind::DotDotDot => SyntaxKind::DOT3,
            tt::TokenKind::DotDotEq => SyntaxKind::DOT2EQ,
            tt::TokenKind::Comma => SyntaxKind::COMMA,
            tt::TokenKind::Semi => SyntaxKind::SEMICOLON,
            tt::TokenKind::Colon => SyntaxKind::COLON,
            tt::TokenKind::PathSep => SyntaxKind::COLON2,
            tt::TokenKind::RArrow => SyntaxKind::THIN_ARROW,
            tt::TokenKind::LArrow => todo!(),
            tt::TokenKind::FatArrow => SyntaxKind::FAT_ARROW,
            tt::TokenKind::Pound => SyntaxKind::POUND,
            tt::TokenKind::Dollar => SyntaxKind::DOLLAR,
            tt::TokenKind::Question => SyntaxKind::QUESTION,
            tt::TokenKind::SingleQuote => {
                assert!(matches!(
                    cursor.next(),
                    Some((tt::Token { kind: tt::TokenKind::Ident(..), .. }, _))
                ));
                SyntaxKind::LIFETIME
            }
            tt::TokenKind::OpenDelim(delim) => match delim {
                tt::Delimiter::Parenthesis => SyntaxKind::L_PAREN,
                tt::Delimiter::Brace => SyntaxKind::L_CURLY,
                tt::Delimiter::Bracket => SyntaxKind::L_BRACK,
                tt::Delimiter::Invisible => continue,
            },
            tt::TokenKind::CloseDelim(delim) => match delim {
                tt::Delimiter::Parenthesis => SyntaxKind::R_PAREN,
                tt::Delimiter::Brace => SyntaxKind::R_CURLY,
                tt::Delimiter::Bracket => SyntaxKind::R_BRACK,
                tt::Delimiter::Invisible => continue,
            },
            tt::TokenKind::Literal(lit) => match lit.kind {
                tt::LitKind::Byte => SyntaxKind::BYTE,
                tt::LitKind::Char => SyntaxKind::CHAR,
                tt::LitKind::Integer => SyntaxKind::INT_NUMBER,
                tt::LitKind::Float => {
                    res.push(SyntaxKind::FLOAT_NUMBER);
                    if lit.suffix.is_none() && !lit.symbol.as_str().ends_with('.') {
                        // Tag the token as joint if it is float with a fractional part
                        // we use this jointness to inform the parser about what token split
                        // event to emit when we encounter a float literal in a field access
                        res.was_joint();
                    }
                    continue;
                }
                tt::LitKind::Str => SyntaxKind::STRING,
                tt::LitKind::StrRaw(_) => SyntaxKind::STRING,
                tt::LitKind::ByteStr => SyntaxKind::BYTE_STRING,
                tt::LitKind::ByteStrRaw(_) => SyntaxKind::BYTE_STRING,
                tt::LitKind::CStr => SyntaxKind::C_STRING,
                tt::LitKind::CStrRaw(_) => SyntaxKind::C_STRING,
                tt::LitKind::Err(_) => SyntaxKind::ERROR,
            },
            tt::TokenKind::Ident(sym, raw) => match sym.as_str() {
                _ if raw.yes() => IDENT,
                "_" => T![_],
                // is this right?
                i if i.starts_with('\'') => LIFETIME_IDENT,
                text => match SyntaxKind::from_keyword(text, edition) {
                    Some(kind) => kind,
                    None => match SyntaxKind::from_contextual_keyword(text, edition) {
                        Some(contextual_keyword) => {
                            res.push_ident(contextual_keyword);
                            continue;
                        }
                        None => SyntaxKind::IDENT,
                    },
                },
            },
            tt::TokenKind::Lifetime(_) => SyntaxKind::LIFETIME,
            tt::TokenKind::DocComment(_, _, _) => todo!(),
            tt::TokenKind::Eof => break,
        };
        res.push(kind);
        if spacing == tt::Spacing::Joint {
            res.was_joint();
        }
    }

    res
}
