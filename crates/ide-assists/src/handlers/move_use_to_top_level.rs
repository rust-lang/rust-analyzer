use crate::assist_context::{AssistContext, Assists};
use ide_db::assists::AssistId;
use ide_db::imports::insert_use::{ImportScope, ImportScopeKind};
use ide_db::source_change::SourceChangeBuilder;
use syntax::{
    AstNode, AstToken,
    ast::{
        BlockExpr, HasModuleItem, Item, Module, SourceFile, Use, Whitespace, edit_in_place::Indent,
        make,
    },
    ted,
};

// Assist: move_use_to_top_level
//
// Moves a use statement from a nested scope to the top level.
//
// ```
// fn main() {
//     use std::collections::HashMap$0;
//     let map = HashMap::new();
// }
// ```
// ->
// ```
// use std::collections::HashMap;
//
// fn main() {
//     let map = HashMap::new();
// }
// ```
pub(crate) fn move_use_to_top_level(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let use_item = ctx.find_node_at_offset::<Use>()?;

    if !is_use_item_movable_to_top(&use_item) {
        return None;
    }

    let scope = ImportScope::find_insert_use_container(use_item.syntax(), &ctx.sema)?;

    acc.add(
        AssistId::refactor_rewrite("move_use_to_top_level"),
        "Move use statement to top-level",
        use_item.syntax().text_range(),
        |builder| {
            move_use_to_top(&builder.make_import_scope_mut(scope), &use_item);
            cleanup_original_location(builder, &use_item);
        },
    )
}

fn is_use_item_movable_to_top(use_item: &Use) -> bool {
    use_item.syntax().ancestors().any(|ancestor| {
        BlockExpr::cast(ancestor.clone()).is_some()
            && !ancestor.ancestors().any(|a| Module::cast(a).is_some())
    })
}

fn move_use_to_top(scope: &ImportScope, use_item: &Use) {
    let use_item = use_item.clone_for_update();
    remove_indents_from_use_item(&use_item);
    insert_use_item_at_top(scope, &use_item);
}

fn remove_indents_from_use_item(use_item: &Use) {
    use_item.dedent(use_item.indent_level());
}

fn insert_use_item_at_top(scope: &ImportScope, use_item: &Use) {
    let source_file: SourceFile = extract_source_file_from_scope(scope);
    let last_top_level_use: Option<Item> = find_last_top_level_use(&source_file);

    match last_top_level_use {
        Some(last_use) => insert_after_existing_use(&last_use, use_item),
        None => insert_at_file_beginning(&source_file, use_item),
    }
}

fn extract_source_file_from_scope(scope: &ImportScope) -> SourceFile {
    match &scope.kind {
        ImportScopeKind::File(file) => file.clone(),
        ImportScopeKind::Module(module) => module
            .syntax()
            .ancestors()
            .find_map(SourceFile::cast)
            .expect("Module must be inside a source file"),
        ImportScopeKind::Block(block) => block
            .syntax()
            .ancestors()
            .find_map(SourceFile::cast)
            .expect("Block must be inside a source file"),
    }
}

fn find_last_top_level_use(source_file: &SourceFile) -> Option<Item> {
    source_file.items().take_while(|item| Use::cast(item.syntax().clone()).is_some()).last()
}

fn insert_after_existing_use(last_use: &Item, use_item: &Use) {
    ted::insert_raw(ted::Position::after(last_use.syntax()), use_item.syntax());
    ted::insert(ted::Position::before(use_item.syntax()), make::tokens::whitespace("\n"));
}

fn insert_at_file_beginning(source_file: &SourceFile, use_item: &Use) {
    if let Some(first_item) = source_file.items().next() {
        ted::insert_raw(ted::Position::before(first_item.syntax()), use_item.syntax());
        ted::insert(ted::Position::after(use_item.syntax()), make::tokens::whitespace("\n\n"));
    }
}

