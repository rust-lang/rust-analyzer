use ide_db::assists::AssistId;
use syntax::{
    AstNode,
    ast::{self, make},
};

use crate::assist_context::{AssistContext, Assists};

// Assist: desugar_self_param
//
// Replaces like `&self` with a `self: &Self` parameter.
//
// ```
// struct Foo;
// impl Foo {
//     fn foo(&$0self) {}
// }
// ```
// ->
// ```
// struct Foo;
// impl Foo {
//     fn foo(self: &Self) {}
// }
// ```
pub(crate) fn desugar_self_param(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let param = ctx.find_node_at_offset::<ast::SelfParam>()?;

    if param.colon_token().is_some() {
        return None;
    }

    let self_ty =
        make::ty_path(make::path_unqualified(make::path_segment(make::name_ref_self_ty())));
    let replace = match param.kind() {
        ast::SelfParamKind::Owned => self_ty,
        ast::SelfParamKind::Ref => make::ty_ref(self_ty, false),
        ast::SelfParamKind::MutRef => make::ty_ref(self_ty, true),
    };

    let target = param.syntax().text_range();
    acc.add(
        AssistId::refactor_rewrite("desugar_self_param"),
        format!("Replace `{param}` to `self: {replace}`"),
        target,
        |edit| {
            edit.replace(target, format!("self: {replace}"));
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::{check_assist, check_assist_not_applicable};

    #[test]
    fn test_desugar_self_param() {
        check_assist(
            desugar_self_param,
            r#"
                struct Foo;
                impl Foo {
                    fn foo(&$0self) {}
                }
            "#,
            r#"
                struct Foo;
                impl Foo {
                    fn foo(self: &Self) {}
                }
            "#,
        );

        check_assist(
            desugar_self_param,
            r#"
                struct Foo;
                impl Foo {
                    fn foo(&$0mut self) {}
                }
            "#,
            r#"
                struct Foo;
                impl Foo {
                    fn foo(self: &mut Self) {}
                }
            "#,
        );

        check_assist(
            desugar_self_param,
            r#"
                struct Foo;
                impl Foo {
                    fn foo($0self) {}
                }
            "#,
            r#"
                struct Foo;
                impl Foo {
                    fn foo(self: Self) {}
                }
            "#,
        );
    }

    #[test]
    fn test_desugar_self_param_not_applicable() {
        check_assist_not_applicable(
            desugar_self_param,
            r#"
                struct Foo;
                impl Foo {
                    fn foo($0self: &Self) {}
                }
            "#,
        );
    }
}
