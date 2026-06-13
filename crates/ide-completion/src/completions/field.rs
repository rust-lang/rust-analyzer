//! Completion of field list position.

use syntax::{AstNode, ast};

use crate::{
    CompletionContext, CompletionItem, CompletionItemKind, Completions,
    context::{PathCompletionCtx, Qualified},
};

pub(crate) fn complete_field_list_tuple_variant(
    acc: &mut Completions,
    ctx: &CompletionContext<'_, '_>,
    path_ctx: &PathCompletionCtx<'_>,
) {
    complete_record_fields(acc, ctx);

    if ctx.qualifier_ctx.vis_node.is_some() {
    } else if let PathCompletionCtx {
        has_macro_bang: false,
        qualified: Qualified::No,
        parent: None,
        has_type_args: false,
        ..
    } = path_ctx
    {
        let mut add_keyword = |kw, snippet| acc.add_keyword_snippet(ctx, kw, snippet);
        add_keyword("pub(crate)", "pub(crate) $0");
        add_keyword("pub(super)", "pub(super) $0");
        add_keyword("pub", "pub $0");
    }
}

pub(crate) fn complete_field_list_record_variant(
    acc: &mut Completions,
    ctx: &CompletionContext<'_, '_>,
) {
    complete_record_fields(acc, ctx);

    if ctx.qualifier_ctx.vis_node.is_none() {
        let mut add_keyword = |kw, snippet| acc.add_keyword_snippet(ctx, kw, snippet);
        add_keyword("pub(crate)", "pub(crate) $0");
        add_keyword("pub(super)", "pub(super) $0");
        add_keyword("pub", "pub $0");
    }
}

fn complete_record_fields(acc: &mut Completions, ctx: &CompletionContext<'_, '_>) {
    let Some(record_field) = ctx
        .original_token
        .parent_ancestors()
        .nth(1)
        .and_then(ast::RecordField::cast)
        .filter(|it| it.ty().is_none() && it.default_val().is_none())
    else {
        return;
    };
    let Some(stmt_list) = record_field.syntax().ancestors().find_map(ast::StmtList::cast) else {
        return;
    };
    super::fn_param::locals_from_stmt_list_scope(ctx, stmt_list, |name, ty| {
        let text = syntax::format_smolstr!("{}: {ty},", name.display(ctx.db, ctx.edition));
        CompletionItem::new(
            CompletionItemKind::Binding,
            record_field.syntax().text_range(),
            text,
            ctx.edition,
        )
        .add_to(acc, ctx.db);
    });
}
