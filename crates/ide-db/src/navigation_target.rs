//! See [`NavigationTarget`].

use std::{fmt, iter};

use arrayvec::ArrayVec;
use either::Either;
use hir::{
    AssocItem, FieldSource, HasContainer, HasCrate, HasSource, HirDisplay, HirFileId, InFile,
    InFileWrapper, LocalSource, ModuleSource, db::ExpandDatabase, symbols::FileSymbol,
};
use span::Edition;
use stdx::{TupleExt, never};
use syntax::{
    AstNode, SmolStr, TextRange, ToSmolStr,
    ast::{self, HasName},
    format_smolstr,
};

use crate::{
    FileId, FileRange, RootDatabase, SymbolKind,
    defs::Definition,
    documentation::{Documentation, HasDocs},
};

pub type RealNavigationTarget = NavigationTarget<FileId>;
pub type HirNavigationTarget = NavigationTarget<HirFileId>;

/// `NavigationTarget` represents an element in the editor's UI which you can
/// click on to navigate to a particular piece of code.
///
/// Typically, a `NavigationTarget` corresponds to some element in the source
/// code, like a function or a struct, but this is not strictly required.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct NavigationTarget<FileId> {
    pub file_id: FileId,
    /// Range which encompasses the whole element.
    ///
    /// Should include body, doc comments, attributes, etc.
    ///
    /// Clients should use this range to answer "is the cursor inside the
    /// element?" question.
    pub full_range: TextRange,
    /// A "most interesting" range within the `full_range`.
    ///
    /// Typically, `full_range` is the whole syntax node, including doc
    /// comments, and `focus_range` is the range of the identifier.
    ///
    /// Clients should place the cursor on this range when navigating to this target.
    ///
    /// This range must be contained within [`Self::full_range`].
    pub focus_range: Option<TextRange>,
    // FIXME: Symbol
    pub name: SmolStr,
    pub kind: Option<SymbolKind>,
    // FIXME: Symbol
    pub container_name: Option<SmolStr>,
    pub description: Option<String>,
    pub docs: Option<Documentation>,
    /// In addition to a `name` field, a `NavigationTarget` may also be aliased
    /// In such cases we want a `NavigationTarget` to be accessible by its alias
    // FIXME: Symbol
    pub alias: Option<SmolStr>,
}

impl<FileId: fmt::Debug> fmt::Debug for NavigationTarget<FileId> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_struct("NavigationTarget");
        macro_rules! opt {
            ($($name:ident)*) => {$(
                if let Some(it) = &self.$name {
                    f.field(stringify!($name), it);
                }
            )*}
        }
        f.field("file_id", &self.file_id).field("full_range", &self.full_range);
        opt!(focus_range);
        f.field("name", &self.name);
        opt!(kind container_name description docs);
        f.finish()
    }
}

pub trait ToNav {
    fn to_nav_hir(&self, db: &RootDatabase) -> HirNavigationTarget;
}

pub trait TryToNav {
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget>;
    fn try_to_nav(&self, db: &RootDatabase) -> Option<UpmappingResult<RealNavigationTarget>> {
        self.try_to_nav_hir(db).map(|it| HirNavigationTarget::upmap(it, db))
    }
}

impl<T: TryToNav, U: TryToNav> TryToNav for Either<T, U> {
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget> {
        match self {
            Either::Left(it) => it.try_to_nav_hir(db),
            Either::Right(it) => it.try_to_nav_hir(db),
        }
    }
}

impl RealNavigationTarget {
    pub fn focus_or_full_file_range(&self) -> FileRange {
        FileRange { file_id: self.file_id, range: self.focus_or_full_range() }
    }
}

impl HirNavigationTarget {
    pub fn focus_or_full_file_range(&self, db: &RootDatabase) -> FileRange {
        InFile { file_id: self.file_id, value: self.focus_or_full_range() }
            .original_node_file_range(db)
            .0
            .into_file_id(db)
    }

    /// Upmaps this nav target to the `target` [`HirFileId`] if possible.
    ///
    /// If successful, the result entries are guaranteed to have the `target` file id.
    pub fn upmap_to(
        self,
        db: &RootDatabase,
        target: HirFileId,
    ) -> Option<Vec<HirNavigationTarget>> {
        if self.file_id == target {
            return Some(vec![self]);
        }
        let ranges =
            orig_ranges_with_focus_in(db, self.file_id, self.full_range, self.focus_range, target)?;
        Some(
            ranges
                .into_iter()
                .map(|(range, focus)| {
                    NavigationTarget {
                        file_id: target,
                        full_range: range,
                        focus_range: focus,
                        name: self.name.clone(),
                        kind: self.kind,
                        container_name: self.container_name.clone(),
                        description: self.description.clone(),
                        // FIXME: possibly expensive clone!
                        docs: self.docs.clone(),
                        alias: self.alias.clone(),
                    }
                })
                .collect(),
        )
    }

