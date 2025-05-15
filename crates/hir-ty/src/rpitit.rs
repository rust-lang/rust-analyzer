//! This module contains the implementation of Return Position Impl Trait In Traits.

use std::{iter, sync::Arc};

use base_db::impl_intern_key_ref;
use chalk_ir::{
    BoundVar, DebruijnIndex,
    cast::Cast,
    fold::{TypeFoldable, TypeFolder, TypeSuperFoldable},
};
use chalk_solve::rust_ir::AssociatedTyValueBound;
use hir_def::{
    AssocItemId, FunctionId, GenericDefId, GenericParamId, ImplId, TraitId,
    hir::generics::{GenericParams, TypeOrConstParamData},
    resolver::HasResolver,
};
use rustc_hash::FxHashMap;
use thin_vec::ThinVec;

use crate::{
    AliasTy, AnyTraitAssocType, Binders, Const, ConstData, ConstValue, DomainGoal, Goal, GoalData,
    ImplTraitLoweringMode, InferenceTable, Interner, Lifetime, LifetimeData, LifetimeElisionKind,
    ParamLoweringMode, PlaceholderIndex, ProjectionTy, Substitution, TraitRef, Ty, TyKind,
    TyLoweringContext, VariableKinds,
    chalk_db::{AssociatedTyValue, inline_bound_to_generic_predicate},
    db::HirDatabase,
    from_assoc_type_id, from_placeholder_idx,
    generics::{Generics, generics},
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
    impl_id: ImplId,
    trait_method_id: FunctionId,
) -> Box<[Arc<AssociatedTyValue>]> {
    let impl_items = db.impl_items(impl_id);
    let trait_method_generics = generics(db, trait_method_id.into());
    let trait_method = db.function_signature(trait_method_id);
    let impl_trait_ref = db.impl_trait(impl_id).expect("invalid impl passed to Chalk");
    let impl_method = impl_items.items.iter().find_map(|(name, id)| {
        if *name == trait_method.name {
            match *id {
                AssocItemId::FunctionId(it) => Some(it),
                _ => None,
            }
        } else {
            None
        }
    });
    let impl_method = match impl_method {
        Some(impl_method) => impl_method,
        None => {
            // Method not in the impl, so it is defaulted.
            return defaulted_impl_method_rpitit_values(
                db,
                impl_id,
                trait_method_id,
                impl_trait_ref,
                &trait_method_generics,
            );
        }
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
    let impl_trait_ref = impl_trait_ref.substitute(Interner, trait_ref_placeholder_subst);
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

    trait_rpitit_to_infer_var
        .into_iter()
        .map(|(trait_assoc_id, infer_var)| {
            let impl_rpitit = table.resolve_completely(infer_var);
            // FIXME: We may be able to get rid of `PlaceholderToBoundVarFolder` and some other stuff if we lower
            // the return type with `ParamLoweringMode::Variable`, but for some reason the unification does not work then.
            let impl_rpitit = impl_rpitit.fold_with(
                &mut PlaceholderToBoundVarFolder {
                    db,
                    method: impl_method.into(),
                    method_generics: impl_method_generics.self_params(),
                    parent: impl_id.into(),
                    parent_generics: impl_method_generics
                        .parent_generics()
                        .expect("parent should be an impl")
                        .self_params(),
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
        .collect()
}

fn defaulted_impl_method_rpitit_values(
    db: &dyn HirDatabase,
    impl_id: ImplId,
    trait_method_id: FunctionId,
    impl_trait_ref: Binders<TraitRef>,
    trait_method_generics: &Generics,
) -> Box<[Arc<AssociatedTyValue>]> {
    let defaulted_rpitit_values = defaulted_trait_method_rpitit_values(db, trait_method_id);
    let impl_generics = generics(db, impl_id.into());
    // The associated type generics as the same as the trait method's, but we take the impl as
    // the parent instead of the trait.
    // The impl generics need to be shifted to account for the associated type generics.
    let trait_method_subst = trait_method_generics.bound_vars_subst(db, DebruijnIndex::INNERMOST);
    let impl_subst = Substitution::from_iter(
        Interner,
        impl_generics.iter_id().enumerate().map(|(idx, id)| match id {
            GenericParamId::ConstParamId(id) => {
                BoundVar::new(DebruijnIndex::INNERMOST, idx + trait_method_generics.len_self())
                    .to_const(Interner, db.const_param_ty(id))
                    .cast(Interner)
            }
            GenericParamId::TypeParamId(_) => {
                BoundVar::new(DebruijnIndex::INNERMOST, idx + trait_method_generics.len_self())
                    .to_ty(Interner)
                    .cast(Interner)
            }
            GenericParamId::LifetimeParamId(_) => {
                BoundVar::new(DebruijnIndex::INNERMOST, idx + trait_method_generics.len_self())
                    .to_lifetime(Interner)
                    .cast(Interner)
            }
        }),
    );
    let impl_trait_ref = impl_trait_ref.substitute(Interner, &impl_subst);
    let impl_rpitit_subst =
        Substitution::from_iter(
            Interner,
            impl_trait_ref.substitution.as_slice(Interner).iter().chain(
                &trait_method_subst.as_slice(Interner)[trait_method_generics.len_parent()..],
            ),
        );
    let binders = VariableKinds::from_iter(
        Interner,
        variable_kinds_from_generics(
            db,
            trait_method_generics.iter_self_id().chain(impl_generics.iter_id()),
        ),
    );
    defaulted_rpitit_values
        .iter()
        .map(|(trait_assoc, trait_rpitit)| {
            let impl_rpitit = trait_rpitit.clone().substitute(Interner, &impl_rpitit_subst);
            Arc::new(AssociatedTyValue {
                associated_ty_id: to_assoc_type_id_rpitit(*trait_assoc),
                impl_id: ImplId::to_chalk(impl_id, db),
                value: Binders::new(binders.clone(), AssociatedTyValueBound { ty: impl_rpitit }),
            })
        })
        .collect()
}

/// This is called only for defaulted trait methods, as there the value of the RPITIT associated
/// items on an impl (if the method body is left defaulted) is the same as with the trait method.
// This returns an `ThinVec` and not `Box<[]>` because this is called from inference,
// and most methods don't have RPITITs.
#[salsa_macros::tracked(return_ref)]
pub(crate) fn defaulted_trait_method_rpitit_values(
    db: &dyn HirDatabase,
    method_id: FunctionId,
) -> ThinVec<(RpititTraitAssocTyId, Binders<Ty>)> {
    let method_generics = generics(db, method_id.into());
    let mut table = InferenceTable::new(db, db.trait_environment(method_id.into()));

    let data = db.function_signature(method_id);
    let resolver = method_id.resolver(db);
    let mut ctx_ret = TyLoweringContext::new(
        db,
        &resolver,
        &data.store,
        method_id.into(),
        LifetimeElisionKind::Infer,
    )
    .with_impl_trait_mode(ImplTraitLoweringMode::Opaque)
    .with_type_param_mode(ParamLoweringMode::Placeholder);
    // This is the return type of the method, with RPITIT lowered as opaques. In other words, like if it was written
    // in an impl.
    let method_opaques_ret = match data.ret_type {
        Some(ret_type) => ctx_ret.lower_ty(ret_type),
        None => TyKind::Tuple(0, Substitution::empty(Interner)).intern(Interner),
    };
    let method_opaques_ret = table.normalize_associated_types_in(method_opaques_ret);

    // This is the return type of the method, with RPITITs lowered as associated types. In other words, like in its
    // signature.
    let method_assocs_ret = db
        .callable_item_signature(method_id.into())
        .substitute(Interner, &method_generics.placeholder_subst(db))
        .ret()
        .clone();
    let mut rpitit_to_infer_var_folder = RpititToInferVarFolder {
        db,
        table: &mut table,
        trait_method_id: method_id,
        trait_rpitit_to_infer_var: FxHashMap::default(),
    };
    let method_assocs_ret =
        method_assocs_ret.fold_with(&mut rpitit_to_infer_var_folder, DebruijnIndex::INNERMOST);
    let trait_rpitit_to_infer_var = rpitit_to_infer_var_folder.trait_rpitit_to_infer_var;
    let method_assocs_ret = table.normalize_associated_types_in(method_assocs_ret);

    table.resolve_obligations_as_possible();
    // Even if unification fails, we want to continue. We will fill the RPITITs with error types.
    table.unify(&method_assocs_ret, &method_opaques_ret);
    table.resolve_obligations_as_possible();

    ThinVec::from_iter(trait_rpitit_to_infer_var.into_iter().map(|(trait_assoc_id, infer_var)| {
        let trait_assoc = trait_assoc_id.loc(db);
        let rpitit = table.resolve_completely(infer_var);
        let rpitit = rpitit.fold_with(
            &mut PlaceholderToBoundVarFolder {
                db,
                method: method_id.into(),
                method_generics: method_generics.self_params(),
                parent: trait_assoc.trait_id.into(),
                parent_generics: method_generics
                    .parent_generics()
                    .expect("method should be inside trait")
                    .self_params(),
            },
            DebruijnIndex::INNERMOST,
        );
        let impl_rpitit = trait_assoc.bounds.as_ref().map(|_| rpitit);
        (trait_assoc_id, impl_rpitit)
    }))
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
            TyKind::Alias(AliasTy::Projection(ProjectionTy { associated_ty_id, substitution }))
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
                            // This is an unrelated binder, therefore `DebruijnIndex::INNERMOST`.
                            let bound = bound.fold_with(self, DebruijnIndex::INNERMOST);
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
    parent: GenericDefId,
    parent_generics: &'a GenericParams,
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
                    + self.method_generics.len_lifetimes()
                    + self.parent_generics.len(),
            )
            .to_ty(Interner)
        } else if placeholder.parent == self.parent {
            BoundVar::new(
                DebruijnIndex::INNERMOST,
                placeholder.local_id.into_raw().into_u32() as usize
                    + self.parent_generics.len_lifetimes(),
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
                    + self.method_generics.len_lifetimes()
                    + self.parent_generics.len(),
            )
            .to_const(Interner, ty)
        } else if placeholder.parent == self.parent {
            BoundVar::new(
                DebruijnIndex::INNERMOST,
                placeholder.local_id.into_raw().into_u32() as usize
                    + self.parent_generics.len_lifetimes(),
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
                placeholder.local_id.into_raw().into_u32() as usize + self.parent_generics.len(),
            )
            .to_lifetime(Interner)
        } else if placeholder.parent == self.parent {
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
