//! Shortcuts that span lexer/parser abstraction.
//!
//! The way Rust works, parser doesn't necessary parse text, and you might
//! tokenize text without parsing it further. So, it makes sense to keep
//! abstract token parsing, and string tokenization as completely separate
//! layers.
//!
//! However, often you do parse text into syntax trees and the glue code for
//! that needs to live somewhere. Rather than putting it to lexer or parser, we
//! use a separate shortcuts module for that.

use std::{fmt, mem};

use crate::{Edition, LexedStr, Step, SyntaxKind};

#[derive(Clone, Copy)]
pub struct Trivia<'a> {
    pub kind: SyntaxKind,
    pub text: &'a str,
}

impl fmt::Debug for Trivia<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}({:?})", self.kind, self.text)
    }
}

#[derive(Debug)]
pub enum StrStep<'a> {
    Token { kind: SyntaxKind, text: &'a str, leading: &'a [Trivia<'a>], trailing: &'a [Trivia<'a>] },
    Enter { kind: SyntaxKind },
    Exit,
    Error { msg: &'a str, pos: usize },
}

impl LexedStr<'_> {
    pub fn to_input(&self, edition: Edition) -> crate::Input {
        let _p = tracing::info_span!("LexedStr::to_input").entered();
        let mut res = crate::Input::with_capacity(self.len());
        let mut was_joint = false;
        for i in 0..self.len() {
            let kind = self.kind(i);
            if kind.is_trivia() {
                was_joint = false
            } else if kind == SyntaxKind::IDENT {
                let token_text = self.text(i);
                res.push_ident(
                    SyntaxKind::from_contextual_keyword(token_text, edition)
                        .unwrap_or(SyntaxKind::IDENT),
                    edition,
                )
            } else {
                if was_joint {
                    res.was_joint();
                }
                res.push(kind, edition);
                // Tag the token as joint if it is float with a fractional part
                // we use this jointness to inform the parser about what token split
                // event to emit when we encounter a float literal in a field access
                if kind == SyntaxKind::FLOAT_NUMBER {
                    if !self.text(i).ends_with('.') {
                        res.was_joint();
                    } else {
                        was_joint = false;
                    }
                } else {
                    was_joint = true;
                }
            }
        }
        res
    }

    /// NB: only valid to call with Output from Reparser/TopLevelEntry.
    pub fn intersperse_trivia(
        &self,
        output: &crate::Output,
        sink: &mut dyn FnMut(StrStep<'_>),
    ) -> bool {
        let mut builder = Builder {
            lexed: self,
            pos: 0,
            last_end: 0,
            after_enter: false,
            leading: Vec::new(),
            trailing: Vec::new(),
            state: State::PendingEnter,
            sink,
        };

        for event in output.iter() {
            match event {
                Step::Token { kind, n_input_tokens: n_raw_tokens } => {
                    builder.token(kind, n_raw_tokens)
                }
                Step::FloatSplit { ends_in_dot: has_pseudo_dot } => {
                    builder.float_split(has_pseudo_dot)
                }
                Step::Enter { kind } => builder.enter(kind),
                Step::Exit => builder.exit(),
                Step::Error { msg } => {
                    let pos = match builder.after_enter {
                        true => builder.next_token_start(),
                        false => builder.last_end,
                    };
                    (builder.sink)(StrStep::Error { msg, pos });
                }
            }
        }

        match mem::replace(&mut builder.state, State::Normal) {
            State::PendingExit => {
                builder.eof();
                (builder.sink)(StrStep::Exit);
            }
            State::PendingEnter | State::Normal => unreachable!(),
        }

        // is_eof?
        builder.pos == builder.lexed.len()
    }
}

struct Builder<'a, 'b> {
    lexed: &'a LexedStr<'a>,
    pos: usize,
    last_end: usize,
    after_enter: bool,
    leading: Vec<Trivia<'a>>,
    trailing: Vec<Trivia<'a>>,
    state: State,
    sink: &'b mut dyn FnMut(StrStep<'_>),
}

enum State {
    PendingEnter,
    Normal,
    PendingExit,
}

impl<'a> Builder<'a, '_> {
    fn token(&mut self, kind: SyntaxKind, n_tokens: u8) {
        match mem::replace(&mut self.state, State::Normal) {
            State::PendingEnter => unreachable!(),
            State::PendingExit => (self.sink)(StrStep::Exit),
            State::Normal => (),
        }
        self.do_token(kind, n_tokens as usize);
    }

    fn float_split(&mut self, has_pseudo_dot: bool) {
        match mem::replace(&mut self.state, State::Normal) {
            State::PendingEnter => unreachable!(),
            State::PendingExit => (self.sink)(StrStep::Exit),
            State::Normal => (),
        }
        self.do_float_split(has_pseudo_dot);
    }

    fn enter(&mut self, kind: SyntaxKind) {
        match mem::replace(&mut self.state, State::Normal) {
            State::PendingEnter => {
                (self.sink)(StrStep::Enter { kind });
                // No need to attach trivias to previous node: there is no
                // previous node.
                return;
            }
            State::PendingExit => (self.sink)(StrStep::Exit),
            State::Normal => (),
        }

        (self.sink)(StrStep::Enter { kind });
        self.after_enter = true;
    }

    fn next_token_start(&self) -> usize {
        let next = (self.pos..self.lexed.len()).find(|&it| !self.lexed.kind(it).is_trivia());
        self.lexed.text_start(next.unwrap_or(self.lexed.len()))
    }

    fn exit(&mut self) {
        match mem::replace(&mut self.state, State::PendingExit) {
            State::PendingEnter => unreachable!(),
            State::PendingExit => (self.sink)(StrStep::Exit),
            State::Normal => (),
        }
    }

    fn take_leading(&mut self) {
        self.leading.clear();
        while self.pos < self.lexed.len() && self.lexed.kind(self.pos).is_trivia() {
            self.leading
                .push(Trivia { kind: self.lexed.kind(self.pos), text: self.lexed.text(self.pos) });
            self.pos += 1;
        }
    }

    fn take_trailing(&mut self) {
        self.trailing.clear();
        while self.pos < self.lexed.len() && self.lexed.kind(self.pos).is_trivia() {
            let kind = self.lexed.kind(self.pos);
            self.trailing.push(Trivia { kind, text: self.lexed.text(self.pos) });
            self.pos += 1;
            if kind == SyntaxKind::NEWLINE {
                break;
            }
        }
    }

    fn eof(&mut self) {
        self.take_leading();
        (self.sink)(StrStep::Token {
            kind: SyntaxKind::EOF,
            text: "",
            leading: &self.leading,
            trailing: &[],
        });
    }

    fn do_token(&mut self, kind: SyntaxKind, n_tokens: usize) {
        self.take_leading();
        let text = self.lexed.range_text(self.pos..self.pos + n_tokens);
        self.pos += n_tokens;
        self.last_end = self.lexed.text_start(self.pos);
        self.after_enter = false;
        self.take_trailing();
        (self.sink)(StrStep::Token {
            kind,
            text,
            leading: &self.leading,
            trailing: &self.trailing,
        });
    }

    fn do_float_split(&mut self, has_pseudo_dot: bool) {
        self.take_leading();
        let start = self.pos;
        let text = self.lexed.range_text(self.pos..self.pos + 1);
        self.pos += 1;
        self.last_end = self.lexed.text_start(self.pos);
        self.after_enter = false;
        self.take_trailing();

        match text.split_once('.') {
            Some((left, right)) => {
                assert!(!left.is_empty());
                (self.sink)(StrStep::Enter { kind: SyntaxKind::NAME_REF });
                (self.sink)(StrStep::Token {
                    kind: SyntaxKind::INT_NUMBER,
                    text: left,
                    leading: &self.leading,
                    trailing: &[],
                });
                (self.sink)(StrStep::Exit);

                // here we move the exit up, the original exit has been deleted in process
                (self.sink)(StrStep::Exit);

                if has_pseudo_dot {
                    assert!(right.is_empty(), "{left}.{right}");
                    (self.sink)(StrStep::Token {
                        kind: SyntaxKind::DOT,
                        text: ".",
                        leading: &[],
                        trailing: &self.trailing,
                    });
                    self.state = State::Normal;
                } else {
                    assert!(!right.is_empty(), "{left}.{right}");
                    (self.sink)(StrStep::Token {
                        kind: SyntaxKind::DOT,
                        text: ".",
                        leading: &[],
                        trailing: &[],
                    });
                    (self.sink)(StrStep::Enter { kind: SyntaxKind::NAME_REF });
                    (self.sink)(StrStep::Token {
                        kind: SyntaxKind::INT_NUMBER,
                        text: right,
                        leading: &[],
                        trailing: &self.trailing,
                    });
                    (self.sink)(StrStep::Exit);

                    // the parser creates an unbalanced start node, we are required to close it here
                    self.state = State::PendingExit;
                }
            }
            None => {
                (self.sink)(StrStep::Error {
                    msg: "illegal float literal",
                    pos: self.lexed.text_start(start),
                });
                (self.sink)(StrStep::Enter { kind: SyntaxKind::ERROR });
                (self.sink)(StrStep::Token {
                    kind: SyntaxKind::FLOAT_NUMBER,
                    text,
                    leading: &self.leading,
                    trailing: &self.trailing,
                });
                (self.sink)(StrStep::Exit);

                // move up
                (self.sink)(StrStep::Exit);

                self.state = if has_pseudo_dot { State::Normal } else { State::PendingExit };
            }
        }
    }
}
