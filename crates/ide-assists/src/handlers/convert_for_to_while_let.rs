use hir::{sym, Name};
use ide_db::famous_defs::FamousDefs;
use syntax::{
    ast::{
        self,
        edit::{AstNodeEdit, IndentLevel},
        make, HasLoopBody,
    },
    AstNode,
};

use crate::{AssistContext, AssistId, AssistKind, Assists};

// Assist: convert_for_loop_to_while_let
//
// Converts a for loop into a while let on the Iterator.
//
// ```
// fn main() {
//     let x = vec![1, 2, 3];
//     for$0 v in x {
//         let y = v * 2;
//     };
// }
// ```
// ->
// ```
// fn main() {
//     let x = vec![1, 2, 3];
//     while let Some(v) = x.into_iter().next() {
//         let y = v * 2;
//     };
// }
// ```
pub(crate) fn convert_for_loop_to_while_let(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
) -> Option<()> {
    let for_loop = ctx.find_node_at_offset::<ast::ForExpr>()?;
    let iterable = for_loop.iterable()?;
    let pat = for_loop.pat()?;
    let body = for_loop.loop_body()?;
    if body.syntax().text_range().start() < ctx.offset() {
        cov_mark::hit!(not_available_in_body);
        return None;
    }

    acc.add(
        AssistId("convert_for_loop_to_while_let", AssistKind::RefactorRewrite),
        "Replace this for loop with `while let`",
        for_loop.syntax().text_range(),
        |builder| {
            let iter = if let Some((expr_behind_ref, method)) =
                is_ref_and_impls_iter_method(&ctx.sema, &iterable)
            {
                // We have either "for x in &col" and col implements a method called iter
                //             or "for x in &mut col" and col implements a method called iter_mut
                make::expr_method_call(
                    expr_behind_ref,
                    make::name_ref(method.as_str()),
                    make::arg_list([]),
                )
            } else if let ast::Expr::RangeExpr(..) = iterable {
                // range expressions need to be parenthesized for the syntax to be correct
                make::expr_paren(iterable)
            } else if impls_core_iter(&ctx.sema, &iterable) {
                // We can't use `iterable` if its immutable, so we create a tmp mutable variable
                let mut_expr = make::let_stmt(
                    make::ident_pat(false, true, make::name("tmp")).into(),
                    None,
                    Some(iterable),
                );
                builder.insert(
                    for_loop.syntax().text_range().start(),
                    mut_expr.reset_indent().to_string(),
                );
                builder.insert(
                    for_loop.syntax().text_range().start(),
                    make::tokens::whitespace(
                        format!("\n{}", IndentLevel::from_node(for_loop.syntax())).as_str(),
                    )
                    .to_string(),
                );
                make::expr_path(make::ext::ident_path("tmp"))
            } else if let ast::Expr::RefExpr(_) = iterable {
                make::expr_method_call(iterable, make::name_ref("into_iter"), make::arg_list([]))
            } else {
                make::expr_method_call(iterable, make::name_ref("into_iter"), make::arg_list([]))
            };

            let opt_pat = make::tuple_struct_pat(make::ext::ident_path("Some"), [pat]);
            let iter_next_expr =
                make::expr_method_call(iter, make::name_ref("next"), make::arg_list([]));
            let cond = make::expr_let(opt_pat.into(), iter_next_expr);

            let expr_for_loop = make::expr_while_loop(cond.into(), body);
            builder.replace(for_loop.syntax().text_range(), expr_for_loop.to_string())
        },
    )
}

/// If iterable is a reference where the expression behind the reference implements a method
/// returning an Iterator called iter or iter_mut (depending on the type of reference) then return
/// the expression behind the reference and the method name
fn is_ref_and_impls_iter_method(
    sema: &hir::Semantics<'_, ide_db::RootDatabase>,
    iterable: &ast::Expr,
) -> Option<(ast::Expr, hir::Name)> {
    let ref_expr = match iterable {
        ast::Expr::RefExpr(r) => r,
        _ => return None,
    };
    let wanted_method = Name::new_symbol_root(if ref_expr.mut_token().is_some() {
        sym::iter_mut.clone()
    } else {
        sym::iter.clone()
    });
    let expr_behind_ref = ref_expr.expr()?;
    let ty = sema.type_of_expr(&expr_behind_ref)?.adjusted();
    let scope = sema.scope(iterable.syntax())?;
    let krate = scope.krate();
    let iter_trait = FamousDefs(sema, krate).core_iter_Iterator()?;

    let has_wanted_method = ty
        .iterate_method_candidates(sema.db, &scope, None, Some(&wanted_method), |func| {
            if func.ret_type(sema.db).impls_trait(sema.db, iter_trait, &[]) {
                return Some(());
            }
            None
        })
        .is_some();
    if !has_wanted_method {
        return None;
    }

    Some((expr_behind_ref, wanted_method))
}

