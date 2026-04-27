//! Defines `ExpressionStore`: a lowered representation of functions, statics and
//! consts.
pub mod body;
mod expander;
pub mod lower;
pub mod path;
pub mod pretty;
pub mod scope;
#[cfg(test)]
mod tests;

use std::{
    borrow::Borrow,
    ops::{Deref, Index},
};

use cfg::{CfgExpr, CfgOptions};
use either::Either;
use hir_expand::{InFile, MacroCallId, mod_path::ModPath, name::Name};
use la_arena::{Arena, ArenaMap};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use span::{Edition, SyntaxContext};
use syntax::{AstPtr, SyntaxNodePtr, ast};
use thin_vec::ThinVec;
use tt::TextRange;

use crate::{
    AdtId, BlockId, ExpressionStoreOwnerId, GenericDefId, SyntheticSyntax,
    db::DefDatabase,
    expr_store::path::Path,
    hir::{
        Array, AsmOperand, Binding, BindingId, Expr, ExprId, ExprOrPatId, InlineAsm, Label,
        LabelId, MatchArm, Pat, PatId, RecordFieldPat, RecordLitField, RecordSpread, Statement,
    },
    nameres::{DefMap, block_def_map},
    signatures::VariantFields,
    type_ref::{LifetimeRef, LifetimeRefId, PathId, TypeRef, TypeRefId},
};

pub use self::body::{Body, BodySourceMap};
pub use self::lower::{
    hir_assoc_type_binding_to_ast, hir_generic_arg_to_ast, hir_segment_to_ast_segment,
};

/// A wrapper around [`span::SyntaxContext`] that is intended only for comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HygieneId(span::SyntaxContext);

impl HygieneId {
    // The edition doesn't matter here, we only use this for comparisons and to lookup the macro.
    pub const ROOT: Self = Self(span::SyntaxContext::root(Edition::Edition2015));

    pub fn new(mut ctx: span::SyntaxContext) -> Self {
        // See `Name` for why we're doing that.
        ctx.remove_root_edition();
        Self(ctx)
    }

    pub(crate) fn syntax_context(self) -> SyntaxContext {
        self.0
    }

    pub(crate) fn is_root(self) -> bool {
        self.0.is_root()
    }
}

pub type ExprPtr = AstPtr<ast::Expr>;
pub type ExprSource = InFile<ExprPtr>;

pub type PatPtr = AstPtr<ast::Pat>;
pub type PatSource = InFile<PatPtr>;

/// BlockExpr -> Desugared label from try block
pub type LabelPtr = AstPtr<Either<ast::Label, ast::BlockExpr>>;
pub type LabelSource = InFile<LabelPtr>;

pub type FieldPtr = AstPtr<ast::RecordExprField>;
pub type FieldSource = InFile<FieldPtr>;

pub type PatFieldPtr = AstPtr<Either<ast::RecordExprField, ast::RecordPatField>>;
pub type PatFieldSource = InFile<PatFieldPtr>;

pub type ExprOrPatPtr = AstPtr<Either<ast::Expr, ast::Pat>>;
pub type ExprOrPatSource = InFile<ExprOrPatPtr>;

pub type SelfParamPtr = AstPtr<ast::SelfParam>;
pub type MacroCallPtr = AstPtr<ast::MacroCall>;

pub type TypePtr = AstPtr<ast::Type>;
pub type TypeSource = InFile<TypePtr>;

pub type LifetimePtr = AstPtr<ast::Lifetime>;
pub type LifetimeSource = InFile<LifetimePtr>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExprRoot {
    root: ExprId,
    // We store, for each root, the range of exprs (and pats and bindings) it holds.
    // We store only the end (exclusive), since the start can be inferred from the previous
    // roots or is zero.
    exprs_end: ExprId,
    pats_end: PatId,
    bindings_end: BindingId,
}

// We split the store into types-only and expressions, because most stores (e.g. generics)
// don't store any expressions and this saves memory. Same thing for the source map.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ExpressionOnlyStore {
    exprs: Arena<Expr>,
    pats: Arena<Pat>,
    bindings: Arena<Binding>,
    labels: Arena<Label>,
    /// Id of the closure/coroutine that owns the corresponding binding. If a binding is owned by the
    /// top level expression, it will not be listed in here.
    binding_owners: FxHashMap<BindingId, ExprId>,
    /// Block expressions in this store that may contain inner items.
    block_scopes: Box<[BlockId]>,

    /// A map from an variable usages to their hygiene ID.
    ///
    /// Expressions (and destructuing patterns) that can be recorded here are single segment path, although not all single segments path refer
    /// to variables and have hygiene (some refer to items, we don't know at this stage).
    ident_hygiene: FxHashMap<ExprOrPatId, HygieneId>,

    /// Maps expression roots to their origin.
    ///
    /// Note: while every root expr is an inference root (aka. an `AnonConst`), there could be other roots that do not appear here.
    /// This can happen when anon consts are nested, for example:
    ///
    /// ```
    /// [
    ///     ();
    ///     {
    ///         // this repeat expr is anon const #1, and *only it* appears in this list.
    ///         [
    ///             ();
    ///             {
    ///                 // this repeat expr is anon const #2.
    ///                 0
    ///             }
    ///         ];
    ///         0
    ///     }
    /// ]
    /// ```
    /// We do this because this allows us to search this list using a binary search,
    /// and it does not bother us because we use this list for two things: constructing `ExprScopes`, which
    /// works fine with nested exprs, and retrieving inference results, and we copy the inner const's inference
    /// into the outer const.
    // FIXME: Array repeat is not problematic indeed, but this could still break with exprs in types,
    // which we do not visit for `ExprScopes` (they're fine for inference though). We either need to visit them,
    // or use a more complicated search.
    expr_roots: SmallVec<[ExprRoot; 1]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionStore {
    expr_only: Option<Box<ExpressionOnlyStore>>,
    pub types: Arena<TypeRef>,
    pub lifetimes: Arena<LifetimeRef>,
}

