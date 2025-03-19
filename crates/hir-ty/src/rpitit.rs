//! This module contains the implementation of Return Position Impl Trait In Traits.

use std::{iter, sync::Arc};

use base_db::impl_intern_key_ref;
use chalk_ir::{
    BoundVar, DebruijnIndex,
    fold::{TypeFoldable, TypeFolder, TypeSuperFoldable},
};
use chalk_solve::rust_ir::AssociatedTyValueBound;
use hir_def::{
    AssocItemId, FunctionId, GenericDefId, ImplId, TraitId,
    hir::generics::{GenericParams, TypeOrConstParamData},
};
use rustc_hash::FxHashMap;

use crate::{
    AliasTy, AnyTraitAssocType, Binders, Const, ConstData, ConstValue, DomainGoal, Goal, GoalData,
    InferenceTable, Interner, Lifetime, LifetimeData, PlaceholderIndex, ProjectionTy, Substitution,
    Ty, TyKind, VariableKinds,
    chalk_db::{AssociatedTyValue, inline_bound_to_generic_predicate},
    db::HirDatabase,
    from_assoc_type_id, from_placeholder_idx,
    generics::generics,
    lt_from_placeholder_idx,
    mapping::{ToChalk, to_assoc_type_id_rpitit},
    variable_kinds_from_generics,
};

/// An associated type synthesized from a Return Position Impl Trait In Trait
/// of the trait (not the impls).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RpititTraitAssocTy {
    pub trait_id: TraitId,
    /// The method that contains this RPITIT.
    pub synthesized_from_method: FunctionId,
    /// The bounds of this associated type (coming from the `impl Bounds`).
    ///
    /// The generics are the generics of the method (with some modifications that we
    /// don't currently implement, see https://rustc-dev-guide.rust-lang.org/return-position-impl-trait-in-trait.html).
    pub bounds: Binders<Vec<chalk_solve::rust_ir::QuantifiedInlineBound<Interner>>>,
}

impl_intern_key_ref!(RpititTraitAssocTyId, RpititTraitAssocTy);

/// An associated type synthesized from a Return Position Impl Trait In Trait
/// of the impl (not the trait).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RpititImplAssocTy {
    pub impl_id: ImplId,
    /// The definition of this associated type in the trait.
    pub trait_assoc: RpititTraitAssocTyId,
}

impl_intern_key_ref!(RpititImplAssocTyId, RpititImplAssocTy);

