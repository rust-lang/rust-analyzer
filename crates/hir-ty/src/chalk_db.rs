//! The implementation of `RustIrDatabase` for Chalk, which provides information
//! about the code that Chalk needs.
use core::ops;
use std::{iter, ops::ControlFlow, sync::Arc};

use hir_expand::name::Name;
use intern::sym;
use rustc_hash::FxHashMap;
use span::Edition;
use tracing::debug;

use chalk_ir::{
    Binders, CanonicalVarKinds,
    cast::{Cast, Caster},
    fold::{TypeFoldable, TypeFolder, TypeSuperFoldable, shift::Shift},
};
use chalk_solve::rust_ir::{self, AssociatedTyDatumBound, OpaqueTyDatumBound, WellKnownTrait};

use base_db::Crate;
use hir_def::{
    AssocItemId, BlockId, CallableDefId, FunctionId, GenericDefId, HasModule, ItemContainerId,
    Lookup, TypeAliasId, VariantId,
    hir::{
        Movability,
        generics::{GenericParams, TypeOrConstParamData},
    },
    lang_item::LangItem,
    signatures::{ImplFlags, StructFlags, TraitFlags},
};

use crate::{
    AliasEq, AliasTy, BoundVar, Const, ConstData, ConstValue, DebruijnIndex, DomainGoal, Goal,
    GoalData, InferenceTable, Interner, Lifetime, LifetimeData, PlaceholderIndex, ProjectionTy,
    ProjectionTyExt, QuantifiedWhereClause, Substitution, TraitRef, TraitRefExt, Ty, TyBuilder,
    TyExt, TyKind, VariableKinds, WhereClause,
    db::{
        HirDatabase, InternedCoroutine, RpititImplAssocTy, RpititImplAssocTyId,
        RpititTraitAssocTyId,
    },
    from_assoc_type_id, from_chalk_trait_id, from_foreign_def_id, from_placeholder_idx,
    generics::generics,
    lower::LifetimeElisionKind,
    lower::trait_fn_signature,
    lt_from_placeholder_idx, make_binders, make_single_type_binders,
    mapping::{
        AnyImplAssocType, AnyTraitAssocType, ToChalk, from_assoc_type_value_id, from_chalk,
        to_assoc_type_id_rpitit, to_assoc_type_value_id, to_assoc_type_value_id_rpitit,
    },
    method_resolution::{ALL_FLOAT_FPS, ALL_INT_FPS, TraitImpls, TyFingerprint},
    to_assoc_type_id, to_chalk_trait_id,
    traits::ChalkContext,
    utils::ClosureSubst,
    variable_kinds_from_generics, wrap_empty_binders,
};

pub(crate) type AssociatedTyDatum = chalk_solve::rust_ir::AssociatedTyDatum<Interner>;
pub(crate) type TraitDatum = chalk_solve::rust_ir::TraitDatum<Interner>;
pub(crate) type AdtDatum = chalk_solve::rust_ir::AdtDatum<Interner>;
pub(crate) type ImplDatum = chalk_solve::rust_ir::ImplDatum<Interner>;
pub(crate) type OpaqueTyDatum = chalk_solve::rust_ir::OpaqueTyDatum<Interner>;

pub(crate) type AssocTypeId = chalk_ir::AssocTypeId<Interner>;
pub(crate) type TraitId = chalk_ir::TraitId<Interner>;
pub(crate) type AdtId = chalk_ir::AdtId<Interner>;
pub(crate) type ImplId = chalk_ir::ImplId<Interner>;
pub(crate) type AssociatedTyValueId = chalk_solve::rust_ir::AssociatedTyValueId<Interner>;
pub(crate) type AssociatedTyValue = chalk_solve::rust_ir::AssociatedTyValue<Interner>;
pub(crate) type FnDefDatum = chalk_solve::rust_ir::FnDefDatum<Interner>;
pub(crate) type Variances = chalk_ir::Variances<Interner>;