#[derive(Debug, Eq, Default)]
struct ExpressionOnlySourceMap {
    // AST expressions can create patterns in destructuring assignments. Therefore, `ExprSource` can also map
    // to `PatId`, and `PatId` can also map to `ExprSource` (the other way around is unaffected).
    expr_map: FxHashMap<ExprSource, ExprOrPatId>,
    expr_map_back: ArenaMap<ExprId, ExprOrPatSource>,

    pat_map: FxHashMap<PatSource, ExprOrPatId>,
    pat_map_back: ArenaMap<PatId, ExprOrPatSource>,

    label_map: FxHashMap<LabelSource, LabelId>,
    label_map_back: ArenaMap<LabelId, LabelSource>,

    binding_definitions:
        ArenaMap<BindingId, SmallVec<[PatId; 2 * size_of::<usize>() / size_of::<PatId>()]>>,

    /// We don't create explicit nodes for record fields (`S { record_field: 92 }`).
    /// Instead, we use id of expression (`92`) to identify the field.
    field_map_back: FxHashMap<ExprId, FieldSource>,
    pat_field_map_back: FxHashMap<PatId, PatFieldSource>,

    template_map: Option<Box<FormatTemplate>>,

    expansions: FxHashMap<InFile<MacroCallPtr>, MacroCallId>,

    /// Diagnostics accumulated during lowering. These contain `AstPtr`s and so are stored in
    /// the source map (since they're just as volatile).
    //
    // We store diagnostics on the `ExpressionOnlySourceMap` because diagnostics are rare (except
    // maybe for cfgs, and they are also not common in type places).
    diagnostics: ThinVec<ExpressionStoreDiagnostics>,
}

impl PartialEq for ExpressionOnlySourceMap {
    fn eq(&self, other: &Self) -> bool {
        // we only need to compare one of the two mappings
        // as the other is a reverse mapping and thus will compare
        // the same as normal mapping
        let Self {
            expr_map: _,
            expr_map_back,
            pat_map: _,
            pat_map_back,
            label_map: _,
            label_map_back,
            // If this changed, our pattern data must have changed
            binding_definitions: _,
            // If this changed, our expression data must have changed
            field_map_back: _,
            // If this changed, our pattern data must have changed
            pat_field_map_back: _,
            template_map,
            expansions,
            diagnostics,
        } = self;
        *expr_map_back == other.expr_map_back
            && *pat_map_back == other.pat_map_back
            && *label_map_back == other.label_map_back
            && *template_map == other.template_map
            && *expansions == other.expansions
            && *diagnostics == other.diagnostics
    }
}

#[derive(Debug, Eq, Default)]
pub struct ExpressionStoreSourceMap {
    expr_only: Option<Box<ExpressionOnlySourceMap>>,

    types_map_back: ArenaMap<TypeRefId, TypeSource>,
    types_map: FxHashMap<TypeSource, TypeRefId>,

    lifetime_map_back: ArenaMap<LifetimeRefId, LifetimeSource>,
    #[expect(
        unused,
        reason = "this is here for completeness, and maybe we'll need it in the future"
    )]
    lifetime_map: FxHashMap<LifetimeSource, LifetimeRefId>,
}

impl PartialEq for ExpressionStoreSourceMap {
    fn eq(&self, other: &Self) -> bool {
        // we only need to compare one of the two mappings
        // as the other is a reverse mapping and thus will compare
        // the same as normal mapping
        let Self { expr_only, types_map_back, types_map: _, lifetime_map_back, lifetime_map: _ } =
            self;
        *expr_only == other.expr_only
            && *types_map_back == other.types_map_back
            && *lifetime_map_back == other.lifetime_map_back
    }
}

/// The body of an item (function, const etc.).
#[derive(Debug, Eq, PartialEq, Default)]
pub struct ExpressionStoreBuilder {
    pub exprs: Arena<Expr>,
    pub pats: Arena<Pat>,
    pub bindings: Arena<Binding>,
    pub labels: Arena<Label>,
    pub lifetimes: Arena<LifetimeRef>,
    pub binding_owners: FxHashMap<BindingId, ExprId>,
    pub types: Arena<TypeRef>,
    block_scopes: Vec<BlockId>,
    ident_hygiene: FxHashMap<ExprOrPatId, HygieneId>,
    inference_roots: Option<SmallVec<[ExprRoot; 1]>>,

    // AST expressions can create patterns in destructuring assignments. Therefore, `ExprSource` can also map
    // to `PatId`, and `PatId` can also map to `ExprSource` (the other way around is unaffected).
    expr_map: FxHashMap<ExprSource, ExprOrPatId>,
    expr_map_back: ArenaMap<ExprId, ExprOrPatSource>,

    pat_map: FxHashMap<PatSource, ExprOrPatId>,
    pat_map_back: ArenaMap<PatId, ExprOrPatSource>,

    label_map: FxHashMap<LabelSource, LabelId>,
    label_map_back: ArenaMap<LabelId, LabelSource>,

