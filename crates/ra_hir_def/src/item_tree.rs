use hir_expand::{ast_id_map::FileAstId, name::Name};
use ra_arena::{Arena, Idx};
use ra_syntax::ast;

use crate::{
    generics::GenericParams,
    path::{ImportAlias, ModPath},
    type_ref::TypeRef,
    visibility::RawVisibility,
};

#[derive(Default)]
pub struct ItemTree {
    imports: Arena<Import>,
    functions: Arena<Function>,
    structs: Arena<Struct>,
    unions: Arena<Union>,
    enums: Arena<Enum>,
    consts: Arena<Const>,
    statics: Arena<Static>,
    traits: Arena<Trait>,
    impls: Arena<Impl>,
    type_aliass: Arena<TypeAlias>,
    mods: Arena<Mod>,
    macro_calls: Arena<MacroCall>,
    exprs: Arena<Expr>,
}

impl ItemTree {
    fn new(syntax: &ast::SourceFile) -> ItemTree {
        ItemTree::default()
    }
}

pub struct Import {
    pub path: ModPath,
    pub alias: Option<ImportAlias>,
    pub visibility: RawVisibility,
    pub is_glob: bool,
    pub is_prelude: bool,
    pub is_extern_crate: bool,
    pub is_macro_use: bool,
}

pub struct Function {
    pub name: Name,
    pub visibility: RawVisibility,
    pub generic_params: GenericParams,
    pub has_self_param: bool,
    pub params: Vec<TypeRef>,
    pub ret_type: TypeRef,
    pub body: Option<Idx<Expr>>,
}

pub struct Struct {
    pub name: Name,
    pub visibility: RawVisibility,
    pub generic_params: GenericParams,
    pub fields: Fields,
}

pub struct Union {
    pub name: Name,
    pub visibility: RawVisibility,
    pub generic_params: GenericParams,
    pub fields: Fields,
}

pub struct Enum {
    pub name: Name,
    pub visibility: RawVisibility,
    pub generic_params: GenericParams,
    pub variants: Arena<Variant>,
}

pub struct Const {
    /// const _: () = ();
    pub name: Option<Name>,
    pub visibility: RawVisibility,
    pub type_ref: TypeRef,
    pub body: Option<Idx<Expr>>,
}

pub struct Static {
    pub name: Name,
    pub visibility: RawVisibility,
    pub type_ref: TypeRef,
    pub body: Option<Idx<Expr>>,
}

pub struct Trait {
    pub name: Name,
    pub visibility: RawVisibility,
    pub generic_params: GenericParams,
    pub auto: bool,
    pub items: Vec<AssocItem>,
}

pub struct Impl {
    pub generic_params: GenericParams,
    pub target_trait: Option<TypeRef>,
    pub target_type: TypeRef,
    pub is_negative: bool,
    pub items: Vec<AssocItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAlias {
    pub name: Name,
    pub visibility: RawVisibility,
    pub generic_params: GenericParams,
    pub type_ref: Option<TypeRef>,
}

pub struct Mod {
    pub name: Name,
    pub visibility: RawVisibility,
    pub items: Vec<ModItem>,
}

pub struct MacroCall {
    pub name: Option<Name>,
    pub path: ModPath,
    pub export: bool,
    pub builtin: bool,
    pub ast_id: FileAstId<ast::MacroCall>,
}

pub struct Expr {
    pub ast_id: FileAstId<ast::Expr>,
}

pub enum ModItem {
    Import(Idx<Import>),
    Function(Idx<Function>),
    Struct(Idx<Struct>),
    Union(Idx<Union>),
    Enum(Idx<Enum>),
    Const(Idx<Const>),
    Static(Idx<Static>),
    Trait(Idx<Trait>),
    Impl(Idx<Impl>),
    TypeAlias(Idx<TypeAlias>),
    Mod(Idx<Mod>),
    MacroCall(Idx<MacroCall>),
}

pub enum AssocItem {
    Function(Idx<Function>),
    TypeAlias(Idx<TypeAlias>),
    Const(Idx<Const>),
    MacroCall(Idx<MacroCall>),
}

pub struct Variant {
    pub name: Name,
    pub fields: Fields,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fields {
    Record(Arena<StructField>),
    Tuple(Arena<StructField>),
    Unit,
}

/// A single field of an enum variant or struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    pub name: Name,
    pub type_ref: TypeRef,
    pub visibility: RawVisibility,
}
