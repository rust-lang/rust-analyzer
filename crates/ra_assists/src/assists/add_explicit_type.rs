use hir::{db::HirDatabase, HirDisplay};
use ra_syntax::{ast::{self, AstNode, LetStmt, NameOwner}, Direction, SyntaxElement, SyntaxKind, T, TextRange, TextUnit};

use crate::{Assist, AssistCtx, AssistId};

enum AssistType {
    PlaceHolder(TextRange),
    Name(TextUnit)
}

// Assist: add_explicit_type
//
// Specify type for a let binding.
//
// ```
// fn main() {
//     let x<|> = 92;
// }
// ```
// ->
// ```
// fn main() {
//     let x: i32 = 92;
// }
// ```
pub(crate) fn add_explicit_type(ctx: AssistCtx<impl HirDatabase>) -> Option<Assist> {
    let stmt = ctx.find_node_at_offset::<LetStmt>()?;
    let expr = stmt.initializer()?;
    let pat = stmt.pat()?;

    // Must be a binding
    let pat = match pat {
        ast::Pat::BindPat(bind_pat) => bind_pat,
        _ => return None,
    };

    let pat_range = pat.syntax().text_range();
    // The binding must have a name
    let name = pat.name()?;
    let name_range = name.syntax().text_range();
    // Assist not applicable if the type has already been specified
    let root = stmt.syntax();

    // Found a color, we need to check if this is a placeholder def.
    let colon: Option<SyntaxElement> =
        root.children_with_tokens().find(|child| child.kind() == T![:]);

    fn followed_by_placeholder_ty(root: &SyntaxElement) -> bool {
        if let Some(n) = root.as_token() {
            for sibling in n.siblings_with_tokens(Direction::Next) {
                if let Some(sib) = sibling.as_token() {
                    if sib.kind() == SyntaxKind::PLACEHOLDER_TYPE {
                        return true;
                    }
                }
                if let Some(sib) = sibling.as_node() {
                    for descendent in sib.descendants_with_tokens() {
                        if descendent.kind() == SyntaxKind::PLACEHOLDER_TYPE {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }


    let assist_type = match colon {
        Some(col) => {
            if !followed_by_placeholder_ty(&col) {
                return None
            }

            let colon_as_token = col.as_token()?;

            let ty_range_start = colon_as_token.text_range().start();

            let ty_range_end = colon_as_token
                .siblings_with_tokens(Direction::Next)
                .find(|s| s.kind() == SyntaxKind::EQ).expect("This is a bind expression").text_range().start();

            AssistType::PlaceHolder(TextRange::from_to(ty_range_start, ty_range_end))
        },
        None => AssistType::Name(name_range.end())
    };

    // Infer type
    let db = ctx.db;
    let analyzer = ctx.source_analyzer(stmt.syntax(), None);
    let ty = analyzer.type_of(db, &expr)?;

    // Assist not applicable if the type is unknown
    if ty.contains_unknown() {
        return None;
    }

    ctx.add_assist(AssistId("add_explicit_type"), "add explicit type", |edit| {
        edit.target(pat_range);

        match assist_type {
            AssistType::PlaceHolder(range) => edit.replace(range, format!(": {} ", ty.display(db))),
            AssistType::Name(pos) => edit.insert(name_range.end(), format!(": {}", ty.display(db)))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::helpers::{check_assist, check_assist_not_applicable, check_assist_target};

    #[test]
    fn add_explicit_type_target() {
        check_assist_target(add_explicit_type, "fn f() { let a<|> = 1; }", "a");
    }

    #[test]
    fn add_explicit_type_works_for_simple_expr() {
        check_assist(
            add_explicit_type,
            "fn f() { let a<|> = 1; }",
            "fn f() { let a<|>: i32 = 1; }",
        );
    }

    #[test]
    fn add_explicit_type_works_for_placeholder_type() {
        check_assist(
            add_explicit_type,
            "fn f() { let a<|>: _ = 1; }",
            "fn f() { let a<|>: i32 = 1; }",
        );
    }

    #[test]
    fn add_explicit_type_not_applicable_if_ty_not_inferred() {
        check_assist_not_applicable(add_explicit_type, "fn f() { let a<|> = None; }");
    }

    #[test]
    fn add_explicit_type_not_applicable_if_ty_already_specified() {
        check_assist_not_applicable(add_explicit_type, "fn f() { let a<|>: i32 = 1; }");
    }

    #[test]
    fn add_explicit_type_not_applicable_if_specified_ty_is_tuple() {
        check_assist_not_applicable(add_explicit_type, "fn f() { let a<|>: (i32, i32) = (3, 4); }");
    }
}
