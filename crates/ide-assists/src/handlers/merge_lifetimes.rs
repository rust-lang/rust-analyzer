use syntax::{
    Direction, NodeOrToken, SyntaxKind, SyntaxNode, T,
    ast::{self, AstNode},
    syntax_editor::SyntaxEditor,
};

use crate::{AssistContext, AssistId, Assists};

// Assist: merge_lifetimes
//
// Merge some lifetime parameter.
//
// ```
// struct Foo<$0'a, 'b$0> {
//     data: &'b [Cow<'a, str>],
// }
// ```
// ->
// ```
// struct Foo<'a> {
//     data: &'a [Cow<'a, str>],
// }
// ```
pub(crate) fn merge_lifetimes(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let param_list = ctx.find_node_at_range::<ast::GenericParamList>()?;
    let params = param_list
        .generic_params()
        .filter(|it| {
            it.syntax()
                .text_range()
                .intersect(ctx.selection_trimmed())
                .is_some_and(|range| !range.is_empty())
        })
        .map(|param| match param {
            ast::GenericParam::LifetimeParam(it) if it.lifetime_bounds().next().is_none() => {
                Some(it)
            }
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;

    if params.len() < 2 {
        return None;
    }
    let lifetimes = params.iter().map(|param| param.lifetime()).collect::<Option<Vec<_>>>()?;

    let first = params.first()?;
    let target = first.syntax().text_range().cover(params.last()?.syntax().text_range());
    acc.add(AssistId::refactor_rewrite("merge_lifetimes"), "Merge lifetimes", target, |builder| {
        let mut edit = builder.make_editor(first.syntax());

        delete_params(&params[1..], &mut edit);
        for next in param_list.syntax().siblings(Direction::Next).skip(1) {
            rename_params(&lifetimes, &next, &mut edit);
        }

        builder.add_file_edits(ctx.vfs_file_id(), edit);
    })
}

fn rename_params(lifetimes: &[ast::Lifetime], node: &SyntaxNode, edit: &mut SyntaxEditor) {
    let Some((first, rest)) = lifetimes.split_first() else { return };

    for child in node.children_with_tokens() {
        if ast::GenericParamList::can_cast(child.kind()) {
            return;
        }
        match child {
            NodeOrToken::Node(node) => rename_params(lifetimes, &node, edit),
            NodeOrToken::Token(token) => {
                if token.kind() == SyntaxKind::LIFETIME_IDENT
                    && rest.iter().any(|it| it.syntax().text() == token.text())
                {
                    edit.replace(token, first.syntax().clone_for_update());
                }
            }
        }
    }
}

fn delete_params(params: &[ast::LifetimeParam], edit: &mut SyntaxEditor) {
    for param in params {
        param
            .syntax()
            .siblings_with_tokens(Direction::Prev)
            .take(3)
            .filter(|it| matches!(it.kind(), T![,] | SyntaxKind::WHITESPACE))
            .for_each(|it| edit.delete(it));
        edit.delete(param.syntax());
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn test_merge_lifetimes() {
        check_assist(
            merge_lifetimes,
            "
            fn foo<'a, $0'b, 'c, 'd$0>(m: &'a str, n: &'b str, h: &'c str) -> (&'a str, &'b str) {
                let x: &'a str = m;
                let y: &'b str = n;
                let z: &'c str = h;
            }
            ",
            "
            fn foo<'a, 'b>(m: &'a str, n: &'b str, h: &'b str) -> (&'a str, &'b str) {
                let x: &'a str = m;
                let y: &'b str = n;
                let z: &'b str = h;
            }
            ",
        );
    }

    #[test]
    fn test_merge_lifetimes_in_struct() {
        check_assist(
            merge_lifetimes,
            "
            struct Foo<$0'a, 'b, 'c$0> {
                s: &'a &'b mut str,
                n: &'c mut i32,
            }
            ",
            "
            struct Foo<'a> {
                s: &'a &'a mut str,
                n: &'a mut i32,
            }
            ",
        );
    }

    #[test]
    fn test_merge_lifetimes_not_rename_next_node() {
        check_assist(
            merge_lifetimes,
            "
            fn foo<'a, $0'b, 'c, 'd$0>(m: &'a str, n: &'b str, h: &'c str) -> (&'a str, &'b str) {
                let x: &'a str = m;
                let y: &'b str = n;
                let z: &'c str = h;
            }
            fn bar<'c>(s: &'c str) {}
            ",
            "
            fn foo<'a, 'b>(m: &'a str, n: &'b str, h: &'b str) -> (&'a str, &'b str) {
                let x: &'a str = m;
                let y: &'b str = n;
                let z: &'b str = h;
            }
            fn bar<'c>(s: &'c str) {}
            ",
        );
    }

    #[test]
    fn test_merge_lifetimes_not_rename_nested_node() {
        check_assist(
            merge_lifetimes,
            "
            fn foo<'a, $0'b, 'c, 'd$0>(m: &'a str, n: &'b str, h: &'c str) -> (&'a str, &'b str) {
                let x: &'a str = m;
                let y: &'b str = n;
                let z: &'c str = h;
                fn bar<'c>(s: &'c str) {}
            }
            ",
            "
            fn foo<'a, 'b>(m: &'a str, n: &'b str, h: &'b str) -> (&'a str, &'b str) {
                let x: &'a str = m;
                let y: &'b str = n;
                let z: &'b str = h;
                fn bar<'c>(s: &'c str) {}
            }
            ",
        );
    }

    #[test]
    fn merge_lifetimes_not_applicable_one_lifetime() {
        check_assist_not_applicable(
            merge_lifetimes,
            "
            fn foo<'a, $0'b$0, 'c, 'd>(m: &'a str, n: &'b str, h: &'c str) -> (&'a str, &'b str) {
                let x: &'a str = m;
                let y: &'b str = n;
                let z: &'c str = h;
            }
            ",
        );
    }

    #[test]
    fn merge_lifetimes_not_applicable_with_bounds() {
        check_assist_not_applicable(
            merge_lifetimes,
            "
            fn foo<'a, $0'b, 'c: 'a, 'd$0>(m: &'a str, n: &'b str, h: &'c str) -> (&'a str, &'b str) {
                let x: &'a str = m;
                let y: &'b str = n;
                let z: &'c str = h;
            }
            ",
        );
    }
}