    pub fn upmap(self, db: &RootDatabase) -> UpmappingResult<RealNavigationTarget> {
        orig_range_with_focus(db, self.file_id, self.full_range, self.focus_range).map(
            |(range, focus)| NavigationTarget {
                file_id: range.file_id,
                full_range: range.range,
                focus_range: focus,
                name: self.name.clone(),
                kind: self.kind,
                container_name: self.container_name.clone(),
                description: self.description.clone(),
                // FIXME: possibly expensive clone!
                docs: self.docs.clone(),
                alias: self.alias.clone(),
            },
        )
    }
}

impl<FileId: fmt::Debug> NavigationTarget<FileId> {
    pub fn debug_render(&self) -> String {
        let mut buf = format!(
            "{} {:?} {:?} {:?}",
            self.name,
            self.kind.unwrap(),
            self.file_id,
            self.full_range
        );
        if let Some(focus_range) = self.focus_range {
            buf.push_str(&format!(" {focus_range:?}"))
        }
        if let Some(container_name) = &self.container_name {
            buf.push_str(&format!(" {container_name}"))
        }
        buf
    }
}

impl HirNavigationTarget {
    pub fn from_module_to_decl(db: &RootDatabase, module: hir::Module) -> HirNavigationTarget {
        let edition = module.krate().edition(db);
        let name =
            module.name(db).map(|it| it.display(db, edition).to_smolstr()).unwrap_or_default();
        match module.declaration_source(db) {
            Some(InFile { value, file_id }) => {
                let display_target = module.krate().to_display_target(db);
                let mut res = NavigationTarget::from_syntax(
                    file_id,
                    name.clone(),
                    value.name().map(|it| it.syntax().text_range()),
                    value.syntax().text_range(),
                    SymbolKind::Module,
                );
                res.docs = module.docs(db);
                res.description = Some(module.display(db, display_target).to_string());
                res
            }
            _ => module.to_nav_hir(db),
        }
    }
}

impl<FileId> NavigationTarget<FileId> {
    pub fn focus_or_full_range(&self) -> TextRange {
        self.focus_range.unwrap_or(self.full_range)
    }

    /// Allows `NavigationTarget` to be created from a `NameOwner`
    pub fn from_named(
        InFileWrapper { file_id, value }: InFileWrapper<FileId, &dyn ast::HasName>,
        kind: SymbolKind,
    ) -> Self {
        let name: SmolStr = value.name().map(|it| it.text().into()).unwrap_or_else(|| "_".into());

        NavigationTarget::from_syntax(
            file_id,
            name.clone(),
            value.name().map(|it| it.syntax().text_range()),
            value.syntax().text_range(),
            kind,
        )
    }

    pub fn from_syntax(
        file_id: FileId,
        name: SmolStr,
        focus_range: Option<TextRange>,
        full_range: TextRange,
        kind: SymbolKind,
    ) -> Self {
        NavigationTarget {
            file_id,
            name,
            kind: Some(kind),
            full_range,
            focus_range,
            container_name: None,
            description: None,
            docs: None,
            alias: None,
        }
    }
}

impl TryToNav for FileSymbol {
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget> {
        let edition =
            self.def.module(db).map(|it| it.krate().edition(db)).unwrap_or(Edition::CURRENT);
        let display_target = self.def.krate(db).to_display_target(db);

        Some(NavigationTarget {
            file_id: self.loc.hir_file_id,
            name: self.is_alias.then(|| self.def.name(db)).flatten().map_or_else(
                || self.name.as_str().into(),
                |it| it.display_no_db(edition).to_smolstr(),
            ),
            alias: self.is_alias.then(|| self.name.as_str().into()),
            kind: Some(self.def.into()),
            full_range: self.loc.ptr.text_range(),
            focus_range: Some(self.loc.name_ptr.text_range()),
            container_name: self.container_name.clone(),
            description: match self.def {
                hir::ModuleDef::Module(it) => Some(it.display(db, display_target).to_string()),
                hir::ModuleDef::Function(it) => Some(it.display(db, display_target).to_string()),
                hir::ModuleDef::Adt(it) => Some(it.display(db, display_target).to_string()),
                hir::ModuleDef::Variant(it) => Some(it.display(db, display_target).to_string()),
                hir::ModuleDef::Const(it) => Some(it.display(db, display_target).to_string()),
                hir::ModuleDef::Static(it) => Some(it.display(db, display_target).to_string()),
                hir::ModuleDef::Trait(it) => Some(it.display(db, display_target).to_string()),
                hir::ModuleDef::TraitAlias(it) => Some(it.display(db, display_target).to_string()),
                hir::ModuleDef::TypeAlias(it) => Some(it.display(db, display_target).to_string()),
                hir::ModuleDef::Macro(it) => Some(it.display(db, display_target).to_string()),
                hir::ModuleDef::BuiltinType(_) => None,
            },
            docs: None,
        })
    }
}

