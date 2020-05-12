use crate::{AssistContext, AssistId, Assists};

use ast::edit::IndentLevel;
use ra_syntax::{ast, AstNode};

// Assist: surround_with_block
//
// This assist surround expressions with a block expression.
//
// ```
// fn foo() {
//     <|>println!("foo");
//     println!("bar");<|>
// }
// ```
// ->
// ```
// fn foo() {
//     {
//         println!("foo");
//         println!("bar");
//     }
// }
// ```
pub(crate) fn surround_with_block(acc: &mut Assists, ctx: &AssistContext) -> Option<()> {
    if ctx.frange.range.is_empty() {
        return None;
    }
    let node: ast::Expr = ctx.find_node_at_offset()?;
    let child = node.syntax().children_with_tokens().find_map(|elt| match elt {
        ra_syntax::NodeOrToken::Node(node) => Some(node),
        ra_syntax::NodeOrToken::Token(_) => None,
    });
    let indent_level = IndentLevel::from_node(&child?);

    let covering_elt = ctx.covering_element();
    let covering_node = match covering_elt {
        ra_syntax::NodeOrToken::Node(node) => node,
        ra_syntax::NodeOrToken::Token(_) => return None,
    };
    let target = covering_node.text_range();

    acc.add(AssistId("surround_with_block"), "Surround with block", target, |edit| {
        edit.set_cursor(ctx.frange.range.start());
        let indent_str = "    ";
        let mut inner_expr_lines = ctx
            .covering_element_raw_string()
            .trim_start()
            .lines()
            .enumerate()
            .map(|(line_idx, line)| {
                if line.is_empty() {
                    return line.to_string();
                }
                let line = if line_idx == 0 && indent_level.0 != 0 {
                    indent_str.repeat(indent_level.0 as usize) + line
                } else {
                    line.to_string()
                };
                format!("{}{}", indent_str, line)
            })
            .collect::<Vec<String>>();
        inner_expr_lines.push(format!("{}}}", indent_str.repeat(indent_level.0 as usize)));

        edit.replace(ctx.frange.range, format!("{{\n{}", inner_expr_lines.join("\n")));
    })
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn not_applicable() {
        check_assist_not_applicable(
            surround_with_block,
            r#"
            fn main() {
                bar();
                <|>foo();

                //comment
                bar();
            }"#,
        );

        check_assist_not_applicable(
            surround_with_block,
            r#"
            fn main() {
                bar();
                <|>fo<|>o();

                //comment
                bar();
            }"#,
        );
    }
    #[test]
    fn simple_statements() {
        check_assist(
            surround_with_block,
            r#"
fn main() {
    bar();
    <|>foo();

    //comment
    bar();<|>
}"#,
            r#"
fn main() {
    bar();
    <|>{
        foo();

        //comment
        bar();
    }
}"#,
        );

        check_assist(
            surround_with_block,
            r#####"
fn foo() {
    <|>println!("foo");
    println!("bar");<|>
}
"#####,
            r#####"
fn foo() {
    <|>{
        println!("foo");
        println!("bar");
    }
}
"#####,
        );
    }

    #[test]
    fn simple_let() {
        check_assist(
            surround_with_block,
            r#"
fn main() {
    bar();
    let info = <|>foo();

    //comment
    bar();<|>
}"#,
            r#"
fn main() {
    bar();
    let info = <|>{
        foo();

        //comment
        bar();
    }
}"#,
        );
    }

    #[test]
    fn simple_in_loop() {
        check_assist(
            surround_with_block,
            r#"
fn main() {
    bar();
    loop {
        <|>foo();

        //comment
        bar();<|>
    }
}"#,
            r#"
fn main() {
    bar();
    loop {
        <|>{
            foo();

            //comment
            bar();
        }
    }
}"#,
        );
    }

    #[test]
    fn stmt_with_let() {
        check_assist(
            surround_with_block,
            r#"
fn main() {
    if true {
        println!("test");
        <|>let bar();
        foo();

        //comment
        bar();<|>
    }
}"#,
            r#"
fn main() {
    if true {
        println!("test");
        <|>{
            let bar();
            foo();

            //comment
            bar();
        }
    }
}"#,
        );
    }
}
