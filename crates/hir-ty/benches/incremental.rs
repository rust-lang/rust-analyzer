use base_db::{
    CrateGraphBuilder, CratesMap, FileSourceRootInput, FileText, RootQueryDb, SourceDatabase,
    SourceRoot, SourceRootId, SourceRootInput,
};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hir_def::{ModuleId, db::DefDatabase, nameres::crate_def_map};
use hir_ty::db::HirDatabase;
use salsa::Durability;
use span::FileId;
use std::{fmt, panic};
use test_fixture::WithFixture;
use triomphe::Arc;

#[salsa_macros::db]
#[derive(Clone)]
pub(crate) struct BenchmarkDB {
    storage: salsa::Storage<Self>,
    files: Arc<base_db::Files>,
    crates_map: Arc<CratesMap>,
}

impl Default for BenchmarkDB {
    fn default() -> Self {
        let mut this = Self {
            storage: salsa::Storage::new(None),
            files: Default::default(),
            crates_map: Default::default(),
        };
        this.set_expand_proc_attr_macros_with_durability(true, Durability::HIGH);
        // This needs to be here otherwise `CrateGraphBuilder` panics.
        this.set_all_crates(Arc::new(Box::new([])));
        CrateGraphBuilder::default().set_in_db(&mut this);
        this
    }
}

impl fmt::Debug for BenchmarkDB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestDB").finish()
    }
}

#[salsa_macros::db]
impl SourceDatabase for BenchmarkDB {
    fn file_text(&self, file_id: base_db::FileId) -> FileText {
        self.files.file_text(file_id)
    }

    fn set_file_text(&mut self, file_id: base_db::FileId, text: &str) {
        let files = Arc::clone(&self.files);
        files.set_file_text(self, file_id, text);
    }

    fn set_file_text_with_durability(
        &mut self,
        file_id: base_db::FileId,
        text: &str,
        durability: Durability,
    ) {
        let files = Arc::clone(&self.files);
        files.set_file_text_with_durability(self, file_id, text, durability);
    }

    /// Source root of the file.
    fn source_root(&self, source_root_id: SourceRootId) -> SourceRootInput {
        self.files.source_root(source_root_id)
    }

    fn set_source_root_with_durability(
        &mut self,
        source_root_id: SourceRootId,
        source_root: Arc<SourceRoot>,
        durability: Durability,
    ) {
        let files = Arc::clone(&self.files);
        files.set_source_root_with_durability(self, source_root_id, source_root, durability);
    }

    fn file_source_root(&self, id: base_db::FileId) -> FileSourceRootInput {
        self.files.file_source_root(id)
    }

    fn set_file_source_root_with_durability(
        &mut self,
        id: base_db::FileId,
        source_root_id: SourceRootId,
        durability: Durability,
    ) {
        let files = Arc::clone(&self.files);
        files.set_file_source_root_with_durability(self, id, source_root_id, durability);
    }

    fn crates_map(&self) -> Arc<CratesMap> {
        self.crates_map.clone()
    }
}

#[salsa_macros::db]
impl salsa::Database for BenchmarkDB {}

impl panic::RefUnwindSafe for BenchmarkDB {}

impl BenchmarkDB {
    pub(crate) fn module_for_file_opt(&self, file_id: impl Into<FileId>) -> Option<ModuleId> {
        let file_id = file_id.into();
        for &krate in self.relevant_crates(file_id).iter() {
            let crate_def_map = crate_def_map(self, krate);
            for (local_id, data) in crate_def_map.modules() {
                if data.origin.file_id().map(|file_id| file_id.file_id(self)) == Some(file_id) {
                    return Some(crate_def_map.module_id(local_id));
                }
            }
        }
        None
    }

    pub(crate) fn module_for_file(&self, file_id: impl Into<FileId>) -> ModuleId {
        self.module_for_file_opt(file_id.into()).unwrap()
    }
}

fn benchmark_incremental_use_expr_addition(c: &mut Criterion) {
    c.bench_function("incremental_use_expr_addition", |b| {
        b.iter_batched(
            || {
                let (db, pos) = BenchmarkDB::with_position(
                    "
//- /lib.rs
fn foo() -> i32 {
    $01 + 1
}",
                );

                let module = db.module_for_file(pos.file_id.file_id(&db));
                let _crate_def_map = module.def_map(&db);
                db.trait_impls_in_crate(module.krate());

                (db, pos)
            },
            |(mut db, pos)| {
                let new_text = "
use std::collections::HashMap;

fn foo() -> i32 {
    $01 + 1
}";

                db.set_file_text(pos.file_id.file_id(&db), new_text);
                let module = db.module_for_file(pos.file_id.file_id(&db));
                let _crate_def_map = module.def_map(&db);

                black_box(db.trait_impls_in_crate(module.krate()));
            },
            criterion::BatchSize::LargeInput,
        );
    });
}

fn benchmark_incremental_struct_addition(c: &mut Criterion) {
    c.bench_function("incremental_struct_addition", |b| {
        b.iter_batched(
            || {
                let (db, pos) = BenchmarkDB::with_position(
                    "
//- /lib.rs
fn foo() -> i32 {
    1 + 1
}

fn bar() -> f32 {
    2.0 * 3.0
}$0",
                );

                let module = db.module_for_file(pos.file_id.file_id(&db));
                let _crate_def_map = module.def_map(&db);
                db.trait_impls_in_crate(module.krate());

                (db, pos)
            },
            |(mut db, pos)| {
                let new_text = "
//- /lib.rs
fn foo() -> i32 {
    1 + 1
}

fn bar() -> f32 {
    2.0 * 3.0
}

pub struct SomeStruct {
    field: i32,
}$0";

                db.set_file_text(pos.file_id.file_id(&db), new_text);
                let module = db.module_for_file(pos.file_id.file_id(&db));
                let _crate_def_map = module.def_map(&db);

                black_box(db.trait_impls_in_crate(module.krate()));
            },
            criterion::BatchSize::LargeInput,
        );
    });
}

criterion_group!(
    benches,
    benchmark_incremental_use_expr_addition,
    benchmark_incremental_struct_addition
);
criterion_main!(benches);