    types_map_back: ArenaMap<TypeRefId, TypeSource>,
    types_map: FxHashMap<TypeSource, TypeRefId>,

    lifetime_map_back: ArenaMap<LifetimeRefId, LifetimeSource>,
    lifetime_map: FxHashMap<LifetimeSource, LifetimeRefId>,

    binding_definitions:
        ArenaMap<BindingId, SmallVec<[PatId; 2 * size_of::<usize>() / size_of::<PatId>()]>>,

    /// We don't create explicit nodes for record fields (`S { record_field: 92 }`).
    /// Instead, we use id of expression (`92`) to identify the field.
    field_map_back: FxHashMap<ExprId, FieldSource>,
    pat_field_map_back: FxHashMap<PatId, PatFieldSource>,

    template_map: Option<Box<FormatTemplate>>,

    expansions: FxHashMap<InFile<MacroCallPtr>, MacroCallId>,

    /// Diagnostics accumulated during lowering. These contain `AstPtr`s and so are stored in
    /// the source map (since they're just as volatile).
    //
    // We store diagnostics on the `ExpressionOnlySourceMap` because diagnostics are rare (except
    // maybe for cfgs, and they are also not common in type places).
    pub(crate) diagnostics: Vec<ExpressionStoreDiagnostics>,
}

#[derive(Default, Debug, Eq, PartialEq)]
struct FormatTemplate {
    /// A map from `format_args!()` expressions to their captures.
    format_args_to_captures: FxHashMap<ExprId, (HygieneId, Vec<(syntax::TextRange, Name)>)>,
    /// A map from `asm!()` expressions to their captures.
    asm_to_captures: FxHashMap<ExprId, Vec<Vec<(syntax::TextRange, usize)>>>,
    /// A map from desugared expressions of implicit captures to their source.
    ///
    /// The value stored for each capture is its template literal and offset inside it. The template literal
    /// is from the `format_args[_nl]!()` macro and so needs to be mapped up once to go to the user-written
    /// template.
    implicit_capture_to_source: FxHashMap<ExprId, InFile<(ExprPtr, TextRange)>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ExpressionStoreDiagnostics {
    InactiveCode { node: InFile<SyntaxNodePtr>, cfg: CfgExpr, opts: CfgOptions },
    UnresolvedMacroCall { node: InFile<MacroCallPtr>, path: ModPath },
    UnreachableLabel { node: InFile<AstPtr<ast::Lifetime>>, name: Name },
    AwaitOutsideOfAsync { node: InFile<AstPtr<ast::AwaitExpr>>, location: String },
    UndeclaredLabel { node: InFile<AstPtr<ast::Lifetime>>, name: Name },
    PatternArgInExternFn { node: InFile<AstPtr<ast::Pat>> },
}

impl ExpressionStoreBuilder {
    pub fn finish(self) -> (ExpressionStore, ExpressionStoreSourceMap) {
        let Self {
            block_scopes,
            mut exprs,
            mut labels,
            mut pats,
            mut bindings,
            mut binding_owners,
            mut ident_hygiene,
            inference_roots: mut expr_roots,
            mut types,
            mut lifetimes,

            mut expr_map,
            mut expr_map_back,
            mut pat_map,
            mut pat_map_back,
            mut label_map,
            mut label_map_back,
            mut types_map_back,
            mut types_map,
            mut lifetime_map_back,
            mut lifetime_map,
            mut binding_definitions,
            mut field_map_back,
            mut pat_field_map_back,
            mut template_map,
            mut expansions,
            diagnostics,
        } = self;
        exprs.shrink_to_fit();
        labels.shrink_to_fit();
        pats.shrink_to_fit();
        bindings.shrink_to_fit();
        binding_owners.shrink_to_fit();
        ident_hygiene.shrink_to_fit();
        types.shrink_to_fit();
        lifetimes.shrink_to_fit();

        expr_map.shrink_to_fit();
        expr_map_back.shrink_to_fit();
        pat_map.shrink_to_fit();
        pat_map_back.shrink_to_fit();
        label_map.shrink_to_fit();
        label_map_back.shrink_to_fit();
        types_map_back.shrink_to_fit();
        types_map.shrink_to_fit();
        lifetime_map_back.shrink_to_fit();
        lifetime_map.shrink_to_fit();
        binding_definitions.shrink_to_fit();
        field_map_back.shrink_to_fit();
        pat_field_map_back.shrink_to_fit();
        if let Some(template_map) = &mut template_map {
            let FormatTemplate {
                format_args_to_captures,
                asm_to_captures,
                implicit_capture_to_source,
            } = &mut **template_map;
            format_args_to_captures.shrink_to_fit();
            asm_to_captures.shrink_to_fit();
            implicit_capture_to_source.shrink_to_fit();
        }
        expansions.shrink_to_fit();

        let has_exprs =
            !exprs.is_empty() || !labels.is_empty() || !pats.is_empty() || !bindings.is_empty();

        let store = {
            let expr_only = if has_exprs {
                if let Some(expr_roots) = &mut expr_roots {
                    expr_roots.shrink_to_fit();
                }
                Some(Box::new(ExpressionOnlyStore {
                    exprs,
                    pats,
                    bindings,
                    labels,
                    binding_owners,
                    block_scopes: block_scopes.into_boxed_slice(),
                    ident_hygiene,
                    expr_roots: expr_roots
                        .expect("should always finish with a `Some(_)` expr_roots"),
                }))
            } else {
                None
            };
            ExpressionStore { expr_only, types, lifetimes }
        };

        let source_map = {
            let expr_only = if has_exprs || !expansions.is_empty() || !diagnostics.is_empty() {
                Some(Box::new(ExpressionOnlySourceMap {
                    expr_map,
                    expr_map_back,
                    pat_map,
                    pat_map_back,
                    label_map,
                    label_map_back,
                    binding_definitions,
                    field_map_back,
                    pat_field_map_back,
                    template_map,
                    expansions,
                    diagnostics: ThinVec::from_iter(diagnostics),
                }))
            } else {
                None
            };
            ExpressionStoreSourceMap {
                expr_only,
                types_map_back,
                types_map,
                lifetime_map_back,
                lifetime_map,
            }
        };

        (store, source_map)
    }
}

impl ExpressionStore {
    const EMPTY: &ExpressionStore =
        &ExpressionStore { expr_only: None, types: Arena::new(), lifetimes: Arena::new() };