impl TryToNav for Definition {
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget> {
        match self {
            Definition::Local(it) => Some(it.to_nav_hir(db)),
            Definition::Label(it) => it.try_to_nav_hir(db),
            Definition::Module(it) => Some(it.to_nav_hir(db)),
            Definition::Crate(it) => Some(it.to_nav_hir(db)),
            Definition::Macro(it) => it.try_to_nav_hir(db),
            Definition::Field(it) => it.try_to_nav_hir(db),
            Definition::SelfType(it) => it.try_to_nav_hir(db),
            Definition::GenericParam(it) => it.try_to_nav_hir(db),
            Definition::Function(it) => it.try_to_nav_hir(db),
            Definition::Adt(it) => it.try_to_nav_hir(db),
            Definition::Variant(it) => it.try_to_nav_hir(db),
            Definition::Const(it) => it.try_to_nav_hir(db),
            Definition::Static(it) => it.try_to_nav_hir(db),
            Definition::Trait(it) => it.try_to_nav_hir(db),
            Definition::TraitAlias(it) => it.try_to_nav_hir(db),
            Definition::TypeAlias(it) => it.try_to_nav_hir(db),
            Definition::ExternCrateDecl(it) => it.try_to_nav_hir(db),
            Definition::InlineAsmOperand(it) => it.try_to_nav_hir(db),
            Definition::BuiltinLifetime(_)
            | Definition::BuiltinType(_)
            | Definition::TupleField(_)
            | Definition::ToolModule(_)
            | Definition::InlineAsmRegOrRegClass(_)
            | Definition::BuiltinAttr(_) => None,
            // FIXME: The focus range should be set to the helper declaration
            Definition::DeriveHelper(it) => it.derive().try_to_nav_hir(db),
        }
    }
}

impl TryToNav for hir::ModuleDef {
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget> {
        match self {
            hir::ModuleDef::Module(it) => Some(it.to_nav_hir(db)),
            hir::ModuleDef::Function(it) => it.try_to_nav_hir(db),
            hir::ModuleDef::Adt(it) => it.try_to_nav_hir(db),
            hir::ModuleDef::Variant(it) => it.try_to_nav_hir(db),
            hir::ModuleDef::Const(it) => it.try_to_nav_hir(db),
            hir::ModuleDef::Static(it) => it.try_to_nav_hir(db),
            hir::ModuleDef::Trait(it) => it.try_to_nav_hir(db),
            hir::ModuleDef::TraitAlias(it) => it.try_to_nav_hir(db),
            hir::ModuleDef::TypeAlias(it) => it.try_to_nav_hir(db),
            hir::ModuleDef::Macro(it) => it.try_to_nav_hir(db),
            hir::ModuleDef::BuiltinType(_) => None,
        }
    }
}

pub(crate) trait ToNavFromAst: Sized {
    const KIND: SymbolKind;
    fn container_name(self, db: &RootDatabase) -> Option<SmolStr> {
        _ = db;
        None
    }
}

fn container_name(db: &RootDatabase, t: impl HasContainer, edition: Edition) -> Option<SmolStr> {
    match t.container(db) {
        hir::ItemContainer::Trait(it) => Some(it.name(db).display_no_db(edition).to_smolstr()),
        // FIXME: Handle owners of blocks correctly here
        hir::ItemContainer::Module(it) => {
            it.name(db).map(|name| name.display_no_db(edition).to_smolstr())
        }
        _ => None,
    }
}

impl ToNavFromAst for hir::Function {
    const KIND: SymbolKind = SymbolKind::Function;
    fn container_name(self, db: &RootDatabase) -> Option<SmolStr> {
        container_name(db, self, self.krate(db).edition(db))
    }
}

