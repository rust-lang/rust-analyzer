use hir_def::DefWithBodyId;
use rustc_type_ir::inherent::Ty as _;
use test_fixture::WithFixture;

use crate::{
    db::HirDatabase,
    mir::{FieldIndex, PlaceElem, PlaceTy, ProjectionElem},
    next_solver::{DbInterner, ParamEnv, Ty, TypingMode, infer::DbInternerInferExt},
    setup_tracing,
    test_db::TestDB,
};

fn lower_mir(#[rust_analyzer::rust_fixture] ra_fixture: &str) {
    let _tracing = setup_tracing();
    let (db, file_ids) = TestDB::with_many_files(ra_fixture);
    crate::attach_db(&db, || {
        let file_id = *file_ids.last().unwrap();
        let module_id = db.module_for_file(file_id.file_id(&db));
        let def_map = module_id.def_map(&db);
        let scope = &def_map[module_id].scope;
        let funcs = scope.declarations().filter_map(|x| match x {
            hir_def::ModuleDefId::FunctionId(it) => Some(it),
            _ => None,
        });
        for func in funcs {
            _ = db.mir_body(func.into());
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
    );
}

fn check_borrowck(#[rust_analyzer::rust_fixture] ra_fixture: &str) {
    let _tracing = setup_tracing();
    let (db, file_ids) = TestDB::with_many_files(ra_fixture);
    crate::attach_db(&db, || {
        let file_id = *file_ids.last().unwrap();
        let module_id = db.module_for_file(file_id.file_id(&db));
        let def_map = module_id.def_map(&db);
        let scope = &def_map[module_id].scope;

        let mut bodies: Vec<DefWithBodyId> = Vec::new();

        for decl in scope.declarations() {
            if let hir_def::ModuleDefId::FunctionId(f) = decl {
                bodies.push(f.into());
            }
        }

        for impl_id in scope.impls() {
            let impl_items = impl_id.impl_items(&db);
            for (_, item) in impl_items.items.iter() {
                if let hir_def::AssocItemId::FunctionId(f) = item {
                    bodies.push((*f).into());
                }
            }
        }

        for body in bodies {
            let _ = db.borrowck(body.into());
        }
    })
}

#[test]
fn regression_21173_const_generic_impl_with_assoc_type() {
    check_borrowck(
        r#"
pub trait Tr {
    type Assoc;
    fn f(&self, handle: Self::Assoc) -> i32;
}

pub struct ConstGeneric<const N: usize>;

impl<const N: usize> Tr for &ConstGeneric<N> {
    type Assoc = AssocTy;

    fn f(&self, a: Self::Assoc) -> i32 {
        a.x
    }
}

pub struct AssocTy {
    x: i32,
}
    "#,
    );
}

#[test]
fn borrowck_tuple_field_projection_recovery_does_not_panic() {
    check_borrowck(
        r#"
//- minicore: sized
fn tuple_field() {
    let t = (1,);
    let x = t.1;
}
    "#,
    );
}

#[test]
fn borrowck_alias_projection_recovery_does_not_panic() {
    check_borrowck(
        r#"
//- minicore: sized
trait Tr { type A; }
fn alias<T: Tr>(x: T::A) {
    let (a, b) = x;
}
    "#,
    );
}

#[test]
fn field_projection_on_slice_recovers_with_error_type() {
    let (db, file_id) = TestDB::with_single_file("");
    crate::attach_db(&db, || {
        let module = db.module_for_file(file_id.file_id(&db));
        let interner = DbInterner::new_with(&db, module.krate(&db));
        let infcx = interner.infer_ctxt().build(TypingMode::PostAnalysis);
        let slice = Ty::new_slice(interner, Ty::new_bool(interner));
        let field: PlaceElem = ProjectionElem::Field(FieldIndex(0));

        let result =
            PlaceTy::from_ty(slice).projection_ty(&infcx, &field, ParamEnv::empty(interner));

        assert!(result.ty.is_ty_error());
    });
}
