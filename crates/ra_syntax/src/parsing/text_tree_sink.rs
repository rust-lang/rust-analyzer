//! FIXME: write short doc here

use std::mem;

use ra_parser::{ParseError, TreeSink};

use crate::{
    parsing::Token,
    syntax_node::GreenNode,
    SmolStr, SyntaxError,
    SyntaxKind::{self, *},
    SyntaxTreeBuilder, TextRange, TextSize,
};

/// Bridges the parser with our specific syntax tree representation.
///
/// `TextTreeSink` also handles attachment of trivia (whitespace) to nodes.
pub(crate) struct TextTreeSink<'a> {
    text: &'a str,
    tokens: &'a [Token],
    token_pos: usize,
    state: State,
    inner: SyntaxTreeBuilder,
}

enum State {
    PendingStart,
    Normal,
    PendingFinish,
}

impl<'a> TreeSink for TextTreeSink<'a> {
    fn token(&mut self, kind: SyntaxKind, n_tokens: u8) {
        match mem::replace(&mut self.state, State::Normal) {
            State::PendingStart => unreachable!(),
            State::PendingFinish => self.inner.finish_node(),
            State::Normal => (),
        }
        self.eat_trivias();
        self.do_token(kind, usize::from(n_tokens));
    }

    fn start_node(&mut self, kind: SyntaxKind) {
        match mem::replace(&mut self.state, State::Normal) {
            State::PendingStart => {
                self.inner.start_node(kind);
                // No need to attach trivias to previous node: there is no
                // previous node.
                return;
            }
            State::PendingFinish => self.inner.finish_node(),
            State::Normal => (),
        }

        let n_trivias =
            self.tokens[self.token_pos..].iter().take_while(|it| it.kind.is_trivia()).count();
        let n_attached_trivias = {
            let leading_trivias = &self.tokens[self.token_pos..self.token_pos + n_trivias];
            let mut trivia_end = self.offset_at(self.token_pos + n_trivias);
            let leading_trivias = leading_trivias.iter().rev().map(|it| {
                let range = TextRange::new(it.offset, trivia_end);
                trivia_end = it.offset;
                (it.kind, &self.text[range])
            });
            n_attached_trivias(kind, leading_trivias)
        };
        self.eat_n_trivias(n_trivias - n_attached_trivias);
        self.inner.start_node(kind);
        self.eat_n_trivias(n_attached_trivias);
    }

    fn finish_node(&mut self) {
        match mem::replace(&mut self.state, State::PendingFinish) {
            State::PendingStart => unreachable!(),
            State::PendingFinish => self.inner.finish_node(),
            State::Normal => (),
        }
    }

    fn error(&mut self, error: ParseError) {
        self.inner.error(error, self.offset_at(self.token_pos))
    }
}

impl<'a> TextTreeSink<'a> {
    pub(super) fn new(text: &'a str, tokens: &'a [Token]) -> Self {
        Self {
            text,
            tokens,
            token_pos: 0,
            state: State::PendingStart,
            inner: SyntaxTreeBuilder::default(),
        }
    }

    pub(super) fn finish(mut self) -> (GreenNode, Vec<SyntaxError>) {
        match mem::replace(&mut self.state, State::Normal) {
            State::PendingFinish => {
                self.eat_trivias();
                self.inner.finish_node()
            }
            State::PendingStart | State::Normal => unreachable!(),
        }

        self.inner.finish_raw()
    }

    fn eat_trivias(&mut self) {
        while let Some(&token) = self.tokens.get(self.token_pos) {
            if !token.kind.is_trivia() {
                break;
            }
            self.do_token(token.kind, 1);
        }
    }

    fn eat_n_trivias(&mut self, n: usize) {
        for _ in 0..n {
            let token = self.tokens[self.token_pos];
            assert!(token.kind.is_trivia());
            self.do_token(token.kind, 1);
        }
    }

    fn do_token(&mut self, kind: SyntaxKind, n_tokens: usize) {
        let range = TextRange::new(
            self.tokens[self.token_pos].offset,
            self.offset_at(self.token_pos + n_tokens),
        );
        let text: SmolStr = self.text[range].into();
        self.token_pos += n_tokens;
        self.inner.token(kind, text);
    }

    fn offset_at(&self, idx: usize) -> TextSize {
        self.tokens.get(idx).map_or_else(|| TextSize::of(self.text), |it| it.offset)
    }
}

fn n_attached_trivias<'a>(
    kind: SyntaxKind,
    trivias: impl Iterator<Item = (SyntaxKind, &'a str)>,
) -> usize {
    match kind {
        MACRO_CALL | CONST_DEF | TYPE_ALIAS_DEF | STRUCT_DEF | ENUM_DEF | ENUM_VARIANT | FN_DEF
        | TRAIT_DEF | MODULE | RECORD_FIELD_DEF | STATIC_DEF => {
            let mut res = 0;
            let mut trivias = trivias.enumerate().peekable();

            while let Some((i, (kind, text))) = trivias.next() {
                match kind {
                    WHITESPACE => {
                        if text.contains("\n\n") {
                            // we check whether the next token is a doc-comment
                            // and skip the whitespace in this case
                            if let Some((peek_kind, peek_text)) =
                                trivias.peek().map(|(_, pair)| pair)
                            {
                                if *peek_kind == COMMENT && peek_text.starts_with("///") {
                                    continue;
                                }
                            }
                            break;
                        }
                    }
                    COMMENT => {
                        res = i + 1;
                    }
                    _ => (),
                }
            }
            res
        }
        _ => 0,
    }
}