impl chalk_solve::RustIrDatabase<Interner> for ChalkContext<'_> {
    fn associated_ty_data(&self, id: AssocTypeId) -> Arc<AssociatedTyDatum> {
        match from_assoc_type_id(self.db, id) {
            AnyTraitAssocType::Normal(id) => self.db.associated_ty_data(id),
            AnyTraitAssocType::Rpitit(assoc_type_id) => {
                let assoc_type = assoc_type_id.loc(self.db);
                Arc::new(AssociatedTyDatum {
                    id,
                    trait_id: to_chalk_trait_id(assoc_type.trait_id),
                    name: sym::synthesized_rpitit_assoc,
                    binders: assoc_type
                        .bounds
                        .clone()
                        .map(|bounds| AssociatedTyDatumBound { bounds, where_clauses: Vec::new() }),
                })
            }
        }
    }
    fn associated_ty_from_impl(
        &self,
        impl_id: chalk_ir::ImplId<Interner>,
        assoc_type_id: chalk_ir::AssocTypeId<Interner>,
    ) -> Option<rust_ir::AssociatedTyValueId<Interner>> {
        match from_assoc_type_id(self.db, assoc_type_id) {
            AnyTraitAssocType::Normal(alias_id) => {
                let trait_sig = self.db.type_alias_signature(alias_id);
                self.db
                    .impl_items(hir_def::ImplId::from_chalk(self.db, impl_id))
                    .items
                    .iter()
                    .find_map(|(name, item)| match item {
                        AssocItemId::TypeAliasId(alias) if &trait_sig.name == name => {
                            Some(to_assoc_type_value_id(*alias))
                        }
                        _ => None,
                    })
            }
            AnyTraitAssocType::Rpitit(trait_assoc) => {
                Some(to_assoc_type_value_id_rpitit(RpititImplAssocTyId::new(
                    self.db,
                    RpititImplAssocTy { impl_id: from_chalk(self.db, impl_id), trait_assoc },
                )))
            }
        }
    }
    fn trait_datum(&self, trait_id: TraitId) -> Arc<TraitDatum> {
        self.db.trait_datum(self.krate, trait_id)
    }
    fn adt_datum(&self, struct_id: AdtId) -> Arc<AdtDatum> {
        self.db.adt_datum(self.krate, struct_id)
    }
    fn adt_repr(&self, _struct_id: AdtId) -> Arc<rust_ir::AdtRepr<Interner>> {
        // FIXME: keep track of these
        Arc::new(rust_ir::AdtRepr { c: false, packed: false, int: None })
    }
    fn discriminant_type(&self, ty: chalk_ir::Ty<Interner>) -> chalk_ir::Ty<Interner> {
        if let chalk_ir::TyKind::Adt(id, _) = ty.kind(Interner) {
            if let hir_def::AdtId::EnumId(e) = id.0 {
                let enum_data = self.db.enum_signature(e);
                let ty = enum_data.repr.unwrap_or_default().discr_type();
                return chalk_ir::TyKind::Scalar(match ty {
                    hir_def::layout::IntegerType::Pointer(is_signed) => match is_signed {
                        true => chalk_ir::Scalar::Int(chalk_ir::IntTy::Isize),
                        false => chalk_ir::Scalar::Uint(chalk_ir::UintTy::Usize),
                    },
                    hir_def::layout::IntegerType::Fixed(size, is_signed) => match is_signed {
                        true => chalk_ir::Scalar::Int(match size {
                            hir_def::layout::Integer::I8 => chalk_ir::IntTy::I8,
                            hir_def::layout::Integer::I16 => chalk_ir::IntTy::I16,
                            hir_def::layout::Integer::I32 => chalk_ir::IntTy::I32,
                            hir_def::layout::Integer::I64 => chalk_ir::IntTy::I64,
                            hir_def::layout::Integer::I128 => chalk_ir::IntTy::I128,
                        }),
                        false => chalk_ir::Scalar::Uint(match size {
                            hir_def::layout::Integer::I8 => chalk_ir::UintTy::U8,
                            hir_def::layout::Integer::I16 => chalk_ir::UintTy::U16,
                            hir_def::layout::Integer::I32 => chalk_ir::UintTy::U32,
                            hir_def::layout::Integer::I64 => chalk_ir::UintTy::U64,
                            hir_def::layout::Integer::I128 => chalk_ir::UintTy::U128,
                        }),
                    },
                })
                .intern(Interner);
            }
        }
        chalk_ir::TyKind::Scalar(chalk_ir::Scalar::Uint(chalk_ir::UintTy::U8)).intern(Interner)
    }
    fn impl_datum(&self, impl_id: ImplId) -> Arc<ImplDatum> {
        self.db.impl_datum(self.krate, impl_id)
    }

    fn fn_def_datum(
        &self,
        fn_def_id: chalk_ir::FnDefId<Interner>,
    ) -> Arc<rust_ir::FnDefDatum<Interner>> {
        self.db.fn_def_datum(from_chalk(self.db, fn_def_id))
    }

    fn impls_for_trait(
        &self,
        trait_id: TraitId,
        parameters: &[chalk_ir::GenericArg<Interner>],
        binders: &CanonicalVarKinds<Interner>,
    ) -> Vec<ImplId> {
        debug!("impls_for_trait {:?}", trait_id);
        let trait_: hir_def::TraitId = from_chalk_trait_id(trait_id);

        let ty: Ty = parameters[0].assert_ty_ref(Interner).clone();

        fn binder_kind(
            ty: &Ty,
            binders: &CanonicalVarKinds<Interner>,
        ) -> Option<chalk_ir::TyVariableKind> {
            if let TyKind::BoundVar(bv) = ty.kind(Interner) {
                let binders = binders.as_slice(Interner);
                if bv.debruijn == DebruijnIndex::INNERMOST {
                    if let chalk_ir::VariableKind::Ty(tk) = binders[bv.index].kind {
                        return Some(tk);
                    }
                }
            }
            None
        }

        let self_ty_fp = TyFingerprint::for_trait_impl(&ty);
        let fps: &[TyFingerprint] = match binder_kind(&ty, binders) {
            Some(chalk_ir::TyVariableKind::Integer) => &ALL_INT_FPS,
            Some(chalk_ir::TyVariableKind::Float) => &ALL_FLOAT_FPS,
            _ => self_ty_fp.as_slice(),
        };

        let id_to_chalk = |id: hir_def::ImplId| id.to_chalk(self.db);

        let mut result = vec![];
        if fps.is_empty() {
            debug!("Unrestricted search for {:?} impls...", trait_);
            _ = self.for_trait_impls(trait_, self_ty_fp, |impls| {
                result.extend(impls.for_trait(trait_).map(id_to_chalk));
                ControlFlow::Continue(())
            });
        } else {
            _ =
                self.for_trait_impls(trait_, self_ty_fp, |impls| {
                    result.extend(fps.iter().flat_map(move |fp| {
                        impls.for_trait_and_self_ty(trait_, *fp).map(id_to_chalk)
                    }));
                    ControlFlow::Continue(())
                });
        };

        debug!("impls_for_trait returned {} impls", result.len());
        result
    }

    fn impl_provided_for(&self, auto_trait_id: TraitId, kind: &chalk_ir::TyKind<Interner>) -> bool {
        debug!("impl_provided_for {:?}, {:?}", auto_trait_id, kind);

        let trait_id = from_chalk_trait_id(auto_trait_id);
        let self_ty = kind.clone().intern(Interner);
        // We cannot filter impls by `TyFingerprint` for the following types:
        let self_ty_fp = match kind {
            // because we need to find any impl whose Self type is a ref with the same mutability
            // (we don't care about the inner type).
            TyKind::Ref(..) => None,
            // because we need to find any impl whose Self type is a tuple with the same arity.
            TyKind::Tuple(..) => None,
            _ => TyFingerprint::for_trait_impl(&self_ty),
        };

        let check_kind = |impl_id| {
            let impl_self_ty = self.db.impl_self_ty(impl_id);
            // NOTE(skip_binders): it's safe to skip binders here as we don't check substitutions.
            let impl_self_kind = impl_self_ty.skip_binders().kind(Interner);

            match (kind, impl_self_kind) {
                (TyKind::Adt(id_a, _), TyKind::Adt(id_b, _)) => id_a == id_b,
                (TyKind::AssociatedType(id_a, _), TyKind::AssociatedType(id_b, _)) => id_a == id_b,
                (TyKind::Scalar(scalar_a), TyKind::Scalar(scalar_b)) => scalar_a == scalar_b,
                (TyKind::Error, TyKind::Error)
                | (TyKind::Str, TyKind::Str)
                | (TyKind::Slice(_), TyKind::Slice(_))
                | (TyKind::Never, TyKind::Never)
                | (TyKind::Array(_, _), TyKind::Array(_, _)) => true,
                (TyKind::Tuple(arity_a, _), TyKind::Tuple(arity_b, _)) => arity_a == arity_b,
                (TyKind::OpaqueType(id_a, _), TyKind::OpaqueType(id_b, _)) => id_a == id_b,
                (TyKind::FnDef(id_a, _), TyKind::FnDef(id_b, _)) => id_a == id_b,
                (TyKind::Ref(id_a, _, _), TyKind::Ref(id_b, _, _))
                | (TyKind::Raw(id_a, _), TyKind::Raw(id_b, _)) => id_a == id_b,
                (TyKind::Closure(id_a, _), TyKind::Closure(id_b, _)) => id_a == id_b,
                (TyKind::Coroutine(id_a, _), TyKind::Coroutine(id_b, _))
                | (TyKind::CoroutineWitness(id_a, _), TyKind::CoroutineWitness(id_b, _)) => {
                    id_a == id_b
                }
                (TyKind::Foreign(id_a), TyKind::Foreign(id_b)) => id_a == id_b,
                (_, _) => false,
            }
        };

        if let Some(fp) = self_ty_fp {
            self.for_trait_impls(trait_id, self_ty_fp, |impls| {
                match impls.for_trait_and_self_ty(trait_id, fp).any(check_kind) {
                    true => ControlFlow::Break(()),
                    false => ControlFlow::Continue(()),
                }
            })
        } else {
            self.for_trait_impls(trait_id, self_ty_fp, |impls| {
                match impls.for_trait(trait_id).any(check_kind) {
                    true => ControlFlow::Break(()),
                    false => ControlFlow::Continue(()),
                }
            })
        }
        .is_break()
    }

    fn associated_ty_value(&self, id: AssociatedTyValueId) -> Arc<AssociatedTyValue> {
        self.db.associated_ty_value(self.krate, id)
    }

    fn custom_clauses(&self) -> Vec<chalk_ir::ProgramClause<Interner>> {
        vec![]
    }
    fn local_impls_to_coherence_check(&self, _trait_id: TraitId) -> Vec<ImplId> {
        // We don't do coherence checking (yet)
        unimplemented!()
    }
    fn interner(&self) -> Interner {
        Interner
    }
    fn well_known_trait_id(
        &self,
        well_known_trait: rust_ir::WellKnownTrait,
    ) -> Option<chalk_ir::TraitId<Interner>> {
        let lang_attr = lang_item_from_well_known_trait(well_known_trait);
        let trait_ = lang_attr.resolve_trait(self.db, self.krate)?;
        Some(to_chalk_trait_id(trait_))
    }

    fn program_clauses_for_env(
        &self,
        environment: &chalk_ir::Environment<Interner>,
    ) -> chalk_ir::ProgramClauses<Interner> {
        self.db.program_clauses_for_chalk_env(self.krate, self.block, environment.clone())
    }

    fn opaque_ty_data(&self, id: chalk_ir::OpaqueTyId<Interner>) -> Arc<OpaqueTyDatum> {
        let full_id = self.db.lookup_intern_impl_trait_id(id.into());
        let bound = match full_id {
            crate::ImplTraitId::ReturnTypeImplTrait(func, idx) => {
                let datas = self
                    .db
                    .return_type_impl_traits(func)
                    .expect("impl trait id without impl traits");
                let (datas, binders) = (*datas).as_ref().into_value_and_skipped_binders();
                let data = &datas.impl_traits[idx];
                let bound = OpaqueTyDatumBound {
                    bounds: make_single_type_binders(data.bounds.skip_binders().to_vec()),
                    where_clauses: chalk_ir::Binders::empty(Interner, vec![]),
                };
                chalk_ir::Binders::new(binders, bound)
            }
            crate::ImplTraitId::TypeAliasImplTrait(alias, idx) => {
                let datas = self
                    .db
                    .type_alias_impl_traits(alias)
                    .expect("impl trait id without impl traits");
                let (datas, binders) = (*datas).as_ref().into_value_and_skipped_binders();
                let data = &datas.impl_traits[idx];
                let bound = OpaqueTyDatumBound {
                    bounds: make_single_type_binders(data.bounds.skip_binders().to_vec()),
                    where_clauses: chalk_ir::Binders::empty(Interner, vec![]),
                };
                chalk_ir::Binders::new(binders, bound)
            }
            crate::ImplTraitId::AsyncBlockTypeImplTrait(..) => {
                if let Some((future_trait, future_output)) =
                    LangItem::Future.resolve_trait(self.db, self.krate).and_then(|trait_| {
                        let alias = self
                            .db
                            .trait_items(trait_)
                            .associated_type_by_name(&Name::new_symbol_root(sym::Output))?;
                        Some((trait_, alias))
                    })
                {
                    // Making up Symbol’s value as variable is void: AsyncBlock<T>:
                    //
                    // |--------------------OpaqueTyDatum-------------------|
                    //        |-------------OpaqueTyDatumBound--------------|
                    // for<T> <Self> [Future<Self>, Future::Output<Self> = T]
                    //     ^1  ^0            ^0                    ^0      ^1
                    let impl_bound = WhereClause::Implemented(TraitRef {
                        trait_id: to_chalk_trait_id(future_trait),
                        // Self type as the first parameter.
                        substitution: Substitution::from1(
                            Interner,
                            TyKind::BoundVar(BoundVar {
                                debruijn: DebruijnIndex::INNERMOST,
                                index: 0,
                            })
                            .intern(Interner),
                        ),
                    });
                    let mut binder = vec![];
                    binder.push(crate::wrap_empty_binders(impl_bound));
                    let sized_trait = LangItem::Sized.resolve_trait(self.db, self.krate);
                    if let Some(sized_trait_) = sized_trait {
                        let sized_bound = WhereClause::Implemented(TraitRef {
                            trait_id: to_chalk_trait_id(sized_trait_),
                            // Self type as the first parameter.
                            substitution: Substitution::from1(
                                Interner,
                                TyKind::BoundVar(BoundVar {
                                    debruijn: DebruijnIndex::INNERMOST,
                                    index: 0,
                                })
                                .intern(Interner),
                            ),
                        });
                        binder.push(crate::wrap_empty_binders(sized_bound));
                    }
                    let proj_bound = WhereClause::AliasEq(AliasEq {
                        alias: AliasTy::Projection(ProjectionTy {
                            associated_ty_id: to_assoc_type_id(future_output),
                            // Self type as the first parameter.
                            substitution: Substitution::from1(
                                Interner,
                                TyKind::BoundVar(BoundVar::new(DebruijnIndex::INNERMOST, 0))
                                    .intern(Interner),
                            ),
                        }),
                        // The parameter of the opaque type.
                        ty: TyKind::BoundVar(BoundVar { debruijn: DebruijnIndex::ONE, index: 0 })
                            .intern(Interner),
                    });
                    binder.push(crate::wrap_empty_binders(proj_bound));
                    let bound = OpaqueTyDatumBound {
                        bounds: make_single_type_binders(binder),
                        where_clauses: chalk_ir::Binders::empty(Interner, vec![]),
                    };
                    // The opaque type has 1 parameter.
                    make_single_type_binders(bound)
                } else {
                    // If failed to find Symbol’s value as variable is void: Future::Output, return empty bounds as fallback.
                    let bound = OpaqueTyDatumBound {
                        bounds: chalk_ir::Binders::empty(Interner, vec![]),
                        where_clauses: chalk_ir::Binders::empty(Interner, vec![]),
                    };
                    // The opaque type has 1 parameter.
                    make_single_type_binders(bound)
                }
            }
        };

        Arc::new(OpaqueTyDatum { opaque_ty_id: id, bound })
    }

    fn hidden_opaque_type(&self, _id: chalk_ir::OpaqueTyId<Interner>) -> chalk_ir::Ty<Interner> {
        // FIXME: actually provide the hidden type; it is relevant for auto traits
        TyKind::Error.intern(Interner)
    }

    // object safety was renamed to dyn-compatibility but still remains here in chalk.
    // This will be removed since we are going to migrate to next-gen trait solver.
    fn is_object_safe(&self, trait_id: chalk_ir::TraitId<Interner>) -> bool {
        let trait_ = from_chalk_trait_id(trait_id);
        crate::dyn_compatibility::dyn_compatibility(self.db, trait_).is_none()
    }

    fn closure_kind(
        &self,
        _closure_id: chalk_ir::ClosureId<Interner>,
        _substs: &chalk_ir::Substitution<Interner>,
    ) -> rust_ir::ClosureKind {
        // Fn is the closure kind that implements all three traits
        rust_ir::ClosureKind::Fn
    }
    fn closure_inputs_and_output(
        &self,
        _closure_id: chalk_ir::ClosureId<Interner>,
        substs: &chalk_ir::Substitution<Interner>,
    ) -> chalk_ir::Binders<rust_ir::FnDefInputsAndOutputDatum<Interner>> {
        let sig_ty = ClosureSubst(substs).sig_ty();
        let sig = &sig_ty.callable_sig(self.db).expect("first closure param should be fn ptr");
        let io = rust_ir::FnDefInputsAndOutputDatum {
            argument_types: sig.params().to_vec(),
            return_type: sig.ret().clone(),
        };
        chalk_ir::Binders::empty(Interner, io.shifted_in(Interner))
    }
    fn closure_upvars(
        &self,
        _closure_id: chalk_ir::ClosureId<Interner>,
        _substs: &chalk_ir::Substitution<Interner>,
    ) -> chalk_ir::Binders<chalk_ir::Ty<Interner>> {
        let ty = TyBuilder::unit();
        chalk_ir::Binders::empty(Interner, ty)
    }
    fn closure_fn_substitution(
        &self,
        _closure_id: chalk_ir::ClosureId<Interner>,
        _substs: &chalk_ir::Substitution<Interner>,
    ) -> chalk_ir::Substitution<Interner> {
        Substitution::empty(Interner)
    }

    fn trait_name(&self, trait_id: chalk_ir::TraitId<Interner>) -> String {
        let id = from_chalk_trait_id(trait_id);
        self.db.trait_signature(id).name.display(self.db, self.edition()).to_string()
    }
    fn adt_name(&self, chalk_ir::AdtId(adt_id): AdtId) -> String {
        let edition = self.edition();
        match adt_id {
            hir_def::AdtId::StructId(id) => {
                self.db.struct_signature(id).name.display(self.db, edition).to_string()
            }
            hir_def::AdtId::EnumId(id) => {
                self.db.enum_signature(id).name.display(self.db, edition).to_string()
            }
            hir_def::AdtId::UnionId(id) => {
                self.db.union_signature(id).name.display(self.db, edition).to_string()
            }
        }
    }
    fn adt_size_align(&self, _id: chalk_ir::AdtId<Interner>) -> Arc<rust_ir::AdtSizeAlign> {
        // FIXME
        Arc::new(rust_ir::AdtSizeAlign::from_one_zst(false))
    }
    fn assoc_type_name(&self, assoc_ty_id: chalk_ir::AssocTypeId<Interner>) -> String {
        let name = match from_assoc_type_id(self.db, assoc_ty_id) {
            AnyTraitAssocType::Normal(id) => self.db.type_alias_signature(id).name.clone(),
            AnyTraitAssocType::Rpitit(id) => {
                self.db.function_signature(id.loc(self.db).synthesized_from_method).name.clone()
            }
        };
        name.display(self.db, self.edition()).to_string()
    }
    fn opaque_type_name(&self, opaque_ty_id: chalk_ir::OpaqueTyId<Interner>) -> String {
        format!("Opaque_{:?}", opaque_ty_id.0)
    }
    fn fn_def_name(&self, fn_def_id: chalk_ir::FnDefId<Interner>) -> String {
        format!("fn_{:?}", fn_def_id.0)
    }
    fn coroutine_datum(
        &self,
        id: chalk_ir::CoroutineId<Interner>,
    ) -> Arc<chalk_solve::rust_ir::CoroutineDatum<Interner>> {
        let InternedCoroutine(parent, expr) = self.db.lookup_intern_coroutine(id.into());

        // We fill substitution with unknown type, because we only need to know whether the generic
        // params are types or consts to build `Binders` and those being filled up are for
        // `resume_type`, `yield_type`, and `return_type` of the coroutine in question.
        let subst = TyBuilder::subst_for_coroutine(self.db, parent).fill_with_unknown().build();

        let len = subst.len(Interner);
        let input_output = rust_ir::CoroutineInputOutputDatum {
            resume_type: TyKind::BoundVar(BoundVar::new(DebruijnIndex::INNERMOST, len - 3))
                .intern(Interner),
            yield_type: TyKind::BoundVar(BoundVar::new(DebruijnIndex::INNERMOST, len - 2))
                .intern(Interner),
            return_type: TyKind::BoundVar(BoundVar::new(DebruijnIndex::INNERMOST, len - 1))
                .intern(Interner),
            // FIXME: calculate upvars
            upvars: vec![],
        };

        let it = subst
            .iter(Interner)
            .map(|it| it.constant(Interner).map(|c| c.data(Interner).ty.clone()));
        let input_output = crate::make_type_and_const_binders(it, input_output);

        let movability = match self.db.body(parent)[expr] {
            hir_def::hir::Expr::Closure {
                closure_kind: hir_def::hir::ClosureKind::Coroutine(movability),
                ..
            } => movability,
            _ => unreachable!("non coroutine expression interned as coroutine"),
        };
        let movability = match movability {
            Movability::Static => rust_ir::Movability::Static,
            Movability::Movable => rust_ir::Movability::Movable,
        };

        Arc::new(rust_ir::CoroutineDatum { movability, input_output })
    }
    fn coroutine_witness_datum(
        &self,
        id: chalk_ir::CoroutineId<Interner>,
    ) -> Arc<chalk_solve::rust_ir::CoroutineWitnessDatum<Interner>> {
        // FIXME: calculate inner types
        let inner_types =
            rust_ir::CoroutineWitnessExistential { types: wrap_empty_binders(vec![]) };

        let InternedCoroutine(parent, _) = self.db.lookup_intern_coroutine(id.into());
        // See the comment in `coroutine_datum()` for unknown types.
        let subst = TyBuilder::subst_for_coroutine(self.db, parent).fill_with_unknown().build();
        let it = subst
            .iter(Interner)
            .map(|it| it.constant(Interner).map(|c| c.data(Interner).ty.clone()));
        let inner_types = crate::make_type_and_const_binders(it, inner_types);

        Arc::new(rust_ir::CoroutineWitnessDatum { inner_types })
    }

    fn unification_database(&self) -> &dyn chalk_ir::UnificationDatabase<Interner> {
        &self.db
    }
}

