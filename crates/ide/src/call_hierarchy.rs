//! Entry point for call-hierarchy

use hir::{HirFileRange, Semantics};
use ide_db::{
    FxIndexMap, RootDatabase,
    defs::{Definition, NameClass, NameRefClass},
    helpers::pick_best_token,
    search::FileReference,
};
use syntax::{AstNode, SyntaxKind::IDENT, ast};

use crate::{
    HirFilePosition, RangeInfo, TryToNav, goto_definition, navigation_target::HirNavigationTarget,
};

#[derive(Debug, Clone)]
pub struct CallItem {
    pub target: HirNavigationTarget,
    pub ranges: Vec<HirFileRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallHierarchyConfig {
    /// Whether to exclude tests from the call hierarchy
    pub exclude_tests: bool,
}

pub(crate) fn call_hierarchy(
    db: &RootDatabase,
    position: HirFilePosition,
) -> Option<RangeInfo<Vec<HirNavigationTarget>>> {
    goto_definition::goto_definition(db, position)
}

pub(crate) fn incoming_calls(
    db: &RootDatabase,
    CallHierarchyConfig { exclude_tests }: CallHierarchyConfig,
    HirFilePosition { file_id, offset }: HirFilePosition,
) -> Option<Vec<CallItem>> {
    let sema = &Semantics::new(db);

    let file = &sema.parse_or_expand(sema.adjust_edition(file_id));
    let mut calls = CallLocations::default();

    let references = sema
        .find_nodes_at_offset_with_descend(file, offset)
        .filter_map(move |node| match node {
            ast::NameLike::NameRef(name_ref) => match NameRefClass::classify(sema, &name_ref)? {
                NameRefClass::Definition(def @ Definition::Function(_), _) => Some(def),
                _ => None,
            },
            ast::NameLike::Name(name) => match NameClass::classify(sema, &name)? {
                NameClass::Definition(def @ Definition::Function(_)) => Some(def),
                _ => None,
            },
            ast::NameLike::Lifetime(_) => None,
        })
        .flat_map(|func| func.usages(sema).all());

    for (file_id, references) in references {
        let references =
            references.iter().filter_map(|FileReference { name, .. }| name.as_name_ref());
        for name in references {
            // This target is the containing function
            let def_nav = sema.ancestors_with_macros(name.syntax().clone()).find_map(|node| {
                let def = ast::Fn::cast(node).and_then(|fn_| sema.to_def(&fn_))?;
                // We should return def before check if it is a test, so that we
                // will not continue to search for outer fn in nested fns
                def.try_to_nav_hir(sema.db).map(|nav| (def, nav))
            });

            if let Some((def, nav)) = def_nav {
                if exclude_tests && def.is_test(db) {
                    continue;
                }

                let range = HirFileRange { file_id, range: name.syntax().text_range() };
                calls.add(nav, range);
            }
        }
    }

    Some(calls.into_items())
}

pub(crate) fn outgoing_calls(
    db: &RootDatabase,
    CallHierarchyConfig { exclude_tests }: CallHierarchyConfig,
    HirFilePosition { file_id, offset }: HirFilePosition,
) -> Option<Vec<CallItem>> {
    let sema = Semantics::new(db);
    let file = &sema.parse_or_expand(sema.adjust_edition(file_id));
    let token = pick_best_token(file.token_at_offset(offset), |kind| match kind {
        IDENT => 1,
        _ => 0,
    })?;
    let mut calls = CallLocations::default();

    sema.descend_into_macros_exact(token)
        .into_iter()
        .filter_map(|it| it.parent_ancestors().nth(1).and_then(ast::Item::cast))
        .filter_map(|item| match item {
            ast::Item::Const(c) => c.body().map(|it| it.syntax().descendants()),
            ast::Item::Fn(f) => f.body().map(|it| it.syntax().descendants()),
            ast::Item::Static(s) => s.body().map(|it| it.syntax().descendants()),
            _ => None,
        })
        .flatten()
        .filter_map(ast::CallableExpr::cast)
        .filter_map(|call_node| {
            let (nav_target, range) = match call_node {
                ast::CallableExpr::Call(call) => {
                    let expr = call.expr()?;
                    let callable = sema.type_of_expr(&expr)?.original.as_callable(db)?;
                    match callable.kind() {
                        hir::CallableKind::Function(it) => {
                            if exclude_tests && it.is_test(db) {
                                return None;
                            }
                            it.try_to_nav_hir(db)
                        }
                        hir::CallableKind::TupleEnumVariant(it) => it.try_to_nav_hir(db),
                        hir::CallableKind::TupleStruct(it) => it.try_to_nav_hir(db),
                        _ => None,
                    }
                    .zip(Some(HirFileRange { file_id, range: expr.syntax().text_range() }))
                }
                ast::CallableExpr::MethodCall(expr) => {
                    let function = sema.resolve_method_call(&expr)?;
                    if exclude_tests && function.is_test(db) {
                        return None;
                    }
                    function.try_to_nav_hir(db).zip(Some(HirFileRange {
                        file_id,
                        range: expr.name_ref()?.syntax().text_range(),
                    }))
                }
            }?;
            Some((nav_target, range))
        })
        .for_each(|(nav, range)| calls.add(nav, range));

    Some(calls.into_items())
}

#[derive(Default)]
struct CallLocations {
    funcs: FxIndexMap<HirNavigationTarget, Vec<HirFileRange>>,
}

impl CallLocations {
    fn add(&mut self, target: HirNavigationTarget, range: HirFileRange) {
        self.funcs.entry(target).or_default().push(range);
    }

