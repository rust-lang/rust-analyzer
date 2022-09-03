//! Name resolution façade.
use std::{hash::BuildHasherDefault, iter, sync::Arc};

use arrayvec::ArrayVec;
use base_db::CrateId;
use either::Either;
use hir_expand::name::{name, Name};
use indexmap::IndexMap;
use rustc_hash::FxHashSet;
use smallvec::SmallVec;

use crate::{
    body::scope::{ExprScopes, ScopeId},
    builtin_type::BuiltinType,
    db::DefDatabase,
    expr::{ExprId, LabelId, PatId},
    generics::{GenericParams, TypeOrConstParamData},
    intern::Interned,
    item_scope::{BuiltinShadowMode, BUILTIN_SCOPE},
    nameres::DefMap,
    path::{ModPath, PathKind},
    per_ns::PerNs,
    visibility::{RawVisibility, Visibility},
    AdtId, AssocItemId, ConstId, ConstParamId, DefWithBodyId, EnumId, EnumVariantId, ExternBlockId,
    FunctionId, GenericDefId, GenericParamId, HasModule, ImplId, ItemContainerId, LifetimeParamId,
    LocalModuleId, Lookup, Macro2Id, MacroId, MacroRulesId, ModuleDefId, ModuleId, ProcMacroId,
    StaticId, StructId, TraitId, TypeAliasId, TypeOrConstParamId, TypeParamId, VariantId,
};

#[derive(Debug, Clone)]
pub struct Resolver {
    /// This is the module scope of this resolver. It is always the outermost scope.
    module_scope: ModuleItemMap,
    self_scope: Option<Either<ImplId, AdtId>>,
    generics_scope: ArrayVec<(GenericDefId, Interned<GenericParams>), 2>,
    /// The inner most block scope if there is one and its position in the `expression_scopes`.
    block_scope: Option<ModuleItemMap>,
    /// The expression scopes, iterate these in reverse order.
    expression_scopes: Option<(Vec<ScopeId>, Arc<ExprScopes>, DefWithBodyId)>,
}