impl ChalkContext<'_> {
    fn edition(&self) -> Edition {
        self.krate.data(self.db).edition
    }

    fn for_trait_impls(
        &self,
        trait_id: hir_def::TraitId,
        self_ty_fp: Option<TyFingerprint>,
        mut f: impl FnMut(&TraitImpls) -> ControlFlow<()>,
    ) -> ControlFlow<()> {
        // Note: Since we're using `impls_for_trait` and `impl_provided_for`,
        // only impls where the trait can be resolved should ever reach Chalk.
        // `impl_datum` relies on that and will panic if the trait can't be resolved.
        let in_deps = self.db.trait_impls_in_deps(self.krate);
        let in_self = self.db.trait_impls_in_crate(self.krate);
        let trait_module = trait_id.module(self.db);
        let type_module = match self_ty_fp {
            Some(TyFingerprint::Adt(adt_id)) => Some(adt_id.module(self.db)),
            Some(TyFingerprint::ForeignType(type_id)) => {
                Some(from_foreign_def_id(type_id).module(self.db))
            }
            Some(TyFingerprint::Dyn(trait_id)) => Some(trait_id.module(self.db)),
            _ => None,
        };

        let mut def_blocks =
            [trait_module.containing_block(), type_module.and_then(|it| it.containing_block())];

        let block_impls = iter::successors(self.block, |&block_id| {
            cov_mark::hit!(block_local_impls);
            block_id.loc(self.db).module.containing_block()
        })
        .inspect(|&block_id| {
            // make sure we don't search the same block twice
            def_blocks.iter_mut().for_each(|block| {
                if *block == Some(block_id) {
                    *block = None;
                }
            });
        })
        .filter_map(|block_id| self.db.trait_impls_in_block(block_id));
        f(&in_self)?;
        for it in in_deps.iter().map(ops::Deref::deref) {
            f(it)?;
        }
        for it in block_impls {
            f(&it)?;
        }
        for it in def_blocks.into_iter().flatten().filter_map(|it| self.db.trait_impls_in_block(it))
        {
            f(&it)?;
        }
        ControlFlow::Continue(())
    }
}