    fn into_items(self) -> Vec<CallItem> {
        self.funcs.into_iter().map(|(target, ranges)| CallItem { target, ranges }).collect()
    }
}

#[cfg(test)]
mod tests {
    use expect_test::{Expect, expect};
    use hir::HirFilePosition;
    use itertools::Itertools;

    use crate::fixture;

    fn check_hierarchy(
        exclude_tests: bool,
        #[rust_analyzer::rust_fixture] ra_fixture: &str,
        expected_nav: Expect,
        expected_incoming: Expect,
        expected_outgoing: Expect,
    ) {
        fn debug_render(item: crate::CallItem) -> String {
            format!(
                "{} : {}",
                item.target.debug_render(),
                item.ranges.iter().format_with(", ", |range, f| f(&format_args!(
                    "{:?}:{:?}",
                    range.file_id, range.range
                )))
            )
        }

        let (analysis, pos) = fixture::position(ra_fixture);

        let mut navs = analysis.call_hierarchy(pos.into()).unwrap().unwrap().info;
        assert_eq!(navs.len(), 1);
        let nav = navs.pop().unwrap();
        expected_nav.assert_eq(&nav.debug_render());

        let config = crate::CallHierarchyConfig { exclude_tests };

        let item_pos =
            HirFilePosition { file_id: nav.file_id, offset: nav.focus_or_full_range().start() };
        let incoming_calls = analysis.incoming_calls(config, item_pos).unwrap().unwrap();
        expected_incoming.assert_eq(&incoming_calls.into_iter().map(debug_render).join("\n"));

        let outgoing_calls = analysis.outgoing_calls(config, item_pos).unwrap().unwrap();
        expected_outgoing.assert_eq(&outgoing_calls.into_iter().map(debug_render).join("\n"));
    }

    #[test]
    fn test_call_hierarchy_on_ref() {
        check_hierarchy(
            false,
            r#"
//- /lib.rs
fn callee() {}
fn caller() {
    call$0ee();
}
"#,
            expect!["callee Function FileId(EditionedFileId(Id(1800))) 0..14 3..9"],
            expect![
                "caller Function FileId(EditionedFileId(Id(1800))) 15..44 18..24 : FileId(EditionedFileId(Id(1800))):33..39"
            ],
            expect![],
        );
    }

    #[test]
    fn test_call_hierarchy_on_def() {
        check_hierarchy(
            false,
            r#"
//- /lib.rs
fn call$0ee() {}
fn caller() {
    callee();
}
"#,
            expect!["callee Function FileId(EditionedFileId(Id(1800))) 0..14 3..9"],
            expect![
                "caller Function FileId(EditionedFileId(Id(1800))) 15..44 18..24 : FileId(EditionedFileId(Id(1800))):33..39"
            ],
            expect![],
        );
    }

    #[test]
    fn test_call_hierarchy_in_same_fn() {
        check_hierarchy(
            false,
            r#"
//- /lib.rs
fn callee() {}
fn caller() {
    call$0ee();
    callee();
}
"#,
            expect!["callee Function FileId(EditionedFileId(Id(1800))) 0..14 3..9"],
            expect![
                "caller Function FileId(EditionedFileId(Id(1800))) 15..58 18..24 : FileId(EditionedFileId(Id(1800))):33..39, FileId(EditionedFileId(Id(1800))):47..53"
            ],
            expect![],
        );
    }

