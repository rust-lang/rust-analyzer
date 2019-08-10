use ra_syntax::{
    SyntaxElement, SyntaxKind::*, NodeOrToken,
    SyntaxNode, SyntaxToken, WalkEvent,
};

pub(crate) fn walk(node: &SyntaxNode) -> impl Iterator<Item = SyntaxElement> {
    node.preorder_with_tokens().filter_map(|event| match event {
        WalkEvent::Enter(element) => Some(element),
        WalkEvent::Leave(_) => None,
    })
}
pub(crate) fn walk_non_whitespace(node: &SyntaxNode) -> impl Iterator<Item = SyntaxElement> {
    node.preorder_with_tokens().filter_map(|event| match event {
        WalkEvent::Enter(element) => Some(element).filter(|it| it.kind() != WHITESPACE),
        WalkEvent::Leave(_) => None,
    })
}
pub(crate) fn walk_tokens(node: &SyntaxNode) -> impl Iterator<Item = SyntaxToken> {
    walk(node).filter_map(|element| match element {
        NodeOrToken::Token(token) => Some(token),
        _ => None,
    })
}

pub(crate) fn has_newline(node: &SyntaxNode) -> bool {
    walk_tokens(node).any(|it| it.text().contains('\n'))
}