impl chalk_ir::UnificationDatabase<Interner> for &dyn HirDatabase {
    fn fn_def_variance(
        &self,
        fn_def_id: chalk_ir::FnDefId<Interner>,
    ) -> chalk_ir::Variances<Interner> {
        HirDatabase::fn_def_variance(*self, from_chalk(*self, fn_def_id))
    }

    fn adt_variance(&self, adt_id: chalk_ir::AdtId<Interner>) -> chalk_ir::Variances<Interner> {
        HirDatabase::adt_variance(*self, adt_id.0)
    }
}

pub(crate) fn program_clauses_for_chalk_env_query(
    db: &dyn HirDatabase,
    krate: Crate,
    block: Option<BlockId>,
    environment: chalk_ir::Environment<Interner>,
) -> chalk_ir::ProgramClauses<Interner> {
    chalk_solve::program_clauses_for_env(&ChalkContext { db, krate, block }, &environment)
}

pub(crate) fn associated_ty_data_query(
    db: &dyn HirDatabase,
    type_alias: TypeAliasId,
) -> Arc<AssociatedTyDatum> {
    debug!("associated_ty_data {:?}", type_alias);
    let trait_ = match type_alias.lookup(db).container {
        ItemContainerId::TraitId(t) => t,
        _ => panic!("associated type not in trait"),
    };

    // Lower bounds -- we could/should maybe move this to a separate query in `lower`
    let type_alias_data = db.type_alias_signature(type_alias);
    let generic_params = generics(db, type_alias.into());
    let resolver = hir_def::resolver::HasResolver::resolver(type_alias, db);
    let mut ctx = crate::TyLoweringContext::new(
        db,
        &resolver,
        &type_alias_data.store,
        type_alias.into(),
        LifetimeElisionKind::AnonymousReportError,
    )
    .with_type_param_mode(crate::lower::ParamLoweringMode::Variable);

    let trait_subst = TyBuilder::subst_for_def(db, trait_, None)
        .fill_with_bound_vars(crate::DebruijnIndex::INNERMOST, 0)
        .build();
    let pro_ty = TyBuilder::assoc_type_projection(db, type_alias, Some(trait_subst))
        .fill_with_bound_vars(
            crate::DebruijnIndex::INNERMOST,
            generic_params.parent_generics().map_or(0, |it| it.len()),
        )
        .build();
    let self_ty = TyKind::Alias(AliasTy::Projection(pro_ty)).intern(Interner);

    let mut bounds = Vec::new();
    for bound in &type_alias_data.bounds {
        ctx.lower_type_bound(bound, self_ty.clone(), false).for_each(|pred| {
            if let Some(pred) = generic_predicate_to_inline_bound(db, &pred, &self_ty) {
                bounds.push(pred);
            }
        });
    }

    if !ctx.unsized_types.contains(&self_ty) {
        let sized_trait =
            LangItem::Sized.resolve_trait(db, resolver.krate()).map(to_chalk_trait_id);
        let sized_bound = sized_trait.into_iter().map(|sized_trait| {
            let trait_bound =
                rust_ir::TraitBound { trait_id: sized_trait, args_no_self: Default::default() };
            let inline_bound = rust_ir::InlineBound::TraitBound(trait_bound);
            chalk_ir::Binders::empty(Interner, inline_bound)
        });
        bounds.extend(sized_bound);
        bounds.shrink_to_fit();
    }

    // FIXME: Re-enable where clauses on associated types when an upstream chalk bug is fixed.
    //        (rust-analyzer#9052)
    // let where_clauses = convert_where_clauses(db, type_alias.into(), &bound_vars);
    let bound_data = rust_ir::AssociatedTyDatumBound { bounds, where_clauses: vec![] };
    let datum = AssociatedTyDatum {
        trait_id: to_chalk_trait_id(trait_),
        id: to_assoc_type_id(type_alias),
        name: type_alias_data.name.symbol().clone(),
        binders: make_binders(db, &generic_params, bound_data),
    };
    Arc::new(datum)
}

