use crate::{
    AssistId,
    assist_context::{AssistContext, Assists},
};
use syntax::{
    AstNode, T,
    ast::{self, edit::AstNodeEdit, syntax_factory::SyntaxFactory},
};

// Assist: unmerge_else_if
//
// Unmerge `else if` into the `else {}`.
//
// ```
// fn foo() {
//     if cond1 {
//         xxx()
//     } else if$0 cond2 {
//         yyy()
//     } else {
//         zzz()
//     }
// }
// ```
// ->
// ```
// fn foo() {
//     if cond1 {
//         xxx()
//     } else {
//         if cond2 {
//             yyy()
//         } else {
//             zzz()
//         }
//     }
// }
// ```
pub(crate) fn unmerge_else_if(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let if_keyword = ctx.find_token_syntax_at_offset(T![if])?;
    let if_expr = if_keyword.parent().and_then(ast::IfExpr::cast)?;
    let _parent_if = if_expr.syntax().parent().and_then(ast::IfExpr::cast)?;

    let target = if_expr.syntax().text_range();
    acc.add(AssistId::refactor_rewrite("unmerge_else_if"), "Unmerge else if", target, |builder| {
        let mut edit = builder.make_editor(if_expr.syntax());
        let make = SyntaxFactory::with_mappings();

        let new_if_expr = if_expr.reset_indent().indent(1.into()).into();
        let new_block = make.block_expr(None, Some(new_if_expr)).indent(if_expr.indent_level());
        edit.replace(if_expr.syntax(), new_block.syntax());

        edit.add_mappings(make.finish_with_mappings());
        builder.add_file_edits(ctx.vfs_file_id(), edit);
    })
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn with_indent() {
        check_assist(
            unmerge_else_if,
            r#"
            fn foo() {
                {
                    if cond1 {
                        xxx()
                    } else if$0 cond2 {
                        yyy(
                            "..."
                        )
                    } else {
                        zzz(
                            "..."
                        )
                    }
                }
            }"#,
            r#"
            fn foo() {
                {
                    if cond1 {
                        xxx()
                    } else {
                        if cond2 {
                            yyy(
                                "..."
                            )
                        } else {
                            zzz(
                                "..."
                            )
                        }
                    }
                }
            }"#,
        );
    }

    #[test]
    fn not_applicable_outside_if_keyword() {
        check_assist_not_applicable(
            unmerge_else_if,
            r#"
            fn foo() {
                if cond1 {
                    xxx()
                } else if cond2$0 {
                    yyy()
                } else {
                    zzz()
                }
            }"#,
        );

        check_assist_not_applicable(
            unmerge_else_if,
            r#"
            fn foo() {
                if cond1 {
                    xxx()
                } else$0 if cond2 {
                    yyy()
                } else {
                    zzz()
                }
            }"#,
        );
    }
}
