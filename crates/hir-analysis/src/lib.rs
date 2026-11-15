//! Various type-based analyses that aren't large enough to warrant their own crate: layout, drop, etc..

extern crate ra_ap_rustc_type_ir as rustc_type_ir;

#[doc(hidden)]
pub mod drop;
pub mod dyn_compatibility;
#[doc(hidden)]
pub mod layout;
#[doc(hidden)]
pub mod specialization;

#[macro_export]
macro_rules! impl_db_methods {
    () => {
        fn specializes(
            &self,
            specializing_impl_def_id: ImplId,
            parent_impl_def_id: ImplId,
        ) -> bool {
            $crate::specialization::specializes(self, specializing_impl_def_id, parent_impl_def_id)
        }

        fn destructor(&self, adt: AdtId) -> Option<ImplId> {
            $crate::drop::destructor(self, adt)
        }

        fn has_drop_glue<'db>(&'db self, ty: Ty<'db>, env: TypingEnv<'db>) -> hir_ty::db::DropGlue {
            $crate::drop::has_drop_glue(self, ty, env)
        }

        fn generics_require_sized_self(&self, def_id: GenericDefId) -> bool {
            $crate::dyn_compatibility::generics_require_sized_self(self, def_id)
        }

        fn trait_is_dyn_compatible(&self, trait_: TraitId) -> bool {
            $crate::dyn_compatibility::dyn_compatibility_violations(self, trait_).is_none()
        }
    };
}