pub(crate) fn trait_datum_query(
    db: &dyn HirDatabase,
    krate: Crate,
    trait_id: TraitId,
) -> Arc<TraitDatum> {
    debug!("trait_datum {:?}", trait_id);
    let trait_ = from_chalk_trait_id(trait_id);
    let trait_data = db.trait_signature(trait_);
    debug!("trait {:?} = {:?}", trait_id, trait_data.name);
    let generic_params = generics(db, trait_.into());
    let bound_vars = generic_params.bound_vars_subst(db, DebruijnIndex::INNERMOST);
    let flags = rust_ir::TraitFlags {
        auto: trait_data.flags.contains(TraitFlags::AUTO),
        upstream: trait_.lookup(db).container.krate() != krate,
        non_enumerable: true,
        coinductive: false, // only relevant for Chalk testing
        // FIXME: set these flags correctly
        marker: false,
        fundamental: trait_data.flags.contains(TraitFlags::FUNDAMENTAL),
    };
    let where_clauses = convert_where_clauses(db, trait_.into(), &bound_vars);
    let trait_items = db.trait_items(trait_);

    let rpitits = trait_items
        .items
        .iter()
        .filter_map(|&(_, item)| match item {
            AssocItemId::FunctionId(it) => Some(it),
            _ => None,
        })
        .flat_map(|method| &trait_fn_signature(db, method).1)
        .map(|assoc_id| to_assoc_type_id_rpitit(*assoc_id));
    let associated_ty_ids =
        trait_items.associated_types().map(to_assoc_type_id).chain(rpitits).collect();

    let trait_datum_bound = rust_ir::TraitDatumBound { where_clauses };
    let well_known = db.lang_attr(trait_.into()).and_then(well_known_trait_from_lang_item);
    let trait_datum = TraitDatum {
        id: trait_id,
        binders: make_binders(db, &generic_params, trait_datum_bound),
        flags,
        associated_ty_ids,
        well_known,
    };
    Arc::new(trait_datum)
}

