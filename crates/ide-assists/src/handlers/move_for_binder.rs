use ide_db::assists::AssistId;
use syntax::{
    SyntaxKind,
    ast::{self, AstNode, make::tokens},
    syntax_editor::Position,
};

use crate::assist_context::{AssistContext, Assists};

// Assist: move_for_binder
//
// Move for-binder to where predicate.
//
// ```
// fn foo<F>()
// where
//     F: $0for<'a> Fn(&'a str)
// {}
// ```
// ->
// ```
// fn foo<F>()
// where
//     for<'a> F: Fn(&'a str)
// {}
// ```
pub(crate) fn move_for_binder(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let for_binder = ctx.find_node_at_offset::<ast::ForBinder>()?;
    let where_pred = ctx.find_node_at_offset::<ast::WherePred>()?;

    if where_pred.for_binder().is_some() || !ctx.has_empty_selection() {
        return None;
    }

    let label = "Move for-binder to where predicate";
    let target = for_binder.syntax().text_range();
    acc.add(AssistId::refactor_rewrite("move_for_binder"), label, target, |builder| {
        let mut edit = builder.make_editor(for_binder.syntax());

        if let Some(next) = for_binder.syntax().next_sibling_or_token()
            && next.kind() == SyntaxKind::WHITESPACE
        {
            edit.delete(next);
        }

        edit.delete(for_binder.syntax());
        edit.insert(Position::before(where_pred.syntax()), tokens::single_space());
        edit.insert(Position::before(where_pred.syntax()), for_binder.syntax().clone_for_update());

        builder.add_file_edits(ctx.vfs_file_id(), edit);
    })
}

#[cfg(test)]
mod tests {
    use crate::tests::check_assist_not_applicable;

    use super::*;

    #[test]
    fn not_applicable_on_exists() {
        check_assist_not_applicable(
            move_for_binder,
            r#"
            fn foo<F>()
            where
                for<'b> F: $0for<'a> Fn(&'a str)
            {}
            "#,
        );

        check_assist_not_applicable(
            move_for_binder,
            r#"
            fn foo<F>()
            where
                $0for<'a> F: Fn(&'a str)
            {}
            "#,
        );
    }
}