impl ToNavFromAst for hir::Const {
    const KIND: SymbolKind = SymbolKind::Const;
    fn container_name(self, db: &RootDatabase) -> Option<SmolStr> {
        container_name(db, self, self.krate(db).edition(db))
    }
}
impl ToNavFromAst for hir::Static {
    const KIND: SymbolKind = SymbolKind::Static;
    fn container_name(self, db: &RootDatabase) -> Option<SmolStr> {
        container_name(db, self, self.krate(db).edition(db))
    }
}
impl ToNavFromAst for hir::Struct {
    const KIND: SymbolKind = SymbolKind::Struct;
    fn container_name(self, db: &RootDatabase) -> Option<SmolStr> {
        container_name(db, self, self.krate(db).edition(db))
    }
}
impl ToNavFromAst for hir::Enum {
    const KIND: SymbolKind = SymbolKind::Enum;
    fn container_name(self, db: &RootDatabase) -> Option<SmolStr> {
        container_name(db, self, self.krate(db).edition(db))
    }
}
impl ToNavFromAst for hir::Variant {
    const KIND: SymbolKind = SymbolKind::Variant;
}
impl ToNavFromAst for hir::Union {
    const KIND: SymbolKind = SymbolKind::Union;
    fn container_name(self, db: &RootDatabase) -> Option<SmolStr> {
        container_name(db, self, self.krate(db).edition(db))
    }
}
impl ToNavFromAst for hir::TypeAlias {
    const KIND: SymbolKind = SymbolKind::TypeAlias;
    fn container_name(self, db: &RootDatabase) -> Option<SmolStr> {
        container_name(db, self, self.krate(db).edition(db))
    }
}
impl ToNavFromAst for hir::Trait {
    const KIND: SymbolKind = SymbolKind::Trait;
    fn container_name(self, db: &RootDatabase) -> Option<SmolStr> {
        container_name(db, self, self.krate(db).edition(db))
    }
}
impl ToNavFromAst for hir::TraitAlias {
    const KIND: SymbolKind = SymbolKind::TraitAlias;
    fn container_name(self, db: &RootDatabase) -> Option<SmolStr> {
        container_name(db, self, self.krate(db).edition(db))
    }
}

impl<D> TryToNav for D
where
    D: HasSource + ToNavFromAst + Copy + HasDocs + HirDisplay + HasCrate,
    D::Ast: ast::HasName,
{
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget> {
        let src = self.source(db)?;
        let mut navigation_target =
            NavigationTarget::from_named(src.as_ref().map(|it| it as &dyn ast::HasName), D::KIND);
        navigation_target.docs = self.docs(db);
        navigation_target.description =
            Some(self.display(db, self.krate(db).to_display_target(db)).to_string());
        navigation_target.container_name = self.container_name(db);
        Some(navigation_target)
    }
}

impl ToNav for hir::Module {
    fn to_nav_hir(&self, db: &RootDatabase) -> HirNavigationTarget {
        let InFile { file_id, value } = self.definition_source(db);
        let edition = self.krate(db).edition(db);

        let name =
            self.name(db).map(|it| it.display_no_db(edition).to_smolstr()).unwrap_or_default();
        let (syntax, focus) = match &value {
            ModuleSource::SourceFile(node) => (node.syntax(), None),
            ModuleSource::Module(node) => (node.syntax(), node.name()),
            ModuleSource::BlockExpr(node) => (node.syntax(), None),
        };

        NavigationTarget::from_syntax(
            file_id,
            name.clone(),
            focus.map(|it| it.syntax().text_range()),
            syntax.text_range(),
            SymbolKind::Module,
        )
    }
}

impl ToNav for hir::Crate {
    fn to_nav_hir(&self, db: &RootDatabase) -> HirNavigationTarget {
        self.root_module().to_nav_hir(db)
    }
}

impl TryToNav for hir::Impl {
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget> {
        let InFile { file_id, value } = self.source(db)?;
        let derive_path = self.as_builtin_derive_path(db);

        let (file_id, focus, syntax) = match &derive_path {
            Some(attr) => (attr.file_id.into(), None, attr.value.syntax()),
            None => (file_id, value.self_ty(), value.syntax()),
        };
        Some(NavigationTarget::from_syntax(
            file_id,
            "impl".into(),
            focus.map(|it| it.syntax().text_range()),
            syntax.text_range(),
            SymbolKind::Impl,
        ))
    }
}

