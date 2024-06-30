//! Utilities for mapping between hir IDs and the surface syntax.

use std::hash::Hash;

use either::Either;
use hir_expand::{InFile, Lookup};
use la_arena::ArenaMap;
use span::AstIdNode;
use syntax::{ast, AstPtr};

use crate::{
    db::DefDatabase,
    dyn_map::{
        keys::def_to_src::{self, DefIdPolicy},
        DynMap, Key,
    },
    item_tree::{AttrOwner, FieldParent, ItemTreeNode},
    ConstId, EnumId, EnumVariantId, ExternBlockId, ExternCrateId, FunctionId, GenericDefId, ImplId,
    ItemTreeLoc, LocalFieldId, LocalLifetimeParamId, LocalTypeOrConstParamId, Macro2Id,
    MacroRulesId, ProcMacroId, StaticId, StructId, TraitAliasId, TraitId, TypeAliasId, UnionId,
    UseId, VariantId,
};

#[derive(Default)]
pub struct DefToSourceCache {
    pub dynmap_cache: DynMap,
}

pub struct DefToSourceContext<'cache> {
    pub cache: &'cache mut DefToSourceCache,
}

pub trait HasSource
where
    Self: Sized + Copy + Eq + Hash + 'static,
    Self: for<'db> Lookup<Database<'db> = dyn DefDatabase + 'db>,
    <Self as Lookup>::Data: ItemTreeLoc,
    <<Self as Lookup>::Data as ItemTreeLoc>::Id: ItemTreeNode<Source = Self::Value>,
{
    type Value: AstIdNode;

    fn source(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<Self::Value> {
        let InFile { file_id, value } = self.ast_ptr(db, ctx);
        InFile::new(file_id, value.to_node(&db.parse_or_expand(file_id)))
    }

    fn ast_ptr(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<AstPtr<Self::Value>>;

    fn ast_ptr_by_key(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
        map_key: Key<Self, AstPtr<Self::Value>, DefIdPolicy<Self, Self::Value>>,
    ) -> InFile<AstPtr<Self::Value>> {
        let file_id = self.lookup(db).item_tree_id().file_id();
        ctx.as_mut()
            .map(|ctx| {
                let ast_ptr = *ctx.cache.dynmap_cache[map_key]
                    .entry(*self)
                    .or_insert_with(|| self.ast_ptr_without_cache(db).value);
                InFile::new(file_id, ast_ptr)
            })
            .unwrap_or_else(|| self.ast_ptr_without_cache(db))
    }

    fn ast_ptr_without_cache(&self, db: &dyn DefDatabase) -> InFile<AstPtr<Self::Value>> {
        let file_id = self.lookup(db).item_tree_id().file_id();
        let loc = self.lookup(db);
        let id = loc.item_tree_id();
        let tree = id.item_tree(db);
        let ast_id_map = db.ast_id_map(file_id);
        let node = &tree[id.value];
        let ast_ptr = ast_id_map.get(node.ast_id());

        InFile::new(file_id, ast_ptr)
    }
}

impl HasSource for StructId {
    type Value = ast::Struct;

    fn ast_ptr(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<AstPtr<Self::Value>> {
        self.ast_ptr_by_key(db, ctx, def_to_src::STRUCT)
    }
}

impl HasSource for UnionId {
    type Value = ast::Union;

    fn ast_ptr(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<AstPtr<Self::Value>> {
        self.ast_ptr_by_key(db, ctx, def_to_src::UNION)
    }
}

impl HasSource for EnumId {
    type Value = ast::Enum;

    fn ast_ptr(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<AstPtr<Self::Value>> {
        self.ast_ptr_by_key(db, ctx, def_to_src::ENUM)
    }
}

impl HasSource for EnumVariantId {
    type Value = ast::Variant;

    fn ast_ptr(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<AstPtr<Self::Value>> {
        self.ast_ptr_by_key(db, ctx, def_to_src::ENUM_VARIANT)
    }
}

impl HasSource for FunctionId {
    type Value = ast::Fn;

    fn ast_ptr(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<AstPtr<Self::Value>> {
        self.ast_ptr_by_key(db, ctx, def_to_src::FUNCTION)
    }
}

impl HasSource for ConstId {
    type Value = ast::Const;

    fn ast_ptr(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<AstPtr<Self::Value>> {
        self.ast_ptr_by_key(db, ctx, def_to_src::CONST)
    }
}

impl HasSource for StaticId {
    type Value = ast::Static;

    fn ast_ptr(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<AstPtr<Self::Value>> {
        self.ast_ptr_by_key(db, ctx, def_to_src::STATIC)
    }
}

impl HasSource for TraitId {
    type Value = ast::Trait;

    fn ast_ptr(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<AstPtr<Self::Value>> {
        self.ast_ptr_by_key(db, ctx, def_to_src::TRAIT)
    }
}

impl HasSource for TraitAliasId {
    type Value = ast::TraitAlias;

    fn ast_ptr(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<AstPtr<Self::Value>> {
        self.ast_ptr_by_key(db, ctx, def_to_src::TRAIT_ALIAS)
    }
}

impl HasSource for TypeAliasId {
    type Value = ast::TypeAlias;

    fn ast_ptr(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<AstPtr<Self::Value>> {
        self.ast_ptr_by_key(db, ctx, def_to_src::TYPE_ALIAS)
    }
}

impl HasSource for Macro2Id {
    type Value = ast::MacroDef;

    fn ast_ptr(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<AstPtr<Self::Value>> {
        self.ast_ptr_by_key(db, ctx, def_to_src::MACRO2)
    }
}

impl HasSource for MacroRulesId {
    type Value = ast::MacroRules;

    fn ast_ptr(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<AstPtr<Self::Value>> {
        self.ast_ptr_by_key(db, ctx, def_to_src::MACRO_RULES)
    }
}

impl HasSource for ProcMacroId {
    type Value = ast::Fn;

    fn ast_ptr(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<AstPtr<Self::Value>> {
        self.ast_ptr_by_key(db, ctx, def_to_src::PROC_MACRO)
    }
}

impl HasSource for ImplId {
    type Value = ast::Impl;

    fn ast_ptr(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<AstPtr<Self::Value>> {
        self.ast_ptr_by_key(db, ctx, def_to_src::IMPL)
    }
}

impl HasSource for ExternCrateId {
    type Value = ast::ExternCrate;

    fn ast_ptr(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<AstPtr<Self::Value>> {
        self.ast_ptr_by_key(db, ctx, def_to_src::EXTERN_CRATE)
    }
}

impl HasSource for ExternBlockId {
    type Value = ast::ExternBlock;

    fn ast_ptr(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<AstPtr<Self::Value>> {
        self.ast_ptr_by_key(db, ctx, def_to_src::EXTERN_BLOCK)
    }
}

impl HasSource for UseId {
    type Value = ast::Use;

    fn ast_ptr(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<AstPtr<Self::Value>> {
        self.ast_ptr_by_key(db, ctx, def_to_src::USE)
    }
}

pub trait HasChildSource<ChildId> {
    type Value;
    fn child_source(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<ArenaMap<ChildId, Self::Value>>;
}

impl HasChildSource<la_arena::Idx<ast::UseTree>> for UseId {
    type Value = ast::UseTree;
    fn child_source(
        &self,
        db: &dyn DefDatabase,
        _ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<ArenaMap<la_arena::Idx<ast::UseTree>, Self::Value>> {
        let loc = &self.lookup(db);
        let use_ = &loc.id.item_tree(db)[loc.id.value];
        InFile::new(
            loc.id.file_id(),
            use_.use_tree_source_map(db, loc.id.file_id()).into_iter().collect(),
        )
    }
}

impl HasChildSource<LocalTypeOrConstParamId> for GenericDefId {
    type Value = Either<ast::TypeOrConstParam, ast::TraitOrAlias>;
    fn child_source(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<ArenaMap<LocalTypeOrConstParamId, Self::Value>> {
        let generic_params = db.generic_params(*self);
        let mut idx_iter = generic_params.iter_type_or_consts().map(|(idx, _)| idx);

        let (file_id, generic_params_list) = self.file_id_and_params_of(db, ctx);

        let mut params = ArenaMap::default();

        // For traits and trait aliases the first type index is `Self`, we need to add it before
        // the other params.
        match *self {
            GenericDefId::TraitId(id) => {
                let trait_ref = id.source(db, ctx).value;
                let idx = idx_iter.next().unwrap();
                params.insert(idx, Either::Right(ast::TraitOrAlias::Trait(trait_ref)));
            }
            GenericDefId::TraitAliasId(id) => {
                let alias = id.source(db, ctx).value;
                let idx = idx_iter.next().unwrap();
                params.insert(idx, Either::Right(ast::TraitOrAlias::TraitAlias(alias)));
            }
            _ => {}
        }

        if let Some(generic_params_list) = generic_params_list {
            for (idx, ast_param) in idx_iter.zip(generic_params_list.type_or_const_params()) {
                params.insert(idx, Either::Left(ast_param));
            }
        }

        InFile::new(file_id, params)
    }
}

impl HasChildSource<LocalLifetimeParamId> for GenericDefId {
    type Value = ast::LifetimeParam;
    fn child_source(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<ArenaMap<LocalLifetimeParamId, Self::Value>> {
        let generic_params = db.generic_params(*self);
        let idx_iter = generic_params.iter_lt().map(|(idx, _)| idx);

        let (file_id, generic_params_list) = self.file_id_and_params_of(db, ctx);

        let mut params = ArenaMap::default();

        if let Some(generic_params_list) = generic_params_list {
            for (idx, ast_param) in idx_iter.zip(generic_params_list.lifetime_params()) {
                params.insert(idx, ast_param);
            }
        }

        InFile::new(file_id, params)
    }
}

impl HasChildSource<LocalFieldId> for VariantId {
    type Value = Either<ast::TupleField, ast::RecordField>;

    fn child_source(
        &self,
        db: &dyn DefDatabase,
        ctx: &mut Option<&mut DefToSourceContext<'_>>,
    ) -> InFile<ArenaMap<LocalFieldId, Self::Value>> {
        let item_tree;
        let (src, parent, container) = match *self {
            VariantId::EnumVariantId(it) => {
                let lookup = it.lookup(db);
                item_tree = it.lookup(db).id.item_tree(db);
                (
                    it.source(db, ctx).map(|it| it.kind()),
                    FieldParent::Variant(lookup.id.value),
                    lookup.parent.lookup(db).container,
                )
            }
            VariantId::StructId(it) => {
                let lookup = it.lookup(db);
                item_tree = lookup.id.item_tree(db);
                (
                    it.source(db, ctx).map(|it| it.kind()),
                    FieldParent::Struct(lookup.id.value),
                    lookup.container,
                )
            }
            VariantId::UnionId(it) => {
                let lookup = it.lookup(db);
                item_tree = lookup.id.item_tree(db);
                (
                    it.source(db, ctx).map(|it| it.kind()),
                    FieldParent::Union(lookup.id.value),
                    lookup.container,
                )
            }
        };

        let mut map = ArenaMap::new();
        match &src.value {
            ast::StructKind::Tuple(fl) => {
                let cfg_options = &db.crate_graph()[container.krate].cfg_options;
                let mut idx = 0;
                for (i, fd) in fl.fields().enumerate() {
                    let attrs = item_tree.attrs(
                        db,
                        container.krate,
                        AttrOwner::make_field_indexed(parent, i),
                    );
                    if !attrs.is_cfg_enabled(cfg_options) {
                        continue;
                    }
                    map.insert(
                        LocalFieldId::from_raw(la_arena::RawIdx::from(idx)),
                        Either::Left(fd.clone()),
                    );
                    idx += 1;
                }
            }
            ast::StructKind::Record(fl) => {
                let cfg_options = &db.crate_graph()[container.krate].cfg_options;
                let mut idx = 0;
                for (i, fd) in fl.fields().enumerate() {
                    let attrs = item_tree.attrs(
                        db,
                        container.krate,
                        AttrOwner::make_field_indexed(parent, i),
                    );
                    if !attrs.is_cfg_enabled(cfg_options) {
                        continue;
                    }
                    map.insert(
                        LocalFieldId::from_raw(la_arena::RawIdx::from(idx)),
                        Either::Right(fd.clone()),
                    );
                    idx += 1;
                }
            }
            _ => (),
        }
        InFile::new(src.file_id, map)
    }
}
