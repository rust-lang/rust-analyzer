//! Provides set of implementation for hir's objects that allows get back location in file.

use either::Either;
use hir_def::{
    nameres::{ModuleOrigin, ModuleSource},
    src::{HasChildSource, HasSource as _},
    CallableDefId, MacroId, VariantId,
};
use hir_expand::{HirFileId, InFile};
use hir_ty::db::InternedClosure;
use span::EditionedFileId;
use syntax::ast;
use tt::TextRange;

use crate::{
    db::HirDatabase, Adt, Callee, Const, Enum, ExternCrateDecl, Field, FieldSource, Function, Impl,
    Label, LifetimeParam, LocalSource, Macro, Module, Param, SelfParam, Semantics, Static, Struct,
    Trait, TraitAlias, TypeAlias, TypeOrConstParam, Union, Variant,
};

pub trait HasSource {
    type Ast;
    /// Fetches the definition's source node.
    /// Using [`crate::Semantics::source`] is preferred when working with [`crate::Semantics`],
    /// as that caches the parsed file in the semantics' cache.
    ///
    /// The current some implementations can return `InFile` instead of `Option<InFile>`.
    /// But we made this method `Option` to support rlib in the future
    /// by <https://github.com/rust-lang/rust-analyzer/issues/6913>
    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>>;
}

/// NB: Module is !HasSource, because it has two source nodes at the same time:
/// definition and declaration.
impl Module {
    /// Returns a node which defines this module. That is, a file or a `mod foo {}` with items.
    pub fn definition_source(self, db: &dyn HirDatabase) -> InFile<ModuleSource> {
        let def_map = self.id.def_map(db.upcast());
        def_map[self.id.local_id].definition_source(db.upcast())
    }

    /// Returns a node which defines this module. That is, a file or a `mod foo {}` with items.
    pub fn definition_source_range(self, db: &dyn HirDatabase) -> InFile<TextRange> {
        let def_map = self.id.def_map(db.upcast());
        def_map[self.id.local_id].definition_source_range(db.upcast())
    }

    pub fn definition_source_file_id(self, db: &dyn HirDatabase) -> HirFileId {
        let def_map = self.id.def_map(db.upcast());
        def_map[self.id.local_id].definition_source_file_id()
    }

    pub fn is_mod_rs(self, db: &dyn HirDatabase) -> bool {
        let def_map = self.id.def_map(db.upcast());
        match def_map[self.id.local_id].origin {
            ModuleOrigin::File { is_mod_rs, .. } => is_mod_rs,
            _ => false,
        }
    }

    pub fn as_source_file_id(self, db: &dyn HirDatabase) -> Option<EditionedFileId> {
        let def_map = self.id.def_map(db.upcast());
        match def_map[self.id.local_id].origin {
            ModuleOrigin::File { definition, .. } | ModuleOrigin::CrateRoot { definition, .. } => {
                Some(definition)
            }
            _ => None,
        }
    }

    pub fn is_inline(self, db: &dyn HirDatabase) -> bool {
        let def_map = self.id.def_map(db.upcast());
        def_map[self.id.local_id].origin.is_inline()
    }

    /// Returns a node which declares this module, either a `mod foo;` or a `mod foo {}`.
    /// `None` for the crate root.
    pub fn declaration_source(self, db: &dyn HirDatabase) -> Option<InFile<ast::Module>> {
        let def_map = self.id.def_map(db.upcast());
        def_map[self.id.local_id].declaration_source(db.upcast())
    }

    /// Returns a text range which declares this module, either a `mod foo;` or a `mod foo {}`.
    /// `None` for the crate root.
    pub fn declaration_source_range(self, db: &dyn HirDatabase) -> Option<InFile<TextRange>> {
        let def_map = self.id.def_map(db.upcast());
        def_map[self.id.local_id].declaration_source_range(db.upcast())
    }
}

impl HasSource for Field {
    type Ast = FieldSource;
    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        let var = VariantId::from(self.parent);
        let src = sema.with_d2s_ctx(|ctx| var.child_source(sema.db.upcast(), &mut Some(ctx)));
        let field_source = src.map(|it| match it[self.id].clone() {
            Either::Left(it) => FieldSource::Pos(it),
            Either::Right(it) => FieldSource::Named(it),
        });
        Some(field_source)
    }
}
impl HasSource for Adt {
    type Ast = ast::Adt;
    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        match self {
            Adt::Struct(s) => Some(s.source(sema)?.map(ast::Adt::Struct)),
            Adt::Union(u) => Some(u.source(sema)?.map(ast::Adt::Union)),
            Adt::Enum(e) => Some(e.source(sema)?.map(ast::Adt::Enum)),
        }
    }
}
impl HasSource for Struct {
    type Ast = ast::Struct;
    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        sema.with_d2s_ctx(|ctx| Some(self.id.source(sema.db.upcast(), &mut Some(ctx))))
    }
}

