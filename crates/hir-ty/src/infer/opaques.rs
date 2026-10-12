//! Defining opaque types via inference.

use rustc_hash::{FxHashMap, FxHashSet};
use rustc_type_ir::{
    GenericArgKind, TypeFoldable, TypeFolder, TypeSuperFoldable, TypeVisitableExt, fold_regions,
    inherent::{GenericArgs as _, IntoKind},
};
use tracing::{debug, instrument};

use crate::{
    Span,
    infer::InferenceContext,
    next_solver::{
        Const, ConstKind, DbInterner, EarlyBinder, GenericArg, GenericArgs, OpaqueTypeKey, Region,
        RegionKind, SolverDefId, Ty, TyKind, TypingMode,
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

#[expect(dead_code, reason = "payloads are retained for non-defining-use diagnostics")]
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

struct ReverseMapper<'db> {
    interner: DbInterner<'db>,
    map: FxHashMap<GenericArg<'db>, GenericArg<'db>>,
}

impl<'db> ReverseMapper<'db> {
    fn new(interner: DbInterner<'db>, opaque_type_key: OpaqueTypeKey<'db>) -> Self {
        let id_args = GenericArgs::identity_for_item(interner, opaque_type_key.def_id.into());
        let map = opaque_type_key.args.iter().zip(id_args.iter()).collect();
        Self { interner, map }
    }
}

impl<'db> TypeFolder<DbInterner<'db>> for ReverseMapper<'db> {
    fn cx(&self) -> DbInterner<'db> {
        self.interner
    }

    fn fold_ty(&mut self, ty: Ty<'db>) -> Ty<'db> {
        match ty.kind() {
            // A parameter outside the opaque declaration is malformed input. Keep the
            // recovered type foldable so the caller never passes it to an incompatible binder.
            TyKind::Param(_) => match self.map.get(&ty.into()).map(|arg| arg.kind()) {
                Some(GenericArgKind::Type(ty)) => ty,
                Some(_) | None => self.interner.default_types().types.error,
            },
            _ => ty.super_fold_with(self),
        }
    }

    fn fold_const(&mut self, konst: Const<'db>) -> Const<'db> {
        match konst.kind() {
            ConstKind::Param(_) => match self.map.get(&konst.into()).map(|arg| arg.kind()) {
                Some(GenericArgKind::Const(konst)) => konst,
                Some(_) | None => self.interner.default_types().consts.error,
            },
            _ => konst.super_fold_with(self),
        }
    }

    fn fold_region(&mut self, region: Region<'db>) -> Region<'db> {
        match region.kind() {
            RegionKind::ReEarlyParam(_) | RegionKind::ReLateParam(_) => {
                match self.map.get(&region.into()).map(|arg| arg.kind()) {
                    Some(GenericArgKind::Lifetime(region)) => region,
                    Some(_) | None => self.interner.default_types().regions.erased,
                }
            }
            _ => region,
        }
    }
}

fn opaque_type_has_defining_use_args<'db>(
    interner: DbInterner<'db>,
    opaque_type_key: OpaqueTypeKey<'db>,
) -> bool {
    let mut seen = FxHashSet::default();
    opaque_type_key.iter_captured_args(interner).all(|(_, arg)| match arg.kind() {
        GenericArgKind::Lifetime(_) => true,
        GenericArgKind::Type(ty) => matches!(ty.kind(), TyKind::Param(_)) && seen.insert(arg),
        GenericArgKind::Const(konst) => {
            matches!(konst.kind(), ConstKind::Param(_)) && seen.insert(arg)
        }
    })
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
        if !opaque_type_has_defining_use_args(self.interner(), opaque_type_key) {
            return UsageKind::NonDefiningUse(opaque_type_key, hidden_type);
        }

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
        let hidden_type =
            hidden_type.fold_with(&mut ReverseMapper::new(self.interner(), opaque_type_key));
        UsageKind::HasDefiningUse(hidden_type)
    }
}