fn well_known_trait_from_lang_item(item: LangItem) -> Option<WellKnownTrait> {
    Some(match item {
        LangItem::Clone => WellKnownTrait::Clone,
        LangItem::CoerceUnsized => WellKnownTrait::CoerceUnsized,
        LangItem::Copy => WellKnownTrait::Copy,
        LangItem::DiscriminantKind => WellKnownTrait::DiscriminantKind,
        LangItem::DispatchFromDyn => WellKnownTrait::DispatchFromDyn,
        LangItem::Drop => WellKnownTrait::Drop,
        LangItem::Fn => WellKnownTrait::Fn,
        LangItem::FnMut => WellKnownTrait::FnMut,
        LangItem::FnOnce => WellKnownTrait::FnOnce,
        LangItem::AsyncFn => WellKnownTrait::AsyncFn,
        LangItem::AsyncFnMut => WellKnownTrait::AsyncFnMut,
        LangItem::AsyncFnOnce => WellKnownTrait::AsyncFnOnce,
        LangItem::Coroutine => WellKnownTrait::Coroutine,
        LangItem::Sized => WellKnownTrait::Sized,
        LangItem::Unpin => WellKnownTrait::Unpin,
        LangItem::Unsize => WellKnownTrait::Unsize,
        LangItem::Tuple => WellKnownTrait::Tuple,
        LangItem::PointeeTrait => WellKnownTrait::Pointee,
        LangItem::FnPtrTrait => WellKnownTrait::FnPtr,
        LangItem::Future => WellKnownTrait::Future,
        _ => return None,
    })
}

fn lang_item_from_well_known_trait(trait_: WellKnownTrait) -> LangItem {
    match trait_ {
        WellKnownTrait::Clone => LangItem::Clone,
        WellKnownTrait::CoerceUnsized => LangItem::CoerceUnsized,
        WellKnownTrait::Copy => LangItem::Copy,
        WellKnownTrait::DiscriminantKind => LangItem::DiscriminantKind,
        WellKnownTrait::DispatchFromDyn => LangItem::DispatchFromDyn,
        WellKnownTrait::Drop => LangItem::Drop,
        WellKnownTrait::Fn => LangItem::Fn,
        WellKnownTrait::FnMut => LangItem::FnMut,
        WellKnownTrait::FnOnce => LangItem::FnOnce,
        WellKnownTrait::AsyncFn => LangItem::AsyncFn,
        WellKnownTrait::AsyncFnMut => LangItem::AsyncFnMut,
        WellKnownTrait::AsyncFnOnce => LangItem::AsyncFnOnce,
        WellKnownTrait::Coroutine => LangItem::Coroutine,
        WellKnownTrait::Sized => LangItem::Sized,
        WellKnownTrait::Tuple => LangItem::Tuple,
        WellKnownTrait::Unpin => LangItem::Unpin,
        WellKnownTrait::Unsize => LangItem::Unsize,
        WellKnownTrait::Pointee => LangItem::PointeeTrait,
        WellKnownTrait::FnPtr => LangItem::FnPtrTrait,
        WellKnownTrait::Future => LangItem::Future,
    }
}

pub(crate) fn adt_datum_query(
    db: &dyn HirDatabase,
    krate: Crate,
    chalk_ir::AdtId(adt_id): AdtId,
) -> Arc<AdtDatum> {
    debug!("adt_datum {:?}", adt_id);
    let generic_params = generics(db, adt_id.into());
    let bound_vars_subst = generic_params.bound_vars_subst(db, DebruijnIndex::INNERMOST);
    let where_clauses = convert_where_clauses(db, adt_id.into(), &bound_vars_subst);

    let (fundamental, phantom_data) = match adt_id {
        hir_def::AdtId::StructId(s) => {
            let flags = db.struct_signature(s).flags;
            (flags.contains(StructFlags::FUNDAMENTAL), flags.contains(StructFlags::IS_PHANTOM_DATA))
        }
        // FIXME set fundamental flags correctly
        hir_def::AdtId::UnionId(_) => (false, false),
        hir_def::AdtId::EnumId(_) => (false, false),
    };
    let flags = rust_ir::AdtFlags {
        upstream: adt_id.module(db).krate() != krate,
        fundamental,
        phantom_data,
    };

    // this slows down rust-analyzer by quite a bit unfortunately, so enabling this is currently not worth it
    let _variant_id_to_fields = |id: VariantId| {
        let variant_data = &id.variant_data(db);
        let fields = if variant_data.fields().is_empty() {
            vec![]
        } else {
            let field_types = db.field_types(id);
            variant_data
                .fields()
                .iter()
                .map(|(idx, _)| field_types[idx].clone().substitute(Interner, &bound_vars_subst))
                .filter(|it| !it.contains_unknown())
                .collect()
        };
        rust_ir::AdtVariantDatum { fields }
    };
    let variant_id_to_fields = |_: VariantId| rust_ir::AdtVariantDatum { fields: vec![] };

    let (kind, variants) = match adt_id {
        hir_def::AdtId::StructId(id) => {
            (rust_ir::AdtKind::Struct, vec![variant_id_to_fields(id.into())])
        }
        hir_def::AdtId::EnumId(id) => {
            let variants = db
                .enum_variants(id)
                .variants
                .iter()
                .map(|&(variant_id, _)| variant_id_to_fields(variant_id.into()))
                .collect();
            (rust_ir::AdtKind::Enum, variants)
        }
        hir_def::AdtId::UnionId(id) => {
            (rust_ir::AdtKind::Union, vec![variant_id_to_fields(id.into())])
        }
    };

    let struct_datum_bound = rust_ir::AdtDatumBound { variants, where_clauses };
    let struct_datum = AdtDatum {
        kind,
        id: chalk_ir::AdtId(adt_id),
        binders: make_binders(db, &generic_params, struct_datum_bound),
        flags,
    };
    Arc::new(struct_datum)
}

pub(crate) fn impl_datum_query(
    db: &dyn HirDatabase,
    krate: Crate,
    impl_id: ImplId,
) -> Arc<ImplDatum> {
    let _p = tracing::info_span!("impl_datum_query").entered();
    debug!("impl_datum {:?}", impl_id);
    let impl_: hir_def::ImplId = from_chalk(db, impl_id);
    impl_def_datum(db, krate, impl_)
}

