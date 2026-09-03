//! This module defines Concrete Syntax Tree (CST), used by rust-analyzer.
//!
//! The CST includes comments and whitespace, provides a single node type,
//! `SyntaxNode`, and a basic traversal API (parent, children, siblings).
//!
//! The *real* implementation is in the (language-agnostic) `rowan` crate, this
//! module just wraps its API.

use std::iter;

use rowan::{GreenNodeBuilder, Language};

use crate::{Parse, SyntaxError, SyntaxKind, TextRange, TextSize};

pub(crate) use rowan::{GreenNode, GreenToken, NodeOrToken};

const ERROR_BIT: u16 = 1 << 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RustLanguage {}
impl Language for RustLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> SyntaxKind {
        SyntaxKind::from(raw.0 & !ERROR_BIT)
    }

    fn kind_to_raw(kind: SyntaxKind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind.into())
    }
}

pub fn is_error(token: &SyntaxToken) -> bool {
    token.green().kind().0 & ERROR_BIT != 0
}

pub type SyntaxNode = rowan::SyntaxNode<RustLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<RustLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<RustLanguage>;
pub type SyntaxNodeChildren = rowan::SyntaxNodeChildren<RustLanguage>;
pub type SyntaxElementChildren = rowan::SyntaxElementChildren<RustLanguage>;
pub type PreorderWithTokens = rowan::api::PreorderWithTokens<RustLanguage>;

pub(crate) fn make_token(kind: SyntaxKind, text: &str) -> SyntaxToken {
    make_token_with_trivia(kind, text, "", "")
}

pub(crate) fn trivia_pieces(text: &str) -> Vec<(rowan::SyntaxKind, &str)> {
    let whitespace = RustLanguage::kind_to_raw(SyntaxKind::WHITESPACE);
    let newline = RustLanguage::kind_to_raw(SyntaxKind::NEWLINE);
    let mut res = Vec::new();
    let mut rest = text;
    while let Some(idx) = rest.find('\n') {
        let (line, tail) = rest.split_at(idx + 1);
        let (ws, nl) = match line.strip_suffix("\r\n") {
            Some(ws) => (ws, &line[line.len() - 2..]),
            None => (&line[..idx], &line[idx..]),
        };
        if !ws.is_empty() {
            res.push((whitespace, ws));
        }
        res.push((newline, nl));
        rest = tail;
    }
    if !rest.is_empty() {
        res.push((whitespace, rest));
    }
    res
}

pub(crate) type TriviaPiece = (rowan::SyntaxKind, String);

pub(crate) fn trivia_of(token: &SyntaxToken, leading: bool) -> Vec<TriviaPiece> {
    let piece = |it: SyntaxToken| (it.green().kind(), it.text().to_owned());
    match leading {
        true => token.leading_trivia().map(piece).collect(),
        false => token.trailing_trivia().map(piece).collect(),
    }
}

pub(crate) fn make_token_with_trivia(
    kind: SyntaxKind,
    text: &str,
    leading: &str,
    trailing: &str,
) -> SyntaxToken {
    let owned = |text: &str| -> Vec<TriviaPiece> {
        trivia_pieces(text).into_iter().map(|(kind, it)| (kind, it.to_owned())).collect()
    };
    make_token_with_raw_trivia(kind, text, &owned(leading), &owned(trailing))
}

pub(crate) fn make_token_with_raw_trivia(
    kind: SyntaxKind,
    text: &str,
    leading: &[TriviaPiece],
    trailing: &[TriviaPiece],
) -> SyntaxToken {
    let mut builder = GreenNodeBuilder::new();
    builder.start_node(RustLanguage::kind_to_raw(SyntaxKind::SOURCE_FILE));
    builder.token_with_trivia(
        RustLanguage::kind_to_raw(kind),
        text,
        leading.iter().map(|(kind, text)| (*kind, text.as_str())),
        trailing.iter().map(|(kind, text)| (*kind, text.as_str())),
    );
    builder.finish_node();
    SyntaxNode::new_root(builder.finish()).first_token().unwrap()
}