    #[inline]
    pub fn empty() -> &'static ExpressionStore {
        ExpressionStore::EMPTY
    }

    pub fn of(db: &dyn DefDatabase, def: ExpressionStoreOwnerId) -> &ExpressionStore {
        match def {
            ExpressionStoreOwnerId::Signature(def) => {
                use crate::signatures::{
                    ConstSignature, EnumSignature, FunctionSignature, ImplSignature,
                    StaticSignature, StructSignature, TraitSignature, TypeAliasSignature,
                    UnionSignature,
                };
                match def {
                    GenericDefId::AdtId(AdtId::EnumId(id)) => &EnumSignature::of(db, id).store,
                    GenericDefId::AdtId(AdtId::StructId(id)) => &StructSignature::of(db, id).store,
                    GenericDefId::AdtId(AdtId::UnionId(id)) => &UnionSignature::of(db, id).store,
                    GenericDefId::ConstId(id) => &ConstSignature::of(db, id).store,
                    GenericDefId::FunctionId(id) => &FunctionSignature::of(db, id).store,
                    GenericDefId::ImplId(id) => &ImplSignature::of(db, id).store,
                    GenericDefId::StaticId(id) => &StaticSignature::of(db, id).store,
                    GenericDefId::TraitId(id) => &TraitSignature::of(db, id).store,
                    GenericDefId::TypeAliasId(id) => &TypeAliasSignature::of(db, id).store,
                }
            }
            ExpressionStoreOwnerId::Body(body) => &Body::of(db, body).store,
            ExpressionStoreOwnerId::VariantFields(variant_id) => {
                &VariantFields::of(db, variant_id).store
            }
        }
    }

    pub fn with_source_map(
        db: &dyn DefDatabase,
        def: ExpressionStoreOwnerId,
    ) -> (&ExpressionStore, &ExpressionStoreSourceMap) {
        match def {
            ExpressionStoreOwnerId::Signature(def) => {
                use crate::signatures::{
                    ConstSignature, EnumSignature, FunctionSignature, ImplSignature,
                    StaticSignature, StructSignature, TraitSignature, TypeAliasSignature,
                    UnionSignature,
                };
                match def {
                    GenericDefId::AdtId(AdtId::EnumId(id)) => {
                        let sig = EnumSignature::with_source_map(db, id);
                        (&sig.0.store, &sig.1)
                    }
                    GenericDefId::AdtId(AdtId::StructId(id)) => {
                        let sig = StructSignature::with_source_map(db, id);
                        (&sig.0.store, &sig.1)
                    }
                    GenericDefId::AdtId(AdtId::UnionId(id)) => {
                        let sig = UnionSignature::with_source_map(db, id);
                        (&sig.0.store, &sig.1)
                    }
                    GenericDefId::ConstId(id) => {
                        let sig = ConstSignature::with_source_map(db, id);
                        (&sig.0.store, &sig.1)
                    }
                    GenericDefId::FunctionId(id) => {
                        let sig = FunctionSignature::with_source_map(db, id);
                        (&sig.0.store, &sig.1)
                    }
                    GenericDefId::ImplId(id) => {
                        let sig = ImplSignature::with_source_map(db, id);
                        (&sig.0.store, &sig.1)
                    }
                    GenericDefId::StaticId(id) => {
                        let sig = StaticSignature::with_source_map(db, id);
                        (&sig.0.store, &sig.1)
                    }
                    GenericDefId::TraitId(id) => {
                        let sig = TraitSignature::with_source_map(db, id);
                        (&sig.0.store, &sig.1)
                    }
                    GenericDefId::TypeAliasId(id) => {
                        let sig = TypeAliasSignature::with_source_map(db, id);
                        (&sig.0.store, &sig.1)
                    }
                }
            }
            ExpressionStoreOwnerId::Body(body) => {
                let (store, sm) = Body::with_source_map(db, body);
                (&store.store, &sm.store)
            }
            ExpressionStoreOwnerId::VariantFields(variant_id) => {
                let (store, sm) = VariantFields::with_source_map(db, variant_id);
                (&store.store, sm)
            }
        }
    }

    /// Returns all expression root `ExprId`s found in this store.
    pub fn expr_roots(&self) -> impl DoubleEndedIterator<Item = ExprId> {
        self.expr_only
            .as_ref()
            .map_or(&[][..], |expr_only| &expr_only.expr_roots)
            .iter()
            .map(|root| root.root)
    }