    #[test]
    fn test_call_hierarchy_in_different_fn() {
        check_hierarchy(
            false,
            r#"
//- /lib.rs
fn callee() {}
fn caller1() {
    call$0ee();
}

fn caller2() {
    callee();
}
"#,
            expect!["callee Function FileId(EditionedFileId(Id(1800))) 0..14 3..9"],
            expect![[r#"
                caller1 Function FileId(EditionedFileId(Id(1800))) 15..45 18..25 : FileId(EditionedFileId(Id(1800))):34..40
                caller2 Function FileId(EditionedFileId(Id(1800))) 47..77 50..57 : FileId(EditionedFileId(Id(1800))):66..72"#]],
            expect![],
        );
    }

    #[test]
    fn test_call_hierarchy_in_tests_mod() {
        check_hierarchy(
            false,
            r#"
//- /lib.rs cfg:test
fn callee() {}
fn caller1() {
    call$0ee();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caller() {
        callee();
    }
}
"#,
            expect!["callee Function FileId(EditionedFileId(Id(1800))) 0..14 3..9"],
            expect![[r#"
                caller1 Function FileId(EditionedFileId(Id(1800))) 15..45 18..25 : FileId(EditionedFileId(Id(1800))):34..40
                test_caller Function FileId(EditionedFileId(Id(1800))) 95..149 110..121 tests : FileId(EditionedFileId(Id(1800))):134..140"#]],
            expect![],
        );
    }

    #[test]
    fn test_call_hierarchy_in_different_files() {
        check_hierarchy(
            false,
            r#"
//- /lib.rs
mod foo;
use foo::callee;

fn caller() {
    call$0ee();
}

//- /foo/mod.rs
pub fn callee() {}
"#,
            expect!["callee Function FileId(EditionedFileId(Id(1801))) 0..18 7..13 foo"],
            expect![
                "caller Function FileId(EditionedFileId(Id(1800))) 27..56 30..36 : FileId(EditionedFileId(Id(1800))):45..51"
            ],
            expect![],
        );
    }

    #[test]
    fn test_call_hierarchy_outgoing() {
        check_hierarchy(
            false,
            r#"
//- /lib.rs
fn callee() {}
fn call$0er() {
    callee();
    callee();
}
"#,
            expect!["caller Function FileId(EditionedFileId(Id(1800))) 15..58 18..24"],
            expect![],
            expect![
                "callee Function FileId(EditionedFileId(Id(1800))) 0..14 3..9 : FileId(EditionedFileId(Id(1800))):33..39, FileId(EditionedFileId(Id(1800))):47..53"
            ],
        );
    }

    #[test]
    fn test_call_hierarchy_outgoing_in_different_files() {
        check_hierarchy(
            false,
            r#"
//- /lib.rs
mod foo;
use foo::callee;

fn call$0er() {
    callee();
}

//- /foo/mod.rs
pub fn callee() {}
"#,
            expect!["caller Function FileId(EditionedFileId(Id(1800))) 27..56 30..36"],
            expect![],
            expect!["callee Function FileId(EditionedFileId(Id(1801))) 0..18 7..13 foo : FileId(EditionedFileId(Id(1800))):45..51"],
        );
    }

    #[test]
    fn test_call_hierarchy_incoming_outgoing() {
        check_hierarchy(
            false,
            r#"
//- /lib.rs
fn caller1() {
    call$0er2();
}

fn caller2() {
    caller3();
}

fn caller3() {

}
"#,
            expect!["caller2 Function FileId(EditionedFileId(Id(1800))) 33..64 36..43"],
            expect![
                "caller1 Function FileId(EditionedFileId(Id(1800))) 0..31 3..10 : FileId(EditionedFileId(Id(1800))):19..26"
            ],
            expect![
                "caller3 Function FileId(EditionedFileId(Id(1800))) 66..83 69..76 : FileId(EditionedFileId(Id(1800))):52..59"
            ],
        );
    }

    #[test]
    fn test_call_hierarchy_issue_5103() {
        check_hierarchy(
            false,
            r#"
fn a() {
    b()
}

fn b() {}

fn main() {
    a$0()
}
"#,
            expect!["a Function FileId(EditionedFileId(Id(1800))) 0..18 3..4"],
            expect![
                "main Function FileId(EditionedFileId(Id(1800))) 31..52 34..38 : FileId(EditionedFileId(Id(1800))):47..48"
            ],
            expect![
                "b Function FileId(EditionedFileId(Id(1800))) 20..29 23..24 : FileId(EditionedFileId(Id(1800))):13..14"
            ],
        );

        check_hierarchy(
            false,
            r#"
fn a() {
    b$0()
}

fn b() {}

fn main() {
    a()
}
"#,
            expect!["b Function FileId(EditionedFileId(Id(1800))) 20..29 23..24"],
            expect![
                "a Function FileId(EditionedFileId(Id(1800))) 0..18 3..4 : FileId(EditionedFileId(Id(1800))):13..14"
            ],
            expect![],
        );
    }

    #[test]
    fn test_call_hierarchy_in_macros_incoming() {
        check_hierarchy(
            false,
            r#"
macro_rules! define {
    ($ident:ident) => {
        fn $ident {}
    }
}
macro_rules! call {
    ($ident:ident) => {
        $ident()
    }
}
define!(callee)
fn caller() {
    call!(call$0ee);
}
"#,
            expect!["callee Function MacroFile(MacroCallId(Id(3400))) 0..10 2..8"],
            expect![
                "caller Function FileId(EditionedFileId(Id(1800))) 160..194 163..169 : MacroFile(MacroCallId(Id(3401))):0..6"
            ],
            expect![],
        );
        check_hierarchy(
            false,
            r#"
macro_rules! define {
    ($ident:ident) => {
        fn $ident {}
    }
}
macro_rules! call {
    ($ident:ident) => {
        $ident()
    }
}
define!(cal$0lee)
fn caller() {
    call!(callee);
}
"#,
            expect!["callee Function MacroFile(MacroCallId(Id(3400))) 0..10 2..8"],
            expect![
                "caller Function FileId(EditionedFileId(Id(1800))) 160..194 163..169 : MacroFile(MacroCallId(Id(3401))):0..6"
            ],
            expect![],
        );
    }

    #[test]
    fn test_call_hierarchy_in_macros_outgoing() {
        check_hierarchy(
            false,
            r#"
macro_rules! define {
    ($ident:ident) => {
        fn $ident {}
    }
}
macro_rules! call {
    ($ident:ident) => {
        $ident()
    }
}
define!(callee)
fn caller$0() {
    call!(callee);
}
"#,
            expect!["caller Function FileId(EditionedFileId(Id(1800))) 160..194 163..169"],
            expect![],
            // FIXME
            expect![],
        );
    }

    #[test]
    fn test_call_hierarchy_in_macros_incoming_different_files() {
        check_hierarchy(
            false,
            r#"
//- /lib.rs
#[macro_use]
mod foo;
define!(callee)
fn caller() {
    call!(call$0ee);
}
//- /foo.rs
macro_rules! define {
    ($ident:ident) => {
        fn $ident {}
    }
}
macro_rules! call {
    ($ident:ident) => {
        $ident()
    }
}
"#,
            expect!["callee Function MacroFile(MacroCallId(Id(3400))) 0..10 2..8"],
            expect![
                "caller Function FileId(EditionedFileId(Id(1800))) 38..72 41..47 : MacroFile(MacroCallId(Id(3401))):0..6"
            ],
            expect![],
        );
        check_hierarchy(
            false,
            r#"
//- /lib.rs
#[macro_use]
mod foo;
define!(cal$0lee)
fn caller() {
    call!(callee);
}
//- /foo.rs
macro_rules! define {
    ($ident:ident) => {
        fn $ident {}
    }
}
macro_rules! call {
    ($ident:ident) => {
        $ident()
    }
}
"#,
            expect!["callee Function MacroFile(MacroCallId(Id(3400))) 0..10 2..8"],
            expect![
                "caller Function FileId(EditionedFileId(Id(1800))) 38..72 41..47 : MacroFile(MacroCallId(Id(3401))):0..6"
            ],
            expect![],
        );
        check_hierarchy(
            false,
            r#"
//- /lib.rs
#[macro_use]
mod foo;
define!(cal$0lee)
call!(callee);
//- /foo.rs
macro_rules! define {
    ($ident:ident) => {
        fn $ident {}
    }
}
macro_rules! call {
    ($ident:ident) => {
        fn caller() {
            $ident()
        }
        fn $ident() {
            $ident()
        }
    }
}
"#,
            expect!["callee Function MacroFile(MacroCallId(Id(3400))) 0..10 2..8"],
            expect![[r#"
                caller Function MacroFile(MacroCallId(Id(3401))) 0..20 2..8 : MacroFile(MacroCallId(Id(3401))):11..17
                callee Function MacroFile(MacroCallId(Id(3401))) 20..40 22..28 : MacroFile(MacroCallId(Id(3401))):31..37"#]],
            expect![],
        );
    }

    #[test]
    fn test_call_hierarchy_in_macros_outgoing_different_files() {
        check_hierarchy(
            false,
            r#"
//- /lib.rs
#[macro_use]
mod foo;
define!(callee)
fn caller$0() {
    call!(callee);
}
//- /foo.rs
macro_rules! define {
    ($ident:ident) => {
        fn $ident {}
    }
}
macro_rules! call {
    ($ident:ident) => {
        $ident()
        callee()
    }
}
"#,
            expect!["caller Function FileId(EditionedFileId(Id(1800))) 38..72 41..47"],
            expect![],
            // FIXME
            expect![],
        );
        check_hierarchy(
            false,
            r#"
//- /lib.rs
#[macro_use]
mod foo;
define!(callee)
fn caller$0() {
    call!(callee);
}
//- /foo.rs
macro_rules! define {
    () => {
        fn callee {}
    }
}
macro_rules! call {
    ($ident:ident) => {
        $ident()
        callee()
    }
}
"#,
            expect!["caller Function FileId(EditionedFileId(Id(1800))) 38..72 41..47"],
            expect![],
            // FIXME
            expect![],
        );
    }

    #[test]
    fn test_trait_method_call_hierarchy() {
        check_hierarchy(
            false,
            r#"
trait T1 {
    fn call$0ee();
}

struct S1;

impl T1 for S1 {
    fn callee() {}
}

fn caller() {
    S1::callee();
}
"#,
            expect!["callee Function FileId(EditionedFileId(Id(1800))) 15..27 18..24 T1"],
            expect![
                "caller Function FileId(EditionedFileId(Id(1800))) 82..115 85..91 : FileId(EditionedFileId(Id(1800))):104..110"
            ],
            expect![],
        );
    }

    #[test]
    fn test_call_hierarchy_excluding_tests() {
        check_hierarchy(
            false,
            r#"
fn main() {
    f1();
}

fn f1$0() {
    f2(); f3();
}

fn f2() {
    f1(); f3();
}

#[test]
fn f3() {
    f1(); f2();
}
"#,
            expect!["f1 Function FileId(EditionedFileId(Id(1800))) 25..52 28..30"],
            expect![
                r#"
                main Function FileId(EditionedFileId(Id(1800))) 0..23 3..7 : FileId(EditionedFileId(Id(1800))):16..18
                f2 Function FileId(EditionedFileId(Id(1800))) 54..81 57..59 : FileId(EditionedFileId(Id(1800))):68..70
                f3 Function FileId(EditionedFileId(Id(1800))) 83..118 94..96 : FileId(EditionedFileId(Id(1800))):105..107"#
            ],
            expect![
                r#"
                f2 Function FileId(EditionedFileId(Id(1800))) 54..81 57..59 : FileId(EditionedFileId(Id(1800))):39..41
                f3 Function FileId(EditionedFileId(Id(1800))) 83..118 94..96 : FileId(EditionedFileId(Id(1800))):45..47"#
            ],
        );

        check_hierarchy(
            true,
            r#"
fn main() {
    f1();
}

fn f1$0() {
    f2(); f3();
}

fn f2() {
    f1(); f3();
}

#[test]
fn f3() {
    f1(); f2();
}
"#,
            expect!["f1 Function FileId(EditionedFileId(Id(1800))) 25..52 28..30"],
            expect![[r#"
                main Function FileId(EditionedFileId(Id(1800))) 0..23 3..7 : FileId(EditionedFileId(Id(1800))):16..18
                f2 Function FileId(EditionedFileId(Id(1800))) 54..81 57..59 : FileId(EditionedFileId(Id(1800))):68..70"#]],
            expect![
                "f2 Function FileId(EditionedFileId(Id(1800))) 54..81 57..59 : FileId(EditionedFileId(Id(1800))):39..41"
            ],
        );
    }
}
