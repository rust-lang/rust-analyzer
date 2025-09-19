use either::Either;
use itertools::Itertools;
use std::iter::successors;
use syntax::{
    AstNode, NodeOrToken, SyntaxKind, SyntaxNode, SyntaxToken, T,
    ast::{self, edit::AstNodeEdit, make},
};

use crate::{AssistContext, AssistId, Assists};

// Assist: replace_if_matches_to_if_let
//
// Replace matches!() to let-chain in if condition.
//
// ```
// fn foo(x: Option<i32>) {
//     if let Some(n) = x && $0matches!(n.checked_div(2), Some(5..8)) {
//     }
// }
// ```
// ->
// ```
// fn foo(x: Option<i32>) {
//     if let Some(n) = x && let Some(5..8) = n.checked_div(2) {
//     }
// }
// ```
pub(crate) fn replace_if_matches_to_if_let(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
) -> Option<()> {
    if !ctx.has_empty_selection() {
        return None;
    }
    let macro_call_expr = ctx.find_node_at_offset::<ast::MacroExpr>()?;
    let macro_call = macro_call_expr.macro_call()?;

    if macro_call.path()?.segment()?.name_ref()?.text() != "matches" {
        return None;
    }
    let has_cond_expr = has_cond_expr_of_let_chain(macro_call_expr.syntax())?;
    let condition = either::for_both!(&has_cond_expr, it => it.condition())?;

    let token_tree = macro_call.token_tree()?.clone_for_update();
    let tts = tt_content(token_tree);
    let (expr_tts, pat_tts, guard_tts) = split_matches_args(&tts)?;

    let target = macro_call.syntax().text_range();
    acc.add(
        AssistId::refactor_rewrite("replace_if_matches_to_if_let"),
        "Replace matches to let-chain",
        target,
        |builder| {
            let mut edit = builder.make_editor(macro_call.syntax());

            let mut new_tts =
                vec![make::token(T![let]).into(), make::tokens::whitespace(" ").into()];
            new_tts.extend(pat_tts.iter().map(to_syntax_element));
            new_tts.extend([
                make::tokens::whitespace(" ").into(),
                make::token(T![=]).into(),
                make::tokens::whitespace(" ").into(),
            ]);
            new_tts.extend(expr_tts.iter().map(to_syntax_element));

            if let Some(guard_tts) = guard_tts {
                let whitespace = if condition.syntax().text().contains_char('\n') {
                    let indent = has_cond_expr.indent_level() + 1;
                    format!("\n{indent}")
                } else {
                    " ".to_owned()
                };
                new_tts.extend(
                    [
                        make::tokens::whitespace(&whitespace).into(),
                        make::token(T![&&]).into(),
                        make::tokens::whitespace(" ").into(),
                    ]
                    .into_iter()
                    .chain(guard_tts.iter().map(to_syntax_element)),
                );
            }

            edit.replace_with_many(macro_call.syntax(), new_tts);

            builder.add_file_edits(ctx.vfs_file_id(), edit);
        },
    )
}

type TT = NodeOrToken<ast::TokenTree, SyntaxToken>;

fn split_matches_args(tts: &[TT]) -> Option<(&[TT], &[TT], Option<&[TT]>)> {
    let (expr_tts, rest_tts) =
        tts.split(|tt| tt.as_token().is_some_and(|it| it.kind() == T![,])).next_tuple()?;
    let (pat_tts, guard_tts) = rest_tts
        .split(|tt| tt.as_token().is_some_and(|it| it.kind() == T![if]))
        .next_tuple()
        .unzip();
    let pat_tts = pat_tts.unwrap_or(rest_tts);
    Some((expr_tts, trim_tts(pat_tts), guard_tts.map(trim_tts)))
}

fn trim_tts(mut tts: &[TT]) -> &[TT] {
    let is_whitespace: fn(&(&TT, &[TT])) -> bool =
        |&(tt, _)| tt.as_token().is_some_and(|it| it.kind() == SyntaxKind::WHITESPACE);
    while let Some((_, rest)) =
        tts.split_first().filter(is_whitespace).or_else(|| tts.split_last().filter(is_whitespace))
    {
        tts = rest
    }
    tts
}