    fn find_root_for(
        &self,
        mut get: impl FnMut(&ExprRoot) -> la_arena::RawIdx,
        find: la_arena::RawIdx,
    ) -> ExprId {
        let expr_only = self.assert_expr_only();
        let find = find.into_u32();
        let entry = expr_only.expr_roots.partition_point(|root| get(root).into_u32() <= find);
        expr_only.expr_roots[entry].root
    }

    pub fn find_root_for_expr(&self, expr: ExprId) -> ExprId {
        self.find_root_for(|root| root.exprs_end.into_raw(), expr.into_raw())
    }

    pub fn find_root_for_pat(&self, pat: PatId) -> ExprId {
        self.find_root_for(|root| root.pats_end.into_raw(), pat.into_raw())
    }

    pub fn find_root_for_binding(&self, binding: BindingId) -> ExprId {
        self.find_root_for(|root| root.bindings_end.into_raw(), binding.into_raw())
    }

    /// Returns an iterator over all block expressions in this store that define inner items.
    pub fn blocks<'a>(
        &'a self,
        db: &'a dyn DefDatabase,
    ) -> impl Iterator<Item = (BlockId, &'a DefMap)> + 'a {
        self.expr_only
            .as_ref()
            .map(|it| &*it.block_scopes)
            .unwrap_or_default()
            .iter()
            .map(move |&block| (block, block_def_map(db, block)))
    }

    pub fn walk_bindings_in_pat(&self, pat_id: PatId, mut f: impl FnMut(BindingId)) {
        self.walk_pats(pat_id, &mut |pat| {
            if let Pat::Bind { id, .. } = &self[pat] {
                f(*id);
            }
        });
    }

    pub fn walk_pats_shallow(&self, pat_id: PatId, mut f: impl FnMut(PatId)) {
        // Do not use `..` patterns or field accesses here, only destructuring, to ensure we cover all cases
        // (we've had multiple bugs with this in the past).
        let pat = &self[pat_id];
        match pat {
            Pat::Range { start: _, end: _, range_type: _ }
            | Pat::Lit(_)
            | Pat::Path(_)
            | Pat::ConstBlock(_)
            | Pat::Wild
            | Pat::Missing
            | Pat::Rest
            | Pat::Expr(_) => {}
            &Pat::Bind { subpat, id: _ } => {
                if let Some(subpat) = subpat {
                    f(subpat);
                }
            }
            Pat::Or(args)
            | Pat::Tuple { args, ellipsis: _ }
            | Pat::TupleStruct { args, ellipsis: _, path: _ } => {
                args.iter().copied().for_each(f);
            }
            Pat::Ref { pat, mutability: _ } => f(*pat),
            Pat::Slice { prefix, slice, suffix } => {
                let total_iter = prefix.iter().chain(slice.iter()).chain(suffix.iter());
                total_iter.copied().for_each(f);
            }
            Pat::Record { args, ellipsis: _, path: _ } => {
                args.iter().for_each(|RecordFieldPat { pat, name: _ }| f(*pat));
            }
            Pat::Box { inner } => f(*inner),
        }
    }

    pub fn walk_pats(&self, pat_id: PatId, f: &mut impl FnMut(PatId)) {
        f(pat_id);
        self.walk_pats_shallow(pat_id, |p| self.walk_pats(p, f));
    }

    pub fn is_binding_upvar(&self, binding: BindingId, relative_to: ExprId) -> bool {
        let Some(expr_only) = &self.expr_only else { return false };
        match expr_only.binding_owners.get(&binding) {
            Some(it) => {
                // We assign expression ids in a way that outer closures will receive
                // a higher id (allocated after their body is collected)
                it.into_raw() > relative_to.into_raw()
            }
            None => true,
        }
    }

    #[inline]
    pub fn binding_owner(&self, id: BindingId) -> Option<ExprId> {
        self.expr_only.as_ref()?.binding_owners.get(&id).copied()
    }

    fn walk_child_exprs_impl(&self, expr_id: ExprId, mut visitor: impl ExprVisitor) {
        // Do not use `..` patterns or field accesses here, only destructuring, to ensure we cover all cases
        // (we've had multiple bugs with this in the past).
        match &self[expr_id] {
            Expr::Continue { label: _ }
            | Expr::Missing
            | Expr::Path(_)
            | Expr::OffsetOf(_)
            | Expr::Literal(_)
            | Expr::Underscore => {}
            Expr::InlineAsm(InlineAsm { operands, options: _, kind: _ }) => {
                operands.iter().for_each(|(_, op)| match op {
                    AsmOperand::In { expr, reg: _ }
                    | AsmOperand::Out { expr: Some(expr), late: _, reg: _ }
                    | AsmOperand::InOut { expr, late: _, reg: _ }
                    | AsmOperand::Const(expr)
                    | AsmOperand::Label(expr) => visitor.on_expr(*expr),
                    AsmOperand::SplitInOut { in_expr, out_expr, late: _, reg: _ } => {
                        visitor.on_expr(*in_expr);
                        visitor.on_expr_opt(*out_expr);
                    }
                    AsmOperand::Out { expr: None, late: _, reg: _ } | AsmOperand::Sym(_) => (),
                })
            }
            Expr::If { condition, then_branch, else_branch } => {
                visitor.on_expr(*condition);
                visitor.on_expr(*then_branch);
                visitor.on_expr_opt(*else_branch);
            }
            Expr::Let { expr, pat } => {
                visitor.on_pat(*pat);
                visitor.on_expr(*expr);
            }
            Expr::Block { statements, tail, id: _, label: _ }
            | Expr::Unsafe { statements, tail, id: _ } => {
                for stmt in statements {
                    match stmt {
                        Statement::Let { initializer, else_branch, pat, type_ref: _ } => {
                            visitor.on_expr_opt(*initializer);
                            visitor.on_expr_opt(*else_branch);
                            visitor.on_pat(*pat);
                        }
                        Statement::Expr { expr: expression, has_semi: _ } => {
                            visitor.on_expr(*expression)
                        }
                        Statement::Item(_) => (),
                    }
                }
                visitor.on_expr_opt(*tail);
            }
            Expr::Loop { body, label: _ } => visitor.on_expr(*body),
            Expr::Call { callee, args } => {
                visitor.on_expr(*callee);
                visitor.on_exprs(args);
            }
            Expr::MethodCall { receiver, args, generic_args: _, method_name: _ } => {
                visitor.on_expr(*receiver);
                visitor.on_exprs(args);
            }
            Expr::Match { expr, arms } => {
                visitor.on_expr(*expr);
                arms.iter().for_each(|MatchArm { pat, guard, expr }| {
                    visitor.on_expr(*expr);
                    visitor.on_expr_opt(*guard);
                    visitor.on_pat(*pat);
                });
            }
            Expr::Break { expr, label: _ }
            | Expr::Return { expr }
            | Expr::Yield { expr }
            | Expr::Yeet { expr } => visitor.on_expr_opt(*expr),
            Expr::Become { expr } => visitor.on_expr(*expr),
            Expr::RecordLit { fields, spread, path: _ } => {
                for RecordLitField { name: _, expr } in fields.iter() {
                    visitor.on_expr(*expr);
                }
                match spread {
                    RecordSpread::Expr(expr) => visitor.on_expr(*expr),
                    RecordSpread::None | RecordSpread::FieldDefaults => {}
                }
            }
            Expr::Closure {
                body,
                args,
                arg_types: _,
                capture_by: _,
                closure_kind: _,
                ret_type: _,
            } => {
                visitor.on_expr(*body);
                visitor.on_pats(args);
            }
            Expr::BinaryOp { lhs, rhs, op: _ } => {
                visitor.on_expr(*lhs);
                visitor.on_expr(*rhs);
            }
            Expr::Range { lhs, rhs, range_type: _ } => {
                visitor.on_expr_opt(*lhs);
                visitor.on_expr_opt(*rhs);
            }
            Expr::Index { base, index } => {
                visitor.on_expr(*base);
                visitor.on_expr(*index);
            }
            Expr::Field { expr, name: _ }
            | Expr::Await { expr }
            | Expr::Cast { expr, type_ref: _ }
            | Expr::Ref { expr, mutability: _, rawness: _ }
            | Expr::UnaryOp { expr, op: _ }
            | Expr::Box { expr }
            | Expr::Const(expr) => {
                visitor.on_expr(*expr);
            }
            Expr::Tuple { exprs } => visitor.on_exprs(exprs),
            Expr::Array(a) => match a {
                Array::ElementList { elements } => visitor.on_exprs(elements),
                Array::Repeat { initializer, repeat } => {
                    visitor.on_expr(*initializer);
                    visitor.on_expr(*repeat)
                }
            },
            &Expr::Assignment { target, value } => {
                visitor.on_pat(target);
                visitor.on_expr(value);
            }
        }
    }

    /// Walks the immediate children expressions and calls `f` for each child expression.
    pub fn walk_child_exprs(&self, expr_id: ExprId, callback: impl FnMut(ExprId)) {
        return self.walk_child_exprs_impl(expr_id, Visitor { callback, store: self });

        struct Visitor<'a, F> {
            callback: F,
            store: &'a ExpressionStore,
        }

        impl<F: FnMut(ExprId)> ExprVisitor for Visitor<'_, F> {
            fn on_expr(&mut self, expr: ExprId) {
                (self.callback)(expr);
            }

            fn on_pat(&mut self, pat: PatId) {
                self.store.walk_exprs_in_pat(pat, &mut self.callback);
            }
        }
    }

    /// Walks the immediate children expressions and calls `f` for each child expression but does
    /// not walk expressions within patterns.
    pub fn walk_child_exprs_without_pats(&self, expr_id: ExprId, callback: impl FnMut(ExprId)) {
        return self.walk_child_exprs_impl(expr_id, Visitor { callback });

        struct Visitor<F> {
            callback: F,
        }

        impl<F: FnMut(ExprId)> ExprVisitor for Visitor<F> {
            fn on_expr(&mut self, expr: ExprId) {
                (self.callback)(expr);
            }

            fn on_pat(&mut self, _pat: PatId) {}
        }
    }

    pub fn walk_exprs_in_pat(&self, pat_id: PatId, mut f: impl FnMut(ExprId)) {
        self.walk_pats(pat_id, &mut |pat| match self[pat] {
            Pat::Expr(expr) | Pat::ConstBlock(expr) | Pat::Lit(expr) => {
                f(expr);
            }
            Pat::Range { start, end, range_type: _ } => {
                if let Some(start) = start {
                    f(start);
                }
                if let Some(end) = end {
                    f(end);
                }
            }
            Pat::Missing
            | Pat::Rest
            | Pat::Wild
            | Pat::Tuple { .. }
            | Pat::Or(_)
            | Pat::Record { .. }
            | Pat::Slice { .. }
            | Pat::Path(_)
            | Pat::Bind { .. }
            | Pat::TupleStruct { .. }
            | Pat::Ref { .. }
            | Pat::Box { .. } => {}
        });
    }

    #[inline]
    #[track_caller]
    fn assert_expr_only(&self) -> &ExpressionOnlyStore {
        self.expr_only.as_ref().expect("should have `ExpressionStore::expr_only`")
    }

    fn binding_hygiene(&self, binding: BindingId) -> HygieneId {
        self.assert_expr_only().bindings[binding].hygiene
    }

    pub fn expr_path_hygiene(&self, expr: ExprId) -> HygieneId {
        self.assert_expr_only().ident_hygiene.get(&expr.into()).copied().unwrap_or(HygieneId::ROOT)
    }

    pub fn pat_path_hygiene(&self, pat: PatId) -> HygieneId {
        self.assert_expr_only().ident_hygiene.get(&pat.into()).copied().unwrap_or(HygieneId::ROOT)
    }

    pub fn expr_or_pat_path_hygiene(&self, id: ExprOrPatId) -> HygieneId {
        match id {
            ExprOrPatId::ExprId(id) => self.expr_path_hygiene(id),
            ExprOrPatId::PatId(id) => self.pat_path_hygiene(id),
        }
    }

    #[inline]
    pub fn exprs(&self) -> impl Iterator<Item = (ExprId, &Expr)> {
        match &self.expr_only {
            Some(it) => it.exprs.iter(),
            None => const { &Arena::new() }.iter(),
        }
    }

    #[inline]
    pub fn pats(&self) -> impl Iterator<Item = (PatId, &Pat)> {
        match &self.expr_only {
            Some(it) => it.pats.iter(),
            None => const { &Arena::new() }.iter(),
        }
    }

    #[inline]
    pub fn bindings(&self) -> impl Iterator<Item = (BindingId, &Binding)> {
        match &self.expr_only {
            Some(it) => it.bindings.iter(),
            None => const { &Arena::new() }.iter(),
        }
    }

    /// The coroutine associated with a coroutine closure.
    #[inline]
    pub fn coroutine_for_closure(coroutine_closure: ExprId) -> ExprId {
        // We keep the async closure exactly one expr before.
        ExprId::from_raw(la_arena::RawIdx::from_u32(coroutine_closure.into_raw().into_u32() - 1))
    }

    /// The opposite of [`Self::coroutine_for_closure()`].
    #[inline]
    pub fn closure_for_coroutine(coroutine: ExprId) -> ExprId {
        // We keep the async closure exactly one expr before.
        ExprId::from_raw(la_arena::RawIdx::from_u32(coroutine.into_raw().into_u32() + 1))
    }
}

