use ra_syntax::{NodeOrToken, SyntaxElement, SyntaxKind::*, SyntaxNode, SyntaxToken, WalkEvent};
use std::iter::successors;

// pub(crate) fn walk(node: &SyntaxNode) -> impl Iterator<Item = SyntaxElement> {
//     node.preorder_with_tokens().filter_map(|event| match event {
//         WalkEvent::Enter(element) => Some(element),
//         WalkEvent::Leave(_) => None,
//     })
// }
// pub(crate) fn walk_non_whitespace(node: &SyntaxNode) -> impl Iterator<Item = SyntaxElement> {
//     node.preorder_with_tokens().filter_map(|event| match event {
//         WalkEvent::Enter(element) => Some(element).filter(|it| it.kind() != WHITESPACE),
//         WalkEvent::Leave(_) => None,
//     })
// }
// pub(crate) fn walk_tokens(node: &SyntaxNode) -> impl Iterator<Item = SyntaxToken> {
//     walk(node).filter_map(|element| match element {
//         NodeOrToken::Token(token) => Some(token),
//         _ => None,
//     })
// }
// pub(crate) fn walk_nodes(node: &SyntaxNode) -> impl Iterator<Item = SyntaxNode> {
//     walk(node).filter_map(|element| match element {
//         NodeOrToken::Node(node) => Some(node),
//         _ => None,
//     })
// }
// pub(crate) fn has_newline(node: &SyntaxNode) -> bool {
//     walk_tokens(node).inspect(|t| println!("newline {:?}", t)).any(|it| it.text().contains('\n'))
// }
pub(crate) fn prev_non_whitespace_sibling(element: &SyntaxElement) -> Option<SyntaxElement> {
    successors(element.prev_sibling_or_token(), |it| it.prev_sibling_or_token())
        .inspect(|t| println!("prev {:?}", t))
        .find(|it| it.kind() != WHITESPACE)
}

pub(crate) fn next_non_whitespace_sibling(element: &SyntaxElement) -> Option<SyntaxElement> {
    successors(element.next_sibling_or_token(), |it| it.next_sibling_or_token())
        .inspect(|t| println!("{:?}", t))
        .find(|it| it.kind() != WHITESPACE)
}