fn cleanup_original_location(builder: &mut SourceChangeBuilder, use_item: &Use) {
    builder.delete(use_item.syntax().text_range());

    // Clean up any trailing whitespace after the removed use statement
    if let Some(whitespace) = use_item
        .syntax()
        .next_sibling_or_token()
        .and_then(|token| token.into_token())
        .and_then(Whitespace::cast)
    {
        if whitespace.syntax().text().starts_with('\n') {
            builder.delete(whitespace.syntax().text_range());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable, check_assist_target};

    use super::*;

    // 1. Basic functionality
    #[test]
    fn test_move_use_from_function() {
        check_assist(
            move_use_to_top_level,
            r#"
fn main() {
    use std::collections::HashMap$0;
    let map = HashMap::new();
}
"#,
            r#"
use std::collections::HashMap;

fn main() {
    let map = HashMap::new();
}
"#,
        );
    }

    #[test]
    fn test_move_use_from_closure() {
        check_assist(
            move_use_to_top_level,
            r#"
fn main() {
    let closure = || {
        use std::collections::HashMap$0;
        HashMap::new()
    };
}
"#,
            r#"
use std::collections::HashMap;

fn main() {
    let closure = || {
        HashMap::new()
    };
}
"#,
        );
    }

    #[test]
    fn test_move_use_from_block() {
        check_assist(
            move_use_to_top_level,
            r#"
fn main() {
    {
        use std::collections::HashMap$0;
        let map = HashMap::new();
    }
}
"#,
            r#"
use std::collections::HashMap;

fn main() {
    {
        let map = HashMap::new();
    }
}
"#,
        );
    }

    // 2. Use statement variations
    #[test]
    fn test_move_use_with_rename() {
        check_assist(
            move_use_to_top_level,
            r#"
fn main() {
    use std::collections::HashMap as Map$0;
    let map = Map::new();
}
"#,
            r#"
use std::collections::HashMap as Map;

fn main() {
    let map = Map::new();
}
"#,
        );
    }

    #[test]
    fn test_move_use_with_glob() {
        check_assist(
            move_use_to_top_level,
            r#"
fn main() {
    use std::collections::*$0;
    let map = HashMap::new();
}
"#,
            r#"
use std::collections::*;

fn main() {
    let map = HashMap::new();
}
"#,
        );
    }

    #[test]
    fn test_move_use_with_grouped_imports() {
        check_assist(
            move_use_to_top_level,
            r#"
fn main() {
    use std::collections::{HashMap, Vec}$0;
    let map = HashMap::new();
}
"#,
            r#"
use std::collections::{HashMap, Vec};

fn main() {
    let map = HashMap::new();
}
"#,
        );
    }

    #[test]
    fn test_move_use_preserves_visibility() {
        check_assist(
            move_use_to_top_level,
            r#"
fn main() {
    pub use std::collections::HashMap$0;
    let map = HashMap::new();
}
"#,
            r#"
pub use std::collections::HashMap;

fn main() {
    let map = HashMap::new();
}
"#,
        );
    }

    #[test]
    fn test_move_use_with_attributes() {
        check_assist(
            move_use_to_top_level,
            r#"
fn main() {
    #[allow(unused)]
    use std::collections::HashMap$0;
    let map = HashMap::new();
}
"#,
            r#"
#[allow(unused)]
use std::collections::HashMap;

fn main() {
    let map = HashMap::new();
}
"#,
        );
    }

    #[test]
    fn test_move_use_with_comments() {
        check_assist(
            move_use_to_top_level,
            r#"
fn main() {
    // comment1
    // comment2
    use std::collections::HashMap$0;
    let map = HashMap::new();
}
"#,
            r#"
// comment1
// comment2
use std::collections::HashMap;

fn main() {
    let map = HashMap::new();
}
"#,
        );
    }

    // 3. Context variations
    #[test]
    fn test_move_use_with_existing_imports() {
        check_assist(
            move_use_to_top_level,
            r#"
use std::fmt::Debug;

fn main() {
    use std::collections::HashMap$0;
    let map = HashMap::new();
}
"#,
            r#"
use std::fmt::Debug;
use std::collections::HashMap;

fn main() {
    let map = HashMap::new();
}
"#,
        );
    }

    #[test]
    fn test_move_use_from_method() {
        check_assist(
            move_use_to_top_level,
            r#"
struct Foo;
impl Foo {
    fn bar(&self) {
        use std::collections::HashMap$0;
        let map = HashMap::new();
    }
}
"#,
            r#"
use std::collections::HashMap;

struct Foo;
impl Foo {
    fn bar(&self) {
        let map = HashMap::new();
    }
}
"#,
        );
    }

    #[test]
    fn test_move_use_from_trait_method() {
        check_assist(
            move_use_to_top_level,
            r#"
trait Foo {
    fn bar(&self) {
        use std::collections::HashMap$0;
        let map = HashMap::new();
    }
}
"#,
            r#"
use std::collections::HashMap;

trait Foo {
    fn bar(&self) {
        let map = HashMap::new();
    }
}
"#,
        );
    }

    #[test]
    fn test_move_use_ignores_other_local_uses() {
        check_assist(
            move_use_to_top_level,
            r#"
fn main() {
    use std::collections::HashMap$0;
    let map = HashMap::new();
    use std::fmt::Debug;
}
"#,
            r#"
use std::collections::HashMap;

fn main() {
    let map = HashMap::new();
    use std::fmt::Debug;
}
"#,
        );
    }

    #[test]
    fn test_move_use_with_comments_above() {
        check_assist(
            move_use_to_top_level,
            r#"
// comment

fn main() {
    use std::collections::HashMap$0;
    let map = HashMap::new();
}
"#,
            r#"
// comment

use std::collections::HashMap;

fn main() {
    let map = HashMap::new();
}
"#,
        );
    }

    // 4. Edge cases
    #[test]
    fn test_move_use_from_const() {
        check_assist(
            move_use_to_top_level,
            r#"
const FOO: usize = {
    use std::mem::size_of$0;
    size_of::<i32>()
};
"#,
            r#"
use std::mem::size_of;

const FOO: usize = {
    size_of::<i32>()
};
"#,
        );
    }

    #[test]
    fn test_move_use_from_static() {
        check_assist(
            move_use_to_top_level,
            r#"
static FOO: usize = {
    use std::mem::size_of$0;
    size_of::<i32>()
};
"#,
            r#"
use std::mem::size_of;

static FOO: usize = {
    size_of::<i32>()
};
"#,
        );
    }

    // 5. Test utilities
    #[test]
    fn test_target_range() {
        check_assist_target(
            move_use_to_top_level,
            r#"
fn main() {
    use std::collections::HashMap$0;
    let map = HashMap::new();
}
"#,
            "use std::collections::HashMap;",
        );
    }

    // 6. Negative tests - when assist should NOT be applicable
    #[test]
    fn test_not_applicable_when_use_already_at_top() {
        check_assist_not_applicable(
            move_use_to_top_level,
            r#"
use std::collections::HashMap$0;

fn main() {
    let map = HashMap::new();
}
"#,
        );
    }

    #[test]
    fn test_not_applicable_when_use_in_module_scope() {
        check_assist_not_applicable(
            move_use_to_top_level,
            r#"
mod foo {
    use std::collections::HashMap$0;

    fn bar() {
        let map = HashMap::new();
    }
}
"#,
        );
    }

    #[test]
    fn test_not_applicable_in_module_function() {
        check_assist_not_applicable(
            move_use_to_top_level,
            r#"
mod a {
    fn foo() {
        use std::collections::HashMap$0;
        let map = HashMap::new();
    }
}
"#,
        );
    }

    #[test]
    fn test_not_applicable_when_not_in_use_statement() {
        check_assist_not_applicable(
            move_use_to_top_level,
            r#"
fn main() {
    let map$0 = HashMap::new();
}
"#,
        );
    }
}