trait ExprVisitor {
    fn on_expr(&mut self, expr: ExprId);
    fn on_pat(&mut self, pat: PatId);
    fn on_expr_opt(&mut self, expr: Option<ExprId>) {
        if let Some(expr) = expr {
            self.on_expr(expr);
        }
    }
    fn on_exprs(&mut self, exprs: impl IntoIterator<Item: Borrow<ExprId>>) {
        exprs.into_iter().for_each(|expr| self.on_expr(*expr.borrow()));
    }
    fn on_pats(&mut self, exprs: impl IntoIterator<Item: Borrow<PatId>>) {
        exprs.into_iter().for_each(|expr| self.on_pat(*expr.borrow()));
    }
}

impl Index<ExprId> for ExpressionStore {
    type Output = Expr;

    #[inline]
    fn index(&self, expr: ExprId) -> &Expr {
        &self.assert_expr_only().exprs[expr]
    }
}

impl Index<PatId> for ExpressionStore {
    type Output = Pat;

    #[inline]
    fn index(&self, pat: PatId) -> &Pat {
        &self.assert_expr_only().pats[pat]
    }
}

impl Index<LabelId> for ExpressionStore {
    type Output = Label;

    #[inline]
    fn index(&self, label: LabelId) -> &Label {
        &self.assert_expr_only().labels[label]
    }
}