#[derive(Debug, Clone)]
struct ModuleItemMap {
    def_map: Arc<DefMap>,
    module_id: LocalModuleId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeNs {
    SelfType(ImplId),
    GenericParam(TypeParamId),
    AdtId(AdtId),
    AdtSelfType(AdtId),
    // Yup, enum variants are added to the types ns, but any usage of variant as
    // type is an error.
    EnumVariantId(EnumVariantId),
    TypeAliasId(TypeAliasId),
    BuiltinType(BuiltinType),
    TraitId(TraitId),
    // Module belong to type ns, but the resolver is used when all module paths
    // are fully resolved.
    // ModuleId(ModuleId)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolveValueResult {
    ValueNs(ValueNs),
    Partial(TypeNs, usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueNs {
    ImplSelf(ImplId),
    LocalBinding(PatId),
    FunctionId(FunctionId),
    ConstId(ConstId),
    StaticId(StaticId),
    StructId(StructId),
    EnumVariantId(EnumVariantId),
    GenericParam(ConstParamId),
}

impl Resolver {
    /// Resolve known trait from std, like `std::futures::Future`
    pub fn resolve_known_trait(&self, db: &dyn DefDatabase, path: &ModPath) -> Option<TraitId> {
        let res = self.resolve_module_path(db, path, BuiltinShadowMode::Other).take_types()?;
        match res {
            ModuleDefId::TraitId(it) => Some(it),
            _ => None,
        }
    }

    /// Resolve known struct from std, like `std::boxed::Box`
    pub fn resolve_known_struct(&self, db: &dyn DefDatabase, path: &ModPath) -> Option<StructId> {
        let res = self.resolve_module_path(db, path, BuiltinShadowMode::Other).take_types()?;
        match res {
            ModuleDefId::AdtId(AdtId::StructId(it)) => Some(it),
            _ => None,
        }
    }

    /// Resolve known enum from std, like `std::result::Result`
    pub fn resolve_known_enum(&self, db: &dyn DefDatabase, path: &ModPath) -> Option<EnumId> {
        let res = self.resolve_module_path(db, path, BuiltinShadowMode::Other).take_types()?;
        match res {
            ModuleDefId::AdtId(AdtId::EnumId(it)) => Some(it),
            _ => None,
        }
    }

    pub fn resolve_module_path_in_items(&self, db: &dyn DefDatabase, path: &ModPath) -> PerNs {
        self.resolve_module_path(db, path, BuiltinShadowMode::Module)
    }

    // FIXME: This shouldn't exist
    pub fn resolve_module_path_in_trait_assoc_items(
        &self,
        db: &dyn DefDatabase,
        path: &ModPath,
    ) -> Option<PerNs> {
        let (item_map, module) = self.item_scope();
        let (module_res, idx) = item_map.resolve_path(db, module, path, BuiltinShadowMode::Module);
        match module_res.take_types()? {
            ModuleDefId::TraitId(it) => {
                let idx = idx?;
                let unresolved = &path.segments()[idx..];
                let assoc = match unresolved {
                    [it] => it,
                    _ => return None,
                };
                let &(_, assoc) = db.trait_data(it).items.iter().find(|(n, _)| n == assoc)?;
                Some(match assoc {
                    AssocItemId::FunctionId(it) => PerNs::values(it.into(), Visibility::Public),
                    AssocItemId::ConstId(it) => PerNs::values(it.into(), Visibility::Public),
                    AssocItemId::TypeAliasId(it) => PerNs::types(it.into(), Visibility::Public),
                })
            }
            _ => None,
        }
    }

    pub fn resolve_path_in_type_ns(
        &self,
        db: &dyn DefDatabase,
        path: &ModPath,
    ) -> Option<(TypeNs, Option<usize>)> {
        let first_name = path.segments().first()?;
        let skip_to_mod = path.kind != PathKind::Plain;
        if skip_to_mod {
            return self.module_scope.resolve_path_in_type_ns(db, path);
        }

        if let Some(block_scope) = &self.block_scope {
            if let res @ Some(_) = block_scope.resolve_path_in_type_ns_blocks(db, path) {
                return res;
            }
        }

        let remaining_idx = || if path.segments().len() == 1 { None } else { Some(1) };

        let type_param = self
            .generics_scopes()
            .find_map(|(def, params)| params.find_type_by_name(first_name, *def));

        if let Some(id) = type_param {
            return Some((TypeNs::GenericParam(id), remaining_idx()));
        }

        if let Some(id) = self.self_scope {
            if first_name == &name![Self] {
                return Some((
                    match id {
                        Either::Left(id) => TypeNs::SelfType(id),
                        Either::Right(id) => TypeNs::AdtSelfType(id),
                    },
                    remaining_idx(),
                ));
            }
        }
        self.module_scope.resolve_path_in_type_ns(db, path)
    }

    pub fn resolve_path_in_type_ns_fully(
        &self,
        db: &dyn DefDatabase,
        path: &ModPath,
    ) -> Option<TypeNs> {
        let (res, unresolved) = self.resolve_path_in_type_ns(db, path)?;
        if unresolved.is_some() {
            return None;
        }
        Some(res)
    }

    pub fn resolve_visibility(
        &self,
        db: &dyn DefDatabase,
        visibility: &RawVisibility,
    ) -> Option<Visibility> {
        match visibility {
            RawVisibility::Module(_) => {
                let (item_map, module) = self.item_scope();
                item_map.resolve_visibility(db, module, visibility)
            }
            RawVisibility::Public => Some(Visibility::Public),
        }
    }

    pub fn resolve_path_in_value_ns(
        &self,
        db: &dyn DefDatabase,
        path: &ModPath,
    ) -> Option<ResolveValueResult> {
        static SELF: Name = name![self];
        let first_name = if path.is_self() { &SELF } else { path.segments().first()? };
        let n_segments = path.segments().len();
        let skip_to_mod = path.kind != PathKind::Plain && !path.is_self();
        if skip_to_mod {
            return self.module_scope.resolve_path_in_value_ns(db, path);
        }

        if n_segments <= 1 {
            if let Some((mut scopes, expr_scopes)) = self.expr_scopes() {
                let scope_entry = scopes.find_map(|scope| {
                    expr_scopes.entries(scope).iter().find(|entry| entry.name() == first_name)
                });
                if let Some(e) = scope_entry {
                    return Some(ResolveValueResult::ValueNs(ValueNs::LocalBinding(e.pat())));
                }
            }
        }

        if let Some(block_scope) = &self.block_scope {
            if let res @ Some(_) = block_scope.resolve_path_in_value_ns_blocks(db, path) {
                return res;
            }
        }

        match n_segments {
            0 | 1 => {
                let const_param = self
                    .generics_scopes()
                    .find_map(|(def, params)| params.find_const_by_name(first_name, *def));

                if let Some(id) = const_param {
                    let val = ValueNs::GenericParam(id);
                    return Some(ResolveValueResult::ValueNs(val));
                }
            }
            _ => {
                let type_param = self
                    .generics_scopes()
                    .find_map(|(def, params)| params.find_type_by_name(first_name, *def));

                if let Some(id) = type_param {
                    let ty = TypeNs::GenericParam(id);
                    return Some(ResolveValueResult::Partial(ty, 1));
                }
            }
        }

        if let Some(id) = self.self_scope {
            if first_name == &name![Self] {
                match id {
                    Either::Left(id) if n_segments > 1 => {
                        return Some(ResolveValueResult::Partial(TypeNs::SelfType(id), 1))
                    }
                    Either::Left(id) => {
                        return Some(ResolveValueResult::ValueNs(ValueNs::ImplSelf(id)))
                    }
                    // bare `Self` doesn't work in the value namespace in a struct/enum definition
                    Either::Right(_) if n_segments <= 1 => (),
                    Either::Right(id) => {
                        return Some(ResolveValueResult::Partial(TypeNs::AdtSelfType(id), 1))
                    }
                }
            }
        }

        if let res @ Some(_) = self.module_scope.resolve_path_in_value_ns(db, path) {
            return res;
        }

        // If a path of the shape `u16::from_le_bytes` failed to resolve at all, then we fall back
        // to resolving to the primitive type, to allow this to still work in the presence of
        // `use core::u16;`.
        if path.kind == PathKind::Plain && path.segments().len() > 1 {
            if let Some(builtin) = BuiltinType::by_name(&path.segments()[0]) {
                return Some(ResolveValueResult::Partial(TypeNs::BuiltinType(builtin), 1));
            }
        }

        None
    }

    pub fn resolve_path_in_value_ns_fully(
        &self,
        db: &dyn DefDatabase,
        path: &ModPath,
    ) -> Option<ValueNs> {
        match self.resolve_path_in_value_ns(db, path)? {
            ResolveValueResult::ValueNs(it) => Some(it),
            ResolveValueResult::Partial(..) => None,
        }
    }

    pub fn resolve_path_as_macro(&self, db: &dyn DefDatabase, path: &ModPath) -> Option<MacroId> {
        let (item_map, module) = self.item_scope();
        item_map.resolve_path(db, module, path, BuiltinShadowMode::Other).0.take_macros()
    }

    /// Returns a set of names available in the current scope.
    ///
    /// Note that this is a somewhat fuzzy concept -- internally, the compiler
    /// doesn't necessary follow a strict scoping discipline. Rather, it just
    /// tells for each ident what it resolves to.
    ///
    /// A good example is something like `str::from_utf8`. From scopes point of
    /// view, this code is erroneous -- both `str` module and `str` type occupy
    /// the same type namespace.
    ///
    /// We don't try to model that super-correctly -- this functionality is
    /// primarily exposed for completions.
    ///
    /// Note that in Rust one name can be bound to several items:
    ///
    /// ```
    /// macro_rules! t { () => (()) }
    /// type t = t!();
    /// const t: t = t!()
    /// ```
    ///
    /// That's why we return a multimap.
    ///
    /// The shadowing is accounted for: in
    ///
    /// ```
    /// let x = 92;
    /// {
    ///     let x = 92;
    ///     $0
    /// }
    /// ```
    ///
    /// there will be only one entry for `x` in the result.
    ///
    /// The result is ordered *roughly* from the innermost scope to the
    /// outermost: when the name is introduced in two namespaces in two scopes,
    /// we use the position of the first scope.
    pub fn names_in_scope(
        &self,
        db: &dyn DefDatabase,
    ) -> FxIndexMap<Name, SmallVec<[ScopeDef; 1]>> {
        let mut acc = ScopeNames::default();

        if let Some((scopes, expr_scopes)) = self.expr_scopes() {
            for scope in scopes {
                if let Some((label, name)) = expr_scopes.label(scope) {
                    acc.add(&name, ScopeDef::Label(label))
                }
                expr_scopes.entries(scope).iter().for_each(|e| {
                    acc.add_local(e.name(), e.pat());
                });
            }
        }

        if let Some(m) = &self.block_scope {
            m.with_ancestors(db, |def_map, module_id| {
                def_map[module_id].scope.entries().for_each(|(name, def)| {
                    acc.add_per_ns(name, def);
                });
                def_map[module_id].scope.legacy_macros().for_each(|(name, macs)| {
                    macs.iter().for_each(|&mac| {
                        acc.add(
                            name,
                            ScopeDef::ModuleDef(ModuleDefId::MacroId(MacroId::from(mac))),
                        );
                    })
                });
            })
        }

        self.generics_scopes().for_each(|(parent, params)| {
            let parent = *parent;
            for (local_id, param) in params.type_or_consts.iter() {
                if let Some(name) = &param.name() {
                    let id = TypeOrConstParamId { parent, local_id };
                    let data = &db.generic_params(parent).type_or_consts[local_id];
                    acc.add(
                        name,
                        ScopeDef::GenericParam(match data {
                            TypeOrConstParamData::TypeParamData(_) => {
                                GenericParamId::TypeParamId(TypeParamId::from_unchecked(id))
                            }
                            TypeOrConstParamData::ConstParamData(_) => {
                                GenericParamId::ConstParamId(ConstParamId::from_unchecked(id))
                            }
                        }),
                    );
                }
            }
            for (local_id, param) in params.lifetimes.iter() {
                let id = LifetimeParamId { parent, local_id };
                acc.add(&param.name, ScopeDef::GenericParam(id.into()))
            }
        });

        match self.self_scope {
            Some(Either::Left(id)) => {
                acc.add(&name![Self], ScopeDef::ImplSelfType(id));
            }
            Some(Either::Right(id)) => {
                acc.add(&name![Self], ScopeDef::AdtSelfType(id));
            }
            None => (),
        }
        let ModuleItemMap { ref def_map, module_id } = self.module_scope;
        // FIXME: should we provide `self` here?
        // f(
        //     Name::self_param(),
        //     PerNs::types(Resolution::Def {
        //         def: m.module.into(),
        //     }),
        // );
        def_map[module_id].scope.entries().for_each(|(name, def)| {
            acc.add_per_ns(name, def);
        });
        def_map[module_id].scope.legacy_macros().for_each(|(name, macs)| {
            macs.iter().for_each(|&mac| {
                acc.add(name, ScopeDef::ModuleDef(ModuleDefId::MacroId(MacroId::from(mac))));
            })
        });
        def_map.extern_prelude().for_each(|(name, &def)| {
            acc.add(name, ScopeDef::ModuleDef(ModuleDefId::ModuleId(def)));
        });
        BUILTIN_SCOPE.iter().for_each(|(name, &def)| {
            acc.add_per_ns(name, def);
        });
        if let Some(prelude) = def_map.prelude() {
            let prelude_def_map = prelude.def_map(db);
            for (name, def) in prelude_def_map[prelude.local_id].scope.entries() {
                acc.add_per_ns(name, def)
            }
        }
        acc.map
    }

    pub fn traits_in_scope(&self, db: &dyn DefDatabase) -> FxHashSet<TraitId> {
        let mut traits = FxHashSet::default();

        if let Some(m) = &self.block_scope {
            m.with_ancestors(db, |def_map, module_id| {
                traits.extend(def_map[module_id].scope.traits())
            })
        }

        if let Some(Either::Left(impl_)) = self.self_scope {
            if let Some(target_trait) = &db.impl_data(impl_).target_trait {
                if let Some(TypeNs::TraitId(trait_)) =
                    self.resolve_path_in_type_ns_fully(db, target_trait.path.mod_path())
                {
                    traits.insert(trait_);
                }
            }
        }

        // Fill in the prelude traits
        if let Some(prelude) = self.module_scope.def_map.prelude() {
            let prelude_def_map = prelude.def_map(db);
            traits.extend(prelude_def_map[prelude.local_id].scope.traits());
        }
        // Fill in module visible traits
        traits.extend(self.module_scope.def_map[self.module_scope.module_id].scope.traits());
        traits
    }

    pub fn module(&self) -> ModuleId {
        let (def_map, local_id) = self.item_scope();
        def_map.module_id(local_id)
    }

    pub fn krate(&self) -> CrateId {
        self.module_scope.def_map.krate()
    }

    pub fn def_map(&self) -> &DefMap {
        self.item_scope().0
    }

    pub fn where_predicates_in_scope(
        &self,
    ) -> impl Iterator<Item = &crate::generics::WherePredicate> {
        self.generics_scopes().flat_map(|(_, params)| params.where_predicates.iter())
    }

    pub fn generic_def(&self) -> Option<GenericDefId> {
        self.generics_scopes().next().map(|&(def, _)| def)
    }

    pub fn body_owner(&self) -> Option<DefWithBodyId> {
        self.expression_scopes.as_ref().map(|&(.., id)| id)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScopeDef {
    ModuleDef(ModuleDefId),
    Unknown,
    ImplSelfType(ImplId),
    AdtSelfType(AdtId),
    GenericParam(GenericParamId),
    Local(PatId),
    Label(LabelId),
}

// needs arbitrary_self_types to be a method... or maybe move to the def?
pub fn resolver_for_expr(db: &dyn DefDatabase, owner: DefWithBodyId, expr_id: ExprId) -> Resolver {
    let scopes = db.expr_scopes(owner);
    resolver_for_scope(db, owner, scopes.scope_for(expr_id))
}

pub fn resolver_for_scope(
    db: &dyn DefDatabase,
    owner: DefWithBodyId,
    scope_id: Option<ScopeId>,
) -> Resolver {
    let mut r = owner.resolver(db);
    let scopes = db.expr_scopes(owner);
    r.expression_scopes = Some((
        scopes
            .scope_chain(scope_id)
            .inspect(|&scope_id| {
                // record innermost block scope
                if r.block_scope.is_none() {
                    if let Some(block) = scopes.block(scope_id) {
                        if let Some(def_map) = db.block_def_map(block) {
                            let root = def_map.root();
                            r.block_scope = Some(ModuleItemMap { def_map, module_id: root });
                        }
                    }
                }
            })
            .collect(),
        scopes,
        owner,
    ));
    r
}

impl Resolver {
    fn expr_scopes(&self) -> Option<(impl Iterator<Item = ScopeId> + '_, &ExprScopes)> {
        self.expression_scopes
            .as_ref()
            .map(|(scopes, expr_scopes, _)| (scopes.iter().copied(), &**expr_scopes))
    }

    fn generics_scopes(&self) -> impl Iterator<Item = &(GenericDefId, Interned<GenericParams>)> {
        self.generics_scope.iter().rev()
    }

    fn resolve_module_path(
        &self,
        db: &dyn DefDatabase,
        path: &ModPath,
        shadow: BuiltinShadowMode,
    ) -> PerNs {
        let (item_map, module) = self.item_scope();
        let (module_res, segment_index) = item_map.resolve_path(db, module, path, shadow);
        if segment_index.is_some() {
            return PerNs::none();
        }
        module_res
    }

    /// The innermost block scope that contains items or the module scope that contains this resolver.
    fn item_scope(&self) -> (&DefMap, LocalModuleId) {
        let scope = self.block_scope.as_ref().unwrap_or(&self.module_scope);
        (&scope.def_map, scope.module_id)
    }

    fn push_generic_params_scope(mut self, db: &dyn DefDatabase, def: GenericDefId) -> Resolver {
        let params = db.generic_params(def);
        self.generics_scope.push((def, params));
        self
    }
}

impl ModuleItemMap {
    fn resolve_path_in_value_ns(
        &self,
        db: &dyn DefDatabase,
        path: &ModPath,
    ) -> Option<ResolveValueResult> {
        let (module_def, idx) =
            self.def_map.resolve_path_locally(db, self.module_id, path, BuiltinShadowMode::Other);
        Self::filter_value_res(idx, module_def)
    }

    fn resolve_path_in_value_ns_blocks(
        &self,
        db: &dyn DefDatabase,
        path: &ModPath,
    ) -> Option<ResolveValueResult> {
        self.def_map.with_ancestor_maps(db, self.module_id, &mut |def_map, module_id| {
            let (module_def, idx) =
                def_map.resolve_path_locally(db, module_id, path, BuiltinShadowMode::Other);
            Self::filter_value_res(idx, module_def)
        })
    }

    fn resolve_path_in_type_ns(
        &self,
        db: &dyn DefDatabase,
        path: &ModPath,
    ) -> Option<(TypeNs, Option<usize>)> {
        let (module_def, idx) =
            self.def_map.resolve_path_locally(db, self.module_id, path, BuiltinShadowMode::Other);
        let res = to_type_ns(module_def)?;
        Some((res, idx))
    }

    fn resolve_path_in_type_ns_blocks(
        &self,
        db: &dyn DefDatabase,
        path: &ModPath,
    ) -> Option<(TypeNs, Option<usize>)> {
        self.def_map.with_ancestor_maps(db, self.module_id, &mut |def_map, module_id| {
            let (module_def, idx) =
                def_map.resolve_path_locally(db, module_id, path, BuiltinShadowMode::Other);
            let res = to_type_ns(module_def)?;
            Some((res, idx))
        })
    }

    fn with_ancestors(&self, db: &dyn DefDatabase, mut f: impl FnMut(&DefMap, LocalModuleId)) {
        self.def_map.with_ancestor_maps(db, self.module_id, &mut |def_map, module_id| {
            f(def_map, module_id);
            match def_map.block_id() {
                Some(_) => None,
                None => Some(()),
            }
        });
    }

    fn filter_value_res(idx: Option<usize>, module_def: PerNs) -> Option<ResolveValueResult> {
        match idx {
            None => {
                let value = to_value_ns(module_def)?;
                Some(ResolveValueResult::ValueNs(value))
            }
            Some(idx) => {
                let ty = match module_def.take_types()? {
                    ModuleDefId::AdtId(it) => TypeNs::AdtId(it),
                    ModuleDefId::TraitId(it) => TypeNs::TraitId(it),
                    ModuleDefId::TypeAliasId(it) => TypeNs::TypeAliasId(it),
                    ModuleDefId::BuiltinType(it) => TypeNs::BuiltinType(it),

                    ModuleDefId::ModuleId(_)
                    | ModuleDefId::FunctionId(_)
                    | ModuleDefId::EnumVariantId(_)
                    | ModuleDefId::ConstId(_)
                    | ModuleDefId::MacroId(_)
                    | ModuleDefId::StaticId(_) => return None,
                };
                Some(ResolveValueResult::Partial(ty, idx))
            }
        }
    }
}

fn to_value_ns(per_ns: PerNs) -> Option<ValueNs> {
    let res = match per_ns.take_values()? {
        ModuleDefId::FunctionId(it) => ValueNs::FunctionId(it),
        ModuleDefId::AdtId(AdtId::StructId(it)) => ValueNs::StructId(it),
        ModuleDefId::EnumVariantId(it) => ValueNs::EnumVariantId(it),
        ModuleDefId::ConstId(it) => ValueNs::ConstId(it),
        ModuleDefId::StaticId(it) => ValueNs::StaticId(it),

        ModuleDefId::AdtId(AdtId::EnumId(_) | AdtId::UnionId(_))
        | ModuleDefId::TraitId(_)
        | ModuleDefId::TypeAliasId(_)
        | ModuleDefId::BuiltinType(_)
        | ModuleDefId::MacroId(_)
        | ModuleDefId::ModuleId(_) => return None,
    };
    Some(res)
}

fn to_type_ns(per_ns: PerNs) -> Option<TypeNs> {
    let res = match per_ns.take_types()? {
        ModuleDefId::AdtId(it) => TypeNs::AdtId(it),
        ModuleDefId::EnumVariantId(it) => TypeNs::EnumVariantId(it),

        ModuleDefId::TypeAliasId(it) => TypeNs::TypeAliasId(it),
        ModuleDefId::BuiltinType(it) => TypeNs::BuiltinType(it),

        ModuleDefId::TraitId(it) => TypeNs::TraitId(it),

        ModuleDefId::FunctionId(_)
        | ModuleDefId::ConstId(_)
        | ModuleDefId::MacroId(_)
        | ModuleDefId::StaticId(_)
        | ModuleDefId::ModuleId(_) => return None,
    };
    Some(res)
}

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<rustc_hash::FxHasher>>;
#[derive(Default)]
struct ScopeNames {
    map: FxIndexMap<Name, SmallVec<[ScopeDef; 1]>>,
}

impl ScopeNames {
    fn add(&mut self, name: &Name, def: ScopeDef) {
        let set = self.map.entry(name.clone()).or_default();
        if !set.contains(&def) {
            set.push(def)
        }
    }
    fn add_per_ns(&mut self, name: &Name, def: PerNs) {
        if let &Some((ty, _)) = &def.types {
            self.add(name, ScopeDef::ModuleDef(ty))
        }
        if let &Some((def, _)) = &def.values {
            self.add(name, ScopeDef::ModuleDef(def))
        }
        if let &Some((mac, _)) = &def.macros {
            self.add(name, ScopeDef::ModuleDef(ModuleDefId::MacroId(mac)))
        }
        if def.is_none() {
            self.add(name, ScopeDef::Unknown)
        }
    }
    fn add_local(&mut self, name: &Name, pat: PatId) {
        let set = self.map.entry(name.clone()).or_default();
        // XXX: hack, account for local (and only local) shadowing.
        //
        // This should be somewhat more principled and take namespaces into
        // accounts, but, alas, scoping rules are a hoax. `str` type and `str`
        // module can be both available in the same scope.
        if set.iter().any(|it| matches!(it, &ScopeDef::Local(_))) {
            cov_mark::hit!(shadowing_shows_single_completion);
            return;
        }
        set.push(ScopeDef::Local(pat))
    }
}

pub trait HasResolver: Copy {
    /// Builds a resolver for type references inside this def.
    fn resolver(self, db: &dyn DefDatabase) -> Resolver;
}

impl HasResolver for ModuleId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        let def_map = self.def_map(db);

        // Fetch the top level def map of this block def map
        let non_block_map =
            iter::successors(def_map.parent().map(|m| (def_map.clone(), m)), |(_, parent)| {
                let parent_def_map = parent.def_map(db);
                let grand_parent = parent_def_map.parent()?;
                Some((parent_def_map, grand_parent))
            })
            .last();
        let (def_map, module_id, block_scope) = if let Some((_, m)) = non_block_map {
            // found a non-block DefMap, so `self` is a currently a block DefMap
            (m.def_map(db), m.local_id, Some(ModuleItemMap { def_map, module_id: self.local_id }))
        } else {
            (def_map, self.local_id, None)
        };

        Resolver {
            module_scope: ModuleItemMap { def_map, module_id },
            self_scope: None,
            generics_scope: ArrayVec::new(),
            block_scope,
            expression_scopes: None,
        }
    }
}

impl HasResolver for TraitId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        self.lookup(db).container.resolver(db).push_generic_params_scope(db, self.into())
    }
}

impl<T: Into<AdtId> + Copy> HasResolver for T {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        let def = self.into();
        let mut resolver = def.module(db).resolver(db).push_generic_params_scope(db, def.into());
        resolver.self_scope = Some(Either::Right(def));
        resolver
    }
}

