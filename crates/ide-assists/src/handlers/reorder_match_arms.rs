use hir::{Adt, EnumVariant, ModuleDef, PathResolution};
use ide_db::FxHashMap;
use syntax::{
    AstNode, SyntaxElement, SyntaxKind, SyntaxNode, T, TextRange,
    ast::{self, Pat},
    syntax_editor::SyntaxEditor,
};

use crate::{AssistContext, AssistId, Assists};

// Assist: reorder_match_arms
//
// Reorders match arms and or-pattern alternatives according to the order of
// variants in the enum definition.
//
// ```
// enum Action { Foo, Bar, Baz }
//
// fn handle(action: Action) {
//     $0match action {
//         Action::Bar => (),
//         Action::Baz | Action::Foo => (),
//     }
// }
// ```
// ->
// ```
// enum Action { Foo, Bar, Baz }
//
// fn handle(action: Action) {
//     match action {
//         Action::Foo | Action::Baz => (),
//         Action::Bar => (),
//     }
// }
// ```
pub(crate) fn reorder_match_arms(acc: &mut Assists, ctx: &AssistContext<'_, '_>) -> Option<()> {
    let match_expr = ctx.find_node_at_offset::<ast::MatchExpr>()?;
    let scrutinee = match_expr.expr()?;
    let arm_list = match_expr.match_arm_list()?;
    let arms = arm_list.arms().collect::<Vec<_>>();

    let enum_ = ctx.sema.type_of_expr(&scrutinee)?.adjusted().autoderef(ctx.db()).find_map(
        |ty| match ty.as_adt() {
            Some(Adt::Enum(enum_)) => Some(enum_),
            _ => None,
        },
    )?;
    let ranks = enum_
        .variants(ctx.db())
        .into_iter()
        .enumerate()
        .map(|(rank, variant)| (variant, rank))
        .collect::<FxHashMap<_, _>>();
    let (inner_range, mut arm_chunks) = collect_arm_chunks(ctx, &ranks, &arm_list, &arms)?;
    let original = node_text(arm_list.syntax(), inner_range);

    for run in arm_chunks.split_mut(|arm| arm.ranks.is_none()) {
        for mut index in 1..run.len() {
            while index > 0 {
                let (Some(previous_ranks), Some(current_ranks)) =
                    (&run[index - 1].ranks, &run[index].ranks)
                else {
                    stdx::never!();
                    break;
                };
                let (Some(&previous_key), Some(&current_key)) =
                    (previous_ranks.first(), current_ranks.first())
                else {
                    stdx::never!();
                    break;
                };

                let overlaps = previous_ranks.iter().any(|rank| current_ranks.contains(rank));
                if current_key >= previous_key || overlaps {
                    break;
                }

                run.swap(index - 1, index);
                index -= 1;
            }
        }
    }

    let mut reordered = String::with_capacity(original.len());
    for (chunk, slot) in arm_chunks.iter().zip(&arms) {
        reordered.push_str(&chunk.before_comma);
        if let Some(comma) = slot.comma_token() {
            reordered.push_str(comma.text());
        }
        reordered.push_str(&chunk.after_comma);
    }
    if original == reordered {
        cov_mark::hit!(already_ordered);
        return None;
    }

    let target = arm_list.syntax().text_range();
    acc.add(
        AssistId::refactor_rewrite("reorder_match_arms"),
        "Reorder match arms",
        target,
        |builder| builder.replace(inner_range, reordered),
    )
}

fn variant_rank(
    ctx: &AssistContext<'_, '_>,
    ranks: &FxHashMap<EnumVariant, usize>,
    pat: Pat,
) -> Option<usize> {
    let resolution = match pat {
        Pat::IdentPat(pat) => {
            let Some(ModuleDef::EnumVariant(variant)) = ctx.sema.resolve_bind_pat_to_const(&pat)
            else {
                return None;
            };
            return ranks.get(&variant).copied();
        }
        Pat::PathPat(pat) => ctx.sema.resolve_path(&pat.path()?)?,
        Pat::RecordPat(pat) => ctx.sema.resolve_path(&pat.path()?)?,
        Pat::TupleStructPat(pat) => ctx.sema.resolve_path(&pat.path()?)?,
        _ => return None,
    };

    let PathResolution::Def(ModuleDef::EnumVariant(variant)) = resolution else { return None };
    ranks.get(&variant).copied()
}