impl Index<BindingId> for ExpressionStore {
    type Output = Binding;

    #[inline]
    fn index(&self, b: BindingId) -> &Binding {
        &self.assert_expr_only().bindings[b]
    }
}

impl Index<TypeRefId> for ExpressionStore {
    type Output = TypeRef;

    #[inline]
    fn index(&self, b: TypeRefId) -> &TypeRef {
        &self.types[b]
    }
}

impl Index<LifetimeRefId> for ExpressionStore {
    type Output = LifetimeRef;

    #[inline]
    fn index(&self, b: LifetimeRefId) -> &LifetimeRef {
        &self.lifetimes[b]
    }
}

impl Index<PathId> for ExpressionStore {
    type Output = Path;

    #[inline]
    fn index(&self, index: PathId) -> &Self::Output {
        let TypeRef::Path(path) = &self[index.type_ref()] else {
            unreachable!("`PathId` always points to `TypeRef::Path`");
        };
        path
    }
}

// FIXME: Change `node_` prefix to something more reasonable.
// Perhaps `expr_syntax` and `expr_id`?
impl ExpressionStoreSourceMap {
    pub fn expr_or_pat_syntax(&self, id: ExprOrPatId) -> Result<ExprOrPatSource, SyntheticSyntax> {
        match id {
            ExprOrPatId::ExprId(id) => self.expr_syntax(id),
            ExprOrPatId::PatId(id) => self.pat_syntax(id),
        }
    }

    #[inline]
    fn expr_or_synthetic(&self) -> Result<&ExpressionOnlySourceMap, SyntheticSyntax> {
        self.expr_only.as_deref().ok_or(SyntheticSyntax)
    }