fn impl_def_datum(db: &dyn HirDatabase, krate: Crate, impl_id: hir_def::ImplId) -> Arc<ImplDatum> {
    let trait_ref_binders = db
        .impl_trait(impl_id)
        // ImplIds for impls where the trait ref can't be resolved should never reach Chalk
        .expect("invalid impl passed to Chalk");
    let trait_ref = trait_ref_binders.skip_binders().clone();
    let impl_data = db.impl_signature(impl_id);

    let generic_params = generics(db, impl_id.into());
    let bound_vars = generic_params.bound_vars_subst(db, DebruijnIndex::INNERMOST);
    let trait_ = trait_ref.hir_trait_id();
    let impl_type = if impl_id.lookup(db).container.krate() == krate {
        rust_ir::ImplType::Local
    } else {
        rust_ir::ImplType::External
    };
    let where_clauses = convert_where_clauses(db, impl_id.into(), &bound_vars);
    let negative = impl_data.flags.contains(ImplFlags::NEGATIVE);
    let polarity = if negative { rust_ir::Polarity::Negative } else { rust_ir::Polarity::Positive };

    let impl_datum_bound = rust_ir::ImplDatumBound { trait_ref, where_clauses };
    let trait_data = db.trait_items(trait_);
    let impl_items = db.impl_items(impl_id);
    let trait_datum = db.trait_datum(krate, to_chalk_trait_id(trait_));
    let associated_ty_value_ids = impl_items
        .items
        .iter()
        .filter_map(|(_, item)| match item {
            AssocItemId::TypeAliasId(type_alias) => Some(*type_alias),
            _ => None,
        })
        .filter(|&type_alias| {
            // don't include associated types that don't exist in the trait
            let name = &db.type_alias_signature(type_alias).name;
            trait_data.associated_type_by_name(name).is_some()
        })
        .map(to_assoc_type_value_id)
        .chain(trait_datum.associated_ty_ids.iter().filter_map(|&trait_assoc| {
            match from_assoc_type_id(db, trait_assoc) {
                AnyTraitAssocType::Rpitit(trait_assoc) => Some(to_assoc_type_value_id_rpitit(
                    RpititImplAssocTyId::new(db, RpititImplAssocTy { impl_id, trait_assoc }),
                )),
                AnyTraitAssocType::Normal(_) => None,
            }
        }))
        .collect();
    debug!("impl_datum: {:?}", impl_datum_bound);
    let impl_datum = ImplDatum {
        binders: make_binders(db, &generic_params, impl_datum_bound),
        impl_type,
        polarity,
        associated_ty_value_ids,
    };
    Arc::new(impl_datum)
}

