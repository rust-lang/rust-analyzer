//! Various type-based analyses that aren't large enough to warrant their own crate: layout, drop, etc..

extern crate ra_ap_rustc_abi as rustc_abi;
extern crate ra_ap_rustc_index as rustc_index;
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

        fn layout_of_adt(
            &self,
            def: AdtId,
            args: StoredGenericArgs,
            trait_env: StoredParamEnvAndCrate,
        ) -> Result<Arc<Layout>, LayoutError> {
            $crate::layout::layout_of_adt_query(self, def, args, trait_env)
        }

        fn layout_of_ty(
            &self,
            ty: StoredTy,
            env: StoredParamEnvAndCrate,
        ) -> Result<Arc<Layout>, LayoutError> {
            $crate::layout::layout_of_ty_query(self, ty, env)
        }

        fn target_data_layout(&self, krate: Crate) -> Result<&TargetDataLayout, TargetLoadError> {
            $crate::layout::target_data_layout_query(self, krate).map_err(|err| err.clone())
        }

        fn target_data_layout_or_default(&self, krate: Crate) -> &TargetDataLayout {
            static DEFAULT: LazyLock<TargetDataLayout> = LazyLock::new(TargetDataLayout::default);

            $crate::layout::target_data_layout_query(self, krate).unwrap_or_else(|_| &*DEFAULT)
        }
    };
}