fn unwrap_pat(mut pat: Pat) -> Option<Pat> {
    loop {
        pat = match pat {
            Pat::BoxPat(pat) => pat.pat()?,
            Pat::DerefPat(pat) => pat.pat()?,
            Pat::IdentPat(pat) => match pat.pat() {
                Some(subpat) => subpat,
                None => return Some(Pat::IdentPat(pat)),
            },
            Pat::ParenPat(pat) => pat.pat()?,
            Pat::RefPat(pat) => pat.pat()?,
            pat => return Some(pat),
        };
    }
}

fn reorder_or_pat(
    ctx: &AssistContext<'_, '_>,
    ranks: &FxHashMap<EnumVariant, usize>,
    arm: &ast::MatchArm,
    or_pat: ast::OrPat,
) -> Option<(Vec<usize>, ast::MatchArm)> {
    let mut ranked_indices = or_pat
        .pats()
        .enumerate()
        .map(|(index, pat)| {
            let rank = variant_rank(ctx, ranks, unwrap_pat(pat)?)?;
            Some((index, rank))
        })
        .collect::<Option<Vec<_>>>()?;
    if ranked_indices.is_empty() {
        return None;
    }
    ranked_indices.sort_by_key(|(_, rank)| *rank);
    let (sorted_indices, variant_ranks): (Vec<_>, Vec<_>) = ranked_indices.into_iter().unzip();
    if sorted_indices.iter().copied().eq(0..sorted_indices.len()) {
        return Some((variant_ranks, arm.clone()));
    }

    let (editor, arm) = SyntaxEditor::with_ast_node(arm);
    let Pat::OrPat(or_pat) = arm.pat().and_then(unwrap_pat)? else {
        stdx::never!();
        return None;
    };
    let elements = or_pat.syntax().children_with_tokens().collect::<Vec<_>>();
    let pat_indices = or_pat
        .pats()
        .map(|pat| elements.iter().position(|element| element.as_node() == Some(pat.syntax())))
        .collect::<Option<Vec<_>>>()?;
    if stdx::never!(pat_indices.len() != sorted_indices.len()) {
        return None;
    }

    let alternative_ranges = pat_indices
        .into_iter()
        .map(|pat_idx| {
            let left_boundary = elements[..pat_idx]
                .iter()
                .rposition(|element| element.kind() == T![|])
                .map_or(0, |pipe_idx| pipe_idx + 1);
            let start = elements[left_boundary..pat_idx]
                .iter()
                .position(is_comment)
                .map_or(pat_idx, |offset| left_boundary + offset);

            let right_boundary = elements[pat_idx + 1..]
                .iter()
                .position(|element| element.kind() == T![|])
                .map_or(elements.len(), |offset| pat_idx + 1 + offset);
            let end = elements[pat_idx + 1..right_boundary]
                .iter()
                .rposition(is_comment)
                .map_or(pat_idx + 1, |offset| pat_idx + 2 + offset);

            start..end
        })
        .collect::<Vec<_>>();

    for (destination, source) in alternative_ranges.iter().zip(sorted_indices) {
        let replacement = elements[alternative_ranges[source].clone()].to_vec();
        editor.replace_all(
            elements[destination.start].clone()..=elements[destination.end - 1].clone(),
            replacement,
        );
    }

    let arm = ast::MatchArm::cast(editor.finish().new_root().clone())?;
    Some((variant_ranks, arm))
}

