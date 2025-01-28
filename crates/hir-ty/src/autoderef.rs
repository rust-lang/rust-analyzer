//! In certain situations, rust automatically inserts derefs as necessary: for
//! example, field accesses `foo.bar` still work when `foo` is actually a
//! reference to a type with the field `bar`. This is an approximation of the
//! logic in rustc (which lives in rustc_hir_analysis/check/autoderef.rs).

use std::mem;

use chalk_ir::cast::Cast;
use hir_def::{lang_item::LangItem, TraitId, TypeAliasId};
use hir_expand::name::Name;
use intern::sym;
use limit::Limit;
use triomphe::Arc;

use crate::{
    db::HirDatabase, infer::unify::InferenceTable, Binders, Canonical, DomainGoal, Goal, GoalData,
    Interner, ProgramClauses, ProjectionTyExt, TraitEnvironment, Ty, TyBuilder, TyKind,
};

static AUTODEREF_RECURSION_LIMIT: Limit = Limit::new(20);

#[derive(Debug)]
pub(crate) enum AutoderefKind {
    Builtin,
    Overloaded,
}

/// Returns types that `ty` transitively dereferences to. This function is only meant to be used
/// outside `hir-ty`.
///
/// It is guaranteed that:
/// - the yielded types don't contain inference variables (but may contain `TyKind::Error`).
/// - a type won't be yielded more than once; in other words, the returned iterator will stop if it
///   detects a cycle in the deref chain.
pub fn autoderef(
    db: &dyn HirDatabase,
    env: Arc<TraitEnvironment>,
    ty: Canonical<Ty>,
) -> impl Iterator<Item = Ty> {
    let mut table = InferenceTable::new(db, env);
    let ty = table.instantiate_canonical(ty);
    let mut autoderef = Autoderef::new_no_tracking(&mut table, ty, false, false);
    let mut v = Vec::new();
    while let Some((ty, _steps)) = autoderef.next() {
        // `ty` may contain unresolved inference variables. Since there's no chance they would be
        // resolved, just replace with fallback type.
        let resolved = autoderef.table.resolve_completely(ty);

        // If the deref chain contains a cycle (e.g. `A` derefs to `B` and `B` derefs to `A`), we
        // would revisit some already visited types. Stop here to avoid duplication.
        //
        // XXX: The recursion limit for `Autoderef` is currently 20, so `Vec::contains()` shouldn't
        // be too expensive. Replace this duplicate check with `FxHashSet` if it proves to be more
        // performant.
        if v.contains(&resolved) {
            break;
        }
        v.push(resolved);
    }
    v.into_iter()
}

trait TrackAutoderefSteps {
    fn len(&self) -> usize;
    fn push(&mut self, kind: AutoderefKind, ty: &Ty);
}

impl TrackAutoderefSteps for usize {
    fn len(&self) -> usize {
        *self
    }
    fn push(&mut self, _: AutoderefKind, _: &Ty) {
        *self += 1;
    }
}
impl TrackAutoderefSteps for Vec<(AutoderefKind, Ty)> {
    fn len(&self) -> usize {
        self.len()
    }
    fn push(&mut self, kind: AutoderefKind, ty: &Ty) {
        self.push((kind, ty.clone()));
    }
}

#[derive(Debug)]
pub(crate) struct Autoderef<'table, 'db, T = Vec<(AutoderefKind, Ty)>> {
    pub(crate) table: &'table mut InferenceTable<'db>,
    ty: Ty,
    at_start: bool,
    steps: T,
    explicit: bool,
    use_receiver_trait: bool,
}

impl<'table, 'db> Autoderef<'table, 'db> {
    pub(crate) fn new(
        table: &'table mut InferenceTable<'db>,
        ty: Ty,
        explicit: bool,
        use_receiver_trait: bool,
    ) -> Self {
        let ty = table.resolve_ty_shallow(&ty);
        Autoderef { table, ty, at_start: true, steps: Vec::new(), explicit, use_receiver_trait }
    }

    pub(crate) fn steps(&self) -> &[(AutoderefKind, Ty)] {
        &self.steps
    }
}

impl<'table, 'db> Autoderef<'table, 'db, usize> {
    pub(crate) fn new_no_tracking(
        table: &'table mut InferenceTable<'db>,
        ty: Ty,
        explicit: bool,
        use_receiver_trait: bool,
    ) -> Self {
        let ty = table.resolve_ty_shallow(&ty);
        Autoderef { table, ty, at_start: true, steps: 0, explicit, use_receiver_trait }
    }
}

#[allow(private_bounds)]
impl<T: TrackAutoderefSteps> Autoderef<'_, '_, T> {
    pub(crate) fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub(crate) fn final_ty(&self) -> Ty {
        self.ty.clone()
    }
}

impl<T: TrackAutoderefSteps> Iterator for Autoderef<'_, '_, T> {
    type Item = (Ty, usize);

    #[tracing::instrument(skip_all)]
    fn next(&mut self) -> Option<Self::Item> {
        if mem::take(&mut self.at_start) {
            return Some((self.ty.clone(), 0));
        }

        if AUTODEREF_RECURSION_LIMIT.check(self.steps.len() + 1).is_err() {
            return None;
        }

        let (kind, new_ty) =
            autoderef_step(self.table, self.ty.clone(), self.explicit, self.use_receiver_trait)?;

        self.steps.push(kind, &self.ty);
        self.ty = new_ty;

        Some((self.ty.clone(), self.step_count()))
    }
}