pub(crate) fn map_indent(node: &SyntaxNode, f: &dyn Fn(&str) -> String) -> SyntaxNode {
    let whitespace = RustLanguage::kind_to_raw(SyntaxKind::WHITESPACE);
    let mut after_newline = false;

    let mut builder = GreenNodeBuilder::new();
    for event in node.preorder_with_tokens() {
        match event {
            rowan::WalkEvent::Enter(rowan::NodeOrToken::Node(node)) => {
                builder.start_node(RustLanguage::kind_to_raw(node.kind()));
            }
            rowan::WalkEvent::Leave(rowan::NodeOrToken::Node(_)) => builder.finish_node(),
            rowan::WalkEvent::Enter(rowan::NodeOrToken::Token(token)) => {
                let mut leading = Vec::new();
                for piece in token.leading_trivia().chain(iter::once(token.clone())) {
                    match piece.kind() {
                        SyntaxKind::NEWLINE => after_newline = true,
                        SyntaxKind::WHITESPACE if after_newline => {
                            after_newline = false;
                            leading.push((whitespace, f(piece.text())));
                            continue;
                        }
                        _ if after_newline => {
                            after_newline = false;
                            let indent = f("");
                            if !indent.is_empty() {
                                leading.push((whitespace, indent));
                            }
                        }
                        _ => (),
                    }
                    if piece != token {
                        leading.push((piece.green().kind(), piece.text().to_owned()));
                    }
                }
                let mut trailing = Vec::new();
                for piece in token.trailing_trivia() {
                    if piece.kind() == SyntaxKind::NEWLINE {
                        after_newline = true;
                    }
                    trailing.push((piece.green().kind(), piece.text().to_owned()));
                }
                builder.token_with_trivia(
                    token.green().kind(),
                    token.text(),
                    leading.iter().map(|(kind, text)| (*kind, text.as_str())),
                    trailing.iter().map(|(kind, text)| (*kind, text.as_str())),
                );
            }
            rowan::WalkEvent::Leave(rowan::NodeOrToken::Token(_)) => (),
        }
    }
    SyntaxNode::new_root(builder.finish())
}

pub fn token_span(node: &SyntaxNode) -> TextRange {
    match (node.first_token(), node.last_token()) {
        (Some(first), Some(last)) => {
            TextRange::new(first.text_range().start(), last.text_range().end())
        }
        _ => TextRange::empty(node.text_range().start()),
    }
}

pub fn token_text_with_comments(node: &SyntaxNode) -> String {
    let full = node.text_range();
    let text = node.text().to_string();
    let leading = node.first_token().map_or(0, |token| {
        let pieces: Vec<_> = token.leading_trivia().collect();
        let start = pieces.iter().position(|it| it.kind() == SyntaxKind::COMMENT);
        match start {
            Some(index) => usize::from(pieces[index].text_range().start() - full.start()),
            None => usize::from(token.text_range().start() - full.start()),
        }
    });
    let end = node.last_token().map_or(text.len(), |token| {
        text.len() - usize::from(full.end() - token.text_range().end())
    });
    text[leading..end].to_owned()
}

pub fn token_text(node: &SyntaxNode) -> String {
    let full = node.text_range();
    let text = node.text().to_string();
    let start = node
        .first_token()
        .map_or(0, |token| usize::from(token.text_range().start() - full.start()));
    let end = node.last_token().map_or(text.len(), |token| {
        text.len() - usize::from(full.end() - token.text_range().end())
    });
    text[start..end].to_owned()
}

