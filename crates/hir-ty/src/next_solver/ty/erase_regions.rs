use rustc_type_ir::{
    Interner, TypeFlags, TypeFoldable, TypeFolder, TypeSuperFoldable, TypeVisitableExt,
    inherent::IntoKind as _,
};
use tracing::debug;

use crate::next_solver::{Binder, Clauses, Const, DbInterner, Predicate, Region, RegionKind, Ty};

impl<'db> DbInterner<'db> {
    pub fn erase_and_anonymize_regions_ty(self, ty: Ty<'db>) -> Ty<'db> {
        // N.B., use `super_fold_with` here. If we used `fold_with`, it
        // could invoke the `erase_and_anonymize_regions_ty` query recursively.
        ty.super_fold_with(&mut RegionEraserAndAnonymizerVisitor { interner: self })
    }

    /// Returns an equivalent value with all free regions removed and
    /// bound regions anonymized. (note that bound regions are important
    /// for subtyping and generally type equality so *cannot* be removed)
    pub fn erase_and_anonymize_regions<T>(self, value: T) -> T
    where
        T: TypeFoldable<DbInterner<'db>>,
    {
        // If there's nothing to erase or anonymize, avoid performing the query at all
        if !value.has_type_flags(TypeFlags::HAS_BINDER_VARS | TypeFlags::HAS_FREE_REGIONS) {
            return value;
        }
        debug!("erase_and_anonymize_regions({:?})", value);
        let value1 = value.fold_with(&mut RegionEraserAndAnonymizerVisitor { interner: self });
        debug!("erase_and_anonymize_regions = {:?}", value1);
        value1
    }
}

struct RegionEraserAndAnonymizerVisitor<'db> {
    interner: DbInterner<'db>,
}

impl<'db> TypeFolder<DbInterner<'db>> for RegionEraserAndAnonymizerVisitor<'db> {
    fn cx(&self) -> DbInterner<'db> {
        self.interner
    }

    fn fold_ty(&mut self, ty: Ty<'db>) -> Ty<'db> {
        if !ty.has_type_flags(TypeFlags::HAS_BINDER_VARS | TypeFlags::HAS_FREE_REGIONS) {
            ty
        } else if ty.has_infer() {
            ty.super_fold_with(self)
        } else {
            self.interner.erase_and_anonymize_regions_ty(ty)
        }
    }

    fn fold_binder<T>(&mut self, t: Binder<'db, T>) -> Binder<'db, T>
    where
        T: TypeFoldable<DbInterner<'db>>,
    {
        let u = self.interner.anonymize_bound_vars(t);
        u.super_fold_with(self)
    }

    fn fold_region(&mut self, r: Region<'db>) -> Region<'db> {
        // We must not erase bound regions. `for<'a> fn(&'a ())` and
        // `fn(&'free ())` are different types: they may implement different
        // traits and have a different `TypeId`.
        match r.kind() {
            RegionKind::ReBound(..) => r,
            _ => self.interner.default_types().regions.erased,
        }
    }

    fn fold_const(&mut self, ct: Const<'db>) -> Const<'db> {
        if ct.has_type_flags(TypeFlags::HAS_BINDER_VARS | TypeFlags::HAS_FREE_REGIONS) {
            ct.super_fold_with(self)
        } else {
            ct
        }
    }

    fn fold_predicate(&mut self, p: Predicate<'db>) -> Predicate<'db> {
        if p.has_type_flags(TypeFlags::HAS_BINDER_VARS | TypeFlags::HAS_FREE_REGIONS) {
            p.super_fold_with(self)
        } else {
            p
        }
    }

    fn fold_clauses(&mut self, c: Clauses<'db>) -> Clauses<'db> {
        if c.has_type_flags(TypeFlags::HAS_BINDER_VARS | TypeFlags::HAS_FREE_REGIONS) {
            c.super_fold_with(self)
        } else {
            c
        }
    }
}
