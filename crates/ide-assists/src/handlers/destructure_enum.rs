use ide_db::famous_defs::FamousDefs;
use syntax::ast::{
    self, AstNode, Expr, edit::AstNodeEdit, edit::IndentLevel, make, syntax_factory::SyntaxFactory,
};

use crate::handlers::add_missing_match_arms::{ExtendedEnum, build_pat, resolve_enum_def};
use crate::{AssistContext, AssistId, Assists};

// Assist: destructure_enum
//
// Destructures an enum value to a match expression.
//
// ```
// enum Action { Move { distance: u32 }, Stop }
//
// fn handle(action: Action) {
//     action$0
// }
// ```
// ->
// ```
// enum Action { Move { distance: u32 }, Stop }
//
// fn handle(action: Action) {
//     match action {
//         Action::Move { distance } => ${1:{}}
//         Action::Stop => ${2:{}}$0
//     }
// }
// ```
//
// When matching on an Option, Some is offered first.
//
// ```
// # //- minicore: option
// fn handle(i: Option<i32>) {
//     i$0
// }
// ```
// ->
// ```
// fn handle(i: Option<i32>) {
//     match i {
//         Some(_) => ${1:{}}
//         None => ${2:{}}$0
//     }
// }
// ```
pub(crate) fn destructure_enum(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let expr = ctx.find_node_at_offset::<Expr>()?;
    let enum_def = resolve_enum_def(&ctx.sema, &expr, None)?;

    let scope = ctx.sema.scope(expr.syntax())?;
    let module = scope.module();
    let krate = module.krate(ctx.db());
    let cfg = ctx.config.find_path_config(ctx.sema.is_nightly(scope.krate()));

    // Handle Option specially.
    let option_enum = FamousDefs(&ctx.sema, krate).core_option_Option();
    let is_option =
        matches!(&enum_def, ExtendedEnum::Enum { enum_, .. } if option_enum == Some(*enum_));

    // Check if we need a catch-all `_ => {}` arm.
    let has_hidden_variants =
        enum_def.variants(ctx.db()).iter().any(|v| v.should_be_hidden(ctx.db(), krate));
    let needs_catch_all_arm = enum_def.is_non_exhaustive(ctx.db(), krate) || has_hidden_variants;

    acc.add(
        AssistId::refactor_rewrite("destructure_enum"),
        "Destructure enum with match",
        expr.syntax().text_range(),
        |builder| {
            let make = SyntaxFactory::with_mappings();

            let mut arms: Vec<ast::MatchArm> = enum_def
                .variants(ctx.db())
                .into_iter()
                .filter(|v| !v.should_be_hidden(ctx.db(), krate))
                .filter_map(|variant| {
                    let pat = build_pat(ctx, &make, module, variant, cfg)?;
                    Some(make.match_arm(pat, None, make::expr_empty_block().into()))
                })
                .collect();

            // Option puts None before Some, but users expect Some first.
            if is_option {
                arms.reverse();
            }

            if needs_catch_all_arm {
                let wildcard = make.match_arm(
                    make.wildcard_pat().into(),
                    None,
                    make::expr_empty_block().into(),
                );
                arms.push(wildcard);
            }

            if arms.is_empty() {
                return;
            }

            let match_arm_list = make.match_arm_list(arms.clone());
            let match_expr = make.expr_match(expr.clone(), match_arm_list);
            let indent = IndentLevel::from_node(expr.syntax());
            let match_expr = match_expr.indent(indent);

            // Get the match arm list from the indented expression for annotations
            let match_arm_list = match_expr.match_arm_list().unwrap();

            let mut editor = builder.make_editor(expr.syntax());
            editor.replace(expr.syntax(), match_expr.syntax());

            // Add snippet placeholders if the LSP client supports it.
            if let Some(cap) = ctx.config.snippet_cap {
                for arm in match_arm_list.arms() {
                    if let Some(arm_expr) = arm.expr() {
                        editor.add_annotation(
                            arm_expr.syntax(),
                            builder.make_placeholder_snippet(cap),
                        );
                    }
                }

                if let Some(last_arm) = match_arm_list.arms().last() {
                    editor.add_annotation(last_arm.syntax(), builder.make_tabstop_after(cap));
                }
            }

            editor.add_mappings(make.finish_with_mappings());
            builder.add_file_edits(ctx.vfs_file_id(), editor);
        },
    )
}
