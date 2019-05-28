use hir::db::HirDatabase;
use ra_syntax::{
    ast::{MatchArm, AstNode,Pat,Whitespace,AstToken},
    SyntaxElement,
    Direction,
    SyntaxKind::COMMA,
    TextUnit,
    algo::non_trivia_sibling
};

use crate::{AssistCtx, Assist, AssistId};

pub(crate) fn merge_match_arms(mut ctx: AssistCtx<impl HirDatabase>) -> Option<Assist> {
    let match_arm = ctx.node_at_offset::<MatchArm>().unwrap();

    let next_arm = MatchArm::cast(match_arm.syntax().next_sibling()?)?;

    let match_arm_expr = match_arm.expr()?;

    let next_arm_expr = next_arm.expr()?;

    if match_arm_expr.syntax().text().len() != next_arm_expr.syntax().text().len()
        && match_arm_expr.syntax().text() != next_arm_expr.syntax().text()
    {
        // return None;
    }

    let buf = build_buffer(match_arm.pats());

    let comma = non_trivia_sibling(match_arm.syntax().into(), Direction::Next);

    let white_space_offset = match_arm.syntax().prev_sibling_or_token();

    ctx.add_action(AssistId("merge_match_arms"), "merge match arms", |edit| {
        edit.target(match_arm.syntax().range());

        let offseting_amount = match white_space_offset {
            Some(SyntaxElement::Token(tok)) => {
                if let Some(_) = Whitespace::cast(tok) {
                    let ele = white_space_offset.unwrap().range();
                    ele.len()
                } else {
                    TextUnit::from(0)
                }
            }
            _ => TextUnit::from(0),
        };

        if let Some(comma) = comma {
            if comma.kind() == COMMA {
                edit.delete(comma.range());
            }
        }

        edit.replace_node_and_indent(match_arm.syntax(), buf);

        if let Some(comma) = non_trivia_sibling(match_arm.syntax().into(), Direction::Next) {
            if comma.kind() == COMMA {
                edit.set_cursor(
                    match_arm.syntax().range().end()
                        + offseting_amount
                        + comma.range().len()
                        + TextUnit::from(2),
                );
            } else {
                edit.set_cursor(
                    match_arm.syntax().range().end() + offseting_amount + TextUnit::from(3),
                )
            }
        } else {
            println!("{} vs {}", match_arm.syntax().range().end(), next_arm.syntax().range().end());
            edit.set_cursor(match_arm.syntax().range().end() + offseting_amount)
        }
    });

    ctx.build()
}

fn build_buffer<'a>(pats: impl Iterator<Item = &'a Pat>) -> String {
    let mut buf = String::new();

    pats.for_each(|pat| buf.push_str(&format!("{}", pat.syntax().text())));

    buf.push_str(" |");

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::helpers::{check_assist,check_assist_target,check_assist_not_applicable};

    #[test]
    fn merge_match_arms_target() {
        check_assist_target(
            merge_match_arms,
            r#"
            enum A {
                Foo,
                Bar,
                Baz
            }
            fn f() {
                let x = A::Foo;
               
                match x {
                    Foo <|> => () 
                    Bar => ()
                    Baz => ()
                }
            }
            "#,
            r#"Foo  => ()"#,
        );
    }

    #[test]
    fn merge_match_arm() {
        check_assist(
            merge_match_arms,
            r#"
            enum A {
                Foo,
                Bar,
                Baz
            }
            fn f() {
                let x = A::Foo;
               
                match x {
                    Foo <|> => () ,
                    Bar => (),
                    Baz => (),
                }
            }
            "#,
            r#"
            enum A {
                Foo,
                Bar,
                Baz
            }
            fn f() {
                let x = A::Foo;
               
                match x {
                    Foo |
                    Bar => <|>(),
                    Baz => (),
                }
            }
            "#,
        );
    }

    #[test]
    fn merge_match_arms_no_comma() {
        check_assist(
            merge_match_arms,
            r#"
            enum A {
                Foo,
                Bar,
                Baz
            }
            fn f() {
                let x = A::Foo;
               
                match x {
                    Foo <|> => () 
                    Bar => ()
                    Baz => ()
                }
            }
            "#,
            r#"
            enum A {
                Foo,
                Bar,
                Baz
            }
            fn f() {
                let x = A::Foo;
               
                match x {
                    Foo | 
                    Bar => <|>()
                    Baz => ()
                }
            }
            "#,
        );
    }

    #[test]
    fn test_merge_match_arms_not_applicable() {
        check_assist_not_applicable(
            merge_match_arms,
            r#"
            enum A {
                Foo,
                Bar,
                Baz
            }
            fn f() {
                let x = A::Foo;

                let y = "Foo".into();
               
                match x {
                    Foo <|> => y.+= "Bar"
                    Bar => ()
                    Baz => ()
                }
            }
            "#,
        );
    }

    #[test]
    fn merge_match_arms_complex_patterns() {
        check_assist(
            merge_match_arms,
            r#"
            enum A {
                Foo(i32,i32),
                Bar(i32),
                Baz
            }
            fn f() {
                let x = A::Foo(1,3);
               
                match x {
                    Foo(1,_) <|> | Foo(_,3) => ()
                    Bar(_) => <|>()
                    Baz => ()
                }
            }
            "#,
             r#"
            enum A {
                Foo(i32,i32),
                Bar(i32),
                Baz
            }
            fn f() {
                let x = A::Foo(1,3);

                ley y = "Foo".into()
               
                match x {
                    Foo(1,_) | Foo(_,3) | 
                    Bar(_) => ()
                    Baz => ()
                }
            }
            "#,
        );
    }
}
