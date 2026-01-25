use syntax::{
    AstNode, NodeOrToken, SyntaxKind, SyntaxToken,
    ast::{self, make},
};

use crate::{
    AssistId,
    assist_context::{AssistContext, Assists},
};

// Assist: merge_attributes
//
// Merge some attributes.
//
// ```
// $0#[cfg(feature = "a")]
// #[cfg(feature = "b")]
// #[cfg(feature = "c")]$0
// struct Foo;
// ```
// ->
// ```
// #[cfg(all(feature = "a", feature = "b", feature = "c"))]
// struct Foo;
// ```
pub(crate) fn merge_attributes(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let attr = ctx.find_node_at_offset::<ast::Attr>()?;
    let name = attr.as_simple_call()?.0;
    let extra_call = mergeable_attr(&name)?;
    let attrs = attr
        .syntax()
        .siblings(syntax::Direction::Next)
        .take_while(|node| node.text_range().start() < ctx.selection_trimmed().end())
        .filter_map(ast::Attr::cast)
        .collect::<Vec<_>>();
    let is_outer = attr.excl_token().is_none();

    if attrs.len() < 2
        || attrs.iter().any(|attr| attr.as_simple_call().is_none_or(|it| it.0 != name))
    {
        return None;
    }
    let attr_tts =
        attrs.iter().map(|attr| attr.meta()?.token_tree()).collect::<Option<Vec<_>>>()?;

    let target = attr.syntax().text_range().cover(attrs.last()?.syntax().text_range());
    acc.add(AssistId::refactor_rewrite("merge_attributes"), "Merge attributes", target, |builder| {
        let name_path = make::ext::ident_path(&name);
        let tts = attr_tts.iter().flat_map(|tt| comma_sep().chain(extract_tt_tts(tt))).skip(2);
        let mut tt = make::token_tree(SyntaxKind::L_PAREN, tts);
        if !extra_call.is_empty() {
            tt = make::token_tree(
                SyntaxKind::L_PAREN,
                [NodeOrToken::Token(make::tokens::ident(extra_call)), NodeOrToken::Node(tt)],
            );
        }
        let meta = make::meta_token_tree(name_path, tt);
        let attr = if is_outer { make::attr_outer(meta) } else { make::attr_inner(meta) };
        builder.replace(target, attr.to_string());
    })
}

fn extract_tt_tts(
    tt: &ast::TokenTree,
) -> impl Iterator<Item = NodeOrToken<ast::TokenTree, SyntaxToken>> {
    tt.token_trees_and_tokens().skip(1).take_while(|it| {
        it.as_token().is_none_or(|token| {
            !matches!(token.kind(), SyntaxKind::R_PAREN | SyntaxKind::R_CURLY | SyntaxKind::R_BRACK)
        })
    })
}

fn comma_sep() -> impl Iterator<Item = NodeOrToken<ast::TokenTree, SyntaxToken>> {
    [make::token(SyntaxKind::COMMA), make::tokens::single_space()]
        .into_iter()
        .map(NodeOrToken::Token)
}

fn mergeable_attr(name: &str) -> Option<&'static str> {
    match name {
        "cfg" => Some("all"),
        "derive" | "feature" | "allow" | "expect" | "deny" | "forbid" | "warn" => Some(""),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn test_merge_cfg_attributes() {
        check_assist(
            merge_attributes,
            r#"
$0#[cfg(feature = "a")]
#[cfg(feature = "b")]$0
struct Foo;
            "#,
            r#"
#[cfg(all(feature = "a", feature = "b"))]
struct Foo;
            "#,
        );

        check_assist(
            merge_attributes,
            r#"
$0#[cfg(feature = "a")]
#[cfg(feature = "b")]
#[cfg(feature = "c")]$0
struct Foo;
            "#,
            r#"
#[cfg(all(feature = "a", feature = "b", feature = "c"))]
struct Foo;
            "#,
        );

        check_assist(
            merge_attributes,
            r#"
$0#[cfg(feature = "a")]
#[cfg(feature = "b")]
#[cfg(any(feature = "c", feature = "d"))]$0
struct Foo;
            "#,
            r#"
#[cfg(all(feature = "a", feature = "b", any(feature = "c", feature = "d")))]
struct Foo;
            "#,
        );
    }

    #[test]
    fn test_merge_derive_attributes() {
        check_assist(
            merge_attributes,
            r#"
$0#[derive(PartialEq)]
#[derive(Clone)]$0
struct Foo;
            "#,
            r#"
#[derive(PartialEq, Clone)]
struct Foo;
            "#,
        );
    }

    #[test]
    fn test_merge_lint_attributes() {
        check_assist(
            merge_attributes,
            r#"
$0#[allow(unused)]
#[allow(deprecated)]$0
struct Foo;
            "#,
            r#"
#[allow(unused, deprecated)]
struct Foo;
            "#,
        );

        check_assist(
            merge_attributes,
            r#"
$0#[deny(unused)]
#[deny(deprecated)]$0
struct Foo;
            "#,
            r#"
#[deny(unused, deprecated)]
struct Foo;
            "#,
        );
    }

    #[test]
    fn test_merge_outer_cfg_attributes() {
        check_assist(
            merge_attributes,
            r#"
$0#![cfg(feature = "a")]
#![cfg(feature = "b")]
#![cfg(feature = "c")]$0
            "#,
            r#"
#![cfg(all(feature = "a", feature = "b", feature = "c"))]
            "#,
        );
    }

    #[test]
    fn test_merge_attributes_not_applicable_other() {
        check_assist_not_applicable(
            merge_attributes,
            r#"
$0#![xxx(feature = "a")]
#![xxx(feature = "c")]$0
            "#,
        );
    }

    #[test]
    fn test_merge_attributes_not_applicable_single_attr() {
        check_assist_not_applicable(
            merge_attributes,
            r#"
$0#[cfg(feature = "a")]$0
struct Foo;
            "#,
        );
    }

    #[test]
    fn test_merge_attributes_not_applicable_different() {
        check_assist_not_applicable(
            merge_attributes,
            r#"
$0#![allow(unused)]
#![deny(deprecated)]$0
            "#,
        );
    }
}