impl HasResolver for FunctionId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        self.lookup(db).container.resolver(db).push_generic_params_scope(db, self.into())
    }
}

impl HasResolver for ConstId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        self.lookup(db).container.resolver(db)
    }
}

impl HasResolver for StaticId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        self.lookup(db).container.resolver(db)
    }
}

impl HasResolver for TypeAliasId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        self.lookup(db).container.resolver(db).push_generic_params_scope(db, self.into())
    }
}

impl HasResolver for ImplId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        let mut resolver =
            self.lookup(db).container.resolver(db).push_generic_params_scope(db, self.into());
        resolver.self_scope = Some(Either::Left(self));
        resolver
    }
}

impl HasResolver for ExternBlockId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        // Same as parent's
        self.lookup(db).container.resolver(db)
    }
}

impl HasResolver for DefWithBodyId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        match self {
            DefWithBodyId::ConstId(c) => c.resolver(db),
            DefWithBodyId::FunctionId(f) => f.resolver(db),
            DefWithBodyId::StaticId(s) => s.resolver(db),
        }
    }
}

impl HasResolver for ItemContainerId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        match self {
            ItemContainerId::ModuleId(it) => it.resolver(db),
            ItemContainerId::TraitId(it) => it.resolver(db),
            ItemContainerId::ImplId(it) => it.resolver(db),
            ItemContainerId::ExternBlockId(it) => it.resolver(db),
        }
    }
}

