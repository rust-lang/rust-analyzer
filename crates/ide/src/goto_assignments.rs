use hir::Semantics;
use ide_db::{
    RootDatabase,
    defs::{Definition, IdentClass},
    helpers::pick_best_token,
    search::ReferenceCategory,
};
use syntax::{AstNode, SyntaxKind::IDENT};

use crate::{FilePosition, NavigationTarget, RangeInfo, TryToNav};

// Feature: Go to Assignments
//
// Navigates to the assignments of an identifier.
//
// Returns all locations where the variable is assigned a value, including:
// - Initial definition sites (let bindings, function parameters, etc.)
// - Explicit assignment expressions (x = value)
// - Compound assignment expressions (x += value, x *= value, etc.)
pub(crate) fn goto_assignments(
    db: &RootDatabase,
    position: FilePosition,
) -> Option<RangeInfo<Vec<NavigationTarget>>> {
    let sema = &Semantics::new(db);

    let def = find_definition_at_position(sema, position)?;

    let Definition::Local(_) = def else {
        return None;
    };

    find_assignments_for_def(sema, def, position)
}

fn find_definition_at_position(
    sema: &Semantics<'_, RootDatabase>,
    position: FilePosition,
) -> Option<Definition> {
    let file = sema.parse_guess_edition(position.file_id);
    let token =
        pick_best_token(file.syntax().token_at_offset(position.offset), |kind| match kind {
            IDENT => 1,
            _ => 0,
        })?;

    let token = sema.descend_into_macros_no_opaque(token, false).pop()?;
    let parent = token.value.parent()?;

    IdentClass::classify_node(sema, &parent)?.definitions().pop().map(|(def, _)| def)
}

fn find_assignments_for_def(
    sema: &Semantics<'_, RootDatabase>,
    def: Definition,
    position: FilePosition,
) -> Option<RangeInfo<Vec<NavigationTarget>>> {
    let mut targets = Vec::new();

    if let Some(nav_result) = def.try_to_nav(sema.db) {
        targets.push(nav_result.call_site);
    }

    let usages = def.usages(sema).include_self_refs().all();

    targets.extend(usages.iter().flat_map(|(file_id, refs)| {
        refs.iter().filter(|file_ref| file_ref.category.contains(ReferenceCategory::WRITE)).map(
            move |file_ref| {
                NavigationTarget::from_syntax(
                    file_id.file_id(sema.db),
                    "assignment".into(),
                    Some(file_ref.range),
                    file_ref.range,
                    ide_db::SymbolKind::Local,
                )
            },
        )
    }));

    if targets.is_empty() {
        return None;
    }

    let range = sema
        .parse_guess_edition(position.file_id)
        .syntax()
        .token_at_offset(position.offset)
        .next()
        .map(|token| token.text_range())?;

    Some(RangeInfo::new(range, targets))
}

#[cfg(test)]
mod tests {
    use ide_db::FileRange;
    use itertools::Itertools;

    use crate::fixture;

    fn check(#[rust_analyzer::rust_fixture] ra_fixture: &str) {
        let (analysis, position, expected) = fixture::annotations(ra_fixture);
        let navs = analysis.goto_assignments(position).unwrap().expect("no assignments found").info;
        if navs.is_empty() {
            panic!("unresolved reference")
        }

        let cmp = |&FileRange { file_id, range }: &_| (file_id, range.start());
        let navs = navs
            .into_iter()
            .map(|nav| FileRange { file_id: nav.file_id, range: nav.focus_or_full_range() })
            .sorted_by_key(cmp)
            .collect::<Vec<_>>();
        let expected = expected
            .into_iter()
            .map(|(FileRange { file_id, range }, _)| FileRange { file_id, range })
            .sorted_by_key(cmp)
            .collect::<Vec<_>>();
        assert_eq!(expected, navs);
    }

    #[test]
    fn goto_assignments_reassignments() {
        check(
            r#"
//- /main.rs
fn main() {
    let mut a = 0;
    let mut x = 1;
         // ^
    x$0 = 2;
 // ^
    println!("{}", x);
    x = 3;
 // ^
}
"#,
        )
    }

    #[test]
    fn goto_assignments_compound_operators() {
        check(
            r#"
//- /main.rs
fn main() {
    let mut x = 10;
         // ^
    x += 5;
 // ^
    x$0 *= 2;
 // ^
    println!("{}", x);
}
"#,
        )
    }

    #[test]
    fn goto_assignments_struct_field_mutation() {
        check(
            r#"
//- /main.rs
struct Point { x: i32, y: i32 }

fn main() {
    let mut p = Point { x: 0, y: 0 };
         // ^
    p$0 = Point { x: 10, y: 20 };
 // ^
    p.x = 5;  // This is not an assignment to `p` itself
    println!("{:?}", p);
}
"#,
        )
    }

    #[test]
    fn goto_assignments_immutable_variable() {
        // Immutable variables only have the initial definition, no assignments
        check(
            r#"
//- /main.rs
fn main() {
    let x$0 = 5;
     // ^
    println!("{}", x);
}
"#,
        );
    }

    #[test]
    fn goto_assignments_closure_capture() {
        check(
            r#"
//- /main.rs
fn main() {
    let mut x = 0;
         // ^
    let closure = |mut y: i32| {
        x$0 = 42;
     // ^
        y = 1;  // This is a different variable
    };
    closure(0);
}
"#,
        );
    }

    #[test]
    fn goto_assignments_loop_variable() {
        check(
            r#"
//- /main.rs
fn main() {
    for mut i in 0..3 {
         // ^
        if i > 1 {
            i$0 += 1;
         // ^
        }
    }
}
"#,
        );
    }

    #[test]
    fn goto_assignments_shadowing() {
        // Each `x` is a separate variable, so only assignments to the same binding
        check(
            r#"
//- /main.rs
fn main() {
    let mut x = 1;
    let mut x = 2;  // Different variable (shadowing)
         // ^
    x$0 = 3;
 // ^
    println!("{}", x);
}
"#,
        );
    }
}