pub(crate) fn fold_whitespace(node: &SyntaxNode) -> SyntaxNode {
    let mut pending: Vec<(rowan::SyntaxKind, String)> = Vec::new();
    let mut builder = GreenNodeBuilder::new();
    for event in node.preorder_with_tokens() {
        match event {
            rowan::WalkEvent::Enter(rowan::NodeOrToken::Node(node)) => {
                builder.start_node(RustLanguage::kind_to_raw(node.kind()));
            }
            rowan::WalkEvent::Leave(rowan::NodeOrToken::Node(_)) => {
                for (kind, text) in pending.drain(..) {
                    builder.token(kind, &text);
                }
                builder.finish_node();
            }
            rowan::WalkEvent::Enter(rowan::NodeOrToken::Token(token)) => {
                if token.kind() == SyntaxKind::WHITESPACE && token.leading_trivia().len() == 0 {
                    pending.extend(
                        trivia_pieces(token.text())
                            .into_iter()
                            .map(|(kind, text)| (kind, text.to_owned())),
                    );
                    continue;
                }
                let mut leading: Vec<_> = std::mem::take(&mut pending);
                leading.extend(
                    token.leading_trivia().map(|it| (it.green().kind(), it.text().to_owned())),
                );
                let trailing: Vec<_> = token
                    .trailing_trivia()
                    .map(|it| (it.green().kind(), it.text().to_owned()))
                    .collect();
                builder.token_with_trivia(
                    token.green().kind(),
                    token.text(),
                    leading.iter().map(|(kind, text)| (*kind, text.as_str())),
                    trailing.iter().map(|(kind, text)| (*kind, text.as_str())),
                );
            }
            rowan::WalkEvent::Leave(rowan::NodeOrToken::Token(_)) => (),
        }
    }
    SyntaxNode::new_root(builder.finish())
}

pub fn strip_trivia(node: &SyntaxNode) -> SyntaxNode {
    let is_formatting = |kind: SyntaxKind| {
        matches!(kind, SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE | SyntaxKind::COMMENT)
    };

    let mut builder = GreenNodeBuilder::new();
    for event in node.preorder_with_tokens() {
        match event {
            rowan::WalkEvent::Enter(rowan::NodeOrToken::Node(node)) => {
                builder.start_node(RustLanguage::kind_to_raw(node.kind()));
            }
            rowan::WalkEvent::Leave(rowan::NodeOrToken::Node(_)) => builder.finish_node(),
            rowan::WalkEvent::Enter(rowan::NodeOrToken::Token(token)) => {
                if is_formatting(token.kind()) {
                    continue;
                }
                let kept = |token: SyntaxToken| {
                    (!is_formatting(token.kind()))
                        .then(|| (token.green().kind(), token.text().to_owned()))
                };
                let leading: Vec<_> = token.leading_trivia().filter_map(kept).collect();
                let trailing: Vec<_> = token.trailing_trivia().filter_map(kept).collect();
                builder.token_with_trivia(
                    RustLanguage::kind_to_raw(token.kind()),
                    token.text(),
                    leading.iter().map(|(kind, text)| (*kind, text.as_str())),
                    trailing.iter().map(|(kind, text)| (*kind, text.as_str())),
                );
            }
            rowan::WalkEvent::Leave(rowan::NodeOrToken::Token(_)) => (),
        }
    }
    SyntaxNode::new_root(builder.finish())
}

pub fn clone_subtree_with_outer_trivia(
    node: &SyntaxNode,
    leading: Option<&str>,
    trailing: Option<&str>,
) -> SyntaxNode {
    let owned = |text: Option<&str>| -> Option<Vec<TriviaPiece>> {
        text.map(|text| {
            trivia_pieces(text).into_iter().map(|(kind, it)| (kind, it.to_owned())).collect()
        })
    };
    let (leading, trailing) = (owned(leading), owned(trailing));
    clone_subtree_with_raw_outer_trivia(node, leading.as_deref(), trailing.as_deref())
}

