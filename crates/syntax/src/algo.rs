//! Collection of assorted algorithms for syntax trees.

use std::iter::successors;

use itertools::Itertools;

use crate::{
    AstNode, Direction, NodeOrToken, SyntaxKind, SyntaxNode, SyntaxToken, TextRange, TextSize,
    ast::{self, AstToken, make::tokens::ELASTIC_MARKER},
    syntax_editor::{Element, SyntaxEditor},
};

/// Returns ancestors of the node at the offset, sorted by length. This should
/// do the right thing at an edge, e.g. when searching for expressions at `{
/// $0foo }` we will get the name reference instead of the whole block, which
/// we would get if we just did `find_token_at_offset(...).flat_map(|t|
/// t.parent().ancestors())`.
pub fn ancestors_at_offset(
    node: &SyntaxNode,
    offset: TextSize,
) -> impl Iterator<Item = SyntaxNode> {
    node.token_at_offset(offset)
        .filter_map(|token| token.owning_node())
        .map(|node| node.ancestors())
        .kmerge_by(|node1, node2| node1.text_range().len() < node2.text_range().len())
}

/// Finds a node of specific Ast type at offset. Note that this is slightly
/// imprecise: if the cursor is strictly between two nodes of the desired type,
/// as in
///
/// ```ignore
/// struct Foo {}|struct Bar;
/// ```
///
/// then the shorter node will be silently preferred.
pub fn find_node_at_offset<N: AstNode>(syntax: &SyntaxNode, offset: TextSize) -> Option<N> {
    ancestors_at_offset(syntax, offset).find_map(N::cast)
}

pub fn find_node_at_range<N: AstNode>(syntax: &SyntaxNode, range: TextRange) -> Option<N> {
    syntax.covering_element(range).ancestors().find_map(N::cast)
}

pub(crate) fn strip_blank_edges(node: &SyntaxNode, elastic: bool) -> SyntaxNode {
    let (editor, node) = SyntaxEditor::new(node.clone());
    let blank =
        |it: &SyntaxToken| matches!(it.kind(), SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE);
    let marker =
        |stripped: usize, len: usize| (elastic && stripped == len).then_some(ELASTIC_MARKER);
    if let Some(first) = node.first_non_trivia_token() {
        let stripped = first.leading_trivia().take_while(blank).count();
        let len = first.leading_trivia().len();
        editor.splice_leading_trivia(&first, ..stripped, marker(stripped, len));
    }
    let mut last = node.last_non_trivia_token();
    if let Some(eof) =
        last.take_if(|it| it.kind() == SyntaxKind::EOF && node.kind() != SyntaxKind::SOURCE_FILE)
    {
        editor.delete(&eof);
        last = eof.prev_non_trivia_token();
    }
    if let Some(last) = last {
        let stripped = last.trailing_trivia().rev().take_while(blank).count();
        let len = last.trailing_trivia().len();
        editor.splice_trailing_trivia(&last, len - stripped.., marker(stripped, len));
    }
    editor.finish().new_root().clone()
}

pub struct BlankRun {
    pub range: TextRange,
    pub before: Option<SyntaxToken>,
    pub after: Option<SyntaxToken>,
    pub newlines: usize,
}

pub fn blank_run(token: &SyntaxToken) -> BlankRun {
    let blank =
        |it: &SyntaxToken| matches!(it.kind(), SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE);
    let mut first = token.clone();
    while let Some(it) = first.prev_token().filter(blank) {
        first = it;
    }
    let mut last = token.clone();
    while let Some(it) = last.next_token().filter(blank) {
        last = it;
    }
    let range = TextRange::new(first.text_range().start(), last.text_range().end());
    let newlines = successors(Some(first.clone()), |it| it.next_token())
        .take_while(|it| it.text_range().end() <= range.end())
        .filter(|it| it.kind() == SyntaxKind::NEWLINE)
        .count();
    BlankRun { range, before: first.prev_token(), after: last.next_token(), newlines }
}

pub fn adjacent_comment(token: &SyntaxToken, direction: Direction) -> Option<SyntaxToken> {
    let step = |it: &SyntaxToken| match direction {
        Direction::Next => it.next_token(),
        Direction::Prev => it.prev_token(),
    };
    let mut newlines = 0;
    successors(step(token), step)
        .take_while(|it| {
            newlines += usize::from(it.kind() == SyntaxKind::NEWLINE);
            newlines <= 1
        })
        .find(|it| !matches!(it.kind(), SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE))
        .filter(|it| ast::AnyComment::can_cast(it.kind()))
}

pub fn skip_whitespace_token(mut token: SyntaxToken, direction: Direction) -> Option<SyntaxToken> {
    while matches!(token.kind(), SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE) {
        token = match direction {
            Direction::Next => token.next_token()?,
            Direction::Prev => token.prev_token()?,
        }
    }
    Some(token)
}

pub fn least_common_ancestor(u: &SyntaxNode, v: &SyntaxNode) -> Option<SyntaxNode> {
    if u == v {
        return Some(u.clone());
    }

    let u_depth = u.ancestors().count();
    let v_depth = v.ancestors().count();
    let keep = u_depth.min(v_depth);

    let u_candidates = u.ancestors().skip(u_depth - keep);
    let v_candidates = v.ancestors().skip(v_depth - keep);
    let (res, _) = u_candidates.zip(v_candidates).find(|(x, y)| x == y)?;
    Some(res)
}

pub fn least_common_ancestor_element(u: impl Element, v: impl Element) -> Option<SyntaxNode> {
    let u = u.syntax_element();
    let v = v.syntax_element();
    if u == v {
        return match u {
            NodeOrToken::Node(node) => Some(node),
            NodeOrToken::Token(token) => token.parent(),
        };
    }

    let u_depth = u.ancestors().count();
    let v_depth = v.ancestors().count();
    let keep = u_depth.min(v_depth);

    let u_candidates = u.ancestors().skip(u_depth - keep);
    let v_candidates = v.ancestors().skip(v_depth - keep);
    let (res, _) = u_candidates.zip(v_candidates).find(|(x, y)| x == y)?;
    Some(res)
}

pub fn neighbor<T: AstNode>(me: &T, direction: Direction) -> Option<T> {
    me.syntax().siblings(direction).skip(1).find_map(T::cast)
}

pub fn has_errors(node: &SyntaxNode) -> bool {
    node.children().any(|it| it.kind() == SyntaxKind::ERROR)
}
