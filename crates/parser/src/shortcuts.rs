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

use std::{collections::VecDeque, fmt, mem};

use crate::{Edition, LexedStr, Step, SyntaxKind};

#[derive(Clone, Copy)]
pub struct Trivia<'a> {
    pub kind: SyntaxKind,
    pub text: &'a str,
    pub error: bool,
}

impl fmt::Debug for Trivia<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.error {
            write!(f, "!")?;
        }
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
            pending: Vec::new(),
            split: VecDeque::new(),
            split_offset: 0,
            flatten_depth: 0,
            region_start: None,
            depth: 0,
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
                    let text_pos = match builder.split.is_empty() {
                        true => builder.lexed.text_start(builder.pos),
                        false => builder.split_offset,
                    };
                    (builder.sink)(StrStep::Error { msg, pos: text_pos });
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
    pending: Vec<Trivia<'a>>,
    split: VecDeque<Trivia<'a>>,
    split_offset: usize,
    flatten_depth: usize,
    region_start: Option<usize>,
    depth: usize,
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
        if self.flatten_depth > 0 {
            self.skip_token(n_tokens as usize);
            return;
        }
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
        if self.flatten_depth > 0 {
            self.flatten_depth += 1;
            return;
        }
        if kind == SyntaxKind::ERROR && self.depth > 0 {
            self.flatten_depth = 1;
            return;
        }
        self.depth += 1;
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
    }

    fn exit(&mut self) {
        if self.flatten_depth > 0 {
            self.flatten_depth -= 1;
            if self.flatten_depth == 0 {
                self.finish_skipped_region();
            }
            return;
        }
        self.depth -= 1;
        match mem::replace(&mut self.state, State::PendingExit) {
            State::PendingEnter => unreachable!(),
            State::PendingExit => (self.sink)(StrStep::Exit),
            State::Normal => (),
        }
    }

    fn fill_split(&mut self) {
        if !self.split.is_empty() {
            return;
        }
        if self.pos >= self.lexed.len() || !self.lexed.kind(self.pos).is_trivia() {
            return;
        }
        self.split_offset = self.lexed.text_start(self.pos);
        let kind = self.lexed.kind(self.pos);
        let text = self.lexed.text(self.pos);
        self.pos += 1;
        if kind != SyntaxKind::WHITESPACE {
            self.split.push_back(Trivia { kind, text, error: false });
            return;
        }
        let mut rest = text;
        while let Some(idx) = rest.find('\n') {
            let (line, tail) = rest.split_at(idx + 1);
            let (ws, newline) = match line.strip_suffix("\r\n") {
                Some(ws) => (ws, &line[line.len() - 2..]),
                None => (&line[..idx], &line[idx..]),
            };
            if !ws.is_empty() {
                self.split.push_back(Trivia {
                    kind: SyntaxKind::WHITESPACE,
                    text: ws,
                    error: false,
                });
            }
            self.split.push_back(Trivia { kind: SyntaxKind::NEWLINE, text: newline, error: false });
            rest = tail;
        }
        if !rest.is_empty() {
            self.split.push_back(Trivia { kind: SyntaxKind::WHITESPACE, text: rest, error: false });
        }
    }

    fn take_leading(&mut self) -> Vec<Trivia<'a>> {
        let mut res = Vec::new();
        loop {
            self.fill_split();
            match self.split.pop_front() {
                Some(trivia) => {
                    self.split_offset += trivia.text.len();
                    res.push(trivia)
                }
                None => break,
            }
        }
        res
    }

    fn take_trailing(&mut self) -> Vec<Trivia<'a>> {
        let mut res = Vec::new();
        loop {
            self.fill_split();
            let Some(trivia) = self.split.pop_front() else { break };
            self.split_offset += trivia.text.len();
            let newline = trivia.kind == SyntaxKind::NEWLINE;
            res.push(trivia);
            if newline {
                break;
            }
        }
        res
    }

    fn skip_token(&mut self, n_tokens: usize) {
        if self.region_start.is_none() {
            let leading = self.take_leading();
            self.pending.extend(leading);
            self.region_start = Some(self.pos);
        } else {
            let leftover: Vec<_> = self.split.drain(..).collect();
            self.split_offset += leftover.iter().map(|it| it.text.len()).sum::<usize>();
            self.pending.extend(leftover);
            while self.pos < self.lexed.len() && self.lexed.kind(self.pos).is_trivia() {
                self.pos += 1;
            }
        }
        self.pos += n_tokens;
    }

    fn finish_skipped_region(&mut self) {
        let Some(start) = self.region_start.take() else { return };
        for pos in start..self.pos {
            let kind = self.lexed.kind(pos);
            self.pending.push(Trivia {
                kind,
                text: self.lexed.text(pos),
                error: !kind.is_trivia(),
            });
        }
    }

    fn take_pending_leading(&mut self) -> Vec<Trivia<'a>> {
        let mut leading = mem::take(&mut self.pending);
        leading.extend(self.take_leading());
        leading
    }

    fn eof(&mut self) {
        let leading = self.take_pending_leading();
        (self.sink)(StrStep::Token {
            kind: SyntaxKind::EOF,
            text: "",
            leading: &leading,
            trailing: &[],
        });
    }

    fn do_token(&mut self, kind: SyntaxKind, n_tokens: usize) {
        let leading = self.take_pending_leading();
        let text = self.lexed.range_text(self.pos..self.pos + n_tokens);
        self.pos += n_tokens;
        let trailing = self.take_trailing();
        (self.sink)(StrStep::Token { kind, text, leading: &leading, trailing: &trailing });
    }

    fn do_float_split(&mut self, has_pseudo_dot: bool) {
        let leading = self.take_pending_leading();
        let start = self.pos;
        let text = self.lexed.range_text(self.pos..self.pos + 1);
        self.pos += 1;
        let trailing = self.take_trailing();

        match text.split_once('.') {
            Some((left, right)) => {
                assert!(!left.is_empty());
                (self.sink)(StrStep::Enter { kind: SyntaxKind::NAME_REF });
                (self.sink)(StrStep::Token {
                    kind: SyntaxKind::INT_NUMBER,
                    text: left,
                    leading: &leading,
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
                        trailing: &trailing,
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
                        trailing: &trailing,
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
                self.pending.extend(leading);
                self.pending.push(Trivia { kind: SyntaxKind::ERROR, text, error: true });
                self.pending.extend(trailing);

                // move up
                (self.sink)(StrStep::Exit);

                self.state = if has_pseudo_dot { State::Normal } else { State::PendingExit };
            }
        }
    }
}