fn collect_arm_chunks(
    ctx: &AssistContext<'_, '_>,
    ranks: &FxHashMap<EnumVariant, usize>,
    arm_list: &ast::MatchArmList,
    arms: &[ast::MatchArm],
) -> Option<(TextRange, Vec<ArmChunk>)> {
    let elements = arm_list.syntax().children_with_tokens().collect::<Vec<_>>();
    let left_brace = arm_list.l_curly_token()?;
    let right_brace = arm_list.r_curly_token()?;
    let left_brace_idx = elements.iter().position(|it| it.as_token() == Some(&left_brace))?;
    let right_brace_idx = elements.iter().position(|it| it.as_token() == Some(&right_brace))?;
    let arm_indices = arms
        .iter()
        .map(|arm| elements.iter().position(|it| it.as_node() == Some(arm.syntax())))
        .collect::<Option<Vec<_>>>()?;

    let mut starts = Vec::with_capacity(arm_indices.len());
    starts.push(left_brace_idx + 1);
    for indices in arm_indices.windows(2) {
        let [previous, next] = *indices else { continue };
        let trivia = &elements[previous + 1..next];
        let start = trivia
            .iter()
            .position(starts_new_line)
            .map(|offset| previous + 1 + offset)
            .or_else(|| trivia.iter().rposition(is_comment).map(|offset| previous + 1 + offset + 1))
            .unwrap_or(previous + 1);
        starts.push(start);
    }

    let last_arm_idx = *arm_indices.last()?;
    let trailing_trivia = &elements[last_arm_idx + 1..right_brace_idx];
    let end = trailing_trivia
        .iter()
        .position(starts_new_line)
        .map(|offset| last_arm_idx + 1 + offset)
        .or_else(|| {
            trailing_trivia.iter().rposition(is_comment).map(|offset| last_arm_idx + 1 + offset + 1)
        })
        .unwrap_or(last_arm_idx + 1);
    let inner_range = TextRange::new(
        elements[*starts.first()?].text_range().start(),
        elements[end.checked_sub(1)?].text_range().end(),
    );
    let chunks = starts
        .iter()
        .copied()
        .zip(starts.iter().copied().skip(1).chain([end]))
        .zip(arms)
        .zip(arm_indices)
        .map(|(((start, end), arm), arm_idx)| {
            let (variant_ranks, arm) = match arm.pat().and_then(unwrap_pat) {
                Some(Pat::OrPat(or_pat)) => match reorder_or_pat(ctx, ranks, arm, or_pat) {
                    Some((variant_ranks, arm)) => (Some(variant_ranks), arm),
                    None => (None, arm.clone()),
                },
                Some(pat) => (variant_rank(ctx, ranks, pat).map(|rank| vec![rank]), arm.clone()),
                None => (None, arm.clone()),
            };

            let arm_range = arm.syntax().text_range();
            let comma = arm.comma_token();
            let before_end = comma.as_ref().map_or(arm_range.end(), |it| it.text_range().start());
            let after_start = comma.as_ref().map_or(arm_range.end(), |it| it.text_range().end());
            let arm_before_comma =
                node_text(arm.syntax(), TextRange::new(arm_range.start(), before_end));
            let arm_after_comma =
                node_text(arm.syntax(), TextRange::new(after_start, arm_range.end()));

            let mut before_comma =
                elements[start..arm_idx].iter().map(SyntaxElement::to_string).collect::<String>();
            before_comma.push_str(&arm_before_comma);

            let mut after_comma = arm_after_comma;
            after_comma.extend(elements[arm_idx + 1..end].iter().map(SyntaxElement::to_string));

            Some(ArmChunk { before_comma, after_comma, ranks: variant_ranks })
        })
        .collect::<Option<Vec<_>>>()?;

    Some((inner_range, chunks))
}

struct ArmChunk {
    before_comma: String,
    after_comma: String,
    ranks: Option<Vec<usize>>,
}

fn node_text(node: &SyntaxNode, range: TextRange) -> String {
    node.text().slice(range - node.text_range().start()).to_string()
}

fn starts_new_line(element: &SyntaxElement) -> bool {
    element.as_token().is_some_and(|token| {
        token.kind() == SyntaxKind::WHITESPACE
            && (token.text().contains('\n') || token.text().contains('\r'))
    })
}

