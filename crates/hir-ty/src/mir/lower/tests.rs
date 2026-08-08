use test_fixture::WithFixture;

use crate::{db::HirDatabase, mir::MirLowerError, setup_tracing, test_db::TestDB};

#[derive(Clone, Copy)]
enum MirLoweringExpectation {
    Success,
    HasErrors,
}

fn lower_mir(#[rust_analyzer::rust_fixture] ra_fixture: &str, expectation: MirLoweringExpectation) {
    let _tracing = setup_tracing();
    let (db, file_ids) = TestDB::with_many_files(ra_fixture);
    crate::attach_db(db.as_dyn(), || {
        let file_id = *file_ids.last().unwrap();
        let module_id = db.module_for_file(file_id.file_id(&db));
        let def_map = module_id.def_map(&db);
        let scope = &def_map[module_id].scope;
        let funcs = scope.declarations().filter_map(|x| match x {
            hir_def::ModuleDefId::FunctionId(it) => Some(it),
            _ => None,
        });
        for func in funcs {
            let result = db.mir_body(func.into());
            match expectation {
                MirLoweringExpectation::Success => {
                    result.unwrap();
                }
                MirLoweringExpectation::HasErrors => assert!(
                    matches!(&result, Err(MirLowerError::HasErrors)),
                    "unexpected MIR lowering result: {result:?}"
                ),
            }
        }
    })
}

#[test]
fn dyn_projection_with_auto_traits_regression_next_solver() {
    lower_mir(
        r#"
//- minicore: sized, send
pub trait Deserializer {}

pub trait Strictest {
    type Object: ?Sized;
}

impl Strictest for dyn CustomValue {
    type Object = dyn CustomValue + Send;
}

pub trait CustomValue: Send {}

impl CustomValue for () {}

struct Box<T: ?Sized>;

type DeserializeFn<T> = fn(&mut dyn Deserializer) -> Box<T>;

fn foo() {
    (|deserializer| Box::new(())) as DeserializeFn<<dyn CustomValue as Strictest>::Object>;
}
    "#,
        MirLoweringExpectation::Success,
    );
}

#[test]
fn const_arg_of_wrong_type() {
    lower_mir(
        r#"
//- minicore: index, slice
struct Struct<const N: i64>(pub [u8; N]);

pub fn function(value: Struct<3>) -> u8 {
    value.0[0]
}
        "#,
        MirLoweringExpectation::HasErrors,
    );
}
