//! Defining opaque types via inference.

use rustc_hash::FxHashMap;
use rustc_type_ir::{
    ConstKind, TyKind, TypeFoldable, TypeFolder, TypeSuperFoldable, TypeVisitableExt, fold_regions,
    inherent::{Const as _, GenericArgs as _, IntoKind, Ty as _},
};
use tracing::{debug, instrument};

use crate::{
    Span,
    infer::InferenceContext,
    next_solver::{
        Const, DbInterner, EarlyBinder, ErrorGuaranteed, GenericArg, GenericArgKind, GenericArgs,
        OpaqueTypeKey, Region, SolverDefId, Ty, TypingMode,
        infer::{opaque_types::OpaqueHiddenType, traits::ObligationCause},
    },
};

impl<'db> InferenceContext<'db> {
    /// This takes all the opaque type uses during HIR typeck. It first computes
    /// the concrete hidden type by iterating over all defining uses.
    ///
    /// A use during HIR typeck is defining if all non-lifetime arguments are
    /// unique generic parameters and the hidden type does not reference any
    /// inference variables.
    ///
    /// It then uses these defining uses to guide inference for all other uses.
    #[instrument(level = "debug", skip(self))]
    pub(super) fn handle_opaque_type_uses(&mut self) {
        // We clone the opaques instead of stealing them here as they are still used for
        // normalization in the next generation trait solver.
        let opaque_types: Vec<_> = self.table.infer_ctxt.clone_opaque_types();

        self.compute_definition_site_hidden_types(opaque_types);
    }
}

#[expect(unused, reason = "rustc has this")]
#[derive(Copy, Clone, Debug)]
enum UsageKind<'db> {
    None,
    NonDefiningUse(OpaqueTypeKey<'db>, OpaqueHiddenType<'db>),
    UnconstrainedHiddenType(OpaqueHiddenType<'db>),
    HasDefiningUse(OpaqueHiddenType<'db>),
}

impl<'db> UsageKind<'db> {
    fn merge(&mut self, other: UsageKind<'db>) {
        match (&*self, &other) {
            (UsageKind::HasDefiningUse(_), _) | (_, UsageKind::None) => unreachable!(),
            (UsageKind::None, _) => *self = other,
            // When mergining non-defining uses, prefer earlier ones. This means
            // the error happens as early as possible.
            (
                UsageKind::NonDefiningUse(..) | UsageKind::UnconstrainedHiddenType(..),
                UsageKind::NonDefiningUse(..),
            ) => {}
            // When merging unconstrained hidden types, we prefer later ones. This is
            // used as in most cases, the defining use is the final return statement
            // of our function, and other uses with defining arguments are likely not
            // intended to be defining.
            (
                UsageKind::NonDefiningUse(..) | UsageKind::UnconstrainedHiddenType(..),
                UsageKind::UnconstrainedHiddenType(..) | UsageKind::HasDefiningUse(_),
            ) => *self = other,
        }
    }
}

impl<'db> InferenceContext<'db> {
    fn compute_definition_site_hidden_types(
        &mut self,
        mut opaque_types: Vec<(OpaqueTypeKey<'db>, OpaqueHiddenType<'db>)>,
    ) {
        for entry in opaque_types.iter_mut() {
            *entry = self.resolve_vars_if_possible(*entry);
        }
        debug!(?opaque_types);

        let interner = self.interner();
        let TypingMode::Analysis { defining_opaque_types_and_generators } =
            self.table.infer_ctxt.typing_mode_raw()
        else {
            unreachable!();
        };

        for def_id in defining_opaque_types_and_generators {
            let def_id = match def_id {
                SolverDefId::InternedOpaqueTyId(it) => it,
                _ => continue,
            };

            // We do actually need to check this the second pass (we can't just
            // store this), because we can go from `UnconstrainedHiddenType` to
            // `HasDefiningUse` (because of fallback)
            let mut usage_kind = UsageKind::None;
            for &(opaque_type_key, hidden_type) in &opaque_types {
                if opaque_type_key.def_id != def_id.into() {
                    continue;
                }

                usage_kind.merge(self.consider_opaque_type_use(opaque_type_key, hidden_type));

                if let UsageKind::HasDefiningUse(..) = usage_kind {
                    break;
                }
            }

            if let UsageKind::HasDefiningUse(ty) = usage_kind {
                for &(opaque_type_key, hidden_type) in &opaque_types {
                    if opaque_type_key.def_id != def_id.into() {
                        continue;
                    }

                    let expected = EarlyBinder::bind(ty.ty)
                        .instantiate(interner, opaque_type_key.args)
                        .skip_norm_wip();
                    _ = self.demand_eqtype_fixme_no_diag(expected, hidden_type.ty);
                }

                self.result.type_of_opaque.insert(def_id, ty.ty.store());

                continue;
            }

            self.result.type_of_opaque.insert(def_id, self.types.types.error.store());
        }
    }

    #[tracing::instrument(skip(self), ret)]
    fn consider_opaque_type_use(
        &self,
        opaque_type_key: OpaqueTypeKey<'db>,
        hidden_type: OpaqueHiddenType<'db>,
    ) -> UsageKind<'db> {
        // We ignore uses of the opaque if they have any inference variables
        // as this can frequently happen with recursive calls.
        //
        // See `tests/ui/traits/next-solver/opaques/universal-args-non-defining.rs`.
        if hidden_type.ty.has_non_region_infer() {
            return UsageKind::UnconstrainedHiddenType(hidden_type);
        }

        // FIXME: This should not use a dummy span.
        let cause = ObligationCause::new(Span::Dummy);
        let at = self.table.infer_ctxt.at(&cause, self.table.param_env);
        let hidden_type = match at.deeply_normalize(hidden_type) {
            Ok(hidden_type) => hidden_type,
            Err(_errors) => OpaqueHiddenType { ty: self.types.types.error },
        };
        let hidden_type =
            fold_regions(self.interner(), hidden_type, |_, _| self.types.regions.erased);
        let hidden_type = remap_generic_params_to_declaration_params(
            self.interner(),
            opaque_type_key,
            hidden_type,
        );
        UsageKind::HasDefiningUse(hidden_type)
    }
}