pub(crate) fn clone_subtree_with_raw_outer_trivia(
    node: &SyntaxNode,
    leading: Option<&[TriviaPiece]>,
    trailing: Option<&[TriviaPiece]>,
) -> SyntaxNode {
    let first = node.first_token();
    let last = node.last_token();

    let mut builder = GreenNodeBuilder::new();
    for event in node.preorder_with_tokens() {
        match event {
            rowan::WalkEvent::Enter(rowan::NodeOrToken::Node(node)) => {
                builder.start_node(RustLanguage::kind_to_raw(node.kind()));
            }
            rowan::WalkEvent::Leave(rowan::NodeOrToken::Node(_)) => builder.finish_node(),
            rowan::WalkEvent::Enter(rowan::NodeOrToken::Token(token)) => {
                let trivia = |token: SyntaxToken| (token.green().kind(), token.text().to_owned());
                let is_first = Some(&token) == first.as_ref();
                let is_last = Some(&token) == last.as_ref();

                let mut lead: Vec<_> = if is_first && leading.is_some() {
                    Vec::new()
                } else {
                    token.leading_trivia().map(trivia).collect()
                };
                if is_first && let Some(pieces) = leading.filter(|it| !it.is_empty()) {
                    lead.extend(pieces.iter().cloned());
                }

                let mut trail: Vec<_> = if is_last && trailing.is_some() {
                    Vec::new()
                } else {
                    token.trailing_trivia().map(trivia).collect()
                };
                if is_last && let Some(pieces) = trailing.filter(|it| !it.is_empty()) {
                    trail.extend(pieces.iter().cloned());
                }

                builder.token_with_trivia(
                    RustLanguage::kind_to_raw(token.kind()),
                    token.text(),
                    lead.iter().map(|(kind, text)| (*kind, text.as_str())),
                    trail.iter().map(|(kind, text)| (*kind, text.as_str())),
                );
            }
            rowan::WalkEvent::Leave(rowan::NodeOrToken::Token(_)) => (),
        }
    }
    SyntaxNode::new_root(builder.finish())
}

#[derive(Default)]
pub struct SyntaxTreeBuilder {
    errors: Vec<SyntaxError>,
    inner: GreenNodeBuilder<'static>,
}

impl SyntaxTreeBuilder {
    pub(crate) fn finish_raw(self) -> (GreenNode, Vec<SyntaxError>) {
        let green = self.inner.finish();
        (green, self.errors)
    }

    pub fn finish(self) -> Parse<SyntaxNode> {
        let (green, errors) = self.finish_raw();
        // Disable block validation, see https://github.com/rust-lang/rust-analyzer/pull/10357
        #[allow(clippy::overly_complex_bool_expr)]
        if cfg!(debug_assertions) && false {
            let node = SyntaxNode::new_root(green.clone());
            crate::validation::validate_block_structure(&node);
        }
        Parse::new(green, errors)
    }

    pub fn token(&mut self, kind: SyntaxKind, text: &str) {
        debug_assert!(!kind.is_trivia(), "trivia must be attached to a token, not built as one");
        let kind = RustLanguage::kind_to_raw(kind);
        self.inner.token(kind, text);
    }

    pub fn token_with_trivia<'a>(
        &mut self,
        kind: SyntaxKind,
        text: &str,
        leading: &[parser::Trivia<'a>],
        trailing: &[parser::Trivia<'a>],
    ) {
        let raw = |it: &parser::Trivia<'a>| {
            let kind = RustLanguage::kind_to_raw(it.kind).0;
            (rowan::SyntaxKind(if it.error { kind | ERROR_BIT } else { kind }), it.text)
        };
        let kind = RustLanguage::kind_to_raw(kind);
        self.inner.token_with_trivia(kind, text, leading.iter().map(raw), trailing.iter().map(raw));
    }

    pub fn start_node(&mut self, kind: SyntaxKind) {
        let kind = RustLanguage::kind_to_raw(kind);
        self.inner.start_node(kind);
    }

    pub fn finish_node(&mut self) {
        self.inner.finish_node();
    }

    pub fn error(&mut self, error: String, text_pos: TextSize) {
        self.errors.push(SyntaxError::new_at_offset(error, text_pos));
    }
}