pub(crate) fn autoderef_step(
    table: &mut InferenceTable<'_>,
    ty: Ty,
    explicit: bool,
    use_receiver_trait: bool,
) -> Option<(AutoderefKind, Ty)> {
    if let Some(derefed) = builtin_deref(table.db, &ty, explicit) {
        Some((AutoderefKind::Builtin, table.resolve_ty_shallow(derefed)))
    } else {
        Some((AutoderefKind::Overloaded, deref_by_trait(table, ty, use_receiver_trait)?))
    }
}

pub(crate) fn builtin_deref<'ty>(
    db: &dyn HirDatabase,
    ty: &'ty Ty,
    explicit: bool,
) -> Option<&'ty Ty> {
    match ty.kind(Interner) {
        TyKind::Ref(.., ty) => Some(ty),
        TyKind::Raw(.., ty) if explicit => Some(ty),
        &TyKind::Adt(chalk_ir::AdtId(adt), ref substs) if crate::lang_items::is_box(db, adt) => {
            substs.at(Interner, 0).ty(Interner)
        }
        _ => None,
    }
}

pub(crate) fn deref_by_trait(
    table @ &mut InferenceTable { db, .. }: &mut InferenceTable<'_>,
    ty: Ty,
    use_receiver_trait: bool,
) -> Option<Ty> {
    let _p = tracing::info_span!("deref_by_trait").entered();
    if table.resolve_ty_shallow(&ty).inference_var(Interner).is_some() {
        // don't try to deref unknown variables
        return None;
    }

    let deref_trait =
        db.lang_item(table.trait_env.krate, LangItem::Deref).and_then(|l| l.as_trait())?;
    let deref_target = db
        .trait_data(deref_trait)
        .associated_type_by_name(&Name::new_symbol_root(sym::Target.clone()))?;
    let mut trait_ = || {
        if use_receiver_trait {
            if let Some(receiver_trait) =
                db.lang_item(table.trait_env.krate, LangItem::Receiver).and_then(|l| l.as_trait())
            {
                // FIXME: The following lines check whether the current rust-std contains blancket
                // implementation of `Receiver` trait for the implementors of `Deref`.
                // Remove this once `Receiver` needs to be stabilized, doing so will
                // effectively bump the MSRV of rust-analyzer to 1.84 due to 1.83 and below lacking the
                // blanked impl on `Deref`.
                let proj = |trait_id: TraitId, target: TypeAliasId| {
                    Binders::with_fresh_type_var(Interner, |ty| {
                        let b = TyBuilder::subst_for_def(db, trait_id, None);
                        if b.remaining() != 1 {
                            // the Target type + Deref trait should only have one generic parameter,
                            // namely Deref's Self type
                            return None;
                        }
                        let deref_subst = b.push(ty.clone()).build();
                        Some(
                            TyBuilder::assoc_type_projection(db, target, Some(deref_subst)).build(),
                        )
                    })
                };
                let deref_proj =
                    proj(deref_trait, deref_target).filter_map(std::convert::identity)?;
                let deref_trait_ref = deref_proj.map(|it| it.trait_ref(db));
                let receiver_target = db
                    .trait_data(receiver_trait)
                    .associated_type_by_name(&Name::new_symbol_root(sym::Target.clone()))?;
                let receiver_proj =
                    proj(receiver_trait, receiver_target).filter_map(std::convert::identity)?;
                let receiver_trait_ref = receiver_proj.map(|it| it.trait_ref(db));
                // if P: Deref<Target = T> then P: Receiver<Target = T>
                let implements_goal: Goal = GoalData::Implies(
                    ProgramClauses::from_iter(
                        Interner,
                        [deref_trait_ref.clone().map(|it| it.cast::<DomainGoal>(Interner))],
                    ),
                    receiver_trait_ref.cast(Interner),
                )
                .intern(Interner);
                if table.try_obligation(implements_goal).is_some() {
                    return Some((receiver_trait, receiver_target));
                }
            }
        }
        None
    };

    // Old rust-std versions may not have `Receiver` trait or
    // `impl<P: ?Sized, T: ?Sized> Receiver for P where P: Deref<Target = T>`.
    // Fallback to `Deref` if they don't
    let (trait_id, target) = trait_().unwrap_or((deref_trait, deref_target));

    let projection = {
        let b = TyBuilder::subst_for_def(db, trait_id, None);
        if b.remaining() != 1 {
            // the Target type + Deref trait should only have one generic parameter,
            // namely Deref's Self type
            return None;
        }
        let deref_subst = b.push(ty).build();
        TyBuilder::assoc_type_projection(db, target, Some(deref_subst)).build()
    };

    // Check that the type implements Deref at all
    let trait_ref = projection.trait_ref(db);
    let implements_goal: Goal = trait_ref.cast(Interner);
    table.try_obligation(implements_goal.clone())?;

    table.register_obligation(implements_goal);

    let result = table.normalize_projection_ty(projection);
    Some(table.resolve_ty_shallow(&result))
}