    #[inline]
    fn expr_only(&self) -> Option<&ExpressionOnlySourceMap> {
        self.expr_only.as_deref()
    }

    #[inline]
    #[track_caller]
    fn assert_expr_only(&self) -> &ExpressionOnlySourceMap {
        self.expr_only.as_ref().expect("should have `ExpressionStoreSourceMap::expr_only`")
    }

    pub fn expr_syntax(&self, expr: ExprId) -> Result<ExprOrPatSource, SyntheticSyntax> {
        self.expr_or_synthetic()?.expr_map_back.get(expr).cloned().ok_or(SyntheticSyntax)
    }

    pub fn node_expr(&self, node: InFile<&ast::Expr>) -> Option<ExprOrPatId> {
        let src = node.map(AstPtr::new);
        self.expr_only()?.expr_map.get(&src).cloned()
    }

    pub fn node_macro_file(&self, node: InFile<&ast::MacroCall>) -> Option<MacroCallId> {
        let src = node.map(AstPtr::new);
        self.expr_only()?.expansions.get(&src).cloned()
    }

    pub fn macro_calls(&self) -> impl Iterator<Item = (InFile<MacroCallPtr>, MacroCallId)> + '_ {
        self.expr_only().into_iter().flat_map(|it| it.expansions.iter().map(|(&a, &b)| (a, b)))
    }

    pub fn pat_syntax(&self, pat: PatId) -> Result<ExprOrPatSource, SyntheticSyntax> {
        self.expr_or_synthetic()?.pat_map_back.get(pat).cloned().ok_or(SyntheticSyntax)
    }

    pub fn node_pat(&self, node: InFile<&ast::Pat>) -> Option<ExprOrPatId> {
        self.expr_only()?.pat_map.get(&node.map(AstPtr::new)).cloned()
    }

    pub fn type_syntax(&self, id: TypeRefId) -> Result<TypeSource, SyntheticSyntax> {
        self.types_map_back.get(id).cloned().ok_or(SyntheticSyntax)
    }

    pub fn node_type(&self, node: InFile<&ast::Type>) -> Option<TypeRefId> {
        self.types_map.get(&node.map(AstPtr::new)).cloned()
    }

    pub fn label_syntax(&self, label: LabelId) -> LabelSource {
        self.assert_expr_only().label_map_back[label]
    }

    pub fn patterns_for_binding(&self, binding: BindingId) -> &[PatId] {
        self.assert_expr_only().binding_definitions.get(binding).map_or(&[], Deref::deref)
    }

    pub fn node_label(&self, node: InFile<&ast::Label>) -> Option<LabelId> {
        let src = node.map(AstPtr::new).map(AstPtr::wrap_left);
        self.expr_only()?.label_map.get(&src).cloned()
    }

    pub fn field_syntax(&self, expr: ExprId) -> FieldSource {
        self.assert_expr_only().field_map_back[&expr]
    }

    pub fn pat_field_syntax(&self, pat: PatId) -> PatFieldSource {
        self.assert_expr_only().pat_field_map_back[&pat]
    }

    pub fn macro_expansion_expr(&self, node: InFile<&ast::MacroExpr>) -> Option<ExprOrPatId> {
        let src = node.map(AstPtr::new).map(AstPtr::upcast::<ast::MacroExpr>).map(AstPtr::upcast);
        self.expr_only()?.expr_map.get(&src).copied()
    }

    pub fn expansions(&self) -> impl Iterator<Item = (&InFile<MacroCallPtr>, &MacroCallId)> {
        self.expr_only().into_iter().flat_map(|it| it.expansions.iter())
    }

    pub fn expansion(&self, node: InFile<&ast::MacroCall>) -> Option<MacroCallId> {
        self.expr_only()?.expansions.get(&node.map(AstPtr::new)).copied()
    }

    pub fn implicit_format_args(
        &self,
        node: InFile<&ast::FormatArgsExpr>,
    ) -> Option<(HygieneId, &[(syntax::TextRange, Name)])> {
        let expr_only = self.expr_only()?;
        let src = node.map(AstPtr::new).map(AstPtr::upcast::<ast::Expr>);
        let (hygiene, names) = expr_only
            .template_map
            .as_ref()?
            .format_args_to_captures
            .get(&expr_only.expr_map.get(&src)?.as_expr()?)?;
        Some((*hygiene, &**names))
    }

    pub fn format_args_implicit_capture(
        &self,
        capture_expr: ExprId,
    ) -> Option<InFile<(ExprPtr, TextRange)>> {
        self.expr_only()?
            .template_map
            .as_ref()?
            .implicit_capture_to_source
            .get(&capture_expr)
            .copied()
    }

    pub fn asm_template_args(
        &self,
        node: InFile<&ast::AsmExpr>,
    ) -> Option<(ExprId, &[Vec<(syntax::TextRange, usize)>])> {
        let expr_only = self.expr_only()?;
        let src = node.map(AstPtr::new).map(AstPtr::upcast::<ast::Expr>);
        let expr = expr_only.expr_map.get(&src)?.as_expr()?;
        Some(expr).zip(
            expr_only.template_map.as_ref()?.asm_to_captures.get(&expr).map(std::ops::Deref::deref),
        )
    }

    /// Get a reference to the source map's diagnostics.
    pub fn diagnostics(&self) -> &[ExpressionStoreDiagnostics] {
        self.expr_only().map(|it| &*it.diagnostics).unwrap_or_default()
    }
}
