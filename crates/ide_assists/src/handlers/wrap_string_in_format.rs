use syntax::{ast, AstToken};

use crate::{AssistContext, AssistId, AssistKind, Assists};

// Assist: wrap_string_in_format
//
// Wraps a plain string literal in a format! macro.
//
// ```
// fn main() {
//     "Hello,$0 World!";
// }
// ```
// ->
// ```
// fn main() {
//     format!("Hello, World!");
// }
// ```
pub(crate) fn wrap_string_in_format(acc: &mut Assists, ctx: &AssistContext) -> Option<()> {
    let token = ctx.find_token_at_offset::<ast::String>()?;
    let target = token.syntax().text_range();

    acc.add(
        AssistId("wrap_string_in_format", AssistKind::RefactorRewrite),
        "Wrap in format!()",
        target,
        |edit| {
            edit.insert(token.syntax().text_range().start(), "format!(");
            edit.insert(token.syntax().text_range().end(), ")");
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_target};

    use super::*;

    #[test]
    fn wrap_string_in_format_target() {
        check_assist_target(
            wrap_string_in_format,
            r#"
            fn f() {
                let s = $0"foo";
            }
            "#,
            r#""foo""#,
        );
    }

    #[test]
    fn wrap_string_in_format_works() {
        check_assist(
            wrap_string_in_format,
            r#"
            fn f() {
                $0"foo";
            }
            "#,
            r#"
            fn f() {
                format!("foo");
            }
            "#,
        )
    }
}