fn tt_content(token_tree: ast::TokenTree) -> Vec<TT> {
    token_tree
        .token_trees_and_tokens()
        .skip(1)
        .take_while(|it| {
            it.as_token().is_none_or(|t| {
                !matches!(t.kind(), SyntaxKind::R_PAREN | SyntaxKind::R_CURLY | SyntaxKind::R_BRACK)
            })
        })
        .collect()
}

fn to_syntax_element(
    tt: &NodeOrToken<ast::TokenTree, SyntaxToken>,
) -> NodeOrToken<SyntaxNode, SyntaxToken> {
    match tt {
        NodeOrToken::Node(node) => NodeOrToken::Node(node.syntax().clone()),
        NodeOrToken::Token(tok) => NodeOrToken::Token(tok.clone()),
    }
}

fn has_cond_expr_of_let_chain(node: &SyntaxNode) -> Option<Either<ast::IfExpr, ast::WhileExpr>> {
    let condition = successors(Some(node.clone()), |node| {
        let parent = node.parent()?;
        let bin_expr = ast::BinExpr::cast(parent)?;
        let ast::BinaryOp::LogicOp(ast::LogicOp::And) = bin_expr.op_kind()? else { return None };
        Some(bin_expr.syntax().clone())
    })
    .last()?;
    AstNode::cast(condition.parent()?)
}

#[cfg(test)]
mod tests {
    use crate::tests::check_assist;

    use super::*;

    #[test]
    fn test_replace_if_matches_to_if_let() {
        check_assist(
            replace_if_matches_to_if_let,
            "
fn foo(x: Option<i32>) {
    if $0matches!(n.checked_div(2), Some(5..8)) {
    }
}
            ",
            "
fn foo(x: Option<i32>) {
    if let Some(5..8) = n.checked_div(2) {
    }
}
            ",
        );
    }

    #[test]
    fn test_replace_if_matches_to_if_let_has_guard() {
        check_assist(
            replace_if_matches_to_if_let,
            "
fn foo(x: Option<i32>) {
    if $0matches!(n.checked_div(2), Some(m) if m > 8) {
    }
}
            ",
            "
fn foo(x: Option<i32>) {
    if let Some(m) = n.checked_div(2) && m > 8 {
    }
}
            ",
        );

        check_assist(
            replace_if_matches_to_if_let,
            "
fn foo(x: Option<i32>) {
    if true && $0matches!(n.checked_div(2), Some(m) if m > 8) {
    }
}
            ",
            "
fn foo(x: Option<i32>) {
    if true && let Some(m) = n.checked_div(2) && m > 8 {
    }
}
            ",
        );

        check_assist(
            replace_if_matches_to_if_let,
            "
fn foo(x: Option<i32>) {
    if true
        && $0matches!(n.checked_div(2), Some(m) if m > 8)
    {
    }
}
            ",
            "
fn foo(x: Option<i32>) {
    if true
        && let Some(m) = n.checked_div(2)
        && m > 8
    {
    }
}
            ",
        );
    }

    #[test]
    fn test_replace_if_matches_to_if_let_in_let_chain() {
        check_assist(
            replace_if_matches_to_if_let,
            "
fn foo(x: Option<i32>) {
    if let Some(n) = x && $0matches!(n.checked_div(2), Some(5..8)) {
    }
}
            ",
            "
fn foo(x: Option<i32>) {
    if let Some(n) = x && let Some(5..8) = n.checked_div(2) {
    }
}
            ",
        );

        check_assist(
            replace_if_matches_to_if_let,
            "
fn foo(x: Option<i32>) {
    if let Some(n) = x && true && $0matches!(n.checked_div(2), Some(5..8)) {
    }
}
            ",
            "
fn foo(x: Option<i32>) {
    if let Some(n) = x && true && let Some(5..8) = n.checked_div(2) {
    }
}
            ",
        );
    }
}
