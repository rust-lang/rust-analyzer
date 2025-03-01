use hir::Semantics;
use ide_db::{FilePosition, RootDatabase};
use syntax::{
    algo::find_node_at_offset,
    ast::{self, AstNode},
};

use crate::NavigationTarget;

// Feature: Children Module
//
// Navigates to the children modules of the current module.
//
// | Editor  | Action Name |
// |---------|-------------|
// | VS Code | **rust-analyzer: Locate children module** |
//
// ![Children Module]()

/// This returns `Vec` because a module may be included from several places.
pub(crate) fn children_module(db: &RootDatabase, position: FilePosition) -> Vec<NavigationTarget> {
    let sema = Semantics::new(db);
    let source_file = sema.parse_guess_edition(position.file_id);
    // First go to the parent module which contains the cursor
    let mut module = find_node_at_offset::<ast::Module>(source_file.syntax(), position.offset);

    // If cursor is literally on `mod foo`, go to the grandpa.
    if let Some(m) = &module {
        if !m
            .item_list()
            .is_some_and(|it| it.syntax().text_range().contains_inclusive(position.offset))
        {
            cov_mark::hit!(test_resolve_parent_module_on_module_decl);
            module = m.syntax().ancestors().skip(1).find_map(ast::Module::cast);
        }
    }

    match module {
        Some(module) => {
            // Return all the children module inside the ItemList of the parent module
            module
                .syntax()
                .children()
                .filter_map(ast::ItemList::cast)
                .flat_map(|it| it.syntax().children())
                .filter_map(ast::Module::cast)
                .flat_map(|m| {
                    sema.to_def(&m)
                        .into_iter()
                        .flat_map(|module| NavigationTarget::from_module_to_decl(db, module))
                })
                .collect()
        }
        None => {
            // Return all the children module inside the source file
            source_file
                .syntax()
                .children()
                .filter_map(ast::Module::cast)
                .flat_map(|m| {
                    sema.to_def(&m)
                        .into_iter()
                        .flat_map(|module| NavigationTarget::from_module_to_decl(db, module))
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use ide_db::FileRange;

    use crate::fixture;

    fn check_children_module(#[rust_analyzer::rust_fixture] ra_fixture: &str) {
        let (analysis, position, expected) = fixture::annotations(ra_fixture);
        let navs = analysis.children_module(position).unwrap();
        let navs = navs
            .iter()
            .map(|nav| FileRange { file_id: nav.file_id, range: nav.focus_or_full_range() })
            .collect::<Vec<_>>();
        assert_eq!(expected.into_iter().map(|(fr, _)| fr).collect::<Vec<_>>(), navs);
    }

    #[test]
    fn test_resolve_children_module() {
        check_children_module(
            r#"
//- /lib.rs
$0
mod foo;
  //^^^

//- /foo.rs
// empty
"#,
        );
    }

    #[test]
    fn test_resolve_children_module_on_module_decl() {
        check_children_module(
            r#"
//- /lib.rs
mod $0foo;
  //^^^
//- /foo.rs
mod bar;

//- /foo/bar.rs
// empty
"#,
        );
    }

    #[test]
    fn test_resolve_children_module_for_inline() {
        check_children_module(
            r#"
//- /lib.rs
mod foo {
    mod bar {
        mod $0baz {}
    }     //^^^
}
"#,
        );
    }

    #[test]
    fn test_resolve_multi_child_module() {
        check_children_module(
            r#"
//- /main.rs
$0
mod foo;
  //^^^
mod bar;
  //^^^
//- /foo.rs
//- /bar.rs
"#,
        );
    }
}
