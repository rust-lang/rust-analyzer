use base_db::SourceDatabase;
use expect_test::Expect;
use hir_def::{DefWithBodyId, ModuleDefId};
use salsa::EventKind;
use test_fixture::WithFixture;

use crate::{InferenceResult, method_resolution::TraitImpls, test_db::TestDB};

use super::visit_module;

#[test]
fn typing_whitespace_inside_a_function_should_not_invalidate_types() {
    let (mut db, pos) = TestDB::with_position(
        "
//- /lib.rs
fn foo() -> i32 {
    $01 + 1
}",
    );
    execute_assert_events(
        &db,
        || {
            let module = db.module_for_file(pos.file_id.file_id(&db));
            let crate_def_map = module.def_map(&db);
            visit_module(&db, crate_def_map, module, &mut |def| {
                if let ModuleDefId::FunctionId(it) = def {
                    InferenceResult::of(&db, DefWithBodyId::from(it));
                }
            });
        },
        &[("InferenceResult < 'db >::for_body_", 1)],
        expect_test::expect![[r#"
            [
                "source_root_crates",
                "crate_local_def_map",
                "file_item_tree_query",
                "HirFileId::ast_id_map_",
                "EditionedFileId::parse_",
                "real_span_map",
                "InferenceResult < 'db >::for_body_",
                "FunctionSignature::of_",
                "FunctionSignature::with_source_map_",
                "AttrFlags::query_",
                "Body::of_",
                "Body::with_source_map_",
                "trait_environment_query",
                "lang_items",
                "crate_lang_items",
                "GenericPredicates::query_with_diagnostics_",
                "fn_sig_for_fn",
                "ExprScopes::body_expr_scopes_",
                "body_upvars_mentioned",
            ]
        "#]],
    );

    let new_text = "
fn foo() -> i32 {
    1
    +
    1
}";

    db.set_file_text(pos.file_id.file_id(&db), new_text);

    execute_assert_events(
        &db,
        || {
            let module = db.module_for_file(pos.file_id.file_id(&db));
            let crate_def_map = module.def_map(&db);
            visit_module(&db, crate_def_map, module, &mut |def| {
                if let ModuleDefId::FunctionId(it) = def {
                    InferenceResult::of(&db, DefWithBodyId::from(it));
                }
            });
        },
        &[("InferenceResult < 'db >::for_body_", 0)],
        expect_test::expect![[r#"
            [
                "EditionedFileId::parse_",
                "HirFileId::ast_id_map_",
                "file_item_tree_query",
                "real_span_map",
                "AttrFlags::query_",
                "FunctionSignature::with_source_map_",
                "FunctionSignature::of_",
                "Body::with_source_map_",
                "Body::of_",
            ]
        "#]],
    );
}

#[test]
fn typing_inside_a_function_should_not_invalidate_types_in_another() {
    let (mut db, pos) = TestDB::with_position(
        "
//- /lib.rs
fn foo() -> f32 {
    1.0 + 2.0
}
fn bar() -> i32 {
    $01 + 1
}
fn baz() -> i32 {
    1 + 1
}",
    );
    execute_assert_events(
        &db,
        || {
            let module = db.module_for_file(pos.file_id.file_id(&db));
            let crate_def_map = module.def_map(&db);
            visit_module(&db, crate_def_map, module, &mut |def| {
                if let ModuleDefId::FunctionId(it) = def {
                    InferenceResult::of(&db, DefWithBodyId::from(it));
                }
            });
        },
        &[("InferenceResult < 'db >::for_body_", 3)],
        expect_test::expect![[r#"
            [
                "source_root_crates",
                "crate_local_def_map",
                "file_item_tree_query",
                "HirFileId::ast_id_map_",
                "EditionedFileId::parse_",
                "real_span_map",
                "InferenceResult < 'db >::for_body_",
                "FunctionSignature::of_",
                "FunctionSignature::with_source_map_",
                "AttrFlags::query_",
                "Body::of_",
                "Body::with_source_map_",
                "trait_environment_query",
                "lang_items",
                "crate_lang_items",
                "GenericPredicates::query_with_diagnostics_",
                "fn_sig_for_fn",
                "ExprScopes::body_expr_scopes_",
                "body_upvars_mentioned",
                "InferenceResult < 'db >::for_body_",
                "FunctionSignature::of_",
                "FunctionSignature::with_source_map_",
                "AttrFlags::query_",
                "Body::of_",
                "Body::with_source_map_",
                "trait_environment_query",
                "GenericPredicates::query_with_diagnostics_",
                "fn_sig_for_fn",
                "ExprScopes::body_expr_scopes_",
                "body_upvars_mentioned",
                "InferenceResult < 'db >::for_body_",
                "FunctionSignature::of_",
                "FunctionSignature::with_source_map_",
                "AttrFlags::query_",
                "Body::of_",
                "Body::with_source_map_",
                "trait_environment_query",
                "GenericPredicates::query_with_diagnostics_",
                "fn_sig_for_fn",
                "ExprScopes::body_expr_scopes_",
                "body_upvars_mentioned",
            ]
        "#]],
    );

    let new_text = "
fn foo() -> f32 {
    1.0 + 2.0
}
fn bar() -> i32 {
    53
}
fn baz() -> i32 {
    1 + 1
}
";

    db.set_file_text(pos.file_id.file_id(&db), new_text);

    execute_assert_events(
        &db,
        || {
            let module = db.module_for_file(pos.file_id.file_id(&db));
            let crate_def_map = module.def_map(&db);
            visit_module(&db, crate_def_map, module, &mut |def| {
                if let ModuleDefId::FunctionId(it) = def {
                    InferenceResult::of(&db, DefWithBodyId::from(it));
                }
            });
        },
        &[("InferenceResult < 'db >::for_body_", 1)],
        expect_test::expect![[r#"
            [
                "EditionedFileId::parse_",
                "HirFileId::ast_id_map_",
                "file_item_tree_query",
                "real_span_map",
                "AttrFlags::query_",
                "FunctionSignature::with_source_map_",
                "FunctionSignature::of_",
                "Body::with_source_map_",
                "Body::of_",
                "AttrFlags::query_",
                "FunctionSignature::with_source_map_",
                "FunctionSignature::of_",
                "Body::with_source_map_",
                "Body::of_",
                "InferenceResult < 'db >::for_body_",
                "ExprScopes::body_expr_scopes_",
                "body_upvars_mentioned",
                "AttrFlags::query_",
                "FunctionSignature::with_source_map_",
                "FunctionSignature::of_",
                "Body::with_source_map_",
                "Body::of_",
            ]
        "#]],
    );
}

#[test]
fn adding_struct_invalidates_infer() {
    let (mut db, pos) = TestDB::with_position(
        "
//- /lib.rs
fn foo() -> i32 {
    1 + 1
}

fn bar() -> f32 {
    2.0 * 3.0
}
$0",
    );
    execute_assert_events(
        &db,
        || {
            let module = db.module_for_file(pos.file_id.file_id(&db));
            let _crate_def_map = module.def_map(&db);
            TraitImpls::for_crate(&db, module.krate(&db));
        },
        &[("TraitImpls < 'db >::for_crate_", 1)],
        expect_test::expect![[r#"
            [
                "source_root_crates",
                "crate_local_def_map",
                "file_item_tree_query",
                "HirFileId::ast_id_map_",
                "EditionedFileId::parse_",
                "real_span_map",
                "TraitImpls < 'db >::for_crate_",
                "lang_items",
                "crate_lang_items",
            ]
        "#]],
    );

    let new_text = "
fn foo() -> i32 {
    1 + 1
}

fn bar() -> f32 {
    2.0 * 3.0
}

pub struct NewStruct {
    field: i32,
}
";

    db.set_file_text(pos.file_id.file_id(&db), new_text);

    execute_assert_events(
        &db,
        || {
            let module = db.module_for_file(pos.file_id.file_id(&db));
            let _crate_def_map = module.def_map(&db);
            TraitImpls::for_crate(&db, module.krate(&db));
        },
        &[("TraitImpls < 'db >::for_crate_", 1)],
        expect_test::expect![[r#"
            [
                "EditionedFileId::parse_",
                "HirFileId::ast_id_map_",
                "file_item_tree_query",
                "real_span_map",
                "crate_local_def_map",
                "TraitImpls < 'db >::for_crate_",
                "crate_lang_items",
            ]
        "#]],
    );
}

#[test]
fn adding_enum_query_log() {
    let (mut db, pos) = TestDB::with_position(
        "
//- /lib.rs
fn foo() -> i32 {
    1 + 1
}

fn bar() -> f32 {
    2.0 * 3.0
}
$0",
    );
    execute_assert_events(
        &db,
        || {
            let module = db.module_for_file(pos.file_id.file_id(&db));
            let _crate_def_map = module.def_map(&db);
            TraitImpls::for_crate(&db, module.krate(&db));
        },
        &[("TraitImpls < 'db >::for_crate_", 1)],
        expect_test::expect![[r#"
            [
                "source_root_crates",
                "crate_local_def_map",
                "file_item_tree_query",
                "HirFileId::ast_id_map_",
                "EditionedFileId::parse_",
                "real_span_map",
                "TraitImpls < 'db >::for_crate_",
                "lang_items",
                "crate_lang_items",
            ]
        "#]],
    );

    let new_text = "
fn foo() -> i32 {
    1 + 1
}

fn bar() -> f32 {
    2.0 * 3.0
}

pub enum SomeEnum {
    A,
    B
}
";

    db.set_file_text(pos.file_id.file_id(&db), new_text);

    execute_assert_events(
        &db,
        || {
            let module = db.module_for_file(pos.file_id.file_id(&db));
            let _crate_def_map = module.def_map(&db);
            TraitImpls::for_crate(&db, module.krate(&db));
        },
        &[("TraitImpls < 'db >::for_crate_", 1)],
        expect_test::expect![[r#"
            [
                "EditionedFileId::parse_",
                "HirFileId::ast_id_map_",
                "file_item_tree_query",
                "real_span_map",
                "crate_local_def_map",
                "TraitImpls < 'db >::for_crate_",
                "crate_lang_items",
            ]
        "#]],
    );
}

#[test]
fn adding_use_query_log() {
    let (mut db, pos) = TestDB::with_position(
        "
//- /lib.rs
fn foo() -> i32 {
    1 + 1
}

fn bar() -> f32 {
    2.0 * 3.0
}
$0",
    );
    execute_assert_events(
        &db,
        || {
            let module = db.module_for_file(pos.file_id.file_id(&db));
            let _crate_def_map = module.def_map(&db);
            TraitImpls::for_crate(&db, module.krate(&db));
        },
        &[("TraitImpls < 'db >::for_crate_", 1)],
        expect_test::expect![[r#"
            [
                "source_root_crates",
                "crate_local_def_map",
                "file_item_tree_query",
                "HirFileId::ast_id_map_",
                "EditionedFileId::parse_",
                "real_span_map",
                "TraitImpls < 'db >::for_crate_",
                "lang_items",
                "crate_lang_items",
            ]
        "#]],
    );

    let new_text = "
use std::collections::HashMap;

fn foo() -> i32 {
    1 + 1
}

fn bar() -> f32 {
    2.0 * 3.0
}
";

    db.set_file_text(pos.file_id.file_id(&db), new_text);

    execute_assert_events(
        &db,
        || {
            let module = db.module_for_file(pos.file_id.file_id(&db));
            let _crate_def_map = module.def_map(&db);
            TraitImpls::for_crate(&db, module.krate(&db));
        },
        &[("TraitImpls < 'db >::for_crate_", 1)],
        expect_test::expect![[r#"
            [
                "EditionedFileId::parse_",
                "HirFileId::ast_id_map_",
                "file_item_tree_query",
                "real_span_map",
                "crate_local_def_map",
                "TraitImpls < 'db >::for_crate_",
                "crate_lang_items",
            ]
        "#]],
    );
}

#[test]
fn adding_impl_query_log() {
    let (mut db, pos) = TestDB::with_position(
        "
//- /lib.rs
fn foo() -> i32 {
    1 + 1
}

fn bar() -> f32 {
    2.0 * 3.0
}

pub struct SomeStruct {
    field: i32,
}
$0",
    );
    execute_assert_events(
        &db,
        || {
            let module = db.module_for_file(pos.file_id.file_id(&db));
            let _crate_def_map = module.def_map(&db);
            TraitImpls::for_crate(&db, module.krate(&db));
        },
        &[("TraitImpls < 'db >::for_crate_", 1)],
        expect_test::expect![[r#"
            [
                "source_root_crates",
                "crate_local_def_map",
                "file_item_tree_query",
                "HirFileId::ast_id_map_",
                "EditionedFileId::parse_",
                "real_span_map",
                "TraitImpls < 'db >::for_crate_",
                "lang_items",
                "crate_lang_items",
            ]
        "#]],
    );

    let new_text = "
fn foo() -> i32 {
    1 + 1
}

fn bar() -> f32 {
    2.0 * 3.0
}

pub struct SomeStruct {
    field: i32,
}

impl SomeStruct {
    pub fn new(value: i32) -> Self {
        Self { field: value }
    }
}
";

    db.set_file_text(pos.file_id.file_id(&db), new_text);

    execute_assert_events(
        &db,
        || {
            let module = db.module_for_file(pos.file_id.file_id(&db));
            let _crate_def_map = module.def_map(&db);
            TraitImpls::for_crate(&db, module.krate(&db));
        },
        &[("TraitImpls < 'db >::for_crate_", 1)],
        expect_test::expect![[r#"
            [
                "EditionedFileId::parse_",
                "HirFileId::ast_id_map_",
                "file_item_tree_query",
                "real_span_map",
                "crate_local_def_map",
                "TraitImpls < 'db >::for_crate_",
                "crate_lang_items",
            ]
        "#]],
    );
}

#[test]
fn changing_recursion_limit_invalidates_inferred_types() {
    use hir_def::{expr_store::Body, hir::Expr};

    use crate::display::{DisplayTarget, HirDisplay};

    let source = r#"
#![recursion_limit = "16"]
trait Decode<O> {}
impl Decode<u32> for () {}
impl<T: Decode<O>, O> Decode<O> for (T,) {}
type Eight<T> = ((((((((T,),),),),),),),);
type Deep = Eight<Eight<Eight<Eight<Eight<Eight<Eight<Eight<()>>>>>>>>;
fn decode<T: Decode<O>, O>(_: T) -> O { loop {} }
fn test(value: Deep) { decode(value); }
"#;
    let (mut db, file_id) = TestDB::with_single_file(source);
    let inferred_type = |db: &TestDB| {
        crate::attach_db(db, || {
            let module = db.module_for_file(file_id.file_id(db));
            let display_target = DisplayTarget::from_crate(db, module.krate(db));
            let mut actual = None;
            visit_module(db, module.def_map(db), module, &mut |def| {
                if let ModuleDefId::FunctionId(function) = def {
                    let def = DefWithBodyId::FunctionId(function);
                    let body = Body::of(db, def);
                    let result = InferenceResult::of(db, def);
                    for (expr, ty) in result.type_of_expr.iter() {
                        if matches!(body[expr], Expr::Call { .. }) {
                            actual = Some(ty.as_ref().display_test(db, display_target).to_string());
                        }
                    }
                }
            });
            actual.expect("the fixture contains a call")
        })
    };

    assert_eq!(inferred_type(&db), "{unknown}");
    db.set_file_text(file_id.file_id(&db), &source.replace("\"16\"", "\"128\""));
    assert_eq!(inferred_type(&db), "u32");
    db.set_file_text(file_id.file_id(&db), source);
    assert_eq!(inferred_type(&db), "{unknown}");
}

fn execute_assert_events(
    db: &TestDB,
    f: impl FnOnce(),
    required: &[(&str, usize)],
    expect: Expect,
) {
    crate::attach_db(db, || {
        let (executed, events) = db.log_executed(f);
        expect.assert_debug_eq(&executed);
        for (event, count) in required {
            let n = executed.iter().filter(|it| it.contains(event)).count();
            assert_eq!(
                n,
                *count,
                "Expected {event} to be executed {count} times, but only got {n}:\n \
             Executed: {executed:#?}\n \
             Event log: {events:#?}",
                events = events
                    .iter()
                    .filter(|event| !matches!(event.kind, EventKind::WillCheckCancellation))
                    .map(|event| { format!("{:?}", event.kind) })
                    .collect::<Vec<_>>(),
            );
        }
    });
}
