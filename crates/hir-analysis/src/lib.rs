//! Various type-based analyses that aren't large enough to warrant their own crate: layout, drop, etc..

mod specialization;

#[doc(hidden)]
pub use crate::specialization::specializes;

#[macro_export]
macro_rules! impl_db_methods {
    () => {
        fn specializes(
            &self,
            specializing_impl_def_id: ImplId,
            parent_impl_def_id: ImplId,
        ) -> bool {
            $crate::specializes(self, specializing_impl_def_id, parent_impl_def_id)
        }
    };
}
