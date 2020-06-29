//! See `TextTokenSource` docs.

use ra_parser::{SyntaxKind, TokenSource};

use crate::{
    parsing::lexer::{kinds_with_ranges, Token},
    SyntaxKind::EOF,
    TextRange,
};

/// Implementation of `ra_parser::TokenSource` that takes tokens from source code text.
pub(crate) struct TextTokenSource<'t> {
    text: &'t str,
    /// token and its length (non-whitespace/comment tokens)
    /// ```non-rust
    ///  struct Foo;
    ///  ^------^--^-
    ///  |      |  \__________________
    ///  |      \_______              \
    ///  |              \             |
    ///  (struct, 0..6) (Foo, 7..10) (;, 10..11)
    /// ```
    significant_tokens: Vec<(SyntaxKind, TextRange)>,

    /// Current token and position
    curr: (ra_parser::Token, usize),
}

impl<'t> TokenSource for TextTokenSource<'t> {
    fn current(&self) -> ra_parser::Token {
        self.curr.0
    }

    fn lookahead_nth(&self, n: usize) -> ra_parser::Token {
        mk_token(self.curr.1 + n, &self.significant_tokens)
    }

    fn bump(&mut self) {
        if self.curr.0.kind == EOF {
            return;
        }

        let pos = self.curr.1 + 1;
        self.curr = (mk_token(pos, &self.significant_tokens), pos);
    }

    fn is_keyword(&self, kw: &str) -> bool {
        self.significant_tokens
            .get(self.curr.1)
            .map(|(_, range)| &self.text[*range] == kw)
            .unwrap_or(false)
    }
}

fn mk_token(pos: usize, significant_tokens: &[(SyntaxKind, TextRange)]) -> ra_parser::Token {
    let (kind, is_jointed_to_next) = match significant_tokens.get(pos) {
        Some((cur_kind, cur_range)) => (
            *cur_kind,
            significant_tokens
                .get(pos + 1)
                .map(|(_, next_range)| cur_range.end() == next_range.start())
                .unwrap_or(false),
        ),
        None => (EOF, false),
    };
    ra_parser::Token { kind, is_jointed_to_next }
}

impl<'t> TextTokenSource<'t> {
    /// Generate input from tokens(expect comment and whitespace).
    pub fn new(text: &'t str, raw_tokens: &'t [Token]) -> TextTokenSource<'t> {
        let significant_tokens: Vec<_> =
            kinds_with_ranges(text, raw_tokens).filter(|(kind, _)| !kind.is_trivia()).collect();

        let first = mk_token(0, &significant_tokens);
        TextTokenSource { text, significant_tokens, curr: (first, 0) }
    }
}