/// Whether iterable implements core::Iterator
fn impls_core_iter(sema: &hir::Semantics<'_, ide_db::RootDatabase>, iterable: &ast::Expr) -> bool {
    (|| {
        let it_typ = sema.type_of_expr(iterable)?.adjusted();

        let module = sema.scope(iterable.syntax())?.module();

        let krate = module.krate();
        let iter_trait = FamousDefs(sema, krate).core_iter_Iterator()?;
        cov_mark::hit!(test_already_impls_iterator);
        Some(it_typ.impls_trait(sema.db, iter_trait, &[]))
    })()
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn each_to_for_simple_for() {
        check_assist(
            convert_for_loop_to_while_let,
            r"
fn main() {
    let x = vec![1, 2, 3];
    for $0v in x {
        v *= 2;
    };
}",
            r"
fn main() {
    let x = vec![1, 2, 3];
    while let Some(v) = x.into_iter().next() {
        v *= 2;
    };
}",
        )
    }

    #[test]
    fn each_to_for_for_in_range() {
        check_assist(
            convert_for_loop_to_while_let,
            r#"
//- minicore: range, iterators
impl<T> core::iter::Iterator for core::ops::Range<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

fn main() {
    for $0x in 0..92 {
        print!("{}", x);
    }
}"#,
            r#"
impl<T> core::iter::Iterator for core::ops::Range<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

fn main() {
    while let Some(x) = (0..92).next() {
        print!("{}", x);
    }
}"#,
        )
    }

    #[test]
    fn each_to_for_not_available_in_body() {
        cov_mark::check!(not_available_in_body);
        check_assist_not_applicable(
            convert_for_loop_to_while_let,
            r"
fn main() {
    let x = vec![1, 2, 3];
    for v in x {
        $0v *= 2;
    }
}",
        )
    }

    #[test]
    fn each_to_for_for_borrowed() {
        check_assist(
            convert_for_loop_to_while_let,
            r#"
//- minicore: iterators
use core::iter::{Repeat, repeat};

struct S;
impl S {
    fn iter(&self) -> Repeat<i32> { repeat(92) }
    fn iter_mut(&mut self) -> Repeat<i32> { repeat(92) }
}

fn main() {
    let x = S;
    for $0v in &x {
        let a = v * 2;
    }
}
"#,
            r#"
use core::iter::{Repeat, repeat};

struct S;
impl S {
    fn iter(&self) -> Repeat<i32> { repeat(92) }
    fn iter_mut(&mut self) -> Repeat<i32> { repeat(92) }
}

fn main() {
    let x = S;
    while let Some(v) = x.iter().next() {
        let a = v * 2;
    }
}
"#,
        )
    }

    #[test]
    fn each_to_for_for_borrowed_no_iter_method() {
        check_assist(
            convert_for_loop_to_while_let,
            r"
struct NoIterMethod;
fn main() {
    let x = NoIterMethod;
    for $0v in &x {
        let a = v * 2;
    }
}
",
            r"
struct NoIterMethod;
fn main() {
    let x = NoIterMethod;
    while let Some(v) = &x.into_iter().next() {
        let a = v * 2;
    }
}
",
        )
    }

    #[test]
    fn each_to_for_for_borrowed_mut() {
        check_assist(
            convert_for_loop_to_while_let,
            r#"
//- minicore: iterators
use core::iter::{Repeat, repeat};

struct S;
impl S {
    fn iter(&self) -> Repeat<i32> { repeat(92) }
    fn iter_mut(&mut self) -> Repeat<i32> { repeat(92) }
}

fn main() {
    let x = S;
    for $0v in &mut x {
        let a = v * 2;
    }
}
"#,
            r#"
use core::iter::{Repeat, repeat};

struct S;
impl S {
    fn iter(&self) -> Repeat<i32> { repeat(92) }
    fn iter_mut(&mut self) -> Repeat<i32> { repeat(92) }
}

fn main() {
    let x = S;
    while let Some(v) = x.iter_mut().next() {
        let a = v * 2;
    }
}
"#,
        )
    }

    #[test]
    fn each_to_for_for_borrowed_mut_behind_var() {
        check_assist(
            convert_for_loop_to_while_let,
            r"
fn main() {
    let x = vec![1, 2, 3];
    let y = &mut x;
    for $0v in y {
        *v *= 2;
    }
}",
            r"
fn main() {
    let x = vec![1, 2, 3];
    let y = &mut x;
    while let Some(v) = y.into_iter().next() {
        *v *= 2;
    }
}",
        )
    }

    #[test]
    fn each_to_for_already_impls_iterator() {
        cov_mark::check!(test_already_impls_iterator);
        check_assist(
            convert_for_loop_to_while_let,
            r#"
//- minicore: iterators
fn main() {
    for$0 a in core::iter::repeat(92).take(1) {
        println!("{}", a);
    }
}
"#,
            r#"
fn main() {
    let mut tmp = core::iter::repeat(92).take(1);
    while let Some(a) = tmp.next() {
        println!("{}", a);
    }
}
"#,
        );
    }
}
