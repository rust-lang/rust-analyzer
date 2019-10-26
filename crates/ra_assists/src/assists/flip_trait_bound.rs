//! `flip_trait_bound` provides an assist that flips the positions of adjacent
//! type bounds in a type bound list.

use crate::{Assist, AssistCtx, AssistId};
use hir::db::HirDatabase;
use ra_syntax::ast::{AstNode, TypeBound, TypeBoundList};
use ra_syntax::{NodeOrToken, TextRange};
use std::cmp::{max, min};

/// Flip trait bound assist.
pub(crate) fn flip_trait_bound(mut ctx: AssistCtx<impl HirDatabase>) -> Option<Assist> {
    // The assist should only be applied when the cursor is in a TypeBoundList
    let bound_list = ctx.node_at_offset::<TypeBoundList>()?;
    let bound_list_range = bound_list.syntax().text_range();
    if !ctx.frange.range.is_subrange(&bound_list_range) {
        return None;
    }

    let (bound_at_cursor, other_bound) = type_bound_pair_at_cursor(ctx.frange.range, &bound_list)?;

    let original_cursor_range = ctx.frange.range;
    ctx.add_action(AssistId("flip_trait_bound"), "flip trait bound", |edit| {
        let bound_at_cursor_range = bound_at_cursor.syntax().text_range();
        let other_bound_range = other_bound.syntax().text_range();
        edit.target(TextRange::from_to(
            min(bound_at_cursor_range.start(), other_bound_range.start()),
            max(bound_at_cursor_range.end(), other_bound_range.end()),
        ));
        edit.replace(bound_at_cursor_range, other_bound.syntax().text());
        edit.replace(other_bound_range, bound_at_cursor.syntax().text());
        edit.set_cursor(original_cursor_range.start());
    });

    ctx.build()
}

/// Find the pair of type bounds that should be flipped given `cursor_range`.
/// - If there are fewer than two TypeBounds in the list, return None.
/// - If the cursor is between two TypeBounds, these are flipped.
/// - If the cursor is on a TypeBound, flips with the preceding TypeBound, or the
/// following one if the cursor is on the last TypeBound in the list.
fn type_bound_pair_at_cursor(
    cursor_range: TextRange,
    bound_list: &TypeBoundList,
) -> Option<(TypeBound, TypeBound)> {
    let bounds: Vec<TypeBound> = bound_list.bounds().collect();
    if bounds.len() < 2 {
        return None;
    }
    let mut bound_at_cursor = 0;
    dbg!(&cursor_range);
    for elem in bound_list.syntax().children_with_tokens() {
        match elem {
            NodeOrToken::Node(node) => {
                if node.text_range().contains(cursor_range.start()) {
                    break;
                }
                bound_at_cursor += 1;
            }
            NodeOrToken::Token(token) => {
                if cursor_range.is_subrange(&token.text_range()) {
                    break;
                }
            }
        }
    }
    // if the cursor is after the last bound, swap it with the preceding one
    let bound_at_cursor = min(bound_at_cursor, bounds.len() - 1);
    // if `bound_at_cursor` is the first bound in the list, pick the following TypeBound as `other_bound`
    let other_bound = if bound_at_cursor == 0 { 1 } else { bound_at_cursor - 1 };
    Some((bounds[bound_at_cursor].clone(), bounds[other_bound].clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::helpers::{check_assist, check_assist_not_applicable, check_assist_target};

    #[test]
    fn flip_trait_bound_assist_available() {
        check_assist_target(flip_trait_bound, "struct S<T> where T: A + <|>B + C { }", "A + B")
    }

    #[test]
    fn flip_trait_bound_not_applicable_for_single_trait_bound() {
        check_assist_not_applicable(flip_trait_bound, "struct S<T> where T: <|>A { }")
    }

    #[test]
    fn flip_trait_bound_works_for_struct() {
        check_assist(
            flip_trait_bound,
            "struct S<T> where T: <|>A + B { }",
            "struct S<T> where T: <|>B + A { }",
        )
    }

    #[test]
    fn flip_trait_bound_works_for_trait_impl() {
        check_assist(
            flip_trait_bound,
            "impl X for S<T> where T: <|>A + B { }",
            "impl X for S<T> where T: <|>B + A { }",
        )
    }

    #[test]
    fn flip_trait_bound_works_for_fn() {
        check_assist(flip_trait_bound, "fn f<T: <|>A + B>(t: T) { }", "fn f<T: <|>B + A>(t: T) { }")
    }

    #[test]
    fn flip_trait_bound_works_for_fn_where_clause() {
        check_assist(
            flip_trait_bound,
            "fn f<T>(t: T) where T: <|>A + B { }",
            "fn f<T>(t: T) where T: <|>B + A { }",
        )
    }

    #[test]
    fn flip_trait_bound_works_for_lifetime() {
        check_assist(
            flip_trait_bound,
            "fn f<T>(t: T) where T: <|>A + 'static { }",
            "fn f<T>(t: T) where T: <|>'static + A { }",
        )
    }

    #[test]
    fn flip_trait_bound_works_for_complex_bounds() {
        check_assist(
            flip_trait_bound,
            "struct S<T> where T: A<T> + b_mod::B<|><T> + C<T> { }",
            "struct S<T> where T: b_mod::B<T> + A<|><T> + C<T> { }",
        )
    }

    #[test]
    fn flip_trait_bound_swaps_with_preceding_at_cursor() {
        check_assist(
            flip_trait_bound,
            "struct S<T> where T: A + <|>B + C { }",
            "struct S<T> where T: B + <|>A + C { }",
        )
    }

    #[test]
    fn flip_trait_bound_works_on_first_bound() {
        check_assist(
            flip_trait_bound,
            "struct S<T> where T: <|>A + B { }",
            "struct S<T> where T: <|>B + A { }",
        )
    }

    #[test]
    fn flip_trait_bound_works_after_last_bound() {
        check_assist(
            flip_trait_bound,
            "struct S<T> where T: A + B<|> { }",
            "struct S<T> where T: B + A<|> { }",
        )
    }

    #[test]
    fn flip_trait_bound_works_on_plus_op() {
        check_assist(
            flip_trait_bound,
            "struct S<T> where T: A <|>+ B { }",
            "struct S<T> where T: B <|>+ A { }",
        )
    }

    #[test]
    fn flip_trait_bound_works_on_whitespace_between_bounds() {
        check_assist(
            flip_trait_bound,
            "struct S<T> where T: A + B<|> + C{ }",
            "struct S<T> where T: A + C<|> + B{ }",
        )
    }
}
