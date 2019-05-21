use hir::db::HirDatabase;
use ra_syntax::{
    ast::{MatchArm, AstNode,Pat},
    Direction,
    SyntaxKind::COMMA,
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
        return None;
    }

    let buf = build_buffer(match_arm.pats());

    let comma = non_trivia_sibling( match_arm.syntax().into(), Direction::Next);

    ctx.add_action(AssistId("merge_match_arms"), "merge match arms", |edit| {
       
        edit.target(match_arm.syntax().range());

        if let Some(comma) = comma {
            if comma.kind() == COMMA {
                edit.delete(comma.range());
            }   
        }

        edit.replace_node_and_indent(match_arm.syntax(), buf);

        edit.set_cursor(next_arm.syntax().range().start())

    });

    ctx.build()
}

fn build_buffer<'a>(pats:impl Iterator<Item = &'a Pat>) -> String {
    let mut buf = String::new();

   pats.for_each(|pat| buf.push_str(&format!("{}", pat.syntax().text())));

    buf.push_str(" |");

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::helpers::{check_assist};

    #[test]
    fn merge_match_banches() {
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
                    Bar => <|> (),
                    Baz => (),
                }
            }
            "#,
        );
    }

    #[test]
    fn merge_match_banches_no_comma() {
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
                    Bar => <|> ()
                    Baz => ()
                }
            }
            "#,
        );
    }
}