// We return a list and not a hasmap because the number of RPITITs in a function should be small.
#[salsa_macros::tracked(return_ref)]
fn impl_method_rpitit_values(
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
        &impl_method_placeholder_subst.as_slice(Interner)[impl_method_generics.len_self()..];
    // We want to substitute the TraitRef with placeholders, but placeholders from the method, not the impl.
    let impl_trait_ref = impl_datum
        .binders
        .as_ref()
        .map(|it| it.trait_ref.clone())
        .substitute(Interner, trait_ref_placeholder_subst);
    let trait_to_impl_args = Substitution::from_iter(
        Interner,
        impl_method_placeholder_subst.as_slice(Interner)[..impl_method_generics.len_self()]
            .iter()
            .chain(impl_trait_ref.substitution.as_slice(Interner)),
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
            let impl_rpitit = Binders::new(
                impl_rpitit_binders,
                rust_ir::AssociatedTyValueBound { ty: impl_rpitit },
            );
            Arc::new(AssociatedTyValue {
                associated_ty_id: to_assoc_type_id_rpitit(trait_assoc_id),
                impl_id: hir_def::ImplId::to_chalk(impl_id, db),
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

pub(crate) fn inline_bound_to_generic_predicate(
    bound: &Binders<rust_ir::InlineBound<Interner>>,
    self_ty: Ty,
) -> QuantifiedWhereClause {
    let (bound, binders) = bound.as_ref().into_value_and_skipped_binders();
    match bound {
        rust_ir::InlineBound::TraitBound(trait_bound) => {
            let trait_ref = TraitRef {
                trait_id: trait_bound.trait_id,
                substitution: Substitution::from_iter(
                    Interner,
                    iter::once(self_ty.cast(Interner))
                        .chain(trait_bound.args_no_self.iter().cloned()),
                ),
            };
            chalk_ir::Binders::new(binders, WhereClause::Implemented(trait_ref))
        }
        rust_ir::InlineBound::AliasEqBound(alias_eq) => {
            let substitution = Substitution::from_iter(
                Interner,
                iter::once(self_ty.cast(Interner)).chain(
                    alias_eq
                        .trait_bound
                        .args_no_self
                        .iter()
                        .cloned()
                        .chain(alias_eq.parameters.iter().cloned()),
                ),
            );
            let alias = AliasEq {
                ty: alias_eq.value.clone(),
                alias: AliasTy::Projection(ProjectionTy {
                    associated_ty_id: alias_eq.associated_ty_id,
                    substitution,
                }),
            };
            chalk_ir::Binders::new(binders, WhereClause::AliasEq(alias))
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

pub(crate) fn associated_ty_value_query(
    db: &dyn HirDatabase,
    krate: Crate,
    id: AssociatedTyValueId,
) -> Arc<AssociatedTyValue> {
    match from_assoc_type_value_id(db, id) {
        AnyImplAssocType::Normal(type_alias) => {
            type_alias_associated_ty_value(db, krate, type_alias)
        }
        AnyImplAssocType::Rpitit(assoc_type_id) => rpitit_associated_ty_value(db, assoc_type_id),
    }
}

fn rpitit_associated_ty_value(
    db: &dyn HirDatabase,
    assoc_type_id: RpititImplAssocTyId,
) -> Arc<AssociatedTyValue> {
    let assoc_type = assoc_type_id.loc(db);
    let trait_assoc = assoc_type.trait_assoc.loc(db);
    let all_method_assocs =
        impl_method_rpitit_values(db, assoc_type.impl_id, trait_assoc.synthesized_from_method);
    let trait_assoc_id = to_assoc_type_id_rpitit(assoc_type.trait_assoc);
    all_method_assocs
        .iter()
        .find(|method_assoc| method_assoc.associated_ty_id == trait_assoc_id)
        .cloned()
        .unwrap_or_else(|| {
            let impl_id = hir_def::ImplId::to_chalk(assoc_type.impl_id, db);
            let trait_method_generics = generics(db, trait_assoc.synthesized_from_method.into());
            let impl_generics = generics(db, assoc_type.impl_id.into());
            // In this situation, we don't know even that the trait and impl generics match, therefore
            // the only binders we can give to comply with the trait's binders are the trait's binders.
            // However, for impl associated types chalk wants only their own generics, excluding
            // those of the impl (unlike in traits), therefore we filter them here.
            // Completely unlike the docs, Chalk requires both the impl generics and the associated type
            // generics in the binder.
            let value = Binders::new(
                VariableKinds::from_iter(
                    Interner,
                    trait_assoc.bounds.binders.as_slice(Interner)
                        [..trait_method_generics.len_self()]
                        .iter()
                        .cloned()
                        .chain(variable_kinds_from_generics(db, impl_generics.iter_id())),
                ),
                rust_ir::AssociatedTyValueBound { ty: TyKind::Error.intern(Interner) },
            );
            Arc::new(AssociatedTyValue { associated_ty_id: trait_assoc_id, impl_id, value })
        })
}

fn type_alias_associated_ty_value(
    db: &dyn HirDatabase,
    _krate: Crate,
    type_alias: TypeAliasId,
) -> Arc<AssociatedTyValue> {
    let type_alias_data = db.type_alias_signature(type_alias);
    let impl_id = match type_alias.lookup(db).container {
        ItemContainerId::ImplId(it) => it,
        _ => panic!("assoc ty value should be in impl"),
    };

    let trait_ref = db
        .impl_trait(impl_id)
        .expect("assoc ty value should not exist")
        .into_value_and_skipped_binders()
        .0; // we don't return any assoc ty values if the impl'd trait can't be resolved

    let assoc_ty = db
        .trait_items(trait_ref.hir_trait_id())
        .associated_type_by_name(&type_alias_data.name)
        .expect("assoc ty value should not exist"); // validated when building the impl data as well
    let (ty, binders) = db.ty(type_alias.into()).into_value_and_skipped_binders();
    let value_bound = rust_ir::AssociatedTyValueBound { ty };
    let value = rust_ir::AssociatedTyValue {
        impl_id: impl_id.to_chalk(db),
        associated_ty_id: to_assoc_type_id(assoc_ty),
        value: chalk_ir::Binders::new(binders, value_bound),
    };
    Arc::new(value)
}

pub(crate) fn fn_def_datum_query(
    db: &dyn HirDatabase,
    callable_def: CallableDefId,
) -> Arc<FnDefDatum> {
    let generic_def = GenericDefId::from_callable(db, callable_def);
    let generic_params = generics(db, generic_def);
    let (sig, binders) = db.callable_item_signature(callable_def).into_value_and_skipped_binders();
    let bound_vars = generic_params.bound_vars_subst(db, DebruijnIndex::INNERMOST);
    let where_clauses = convert_where_clauses(db, generic_def, &bound_vars);
    let bound = rust_ir::FnDefDatumBound {
        // Note: Chalk doesn't actually use this information yet as far as I am aware, but we provide it anyway
        inputs_and_output: chalk_ir::Binders::empty(
            Interner,
            rust_ir::FnDefInputsAndOutputDatum {
                argument_types: sig.params().to_vec(),
                return_type: sig.ret().clone(),
            }
            .shifted_in(Interner),
        ),
        where_clauses,
    };
    let datum = FnDefDatum {
        id: callable_def.to_chalk(db),
        sig: chalk_ir::FnSig {
            abi: sig.abi,
            safety: chalk_ir::Safety::Safe,
            variadic: sig.is_varargs,
        },
        binders: chalk_ir::Binders::new(binders, bound),
    };
    Arc::new(datum)
}

pub(crate) fn fn_def_variance_query(
    db: &dyn HirDatabase,
    callable_def: CallableDefId,
) -> Variances {
    Variances::from_iter(
        Interner,
        db.variances_of(GenericDefId::from_callable(db, callable_def))
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|v| match v {
                crate::variance::Variance::Covariant => chalk_ir::Variance::Covariant,
                crate::variance::Variance::Invariant => chalk_ir::Variance::Invariant,
                crate::variance::Variance::Contravariant => chalk_ir::Variance::Contravariant,
                crate::variance::Variance::Bivariant => chalk_ir::Variance::Invariant,
            }),
    )
}

pub(crate) fn adt_variance_query(db: &dyn HirDatabase, adt_id: hir_def::AdtId) -> Variances {
    Variances::from_iter(
        Interner,
        db.variances_of(adt_id.into()).as_deref().unwrap_or_default().iter().map(|v| match v {
            crate::variance::Variance::Covariant => chalk_ir::Variance::Covariant,
            crate::variance::Variance::Invariant => chalk_ir::Variance::Invariant,
            crate::variance::Variance::Contravariant => chalk_ir::Variance::Contravariant,
            crate::variance::Variance::Bivariant => chalk_ir::Variance::Invariant,
        }),
    )
}

/// Returns instantiated predicates.
pub(super) fn convert_where_clauses(
    db: &dyn HirDatabase,
    def: GenericDefId,
    substs: &Substitution,
) -> Vec<chalk_ir::QuantifiedWhereClause<Interner>> {
    db.generic_predicates(def)
        .iter()
        .cloned()
        .map(|pred| pred.substitute(Interner, substs))
        .collect()
}

pub(super) fn generic_predicate_to_inline_bound(
    db: &dyn HirDatabase,
    pred: &QuantifiedWhereClause,
    self_ty: &Ty,
) -> Option<chalk_ir::Binders<rust_ir::InlineBound<Interner>>> {
    // An InlineBound is like a GenericPredicate, except the self type is left out.
    // We don't have a special type for this, but Chalk does.
    let self_ty_shifted_in = self_ty.clone().shifted_in_from(Interner, DebruijnIndex::ONE);
    let (pred, binders) = pred.as_ref().into_value_and_skipped_binders();
    match pred {
        WhereClause::Implemented(trait_ref) => {
            if trait_ref.self_type_parameter(Interner) != self_ty_shifted_in {
                // we can only convert predicates back to type bounds if they
                // have the expected self type
                return None;
            }
            let args_no_self = trait_ref.substitution.as_slice(Interner)[1..]
                .iter()
                .cloned()
                .casted(Interner)
                .collect();
            let trait_bound = rust_ir::TraitBound { trait_id: trait_ref.trait_id, args_no_self };
            Some(chalk_ir::Binders::new(binders, rust_ir::InlineBound::TraitBound(trait_bound)))
        }
        WhereClause::AliasEq(AliasEq { alias: AliasTy::Projection(projection_ty), ty }) => {
            let generic_def = match from_assoc_type_id(db, projection_ty.associated_ty_id) {
                AnyTraitAssocType::Normal(type_alias) => type_alias.into(),
                AnyTraitAssocType::Rpitit(_) => {
                    unreachable!(
                        "there is no way to refer to a RPITIT synthesized \
                        associated type on associated type's self bounds (`type Assoc: Bound`)"
                    )
                }
            };
            let generics = generics(db, generic_def);
            let parent_len = generics.parent_generics().map_or(0, |g| g.len_self());
            let (trait_args, assoc_args) =
                projection_ty.substitution.as_slice(Interner).split_at(parent_len);
            let (self_ty, args_no_self) =
                trait_args.split_first().expect("projection without trait self type");
            if self_ty.assert_ty_ref(Interner) != &self_ty_shifted_in {
                return None;
            }

            let args_no_self = args_no_self.iter().cloned().casted(Interner).collect();
            let parameters = assoc_args.to_vec();

            let alias_eq_bound = rust_ir::AliasEqBound {
                value: ty.clone(),
                trait_bound: rust_ir::TraitBound {
                    trait_id: to_chalk_trait_id(projection_ty.trait_(db)),
                    args_no_self,
                },
                associated_ty_id: projection_ty.associated_ty_id,
                parameters,
            };
            Some(chalk_ir::Binders::new(
                binders,
                rust_ir::InlineBound::AliasEqBound(alias_eq_bound),
            ))
        }
        _ => None,
    }
}
