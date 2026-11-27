//! Things related to the infer context of the next-trait-solver.

pub(crate) mod table;

pub(crate) use table::{OpaqueTypeStorage, OpaqueTypeTable};

use macros::{TypeFoldable, TypeVisitable};
use rustc_hash::FxHashMap;
use rustc_type_ir::{
    TypeFoldable, TypeFolder, TypeSuperFoldable, fold_regions,
    inherent::{GenericArgs as _, IntoKind as _, Ty as _},
};

use crate::next_solver::{
    Const, ConstKind, DbInterner, ErrorGuaranteed, GenericArg, GenericArgs, OpaqueTypeKey, Region,
    Ty, TyKind, infer::InferCtxt,
};

#[derive(Copy, Clone, Debug, TypeVisitable, TypeFoldable)]
pub struct OpaqueHiddenType<'db> {
    pub ty: Ty<'db>,
}

impl<'db> OpaqueHiddenType<'db> {
    /// Express the hidden type in terms of the opaque declaration's parameters.
    pub(crate) fn remap_generic_params_to_declaration_params(
        self,
        opaque_type_key: OpaqueTypeKey<'db>,
        interner: DbInterner<'db>,
    ) -> Self {
        let OpaqueTypeKey { def_id, args } = opaque_type_key;

        // Map arguments from the defining use back to the opaque's parameters.
        let id_args = GenericArgs::identity_for_item(interner, def_id.into());
        let map = args.iter().zip(id_args.iter()).collect();

        // During HIR type checking, erase free regions without anonymizing bound regions.
        let erased = Region::new_erased(interner);
        let ty = fold_regions(interner, self.ty, |_, _| erased);
        let ty = ty.fold_with(&mut ReverseMapper { interner, map });
        Self { ty }
    }
}

struct ReverseMapper<'db> {
    interner: DbInterner<'db>,
    map: FxHashMap<GenericArg<'db>, GenericArg<'db>>,
}

impl<'db> TypeFolder<DbInterner<'db>> for ReverseMapper<'db> {
    fn cx(&self) -> DbInterner<'db> {
        self.interner
    }

    fn fold_ty(&mut self, ty: Ty<'db>) -> Ty<'db> {
        match ty.kind() {
            TyKind::Param(_) => self
                .map
                .get(&ty.into())
                .and_then(|arg| arg.ty())
                .unwrap_or_else(|| Ty::new_error(self.interner, ErrorGuaranteed)),
            _ => ty.super_fold_with(self),
        }
    }

    fn fold_const(&mut self, ct: Const<'db>) -> Const<'db> {
        match ct.kind() {
            ConstKind::Param(_) => self
                .map
                .get(&ct.into())
                .and_then(|arg| arg.konst())
                .unwrap_or_else(|| Const::error(self.interner)),
            // Unevaluated consts can refer to parameters from the defining use in their arguments.
            // Instantiating the hidden type visits those arguments too, so remap them here.
            _ => ct.super_fold_with(self),
        }
    }
}

impl<'db> InferCtxt<'db> {
    /// Insert a hidden type into the opaque type storage, making sure
    /// it hasn't previously been defined. This does not emit any
    /// constraints and it's the responsibility of the caller to make
    /// sure that the item bounds of the opaque are checked.
    pub fn register_hidden_type_in_storage(
        &self,
        opaque_type_key: OpaqueTypeKey<'db>,
        hidden_ty: OpaqueHiddenType<'db>,
    ) -> Option<Ty<'db>> {
        self.inner.borrow_mut().opaque_types().register(opaque_type_key, hidden_ty)
    }
}