impl HasSource for Union {
    type Ast = ast::Union;
    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        sema.with_d2s_ctx(|ctx| Some(self.id.source(sema.db.upcast(), &mut Some(ctx))))
    }
}
impl HasSource for Enum {
    type Ast = ast::Enum;
    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        sema.with_d2s_ctx(|ctx| Some(self.id.source(sema.db.upcast(), &mut Some(ctx))))
    }
}
impl HasSource for Variant {
    type Ast = ast::Variant;
    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        sema.with_d2s_ctx(|ctx| Some(self.id.source(sema.db.upcast(), &mut Some(ctx))))
    }
}
impl HasSource for Function {
    type Ast = ast::Fn;
    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        sema.with_d2s_ctx(|ctx| Some(self.id.source(sema.db.upcast(), &mut Some(ctx))))
    }
}
impl HasSource for Const {
    type Ast = ast::Const;
    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        sema.with_d2s_ctx(|ctx| Some(self.id.source(sema.db.upcast(), &mut Some(ctx))))
    }
}
impl HasSource for Static {
    type Ast = ast::Static;
    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        sema.with_d2s_ctx(|ctx| Some(self.id.source(sema.db.upcast(), &mut Some(ctx))))
    }
}
impl HasSource for Trait {
    type Ast = ast::Trait;
    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        sema.with_d2s_ctx(|ctx| Some(self.id.source(sema.db.upcast(), &mut Some(ctx))))
    }
}
impl HasSource for TraitAlias {
    type Ast = ast::TraitAlias;
    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        sema.with_d2s_ctx(|ctx| Some(self.id.source(sema.db.upcast(), &mut Some(ctx))))
    }
}
impl HasSource for TypeAlias {
    type Ast = ast::TypeAlias;
    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        sema.with_d2s_ctx(|ctx| Some(self.id.source(sema.db.upcast(), &mut Some(ctx))))
    }
}
impl HasSource for Macro {
    type Ast = Either<ast::Macro, ast::Fn>;
    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        match self.id {
            MacroId::Macro2Id(it) => Some(
                sema.with_d2s_ctx(|ctx| it.source(sema.db.upcast(), &mut Some(ctx)))
                    .map(ast::Macro::MacroDef)
                    .map(Either::Left),
            ),
            MacroId::MacroRulesId(it) => Some(
                sema.with_d2s_ctx(|ctx| it.source(sema.db.upcast(), &mut Some(ctx)))
                    .map(ast::Macro::MacroRules)
                    .map(Either::Left),
            ),
            MacroId::ProcMacroId(it) => Some(
                sema.with_d2s_ctx(|ctx| it.source(sema.db.upcast(), &mut Some(ctx)))
                    .map(Either::Right),
            ),
        }
    }
}
impl HasSource for Impl {
    type Ast = ast::Impl;
    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        sema.with_d2s_ctx(|ctx| Some(self.id.source(sema.db.upcast(), &mut Some(ctx))))
    }
}

impl HasSource for TypeOrConstParam {
    type Ast = Either<ast::TypeOrConstParam, ast::TraitOrAlias>;
    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        let child_source =
            sema.with_d2s_ctx(|ctx| self.id.parent.child_source(sema.db.upcast(), &mut Some(ctx)));
        child_source.map(|it| it.get(self.id.local_id).cloned()).transpose()
    }
}

impl HasSource for LifetimeParam {
    type Ast = ast::LifetimeParam;
    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        let child_source =
            sema.with_d2s_ctx(|ctx| self.id.parent.child_source(sema.db.upcast(), &mut Some(ctx)));
        child_source.map(|it| it.get(self.id.local_id).cloned()).transpose()
    }
}

impl HasSource for LocalSource {
    type Ast = Either<ast::IdentPat, ast::SelfParam>;

    fn source<DB: HirDatabase>(self, _sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        Some(self.source)
    }
}

impl HasSource for Param {
    type Ast = Either<ast::SelfParam, ast::Param>;

    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        match self.func {
            Callee::Def(CallableDefId::FunctionId(func)) => {
                let InFile { file_id, value } = Function { id: func }.source(sema)?;
                let params = value.param_list()?;
                if let Some(self_param) = params.self_param() {
                    if let Some(idx) = self.idx.checked_sub(1) {
                        params.params().nth(idx).map(Either::Right)
                    } else {
                        Some(Either::Left(self_param))
                    }
                } else {
                    params.params().nth(self.idx).map(Either::Right)
                }
                .map(|value| InFile { file_id, value })
            }
            Callee::Closure(closure, _) => {
                let InternedClosure(owner, expr_id) = sema.db.lookup_intern_closure(closure.into());
                let (_, source_map) = sema.db.body_with_source_map(owner);
                let ast @ InFile { file_id, value } = source_map.expr_syntax(expr_id).ok()?;
                let root = sema.db.parse_or_expand(file_id);
                match value.to_node(&root) {
                    ast::Expr::ClosureExpr(it) => it
                        .param_list()?
                        .params()
                        .nth(self.idx)
                        .map(Either::Right)
                        .map(|value| InFile { file_id: ast.file_id, value }),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

impl HasSource for SelfParam {
    type Ast = ast::SelfParam;

    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        let InFile { file_id, value } = Function::from(self.func).source(sema)?;
        value
            .param_list()
            .and_then(|params| params.self_param())
            .map(|value| InFile { file_id, value })
    }
}

impl HasSource for Label {
    type Ast = ast::Label;

    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        let (_body, source_map) = sema.db.body_with_source_map(self.parent);
        let src = source_map.label_syntax(self.label_id);
        let root = src.file_syntax(sema.db.upcast());
        Some(src.map(|ast| ast.to_node(&root)))
    }
}

impl HasSource for ExternCrateDecl {
    type Ast = ast::ExternCrate;

    fn source<DB: HirDatabase>(self, sema: &Semantics<'_, DB>) -> Option<InFile<Self::Ast>> {
        sema.with_d2s_ctx(|ctx| Some(self.id.source(sema.db.upcast(), &mut Some(ctx))))
    }
}