// We return a list and not a hasmap because the number of RPITITs in a function should be small.
#[salsa_macros::tracked(return_ref)]
pub(crate) fn impl_method_rpitit_values(
    db: &dyn HirDatabase,
    impl_id: hir_def::ImplId,
    trait_method_id: FunctionId,
) -> Box<[Arc<AssociatedTyValue>]> {
    let impl_items = db.impl_items(impl_id);
    let trait_method_generics = generics(db, trait_method_id.into());
    let impl_datum =
        db.impl_datum(impl_id.loc(db).container.krate(), hir_def::ImplId::to_chalk(impl_id, db));
    let trait_method = db.function_signature(trait_method_id);
    let Some(impl_method) = impl_items.items.iter().find_map(|(name, id)| {
        if *name == trait_method.name {
            match *id {
                AssocItemId::FunctionId(it) => Some(it),
                _ => None,
            }
        } else {
            None
        }
    }) else {
        // FIXME: Handle defaulted methods.
        return Box::default();
    };

    let impl_method_generics = generics(db, impl_method.into());

    // First, just so we won't ICE, check that the impl method generics match the trait method generics.
    if !check_method_generics_are_structurally_compatible(
        trait_method_generics.self_params(),
        impl_method_generics.self_params(),
    ) {
        return Box::default();
    }

    // The inference algorithm works as follows: in the trait method, we replace each RPITIT with an infer var,
    // then we equate the return type of the trait method with the return type of the impl method. The values
    // of the inference vars now represent the value of the RPITIT assoc types.
    let mut table = InferenceTable::new(db, db.trait_environment(impl_method.into()));
    let impl_method_placeholder_subst = impl_method_generics.placeholder_subst(db);

    let impl_method_ret = db
        .callable_item_signature(impl_method.into())
        .substitute(Interner, &impl_method_placeholder_subst)
        .ret()
        .clone();
    let impl_method_ret = table.normalize_associated_types_in(impl_method_ret);

    // Create mapping from trait to impl (i.e. impl trait header + impl method identity args).
    let trait_ref_placeholder_subst =
        &impl_method_placeholder_subst.as_slice(Interner)[..impl_method_generics.len_parent()];
    // We want to substitute the TraitRef with placeholders, but placeholders from the method, not the impl.
    let impl_trait_ref = impl_datum
        .binders
        .as_ref()
        .map(|it| it.trait_ref.clone())
        .substitute(Interner, trait_ref_placeholder_subst);
    let trait_to_impl_args = Substitution::from_iter(
        Interner,
        impl_trait_ref.substitution.as_slice(Interner).iter().chain(
            &impl_method_placeholder_subst.as_slice(Interner)[impl_method_generics.len_parent()..],
        ),
    );
    let trait_method_ret = db
        .callable_item_signature(trait_method_id.into())
        .substitute(Interner, &trait_to_impl_args)
        .ret()
        .clone();
    let mut rpitit_to_infer_var_folder = RpititToInferVarFolder {
        db,
        table: &mut table,
        trait_method_id,
        trait_rpitit_to_infer_var: FxHashMap::default(),
    };
    let trait_method_ret =
        trait_method_ret.fold_with(&mut rpitit_to_infer_var_folder, DebruijnIndex::INNERMOST);
    let trait_rpitit_to_infer_var = rpitit_to_infer_var_folder.trait_rpitit_to_infer_var;
    let trait_method_ret = table.normalize_associated_types_in(trait_method_ret);

    table.resolve_obligations_as_possible();
    // Even if unification fails, we want to continue. We will fill the RPITITs with error types.
    table.unify(&trait_method_ret, &impl_method_ret);
    table.resolve_obligations_as_possible();

    return trait_rpitit_to_infer_var
        .into_iter()
        .map(|(trait_assoc_id, infer_var)| {
            let impl_rpitit = table.resolve_completely(infer_var);
            let impl_rpitit = impl_rpitit.fold_with(
                &mut PlaceholderToBoundVarFolder {
                    db,
                    method: impl_method.into(),
                    method_generics: impl_method_generics.self_params(),
                },
                DebruijnIndex::INNERMOST,
            );
            let trait_assoc = trait_assoc_id.loc(db);
            // Completely unlike the docs, Chalk requires both the impl generics and the associated type
            // generics in the binder.
            let impl_rpitit_binders = VariableKinds::from_iter(
                Interner,
                trait_assoc.bounds.binders.as_slice(Interner)[..trait_method_generics.len()]
                    .iter()
                    .cloned()
                    .chain(variable_kinds_from_generics(db, impl_method_generics.iter_parent_id())),
            );
            let impl_rpitit =
                Binders::new(impl_rpitit_binders, AssociatedTyValueBound { ty: impl_rpitit });
            Arc::new(AssociatedTyValue {
                associated_ty_id: to_assoc_type_id_rpitit(trait_assoc_id),
                impl_id: ImplId::to_chalk(impl_id, db),
                value: impl_rpitit,
            })
        })
        .collect();

    #[derive(chalk_derive::FallibleTypeFolder)]
    #[has_interner(Interner)]
    struct RpititToInferVarFolder<'a, 'b> {
        db: &'a dyn HirDatabase,
        table: &'a mut InferenceTable<'b>,
        trait_rpitit_to_infer_var: FxHashMap<RpititTraitAssocTyId, Ty>,
        trait_method_id: FunctionId,
    }
    impl TypeFolder<Interner> for RpititToInferVarFolder<'_, '_> {
        fn as_dyn(&mut self) -> &mut dyn TypeFolder<Interner> {
            self
        }

        fn interner(&self) -> Interner {
            Interner
        }

        fn fold_ty(&mut self, ty: Ty, outer_binder: DebruijnIndex) -> Ty {
            let result = match ty.kind(Interner) {
                TyKind::Alias(AliasTy::Projection(ProjectionTy {
                    associated_ty_id,
                    substitution,
                }))
                | TyKind::AssociatedType(associated_ty_id, substitution) => {
                    if let AnyTraitAssocType::Rpitit(assoc_id) =
                        from_assoc_type_id(self.db, *associated_ty_id)
                    {
                        let assoc = assoc_id.loc(self.db);
                        if assoc.synthesized_from_method == self.trait_method_id {
                            if let Some(ty) = self.trait_rpitit_to_infer_var.get(&assoc_id) {
                                return ty.clone();
                            }

                            // Replace with new infer var.
                            // This needs to come before we fold the bounds, because they also contain this associated type.
                            let var = self.table.new_type_var();
                            self.trait_rpitit_to_infer_var.insert(assoc_id, var.clone());

                            // Recurse into bounds, so that nested RPITITs will be handled correctly.
                            for bound in assoc.bounds.clone().substitute(Interner, substitution) {
                                let bound = inline_bound_to_generic_predicate(&bound, var.clone());
                                let bound = bound.fold_with(self, outer_binder);
                                let bound = self.table.normalize_associated_types_in(bound);
                                self.table.register_obligation(Goal::new(
                                    Interner,
                                    GoalData::Quantified(
                                        chalk_ir::QuantifierKind::ForAll,
                                        bound.map(|bound| {
                                            Goal::new(
                                                Interner,
                                                GoalData::DomainGoal(DomainGoal::Holds(bound)),
                                            )
                                        }),
                                    ),
                                ));
                            }

                            return var;
                        }
                    }
                    ty.clone()
                }
                _ => ty.clone(),
            };
            result.super_fold_with(self, outer_binder)
        }
    }

    #[derive(chalk_derive::FallibleTypeFolder)]
    #[has_interner(Interner)]
    struct PlaceholderToBoundVarFolder<'a> {
        db: &'a dyn HirDatabase,
        method: GenericDefId,
        method_generics: &'a GenericParams,
    }
    impl TypeFolder<Interner> for PlaceholderToBoundVarFolder<'_> {
        fn as_dyn(&mut self) -> &mut dyn TypeFolder<Interner> {
            self
        }

        fn interner(&self) -> Interner {
            Interner
        }

        fn fold_free_placeholder_ty(
            &mut self,
            universe: PlaceholderIndex,
            _outer_binder: DebruijnIndex,
        ) -> Ty {
            let placeholder = from_placeholder_idx(self.db, universe);
            if placeholder.parent == self.method {
                BoundVar::new(
                    DebruijnIndex::INNERMOST,
                    placeholder.local_id.into_raw().into_u32() as usize
                        + self.method_generics.len_lifetimes(),
                )
                .to_ty(Interner)
            } else {
                TyKind::Placeholder(universe).intern(Interner)
            }
        }

        fn fold_free_placeholder_const(
            &mut self,
            ty: Ty,
            universe: PlaceholderIndex,
            _outer_binder: DebruijnIndex,
        ) -> Const {
            let placeholder = from_placeholder_idx(self.db, universe);
            if placeholder.parent == self.method {
                BoundVar::new(
                    DebruijnIndex::INNERMOST,
                    placeholder.local_id.into_raw().into_u32() as usize
                        + self.method_generics.len_lifetimes(),
                )
                .to_const(Interner, ty)
            } else {
                Const::new(Interner, ConstData { ty, value: ConstValue::Placeholder(universe) })
            }
        }

        fn fold_free_placeholder_lifetime(
            &mut self,
            universe: PlaceholderIndex,
            _outer_binder: DebruijnIndex,
        ) -> Lifetime {
            let placeholder = lt_from_placeholder_idx(self.db, universe);
            if placeholder.parent == self.method {
                BoundVar::new(
                    DebruijnIndex::INNERMOST,
                    placeholder.local_id.into_raw().into_u32() as usize,
                )
                .to_lifetime(Interner)
            } else {
                Lifetime::new(Interner, LifetimeData::Placeholder(universe))
            }
        }
    }
}

fn check_method_generics_are_structurally_compatible(
    trait_method_generics: &GenericParams,
    impl_method_generics: &GenericParams,
) -> bool {
    if trait_method_generics.len_type_or_consts() != impl_method_generics.len_type_or_consts() {
        return false;
    }

    for ((_, trait_arg), (_, impl_arg)) in iter::zip(
        trait_method_generics.iter_type_or_consts(),
        impl_method_generics.iter_type_or_consts(),
    ) {
        match (trait_arg, impl_arg) {
            (TypeOrConstParamData::TypeParamData(_), TypeOrConstParamData::TypeParamData(_))
            | (TypeOrConstParamData::ConstParamData(_), TypeOrConstParamData::ConstParamData(_)) => {
            }
            _ => return false,
        }
    }

    true
}