/// The hidden type is written in terms of the generic parameters of the item containing
/// the defining use, while `type_of_opaque` has to be bound by the generic parameters of
/// the *opaque* itself. Those two lists are not the same, neither in length nor in the
/// position of a given parameter, e.g.
///
/// ```ignore
/// impl Tr for S {
///     // The opaque's parameters are `['a, D]`.
///     type Fut<'a, D: 'a> = impl Sized + 'a;
///     // The hidden type `(S, &'a mut D)` is written with the function's parameters, `[D]`.
///     fn make<D>(self, d: &mut D) -> Self::Fut<'_, D> { (self, d) }
/// }
/// ```
///
/// Without this remapping, instantiating the hidden type with the arguments of a use would
/// look up the function's `D` (index 0) in the opaque's arguments, whose index 0 is `'a`.
fn remap_generic_params_to_declaration_params<'db>(
    interner: DbInterner<'db>,
    opaque_type_key: OpaqueTypeKey<'db>,
    hidden_type: OpaqueHiddenType<'db>,
) -> OpaqueHiddenType<'db> {
    if !hidden_type.ty.has_param() {
        return hidden_type;
    }

    let identity_args = GenericArgs::identity_for_item(interner, opaque_type_key.def_id.into());
    // This zip may pair the same lifetime in `args` with different lifetimes from
    // `identity_args`. Simply collecting is the correct behaviour: it keeps the last one,
    // which is the one introduced by the opaque type itself.
    let map: FxHashMap<GenericArg<'db>, GenericArg<'db>> =
        opaque_type_key.args.iter().zip(identity_args.iter()).collect();
    let ty = hidden_type.ty.fold_with(&mut ReverseMapper { interner, map });
    OpaqueHiddenType { ty }
}

/// Converts generic parameters of a [`TypeFoldable`] from one item's generics to another's,
/// here from the item containing the defining use to the opaque type itself.
struct ReverseMapper<'db> {
    interner: DbInterner<'db>,
    map: FxHashMap<GenericArg<'db>, GenericArg<'db>>,
}

impl<'db> TypeFolder<DbInterner<'db>> for ReverseMapper<'db> {
    fn cx(&self) -> DbInterner<'db> {
        self.interner
    }

    fn fold_ty(&mut self, ty: Ty<'db>) -> Ty<'db> {
        if !ty.has_param() {
            return ty;
        }

        match ty.kind() {
            TyKind::Param(_) => match self.map.get(&ty.into()).map(|arg| arg.kind()) {
                Some(GenericArgKind::Type(ty)) => ty,
                // The hidden type mentions a type parameter the opaque does not capture.
                // rustc reports an error here, we only have the error type.
                _ => Ty::new_error(self.interner, ErrorGuaranteed),
            },
            _ => ty.super_fold_with(self),
        }
    }

    fn fold_const(&mut self, ct: Const<'db>) -> Const<'db> {
        if !ct.has_param() {
            return ct;
        }

        match ct.kind() {
            ConstKind::Param(_) => match self.map.get(&ct.into()).map(|arg| arg.kind()) {
                Some(GenericArgKind::Const(ct)) => ct,
                // Ditto, for const parameters.
                _ => Const::new_error(self.interner, ErrorGuaranteed),
            },
            _ => ct.super_fold_with(self),
        }
    }

    fn fold_region(&mut self, r: Region<'db>) -> Region<'db> {
        // Regions were erased before we got here, there is nothing to map them to.
        r
    }
}
