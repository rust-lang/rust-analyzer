use stdx::format_to;
use syntax::{
    ast::{self},
    AstNode,
};

use super::convert_iter_for_each_to_for::{impls_core_iter, is_ref_and_impls_iter_method};
use crate::{utils::detect_for_loop, AssistContext, AssistId, AssistKind, Assists};

// Assist: convert_for_loop_with_while_let
//
// Converts a for loop into a while let on the Iterator.
//
// ```
// fn main() {
//     let x = vec![1, 2, 3];
//     for$0 v in x {
//         let y = v * 2;
//     }
// }
// ```
// ->
// ```
// fn main() {
//     let x = vec![1, 2, 3];
//     let mut x = x.into_iter();
//     while let Some(v) = x.next() {
//         let y = v * 2;
//     }
// }
// ```
pub(crate) fn convert_for_loop_with_while_let(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
) -> Option<()> {
    let (for_loop, iterable, pat, body) = detect_for_loop(ctx)?;

    acc.add(
        AssistId("convert_for_loop_with_while_let", AssistKind::RefactorRewrite),
        "Replace this for loop with while let",
        for_loop.syntax().text_range(),
        |builder| {
            let mut buf = String::new();

            let iterator_name;
            println!("{}", iterable);
            if let Some((expr_behind_ref, method)) =
                is_ref_and_impls_iter_method(&ctx.sema, &iterable)
            {
                // We have either "for x in &col" and col implements a method called iter
                //             or "for x in &mut col" and col implements a method called iter_mut
                format_to!(
                    buf,
                    "let mut {} = {}.{}();\n    ",
                    expr_behind_ref,
                    expr_behind_ref,
                    method
                );
                iterator_name = expr_behind_ref.to_string();
            } else if let ast::Expr::RangeExpr(..) = iterable {
                // range expressions need assigned with new variable
                format_to!(buf, "let mut iter = {};\n    ", iterable);
                iterator_name = String::from("iter");
            } else if impls_core_iter(&ctx.sema, &iterable) {
                format_to!(buf, "let mut iter = {};\n    ", iterable);
                iterator_name = String::from("iter");
            } else if let ast::Expr::RefExpr(_) = iterable {
                format_to!(buf, "let mut iter = ({}).into_iter();\n    ", iterable);
                iterator_name = String::from("iter");
            } else {
                format_to!(buf, "let mut {} = {}.into_iter();\n    ", iterable, iterable);
                iterator_name = iterable.to_string();
            }

            let while_syn = format!("while let Some({}) = {}.next() {}", pat, iterator_name, body);
            format_to!(buf, "{}", while_syn);

            builder.replace(for_loop.syntax().text_range(), buf)
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;
    #[test]
    fn simple_for_to_while() {
        check_assist(
            convert_for_loop_with_while_let,
            r"
fn main() {
    let x = vec![1, 2, 3];
    for $0v in x {
        v *= 2;
    }
}",
            r"
fn main() {
    let x = vec![1, 2, 3];
    let mut x = x.into_iter();
    while let Some(v) = x.next() {
        v *= 2;
    }
}",
        )
    }
    #[test]
    fn test_for_loop_with_iterator() {
        check_assist(
            convert_for_loop_with_while_let,
            r#"
//- minicore: iterators
fn main() {
    let it = core::iter::repeat(92);
    for $0(x, y) in it {
        println!("x: {}, y: {}", x, y);
    }
}
"#,
            r#"
fn main() {
    let it = core::iter::repeat(92);
    let mut iter = it;
    while let Some((x, y)) = iter.next() {
        println!("x: {}, y: {}", x, y);
    }
}
"#,
        )
    }

    #[test]
    fn for_in_range_to_while() {
        check_assist(
            convert_for_loop_with_while_let,
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
    let mut iter = 0..92;
    while let Some(x) = iter.next() {
        print!("{}", x);
    }
}"#,
        )
    }

    #[test]
    fn for_to_while_not_available_in_body() {
        cov_mark::check!(not_available_in_body);
        check_assist_not_applicable(
            convert_for_loop_with_while_let,
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
    fn for_to_while_for_borrowed() {
        check_assist(
            convert_for_loop_with_while_let,
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
    let mut x = x.iter();
    while let Some(v) = x.next() {
        let a = v * 2;
    }
}
"#,
        )
    }

    #[test]
    fn for_to_while_for_borrowed_no_iter_method() {
        check_assist(
            convert_for_loop_with_while_let,
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
    let mut iter = (&x).into_iter();
    while let Some(v) = iter.next() {
        let a = v * 2;
    }
}
",
        )
    }
    #[test]
    fn each_to_for_for_borrowed_mut() {
        check_assist(
            convert_for_loop_with_while_let,
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
    let mut x = x.iter_mut();
    while let Some(v) = x.next() {
        let a = v * 2;
    }
}
"#,
        )
    }

    #[test]
    fn for_to_while_for_borrowed_mut_behind_var() {
        check_assist(
            convert_for_loop_with_while_let,
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
    let mut y = y.into_iter();
    while let Some(v) = y.next() {
        *v *= 2;
    }
}",
        )
    }
    #[test]
    fn for_to_while_already_impls_iterator() {
        cov_mark::check!(test_already_impls_iterator);
        check_assist(
            convert_for_loop_with_while_let,
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
    let mut iter = core::iter::repeat(92).take(1);
    while let Some(a) = iter.next() {
        println!("{}", a);
    }
}
"#,
        );
    }
}
