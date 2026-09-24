use syntax::{
    T,
    ast::{self, AstNode, edit::AstNodeEdit},
};

use crate::{AssistContext, AssistId, Assists};

// Assist: flip_or_pattern
//
// Flips two patterns in an or-pattern.
//
// ```
// fn foo() {
//     let (a |$0 b) = 1;
// }
// ```
// ->
// ```
// fn foo() {
//     let (b | a) = 1;
// }
// ```
pub(crate) fn flip_or_pattern(acc: &mut Assists, ctx: &AssistContext<'_, '_>) -> Option<()> {
    // Only flip on the `|` token
    let pipe = ctx.find_token_syntax_at_offset(T![|])?;

    let parent = ast::OrPat::cast(pipe.parent()?)?;

    let before = ast::Pat::cast(pipe.prev_sibling_or_token()?.into_node()?)?;
    let after = ast::Pat::cast(pipe.next_sibling_or_token()?.into_node()?)?;

    let target = pipe.text_range();
    acc.add(AssistId::refactor_rewrite("flip_or_pattern"), "Flip patterns", target, |builder| {
        let editor = builder.make_editor(parent.syntax());
        let (first, second) = (before.detached(), after.detached());
        editor.replace(before.syntax(), second.syntax());
        editor.replace(after.syntax(), first.syntax());
        builder.add_file_edits(ctx.vfs_file_id(), editor);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::{check_assist, check_assist_not_applicable, check_assist_target};

    #[test]
    fn flip_or_pattern_assist_available() {
        check_assist_target(flip_or_pattern, "fn main(a |$0 b: ()) {}", "|")
    }

    #[test]
    fn flip_or_pattern_not_applicable_for_leading_pipe() {
        check_assist_not_applicable(flip_or_pattern, "fn main(|$0 b: ()) {}")
    }

    #[test]
    fn flip_or_pattern_works() {
        check_assist(
            flip_or_pattern,
            "fn foo() { let (a | b |$0 c | d) = 1; }",
            "fn foo() { let (a | c | b | d) = 1; }",
        )
    }

    #[test]
    fn flip_or_pattern_works_match_guard() {
        check_assist(
            flip_or_pattern,
            "fn foo() { match() { a |$0 b if true => () }}",
            "fn foo() { match() { b | a if true => () }}",
        )
    }
}
