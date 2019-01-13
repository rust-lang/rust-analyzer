use ra_syntax::{ast::{self, AstNode}, TextUnit};

use crate::assists::{AssistCtx, Assist};

pub fn introduce_match<'a>(ctx: AssistCtx) -> Option<Assist> {
    let node = ctx.covering_node();
    let _expr = node.ancestors().filter_map(ast::Expr::cast).next()?;

    ctx.build("introduce match", move |edit| {
        let first_part = format!("match {} {{\n    ", node.text());
        let match_expr = format!("{}_ => (),\n}}", first_part);
        edit.replace_node_and_indent(node, match_expr.into_boxed_str());

        // FIXME: Set cursor at beginning of first match arm.
        // The following doesn't work because of re-indentation:
        //
        // edit.set_cursor(expr.range().start() + TextUnit::of_str(&first_part));
        //
        // Workaround:
        edit.set_cursor(node.range().start() + TextUnit::of_str("match "));
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assists::{check_assist, check_assist_range};

    #[test]
    fn test_introduce_match_simple() {
        check_assist(
            introduce_match,
            "
fn foo() {
    let x = Some(42);
    x<|>
}",
            "
fn foo() {
    let x = Some(42);
    match <|>x {
        _ => (),
    }
}",
        );
    }

    #[test]
    fn test_introduce_match_expr_stmt() {
        check_assist_range(
            introduce_match,
            "
fn foo() {
    <|>Some(42).map(|x| x + 1)<|>
}",
            "
fn foo() {
    match <|>Some(42).map(|x| x + 1) {
        _ => (),
    }
}",
        );
    }

    #[test]
    fn test_introduce_match_part_of_expr_stmt() {
        check_assist_range(
            introduce_match,
            "
fn foo() {
    <|>Some(21)<|>.map(|x| x + 21);
}",
            "
fn foo() {
    match <|>Some(21) {
        _ => (),
    }.map(|x| x + 21);
}",
        );
    }

    #[test]
    fn test_introduce_match_last_expr() {
        check_assist_range(
            introduce_match,
            "
fn foo() {
    bar(<|>Ok(42)<|>)
}",
    // FIXME: This is the wrong indentation
            "
fn foo() {
    bar(match <|>Ok(42) {
    _ => (),
})
}",
        );
    }

    #[test]
    fn test_introduce_match_last_full_expr() {
        check_assist_range(
            introduce_match,
            "
fn foo() {
    <|>bar(1 + 1)<|>
}",
            "
fn foo() {
    match <|>bar(1 + 1) {
        _ => (),
    }
}",
        );
    }

}