impl TryToNav for hir::ExternCrateDecl {
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget> {
        let src = self.source(db)?;
        let InFile { file_id, value } = src;
        let focus = value
            .rename()
            .map_or_else(|| value.name_ref().map(Either::Left), |it| it.name().map(Either::Right));

        let krate = self.module(db).krate();
        let edition = krate.edition(db);
        let mut nav = NavigationTarget::from_syntax(
            file_id,
            self.alias_or_name(db)
                .unwrap_or_else(|| self.name(db))
                .display(db, edition)
                .to_smolstr(),
            focus.map(|it| it.syntax().text_range()),
            value.syntax().text_range(),
            SymbolKind::Module,
        );

        nav.docs = self.docs(db);
        nav.description = Some(self.display(db, krate.to_display_target(db)).to_string());
        nav.container_name = container_name(db, *self, edition);
        Some(nav)
    }
}

impl TryToNav for hir::Field {
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget> {
        let src = self.source(db)?;
        let krate = self.parent_def(db).module(db).krate();

        Some(match &src.value {
            FieldSource::Named(it) => {
                let mut nav = NavigationTarget::from_named(src.with_value(it), SymbolKind::Field);
                nav.docs = self.docs(db);
                nav.description = Some(self.display(db, krate.to_display_target(db)).to_string());
                nav
            }
            FieldSource::Pos(it) => NavigationTarget::from_syntax(
                src.file_id,
                format_smolstr!("{}", self.index()),
                None,
                it.syntax().text_range(),
                SymbolKind::Field,
            ),
        })
    }
}

impl TryToNav for hir::Macro {
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget> {
        let src = self.source(db)?;
        let name_owner: &dyn ast::HasName = match &src.value {
            Either::Left(it) => it,
            Either::Right(it) => it,
        };
        let mut nav =
            NavigationTarget::from_named(src.as_ref().with_value(name_owner), self.kind(db).into());
        nav.docs = self.docs(db);
        Some(nav)
    }
}

impl TryToNav for hir::Adt {
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget> {
        match self {
            hir::Adt::Struct(it) => it.try_to_nav_hir(db),
            hir::Adt::Union(it) => it.try_to_nav_hir(db),
            hir::Adt::Enum(it) => it.try_to_nav_hir(db),
        }
    }
}

impl TryToNav for hir::AssocItem {
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget> {
        match self {
            AssocItem::Function(it) => it.try_to_nav_hir(db),
            AssocItem::Const(it) => it.try_to_nav_hir(db),
            AssocItem::TypeAlias(it) => it.try_to_nav_hir(db),
        }
    }
}

impl TryToNav for hir::GenericParam {
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget> {
        match self {
            hir::GenericParam::TypeParam(it) => it.try_to_nav_hir(db),
            hir::GenericParam::ConstParam(it) => it.try_to_nav_hir(db),
            hir::GenericParam::LifetimeParam(it) => it.try_to_nav_hir(db),
        }
    }
}

impl ToNav for LocalSource {
    fn to_nav_hir(&self, db: &RootDatabase) -> HirNavigationTarget {
        let InFile { file_id, value } = &self.source;
        let file_id = *file_id;
        let local = self.local;
        let (node, name_n) = match &value {
            Either::Left(bind_pat) => (bind_pat.syntax(), bind_pat.name()),
            Either::Right(it) => (it.syntax(), it.name()),
        };
        let edition = self.local.parent(db).module(db).krate().edition(db);
        let name = local.name(db).display_no_db(edition).to_smolstr();
        let kind = if local.is_self(db) {
            SymbolKind::SelfParam
        } else if local.is_param(db) {
            SymbolKind::ValueParam
        } else {
            SymbolKind::Local
        };
        NavigationTarget {
            file_id,
            name,
            alias: None,
            kind: Some(kind),
            full_range: node.text_range(),
            focus_range: name_n.map(|it| it.syntax().text_range()),
            container_name: None,
            description: None,
            docs: None,
        }
    }
}

impl ToNav for hir::Local {
    fn to_nav_hir(&self, db: &RootDatabase) -> HirNavigationTarget {
        self.primary_source(db).to_nav_hir(db)
    }
}

impl TryToNav for hir::Label {
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget> {
        let InFile { file_id, value } = self.source(db)?;
        // Labels can't be keywords, so no escaping needed.
        let name = self.name(db).display_no_db(Edition::Edition2015).to_smolstr();

        Some(NavigationTarget {
            file_id,
            name: name.clone(),
            alias: None,
            kind: Some(SymbolKind::Label),
            full_range: value.syntax().text_range(),
            focus_range: value.lifetime().map(|it| it.syntax().text_range()),
            container_name: None,
            description: None,
            docs: None,
        })
    }
}