impl HasResolver for GenericDefId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        match self {
            GenericDefId::FunctionId(inner) => inner.resolver(db),
            GenericDefId::AdtId(adt) => adt.resolver(db),
            GenericDefId::TraitId(inner) => inner.resolver(db),
            GenericDefId::TypeAliasId(inner) => inner.resolver(db),
            GenericDefId::ImplId(inner) => inner.resolver(db),
            GenericDefId::EnumVariantId(inner) => inner.parent.resolver(db),
            GenericDefId::ConstId(inner) => inner.resolver(db),
        }
    }
}

impl HasResolver for VariantId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        match self {
            VariantId::EnumVariantId(it) => it.parent.resolver(db),
            VariantId::StructId(it) => it.resolver(db),
            VariantId::UnionId(it) => it.resolver(db),
        }
    }
}

impl HasResolver for MacroId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        match self {
            MacroId::Macro2Id(it) => it.resolver(db),
            MacroId::MacroRulesId(it) => it.resolver(db),
            MacroId::ProcMacroId(it) => it.resolver(db),
        }
    }
}

impl HasResolver for Macro2Id {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        self.lookup(db).container.resolver(db)
    }
}

impl HasResolver for ProcMacroId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        self.lookup(db).container.resolver(db)
    }
}

impl HasResolver for MacroRulesId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        self.lookup(db).container.resolver(db)
    }
}
