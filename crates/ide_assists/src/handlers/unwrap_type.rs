use syntax::{ast, AstNode};

use crate::{AssistContext, AssistId, AssistKind, Assists};

// Assist: unwrap_type
//
// This assist unwrap the type from its wrapper type.
//
// ```
// fn foo() {
//     let bar: Option<$0usize> = 5;
// }
// ```
// ->
// ```
// fn foo() {
//     let bar: usize = 5;
// }
// ```
pub(crate) fn unwrap_type(acc: &mut Assists, ctx: &AssistContext) -> Option<()> {
    let assist_id = AssistId("unwrap_type", AssistKind::RefactorRewrite);
    let assist_label = "Unwrap type";

    let generic_args: ast::GenericArgList = ctx.find_node_at_offset()?;
    let mut generic_args = generic_args.generic_args();
    let first_generic_arg = generic_args.next()?;
    if generic_args.next().is_some() {
        return None;
    }
    let first_generic_arg_node = first_generic_arg.syntax();
    let wrapper_type = first_generic_arg_node.ancestors().find_map(ast::PathType::cast)?;

    return acc.add(assist_id, assist_label, first_generic_arg_node.text_range(), |builder| {
        builder.replace(wrapper_type.syntax().text_range(), first_generic_arg.to_string());
    });
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn unwrap_type_var_decl() {
        check_assist(
            unwrap_type,
            r#"
fn main() {
    let test: Option<$0usize> = 5;
}
"#,
            r#"
fn main() {
    let test: usize = 5;
}
"#,
        );
    }

    #[test]
    fn unwrap_type_var_decl_deep() {
        check_assist(
            unwrap_type,
            r#"
fn main() {
    let test: Option<Vec<$0usize>> = Some(5);
}
"#,
            r#"
fn main() {
    let test: Option<usize> = Some(5);
}
"#,
        );
    }

    #[test]
    fn unwrap_gen_type_func() {
        check_assist(
            unwrap_type,
            r#"
fn foo(arg: Option<$0usize>) {

}
"#,
            r#"
fn foo(arg: usize) {

}
"#,
        );
    }

    #[test]
    fn unwrap_gen_type_in_struct() {
        check_assist(
            unwrap_type,
            r#"
struct Foo<T> {
    inner: Option<$0T>
}
fn main() {

}
"#,
            r#"
struct Foo<T> {
    inner: T
}
fn main() {

}
"#,
        );
    }

    #[test]
    fn unwrap_generics_func() {
        check_assist_not_applicable(
            unwrap_type,
            r#"
fn foo<$0T>(arg: Option<T>) {

}
"#,
        );
    }
}