impl TryToNav for hir::TypeParam {
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget> {
        let InFile { file_id, value } = self.merge().source(db)?;
        let edition = self.module(db).krate().edition(db);
        let name = self.name(db).display_no_db(edition).to_smolstr();

        let value = match value {
            Either::Left(ast::TypeOrConstParam::Type(x)) => Either::Left(x),
            Either::Left(ast::TypeOrConstParam::Const(_)) => {
                never!();
                return None;
            }
            Either::Right(x) => Either::Right(x),
        };

        let syntax = match &value {
            Either::Left(type_param) => type_param.syntax(),
            Either::Right(trait_) => trait_.syntax(),
        };
        let focus = value.as_ref().either(|it| it.name(), |it| it.name());

        Some(NavigationTarget {
            file_id,
            name: name.clone(),
            alias: None,
            kind: Some(SymbolKind::TypeParam),
            full_range: syntax.text_range(),
            focus_range: focus.map(|it| it.syntax().text_range()),
            container_name: None,
            description: None,
            docs: None,
        })
    }
}

impl TryToNav for hir::TypeOrConstParam {
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget> {
        self.split(db).try_to_nav_hir(db)
    }
}

impl TryToNav for hir::LifetimeParam {
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget> {
        let InFile { file_id, value } = self.source(db)?;
        // Lifetimes cannot be keywords, so not escaping needed.
        let name = self.name(db).display_no_db(Edition::Edition2015).to_smolstr();

        Some(NavigationTarget {
            file_id,
            name: name.clone(),
            alias: None,
            kind: Some(SymbolKind::LifetimeParam),
            full_range: value.syntax().text_range(),
            focus_range: value.lifetime().map(|it| it.syntax().text_range()),
            container_name: None,
            description: None,
            docs: None,
        })
    }
}

impl TryToNav for hir::ConstParam {
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget> {
        let InFile { file_id, value } = self.merge().source(db)?;
        let edition = self.module(db).krate().edition(db);
        let name = self.name(db).display_no_db(edition).to_smolstr();

        let value = match value {
            Either::Left(ast::TypeOrConstParam::Const(x)) => x,
            _ => {
                never!();
                return None;
            }
        };

        Some(NavigationTarget {
            file_id,
            name: name.clone(),
            alias: None,
            kind: Some(SymbolKind::ConstParam),
            full_range: value.syntax().text_range(),
            focus_range: value.name().map(|it| it.syntax().text_range()),
            container_name: None,
            description: None,
            docs: None,
        })
    }
}

impl TryToNav for hir::InlineAsmOperand {
    fn try_to_nav_hir(&self, db: &RootDatabase) -> Option<HirNavigationTarget> {
        let InFile { file_id, value } = &self.source(db)?;
        let file_id = *file_id;
        let edition = self.parent(db).module(db).krate().edition(db);
        Some(NavigationTarget {
            file_id,
            name: self
                .name(db)
                .map_or_else(|| "_".into(), |it| it.display(db, edition).to_smolstr()),
            alias: None,
            kind: Some(SymbolKind::Local),
            full_range: value.syntax().text_range(),
            focus_range: value.name().map(|it| it.syntax().text_range()),
            container_name: None,
            description: None,
            docs: None,
        })
    }
}

#[derive(Debug)]
pub struct UpmappingResult<T> {
    /// The macro call site.
    pub call_site: T,
    /// The macro definition site, if relevant.
    pub def_site: Option<T>,
}

impl<T> UpmappingResult<T> {
    pub fn call_site(self) -> T {
        self.call_site
    }

    pub fn collect<FI: FromIterator<T>>(self) -> FI {
        FI::from_iter(self)
    }
}

impl<T> IntoIterator for UpmappingResult<T> {
    type Item = T;

    type IntoIter = <ArrayVec<T, 2> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.def_site
            .into_iter()
            .chain(Some(self.call_site))
            .collect::<ArrayVec<_, 2>>()
            .into_iter()
    }
}

impl<T> UpmappingResult<T> {
    pub(crate) fn map<U>(self, f: impl Fn(T) -> U) -> UpmappingResult<U> {
        UpmappingResult { call_site: f(self.call_site), def_site: self.def_site.map(f) }
    }
}

