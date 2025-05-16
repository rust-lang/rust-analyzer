use stdx::format_to;
use syntax::{
    AstNode, T,
    ast::{self, Expr, LiteralKind, Pat, Type, edit::AstNodeEdit, make},
};

use crate::{AssistContext, AssistId, Assists};

// Assist: unwrap_array
//
// Unwrap the array to different variables.
//
// ```
// # //- minicore: result
// fn main() {
//     $0let [foo, bar] = ["Foo", "Bar"];
// }
// ```
// ->
// ```
// fn main() {
//     let foo = "Foo";
//     let bar = "Bar";
// }
// ```
pub(crate) fn unwrap_array(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let let_kw = ctx.find_token_syntax_at_offset(T![let])?;
    let let_node = let_kw.parent()?;
    let let_stmt = ast::LetStmt::cast(let_node.clone())?;
    let (pat, init) = (let_stmt.pat()?, let_stmt.initializer()?);
    let ty = let_stmt.ty();

    let Pat::SlicePat(slice_pat) = pat else {
        return None;
    };
    let Expr::ArrayExpr(array_init) = init else {
        return None;
    };
    let arr_ty = ty.and_then(|ty| match ty {
        Type::ArrayType(ty) => Some(ty),
        _ => None,
    });
    let arr_ty_len = arr_ty.as_ref().and_then(get_size);
    let arr_ty = arr_ty.and_then(|ty| ty.ty());

    if slice_pat.pats().count() != array_init.exprs().count()
        || arr_ty_len.is_some_and(|len| len != slice_pat.pats().count())
    {
        return None;
    }

    acc.add(
        AssistId::refactor_rewrite("unwrap_array"),
        "Unwrap array",
        let_kw.text_range(),
        |builder| {
            let mut lets_str = String::new();
            let indent = let_stmt.indent_level();

            for (pattern, initializer) in slice_pat.pats().zip(array_init.exprs().map(Some)) {
                let output = make::let_stmt(pattern, arr_ty.clone(), initializer);
                format_to!(lets_str, "{indent}{output}\n");
            }

            builder.replace(let_node.text_range(), lets_str.trim());
        },
    )
}

fn get_size(arr: &ast::ArrayType) -> Option<usize> {
    match arr.const_arg()?.expr()? {
        Expr::Literal(lit) => match lit.kind() {
            LiteralKind::IntNumber(num) => num.value().ok().and_then(|n| n.try_into().ok()),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn unwrap_arrays() {
        check_assist(
            unwrap_array,
            r#"
            fn main() {
                $0let [foo, bar] = ["Foo", "Bar"];
            }
            "#,
            r#"
            fn main() {
                let foo = "Foo";
                let bar = "Bar";
            }
            "#,
        );

        check_assist(
            unwrap_array,
            r#"
            fn main() {
                $0let [foo, bar, baz] = ["Foo", "Bar", "Baz"];
            }
            "#,
            r#"
            fn main() {
                let foo = "Foo";
                let bar = "Bar";
                let baz = "Baz";
            }
            "#,
        );
    }

    #[test]
    fn unwrap_array_with_types() {
        check_assist(
            unwrap_array,
            r#"
            fn main() {
                $0let [foo, bar]: [u8; 2] = [5, 10];
            }
            "#,
            r#"
            fn main() {
                let foo: u8 = 5;
                let bar: u8 = 10;
            }
            "#,
        );
    }

    #[test]
    fn unwrap_array_single_pat() {
        check_assist(
            unwrap_array,
            r#"
            fn main() {
                $0let [foo]: [u8; 1] = [5];
            }
            "#,
            r#"
            fn main() {
                let foo: u8 = 5;
            }
            "#,
        );
        check_assist(
            unwrap_array,
            r#"
            fn main() {
                $0let [foo] = [5];
            }
            "#,
            r#"
            fn main() {
                let foo = 5;
            }
            "#,
        );
    }

    #[test]
    fn unwrap_array_empty() {
        check_assist(
            unwrap_array,
            r#"
            fn main() {
                $0let []: [u8; 0] = [];//...
            }
            "#,
            r#"
            fn main() {
                //...
            }
            "#,
        );
        check_assist(
            unwrap_array,
            r#"
            fn main() {
                $0let [] = [];//...
            }
            "#,
            r#"
            fn main() {
                //...
            }
            "#,
        );
    }

    #[test]
    fn unwrap_array_invalid_len() {
        check_assist_not_applicable(
            unwrap_array,
            r#"
            fn main() {
                $0let [foo, bar]: [u8; 3] = [5, 10];
            }
            "#,
        );
        check_assist_not_applicable(
            unwrap_array,
            r#"
            fn main() {
                $0let [foo, bar] = [5, 10, 15];
            }
            "#,
        );
        check_assist_not_applicable(
            unwrap_array,
            r#"
            fn main() {
                $0let [] = [1];
            }
            "#,
        );
        check_assist_not_applicable(
            unwrap_array,
            r#"
            fn main() {
                $0let [a] = [];
            }
            "#,
        );
        check_assist_not_applicable(
            unwrap_array,
            r#"
            fn main() {
                $0let []: [(); 1] = [];
            }
            "#,
        );
    }

    #[test]
    fn unwrap_array_ignore_complex_len() {
        check_assist(
            unwrap_array,
            r#"
            const N: usize = 2;
            fn main() {
                $0let [a, b]: [u8; N] = [2, 3];
            }
            "#,
            r#"
            const N: usize = 2;
            fn main() {
                let a: u8 = 2;
                let b: u8 = 3;
            }
            "#,
        );
        check_assist(
            unwrap_array,
            r#"
            const N: usize = 3;
            fn main() {
                $0let [a, b]: [u8; N] = [2, 3];
            }
            "#,
            r#"
            const N: usize = 3;
            fn main() {
                let a: u8 = 2;
                let b: u8 = 3;
            }
            "#,
        );
    }

    #[test]
    fn unwrap_array_with_empty_len() {
        check_assist(
            unwrap_array,
            r#"
            fn main() {
                $0let [a, b]: [u8;] = [2, 3];
            }
            "#,
            r#"
            fn main() {
                let a: u8 = 2;
                let b: u8 = 3;
            }
            "#,
        );
        check_assist(
            unwrap_array,
            r#"
            fn main() {
                $0let [a, b]: [u8; _] = [2, 3];
            }
            "#,
            r#"
            fn main() {
                let a: u8 = 2;
                let b: u8 = 3;
            }
            "#,
        );
    }

    #[test]
    fn unwrap_array_with_complex_pattern() {
        check_assist(
            unwrap_array,
            r#"
            fn main() {
                $0let [a | a, b @ c]: [u8; 2] = [2, 3];
            }
            "#,
            r#"
            fn main() {
                let a | a: u8 = 2;
                let b @ c: u8 = 3;
            }
            "#,
        );
        check_assist(
            unwrap_array,
            r#"
            fn main() {
                $0let [a | a, b @ c] = [2, 3];
            }
            "#,
            r#"
            fn main() {
                let a | a = 2;
                let b @ c = 3;
            }
            "#,
        );
        check_assist(
            unwrap_array,
            r#"
            fn main() {
                $0let [m @ (a | a), b @ c] = [2, 3];
            }
            "#,
            r#"
            fn main() {
                let m @ (a | a) = 2;
                let b @ c = 3;
            }
            "#,
        );
        check_assist(
            unwrap_array,
            r#"
            fn main() {
                $0let [_, b @ c] = [2, 3];
            }
            "#,
            r#"
            fn main() {
                let _ = 2;
                let b @ c = 3;
            }
            "#,
        );
    }
}