fn is_comment(element: &SyntaxElement) -> bool {
    element.as_token().is_some_and(|token| token.kind() == SyntaxKind::COMMENT)
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn reorder_variant_patterns() {
        check_assist(
            reorder_match_arms,
            r#"
enum Action {
    Unit,
    Tuple(i32),
    Record { value: i32 },
    Imported,
    End,
}
use Action::Imported;

fn handle(action: &Action) {
    $0match action {
        &ref binding @ Action::End => (),
        Imported => (),
        Action::Record { value } => (),
        (Action::Tuple(value)) => (),
        Action::Unit => (),
    }
}
"#,
            r#"
enum Action {
    Unit,
    Tuple(i32),
    Record { value: i32 },
    Imported,
    End,
}
use Action::Imported;

fn handle(action: &Action) {
    match action {
        Action::Unit => (),
        (Action::Tuple(value)) => (),
        Action::Record { value } => (),
        Imported => (),
        &ref binding @ Action::End => (),
    }
}
"#,
        );
    }

    #[test]
    fn reorder_box_and_deref_patterns() {
        check_assist(
            reorder_match_arms,
            r#"
enum Action { Move, Stop, Wait }

fn handle(action: Action) {
    $0match action {
        box Action::Wait => (),
        builtin # deref(Action::Stop) => (),
        Action::Move => (),
    }
}
"#,
            r#"
enum Action { Move, Stop, Wait }

fn handle(action: Action) {
    match action {
        Action::Move => (),
        builtin # deref(Action::Stop) => (),
        box Action::Wait => (),
    }
}
"#,
        );
    }

    #[test]
    fn reorder_or_patterns_and_arms() {
        check_assist(
            reorder_match_arms,
            r#"
enum Action { Unit, Tuple(i32), Record { value: i32 }, Last }
use Action::{Last, Unit};

fn handle(action: &Action) {
    $0match action {
        &Action::Record { .. } => (),
        &(| Last /* last */ | Action::Tuple(/* payload */ _) | /* unit */ Unit) => (),
    }
}
"#,
            r#"
enum Action { Unit, Tuple(i32), Record { value: i32 }, Last }
use Action::{Last, Unit};

fn handle(action: &Action) {
    match action {
        &(| /* unit */ Unit | Action::Tuple(/* payload */ _) | Last /* last */) => (),
        &Action::Record { .. } => (),
    }
}
"#,
        );

        check_assist(
            reorder_match_arms,
            r#"
enum Action { A, B, C, D }

fn handle(action: Action) {
    $0match action {
        Action::D | /* A */ Action::A |
        // B
        Action::B | Action::C => (),
    }
}
"#,
            r#"
enum Action { A, B, C, D }

fn handle(action: Action) {
    match action {
        /* A */ Action::A | // B
        Action::B |
        Action::C | Action::D => (),
    }
}
"#,
        );
    }

    #[test]
    fn stably_sort_unknown_patterns() {
        check_assist(
            reorder_match_arms,
            r#"
enum Action { A, B, C, D, E }

fn handle(action: Action, condition: bool) {
    $0match action {
        Action::C => (),
        Action::B if condition => (),
        Action::B => (),
        Action::A => (),
        _ => (),
        Action::E => (),
        Action::D => (),
    }
}
"#,
            r#"
enum Action { A, B, C, D, E }

fn handle(action: Action, condition: bool) {
    match action {
        Action::A => (),
        Action::B if condition => (),
        Action::B => (),
        Action::C => (),
        _ => (),
        Action::D => (),
        Action::E => (),
    }
}
"#,
        );
    }

    #[test]
    fn do_not_move_arms_past_overlapping_patterns() {
        check_assist(
            reorder_match_arms,
            r#"
enum Action { A, B, C }

fn handle(action: Action) {
    $0match action {
        Action::C => (),
        Action::B => (),
        Action::A | Action::B => (),
    }
}
"#,
            r#"
enum Action { A, B, C }

fn handle(action: Action) {
    match action {
        Action::B => (),
        Action::A | Action::B => (),
        Action::C => (),
    }
}
"#,
        );
    }

    #[test]
    fn preserve_multiline_comments_and_commas() {
        check_assist(
            reorder_match_arms,
            r#"
enum Action { A, B, C }

fn handle(action: Action) {
    $0match action {
        // C
        Action::C => (), // trailing C
        // A
        Action::A => (), // trailing A
        Action::B => ()
    }
}
"#,
            r#"
enum Action { A, B, C }

fn handle(action: Action) {
    match action {
        // A
        Action::A => (), // trailing A
        Action::B => (),
        // C
        Action::C => () // trailing C
    }
}
"#,
        );
    }

    #[test]
    fn preserve_single_line_comments_and_commas() {
        check_assist(
            reorder_match_arms,
            r#"
enum Action { A, B, C }

fn handle(action: Action) {
    $0match action { Action::C => (), /* C */ Action::A => (), Action::B => () }
}
"#,
            r#"
enum Action { A, B, C }

fn handle(action: Action) {
    match action { Action::A => (), Action::B => (), Action::C => () /* C */ }
}
"#,
        );
    }

    #[test]
    fn unsupported_or_patterns_are_not_reordered() {
        check_assist_not_applicable(
            reorder_match_arms,
            r#"
enum Action { A, B }
const OTHER: Action = Action::A;

fn handle(action: Action) {
    $0match action {
        Action::B | OTHER => (),
    }
}
"#,
        );

        check_assist_not_applicable(
            reorder_match_arms,
            r#"
enum Action { A }

fn handle(action: Action) {
    $0match action {
        | => (),
    }
}
"#,
        );
    }

    #[test]
    fn not_applicable_when_nothing_can_be_reordered() {
        cov_mark::check!(already_ordered);
        check_assist_not_applicable(
            reorder_match_arms,
            r#"
enum Action { A, B }

fn handle(action: Action) {
    $0match action {
        Action::A | Action::B => (),
        _ => (),
    }
}
"#,
        );

        check_assist_not_applicable(
            reorder_match_arms,
            r#"
fn handle(value: i32) {
    $0match value {
        1 => (),
        0 => (),
        _ => (),
    }
}
"#,
        );
    }
}