fn orig_range_with_focus(
    db: &RootDatabase,
    hir_file: HirFileId,
    value: TextRange,
    focus_range: Option<TextRange>,
) -> UpmappingResult<(FileRange, Option<TextRange>)> {
    let macro_file = match hir_file {
        HirFileId::FileId(editioned_file_id) => {
            return UpmappingResult {
                call_site: (
                    FileRange { file_id: editioned_file_id.file_id(db), range: value },
                    focus_range,
                ),
                def_site: None,
            };
        }
        HirFileId::MacroFile(macro_file) => macro_file,
    };
    let call_site_fallback = || UpmappingResult {
        call_site: (
            InFile::new(hir_file, value).original_node_file_range(db).0.into_file_id(db),
            None,
        ),
        def_site: None,
    };

    let Some(name) = focus_range else { return call_site_fallback() };

    let call_kind = || db.lookup_intern_macro_call(macro_file).kind;

    let def_range = || db.lookup_intern_macro_call(macro_file).def.definition_range(db);

    // FIXME: Also make use of the syntax context to determine which site we are at?
    let value_range = InFile::new(hir_file, value).original_node_file_range_opt(db);
    let ((call_site_range, call_site_focus), def_site) =
        match InFile::new(hir_file, name).original_node_file_range_opt(db) {
            // call site name
            Some((focus_range, ctxt)) if ctxt.is_root() => {
                // Try to upmap the node as well, if it ends up in the def site, go back to the call site
                (
                    (
                        match value_range {
                            // name is in the node in the macro input so we can return it
                            Some((range, ctxt))
                                if ctxt.is_root()
                                    && range.file_id == focus_range.file_id
                                    && range.range.contains_range(focus_range.range) =>
                            {
                                range
                            }
                            // name lies outside the node, so instead point to the macro call which
                            // *should* contain the name
                            _ => {
                                let kind = call_kind();
                                let range = kind.clone().original_call_range_with_input(db);
                                //If the focus range is in the attribute/derive body, we
                                // need to point the call site to the entire body, if not, fall back
                                // to the name range of the attribute/derive call
                                // FIXME: Do this differently, this is very inflexible the caller
                                // should choose this behavior
                                if range.file_id == focus_range.file_id
                                    && range.range.contains_range(focus_range.range)
                                {
                                    range
                                } else {
                                    kind.original_call_range(db)
                                }
                            }
                        },
                        Some(focus_range),
                    ),
                    // no def site relevant
                    None,
                )
            }

            // def site name
            // FIXME: This can be improved
            Some((focus_range, _ctxt)) => {
                match value_range {
                    // but overall node is in macro input
                    Some((range, ctxt)) if ctxt.is_root() => (
                        // node mapped up in call site, show the node
                        (range, None),
                        // def site, if the name is in the (possibly) upmapped def site range, show the
                        // def site
                        {
                            let (def_site, _) = def_range().original_node_file_range(db);
                            (def_site.file_id == focus_range.file_id
                                && def_site.range.contains_range(focus_range.range))
                            .then_some((def_site, Some(focus_range)))
                        },
                    ),
                    // node is in macro def, just show the focus
                    _ => (
                        // show the macro call
                        (call_kind().original_call_range(db), None),
                        Some((focus_range, Some(focus_range))),
                    ),
                }
            }
            // lost name? can't happen for single tokens
            None => return call_site_fallback(),
        };

    UpmappingResult {
        call_site: (
            call_site_range.into_file_id(db),
            call_site_focus.and_then(|hir::FileRange { file_id, range }| {
                if call_site_range.file_id == file_id && call_site_range.range.contains_range(range)
                {
                    Some(range)
                } else {
                    None
                }
            }),
        ),
        def_site: def_site.map(|(def_site_range, def_site_focus)| {
            (
                def_site_range.into_file_id(db),
                def_site_focus.and_then(|hir::FileRange { file_id, range }| {
                    if def_site_range.file_id == file_id
                        && def_site_range.range.contains_range(range)
                    {
                        Some(range)
                    } else {
                        None
                    }
                }),
            )
        }),
    }
}

