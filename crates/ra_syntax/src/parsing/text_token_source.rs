//! See `TextTokenSource` docs.

use ra_parser::TokenSource;

use crate::{parsing::lexer::Token, SyntaxKind::EOF};

/// Implementation of `ra_parser::TokenSource` that takes tokens from source code text.
pub(crate) struct TextTokenSource<'t> {
    text: &'t str,
    /// non-whitespace/comment tokens
    significant_tokens: Vec<Token>,

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
            .map(|token| &self.text[token.range] == kw)
            .unwrap_or(false)
    }
}

fn mk_token(pos: usize, token_offset_pairs: &[Token]) -> ra_parser::Token {
    let (kind, is_jointed_to_next) = match token_offset_pairs.get(pos) {
        Some(token) => (
            token.kind,
            token_offset_pairs
                .get(pos + 1)
                .map(|next_token| token.range.end() == next_token.range.start())
                .unwrap_or(false),
        ),
        None => (EOF, false),
    };
    ra_parser::Token { kind, is_jointed_to_next }
}

impl<'t> TextTokenSource<'t> {
    /// Generate input from tokens (expect comment and whitespace).
    pub fn new(text: &'t str, tokens: &'t [Token]) -> TextTokenSource<'t> {
        let significant_tokens: Vec<_> =
            tokens.iter().copied().filter(|it| !it.kind.is_trivia()).collect();

        let first = mk_token(0, &significant_tokens);
        TextTokenSource { text, significant_tokens, curr: (first, 0) }
    }
}