pub(crate) fn orig_ranges_with_focus_in(
    db: &RootDatabase,
    hir_file: HirFileId,
    value: TextRange,
    focus_range: Option<TextRange>,
    target: HirFileId,
) -> Option<Vec<(TextRange, Option<TextRange>)>> {
    let (mut current, target) = match (hir_file, target) {
        (HirFileId::FileId(file_id), HirFileId::FileId(target_file_id))
            if file_id == target_file_id =>
        {
            return Some(vec![(value, focus_range)]);
        }
        (HirFileId::FileId(_), HirFileId::FileId(_) | HirFileId::MacroFile(_)) => return None,
        (HirFileId::MacroFile(_), HirFileId::FileId(target_file_id)) => {
            let r = orig_range_with_focus(db, hir_file, value, focus_range);
            if r.call_site.0.file_id != target_file_id.file_id(db) {
                return None;
            }
            let mut ranges = vec![];
            ranges.push((r.call_site.0.range, r.call_site.1));

            if let Some((def_range, def_focus)) =
                r.def_site.filter(|it| it.0.file_id == target_file_id.file_id(db))
            {
                ranges.push((def_range.range, def_focus));
            }
            return Some(ranges);
        }
        (HirFileId::MacroFile(current), HirFileId::MacroFile(target)) => (current, target),
    };
    let expansion_span_map = db.expansion_span_map(current);
    let span = expansion_span_map.span_at(value.start());
    // FIXME: Use this
    let _focus_span =
        focus_range.map(|focus_range| expansion_span_map.span_at(focus_range.start()));
    loop {
        let parent = current.parent(db).macro_file()?;
        if parent == target {
            let arg_map = db.expansion_span_map(parent);
            let arg_node = current.call_node(db);
            let arg_range = arg_node.text_range();
            break Some(
                arg_map
                    .ranges_with_span_exact(span)
                    .filter(|(range, _)| range.intersect(arg_range).is_some())
                    .map(TupleExt::head)
                    .zip(iter::repeat(None))
                    .collect(),
            );
        }
        current = parent;
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use test_fixture::WithFixture;

    use crate::{
        RootDatabase,
        navigation_target::TryToNav,
        symbol_index::{self, Query},
    };

    #[test]
    fn test_nav_for_symbol() {
        let (db, _) = &RootDatabase::with_single_file(
            r#"
enum FooInner { }
fn foo() { enum FooInner { } }
"#,
        );

        let navs = symbol_index::world_symbols(db, Query::new("FooInner".to_owned()))
            .into_iter()
            .filter_map(|s| s.try_to_nav_hir(db))
            .collect::<Vec<_>>();
        expect![[r#"
            [
                NavigationTarget {
                    file_id: FileId(
                        EditionedFileId(
                            Id(1800),
                        ),
                    ),
                    full_range: 0..17,
                    focus_range: 5..13,
                    name: "FooInner",
                    kind: Enum,
                    description: "enum FooInner",
                },
                NavigationTarget {
                    file_id: FileId(
                        EditionedFileId(
                            Id(1800),
                        ),
                    ),
                    full_range: 29..46,
                    focus_range: 34..42,
                    name: "FooInner",
                    kind: Enum,
                    container_name: "foo",
                    description: "enum FooInner",
                },
            ]
        "#]]
        .assert_debug_eq(&navs);
    }

    #[test]
    fn test_world_symbols_are_case_sensitive() {
        let (db, _) = &RootDatabase::with_single_file(
            r#"
fn foo() {}
struct Foo;
"#,
        );

        let navs = symbol_index::world_symbols(db, Query::new("foo".to_owned()))
            .into_iter()
            .filter_map(|s| s.try_to_nav_hir(db))
            .collect::<Vec<_>>();
        assert_eq!(navs.len(), 2)
    }

    #[test]
    fn test_ensure_hidden_symbols_are_not_returned() {
        let (db, _) = &RootDatabase::with_single_file(
            r#"
fn foo() {}
struct Foo;
static __FOO_CALLSITE: () = ();
"#,
        );

        // It doesn't show the hidden symbol

        let navs = symbol_index::world_symbols(db, Query::new("foo".to_owned()))
            .into_iter()
            .filter_map(|s| s.try_to_nav_hir(db))
            .collect::<Vec<_>>();
        assert_eq!(navs.len(), 2);

        let navs = symbol_index::world_symbols(db, Query::new("_foo".to_owned()))
            .into_iter()
            .filter_map(|s| s.try_to_nav_hir(db))
            .collect::<Vec<_>>();
        assert_eq!(navs.len(), 0);

        // Unless we explicitly search for a `__` prefix
        let navs = symbol_index::world_symbols(db, Query::new("__foo".to_owned()))
            .into_iter()
            .filter_map(|s| s.try_to_nav_hir(db))
            .collect::<Vec<_>>();
        assert_eq!(navs.len(), 1);
    }
}
