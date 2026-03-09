//! Re-export diagnostics such that clients of `hir` don't have to depend on
//! low-level crates.
//!
//! This probably isn't the best way to do this -- ideally, diagnostics should
//! be expressed in terms of hir types themselves.
use cfg::{CfgExpr, CfgOptions};
use either::Either;
use hir_def::{
    DefWithBodyId, GenericParamId, SyntheticSyntax,
    expr_store::{
        ExprOrPatPtr, ExpressionStoreSourceMap, hir_assoc_type_binding_to_ast,
        hir_generic_arg_to_ast, hir_segment_to_ast_segment,
    },
    hir::ExprOrPatId,
};
use hir_expand::{HirFileId, InFile, mod_path::ModPath, name::Name};
use hir_ty::{
    CastError, InferenceDiagnostic, InferenceTyDiagnosticSource, PathGenericsSource,
    PathLoweringDiagnostic, TyLoweringDiagnostic, TyLoweringDiagnosticKind,
    db::HirDatabase,
    diagnostics::{BodyValidationDiagnostic, UnsafetyReason},
};
use syntax::{
    AstNode, AstPtr, SyntaxError, SyntaxNodePtr, TextRange,
    ast::{self, HasGenericArgs},
    match_ast,
};
use triomphe::Arc;

use crate::{AssocItem, Field, Function, GenericDef, Local, Trait, Type};

pub use hir_def::VariantId;
pub use hir_ty::{
    GenericArgsProhibitedReason, IncorrectGenericsLenKind,
    diagnostics::{CaseType, IncorrectCase},
};

/// The macro takes two lists (separated by ;):
/// 1. Structs defined inline. For brevity, `pub` and `struct` can be omitted -- for example, the
///    following struct:
///    ```ignore
///    pub struct BreakOutsideOfLoop {
///        pub expr: InFile<AstPtr<ast::Expr>>,
///        pub is_break: bool,
///        pub bad_value_break: bool,
///    }
///    ```
///    can be written as follows:
///    ```ignore
///    BreakOutsideOfLoop {
///        expr: InFile<AstPtr<ast::Expr>>,
///        is_break: bool,
///        bad_value_break: bool,
///    }
///    ```
/// 2. Names of structs imported from outside.
///
/// It creates:
/// - for the inline structs, their definitions
/// - the `AnyDiagnostic` enum, with variants derived from both lists of structs
/// - `impl From<$Diagnostic> for AnyDiagnostic` for each of the variants
macro_rules! diagnostics {
    ($AnyDiagnostic:ident <$db:lifetime> ->
        $(
            $(#[$attr:meta])*
            $diag:ident $(<$lt:lifetime>)? {$(
                    $(#[$field_attr:meta])* $field:ident: $field_ty:ty,
            )*}
        )*

        ;

        $(
            $imp_diag:ident $(<$imp_lt:lifetime>)?,
        )*
    ) => {
        $(
            $(#[$attr])*
            pub struct $diag $(<$lt>)? {$(
                $(#[$field_attr])* pub $field: $field_ty,
            )*}
        )*

        #[derive(Debug)]
        pub enum $AnyDiagnostic<$db> {
            $(
                $diag(Box<$diag $(<$lt>)?>),
            )*
            $(
                $imp_diag(Box<$imp_diag $(<$imp_lt>)?>),
            )*
        }

        $(
            impl<$db> From<$diag $(<$lt>)?> for $AnyDiagnostic<$db> {
                fn from(d: $diag $(<$lt>)?) -> $AnyDiagnostic<$db> {
                    $AnyDiagnostic::$diag(Box::new(d))
                }
            }
        )*
        $(
            impl<$db> From<$imp_diag $(<$imp_lt>)?> for $AnyDiagnostic<$db> {
                fn from(d: $imp_diag $(<$imp_lt>)?) -> $AnyDiagnostic<$db> {
                    $AnyDiagnostic::$imp_diag(Box::new(d))
                }
            }
        )*
    };
}

diagnostics![AnyDiagnostic<'db> ->
    #[derive(Debug)]
    AwaitOutsideOfAsync {
        node: InFile<AstPtr<ast::AwaitExpr>>,
        location: String,
    }

    #[derive(Debug)]
    BadRtn {
        rtn: InFile<AstPtr<ast::ReturnTypeSyntax>>,
    }

    #[derive(Debug)]
    BreakOutsideOfLoop {
        expr: InFile<ExprOrPatPtr>,
        is_break: bool,
        bad_value_break: bool,
    }

    #[derive(Debug)]
    CastToUnsized<'db> {
        expr: InFile<ExprOrPatPtr>,
        cast_ty: Type<'db>,
    }

    #[derive(Debug)]
    ElidedLifetimesInPath {
        /// Points at the name if there are no generics.
        generics_or_segment: InFile<AstPtr<Either<ast::GenericArgList, ast::NameRef>>>,
        expected: u32,
        def: GenericDef,
        hard_error: bool,
    }

    #[derive(Debug)]
    ExpectedFunction<'db> {
        call: InFile<ExprOrPatPtr>,
        found: Type<'db>,
    }

    #[derive(Debug)]
    GenericArgsProhibited {
        args: InFile<AstPtr<Either<ast::GenericArgList, ast::ParenthesizedArgList>>>,
        reason: GenericArgsProhibitedReason,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    InactiveCode {
        node: InFile<SyntaxNodePtr>,
        cfg: CfgExpr,
        opts: CfgOptions,
    }

    #[derive(Debug, PartialEq, Eq)]
    IncoherentImpl {
        file_id: HirFileId,
        impl_: AstPtr<ast::Impl>,
    }

    #[derive(Debug)]
    IncorrectGenericsLen {
        /// Points at the name if there are no generics.
        generics_or_segment: InFile<AstPtr<Either<ast::GenericArgList, ast::NameRef>>>,
        kind: IncorrectGenericsLenKind,
        provided: u32,
        expected: u32,
        def: GenericDef,
    }

    #[derive(Debug)]
    IncorrectGenericsOrder {
        provided_arg: InFile<AstPtr<ast::GenericArg>>,
        expected_kind: GenericArgKind,
    }

    #[derive(Debug)]
    InvalidCast<'db> {
        expr: InFile<ExprOrPatPtr>,
        error: CastError,
        expr_ty: Type<'db>,
        cast_ty: Type<'db>,
    }

    #[derive(Debug)]
    InvalidDeriveTarget {
        range: InFile<TextRange>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    MacroDefError {
        node: InFile<AstPtr<ast::Macro>>,
        message: String,
        name: Option<TextRange>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    MacroError {
        range: InFile<TextRange>,
        message: String,
        error: bool,
        kind: &'static str,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    MacroExpansionParseError {
        range: InFile<TextRange>,
        errors: Arc<[SyntaxError]>,
    }

    #[derive(Debug)]
    MalformedDerive {
        range: InFile<TextRange>,
    }

    #[derive(Debug)]
    MismatchedArgCount {
        call_expr: InFile<ExprOrPatPtr>,
        expected: usize,
        found: usize,
    }

    #[derive(Debug)]
    MismatchedTupleStructPatArgCount {
        expr_or_pat: InFile<ExprOrPatPtr>,
        expected: usize,
        found: usize,
    }

    #[derive(Debug)]
    MissingFields {
        file: HirFileId,
        field_list_parent: AstPtr<Either<ast::RecordExpr, ast::RecordPat>>,
        field_list_parent_path: Option<AstPtr<ast::Path>>,
        missed_fields: Vec<Name>,
    }

    #[derive(Debug)]
    MissingLifetime {
        /// Points at the name if there are no generics.
        generics_or_segment: InFile<AstPtr<Either<ast::GenericArgList, ast::NameRef>>>,
        expected: u32,
        def: GenericDef,
    }

    #[derive(Debug)]
    MissingMatchArms {
        scrutinee_expr: InFile<AstPtr<ast::Expr>>,
        uncovered_patterns: String,
    }

    #[derive(Debug)]
    MissingUnsafe {
        node: InFile<ExprOrPatPtr>,
        lint: UnsafeLint,
        reason: UnsafetyReason,
    }

    #[derive(Debug)]
    MovedOutOfRef<'db> {
        ty: Type<'db>,
        span: InFile<SyntaxNodePtr>,
    }

    #[derive(Debug)]
    NeedMut {
        local: Local,
        span: InFile<SyntaxNodePtr>,
    }

    #[derive(Debug)]
    NonExhaustiveLet {
        pat: InFile<AstPtr<ast::Pat>>,
        uncovered_patterns: String,
    }

    #[derive(Debug)]
    NoSuchField {
        field: InFile<AstPtr<Either<ast::RecordExprField, ast::RecordPatField>>>,
        private: Option<Field>,
        variant: VariantId,
    }

    #[derive(Debug)]
    PrivateAssocItem {
        expr_or_pat: InFile<ExprOrPatPtr>,
        item: AssocItem,
    }

    #[derive(Debug)]
    PrivateField {
        expr: InFile<ExprOrPatPtr>,
        field: Field,
    }

    #[derive(Debug)]
    RemoveTrailingReturn {
        return_expr: InFile<AstPtr<ast::ReturnExpr>>,
    }

    #[derive(Debug)]
    RemoveUnnecessaryElse {
        if_expr: InFile<AstPtr<ast::IfExpr>>,
    }

    #[derive(Debug)]
    ReplaceFilterMapNextWithFindMap {
        file: HirFileId,
        /// This expression is the whole method chain up to and including `.filter_map(..).next()`.
        next_expr: AstPtr<ast::Expr>,
    }
    // FIXME: Split this off into the corresponding 4 rustc errors

    #[derive(Debug, PartialEq, Eq)]
    TraitImplIncorrectSafety {
        file_id: HirFileId,
        impl_: AstPtr<ast::Impl>,
        should_be_safe: bool,
    }

    #[derive(Debug, PartialEq, Eq)]
    TraitImplMissingAssocItems {
        file_id: HirFileId,
        impl_: AstPtr<ast::Impl>,
        missing: Vec<(Name, AssocItem)>,
    }

    #[derive(Debug, PartialEq, Eq)]
    TraitImplOrphan {
        file_id: HirFileId,
        impl_: AstPtr<ast::Impl>,
    }

    #[derive(Debug, PartialEq, Eq)]
    TraitImplRedundantAssocItems {
        file_id: HirFileId,
        trait_: Trait,
        impl_: AstPtr<ast::Impl>,
        assoc_item: (Name, AssocItem),
    }

    #[derive(Debug)]
    TypedHole<'db> {
        expr: InFile<ExprOrPatPtr>,
        expected: Type<'db>,
    }

    #[derive(Debug)]
    TypeMismatch<'db> {
        expr_or_pat: InFile<ExprOrPatPtr>,
        expected: Type<'db>,
        actual: Type<'db>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    UndeclaredLabel {
        node: InFile<AstPtr<ast::Lifetime>>,
        name: Name,
    }

    #[derive(Debug)]
    UnimplementedBuiltinMacro {
        node: InFile<SyntaxNodePtr>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    UnreachableLabel {
        node: InFile<AstPtr<ast::Lifetime>>,
        name: Name,
    }

    #[derive(Debug)]
    UnresolvedAssocItem {
        expr_or_pat: InFile<ExprOrPatPtr>,
    }

    #[derive(Debug)]
    UnresolvedExternCrate {
        decl: InFile<AstPtr<ast::ExternCrate>>,
    }

    #[derive(Debug)]
    UnresolvedField<'db> {
        expr: InFile<ExprOrPatPtr>,
        receiver: Type<'db>,
        name: Name,
        method_with_same_name_exists: bool,
    }

    #[derive(Debug)]
    UnresolvedImport {
        decl: InFile<AstPtr<ast::UseTree>>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    UnresolvedMacroCall {
        range: InFile<TextRange>,
        path: ModPath,
        is_bang: bool,
    }

    #[derive(Debug)]
    UnresolvedMethodCall<'db> {
        expr: InFile<ExprOrPatPtr>,
        receiver: Type<'db>,
        name: Name,
        field_with_same_name: Option<Type<'db>>,
        assoc_func_with_same_name: Option<Function>,
    }

    #[derive(Debug)]
    UnresolvedModule {
        decl: InFile<AstPtr<ast::Module>>,
        candidates: Box<[String]>,
    }

    #[derive(Debug)]
    UnresolvedIdent {
        node: InFile<(ExprOrPatPtr, Option<TextRange>)>,
    }

    #[derive(Debug)]
    UnusedMut {
        local: Local,
    }

    #[derive(Debug)]
    UnusedVariable {
        local: Local,
    }

    #[derive(Debug)]
    ParenthesizedGenericArgsWithoutFnTrait {
        args: InFile<AstPtr<ast::ParenthesizedArgList>>,
    }

    ;

    // IncorrectCase,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsafeLint {
    HardError,
    UnsafeOpInUnsafeFn,
    DeprecatedSafe2024,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenericArgKind {
    Lifetime,
    Type,
    Const,
}

impl GenericArgKind {
    fn from_id(id: GenericParamId) -> Self {
        match id {
            GenericParamId::TypeParamId(_) => GenericArgKind::Type,
            GenericParamId::ConstParamId(_) => GenericArgKind::Const,
            GenericParamId::LifetimeParamId(_) => GenericArgKind::Lifetime,
        }
    }
}
impl<'db> AnyDiagnostic<'db> {
    pub(crate) fn body_validation_diagnostic(
        db: &'db dyn HirDatabase,
        diagnostic: BodyValidationDiagnostic,
        source_map: &hir_def::expr_store::BodySourceMap,
    ) -> Option<AnyDiagnostic<'db>> {
        match diagnostic {
            BodyValidationDiagnostic::RecordMissingFields { record, variant, missed_fields } => {
                let variant_data = variant.fields(db);
                let missed_fields = missed_fields
                    .into_iter()
                    .map(|idx| variant_data.fields()[idx].name.clone())
                    .collect();

                let record = match record {
                    Either::Left(record_expr) => source_map.expr_syntax(record_expr).ok()?,
                    Either::Right(record_pat) => source_map.pat_syntax(record_pat).ok()?,
                };
                let file = record.file_id;
                let root = record.file_syntax(db);
                match record.value.to_node(&root) {
                    Either::Left(ast::Expr::RecordExpr(record_expr)) => {
                        if record_expr.record_expr_field_list().is_some() {
                            let field_list_parent_path =
                                record_expr.path().map(|path| AstPtr::new(&path));
                            return Some(
                                MissingFields {
                                    file,
                                    field_list_parent: AstPtr::new(&Either::Left(record_expr)),
                                    field_list_parent_path,
                                    missed_fields,
                                }
                                .into(),
                            );
                        }
                    }
                    Either::Right(ast::Pat::RecordPat(record_pat)) => {
                        if record_pat.record_pat_field_list().is_some() {
                            let field_list_parent_path =
                                record_pat.path().map(|path| AstPtr::new(&path));
                            return Some(
                                MissingFields {
                                    file,
                                    field_list_parent: AstPtr::new(&Either::Right(record_pat)),
                                    field_list_parent_path,
                                    missed_fields,
                                }
                                .into(),
                            );
                        }
                    }
                    _ => {}
                }
            }
            BodyValidationDiagnostic::ReplaceFilterMapNextWithFindMap { method_call_expr } => {
                if let Ok(next_source_ptr) = source_map.expr_syntax(method_call_expr) {
                    return Some(
                        ReplaceFilterMapNextWithFindMap {
                            file: next_source_ptr.file_id,
                            next_expr: next_source_ptr.value.cast()?,
                        }
                        .into(),
                    );
                }
            }
            BodyValidationDiagnostic::MissingMatchArms { match_expr, uncovered_patterns } => {
                match source_map.expr_syntax(match_expr) {
                    Ok(source_ptr) => {
                        let root = source_ptr.file_syntax(db);
                        if let Either::Left(ast::Expr::MatchExpr(match_expr)) =
                            &source_ptr.value.to_node(&root)
                        {
                            match match_expr.expr() {
                                Some(scrut_expr) if match_expr.match_arm_list().is_some() => {
                                    return Some(
                                        MissingMatchArms {
                                            scrutinee_expr: InFile::new(
                                                source_ptr.file_id,
                                                AstPtr::new(&scrut_expr),
                                            ),
                                            uncovered_patterns,
                                        }
                                        .into(),
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(SyntheticSyntax) => (),
                }
            }
            BodyValidationDiagnostic::NonExhaustiveLet { pat, uncovered_patterns } => {
                match source_map.pat_syntax(pat) {
                    Ok(source_ptr) => {
                        if let Some(ast_pat) = source_ptr.value.cast::<ast::Pat>() {
                            return Some(
                                NonExhaustiveLet {
                                    pat: InFile::new(source_ptr.file_id, ast_pat),
                                    uncovered_patterns,
                                }
                                .into(),
                            );
                        }
                    }
                    Err(SyntheticSyntax) => {}
                }
            }
            BodyValidationDiagnostic::RemoveTrailingReturn { return_expr } => {
                if let Ok(source_ptr) = source_map.expr_syntax(return_expr) {
                    // Filters out desugared return expressions (e.g. desugared try operators).
                    if let Some(ptr) = source_ptr.value.cast::<ast::ReturnExpr>() {
                        return Some(
                            RemoveTrailingReturn {
                                return_expr: InFile::new(source_ptr.file_id, ptr),
                            }
                            .into(),
                        );
                    }
                }
            }
            BodyValidationDiagnostic::RemoveUnnecessaryElse { if_expr } => {
                if let Ok(source_ptr) = source_map.expr_syntax(if_expr)
                    && let Some(ptr) = source_ptr.value.cast::<ast::IfExpr>()
                {
                    return Some(
                        RemoveUnnecessaryElse { if_expr: InFile::new(source_ptr.file_id, ptr) }
                            .into(),
                    );
                }
            }
        }
        None
    }

    pub(crate) fn inference_diagnostic(
        db: &'db dyn HirDatabase,
        def: DefWithBodyId,
        d: &InferenceDiagnostic,
        source_map: &hir_def::expr_store::BodySourceMap,
        sig_map: &hir_def::expr_store::ExpressionStoreSourceMap,
    ) -> Option<AnyDiagnostic<'db>> {
        let expr_syntax = |expr| {
            source_map
                .expr_syntax(expr)
                .inspect_err(|_| stdx::never!("inference diagnostic in desugared expr"))
                .ok()
        };
        let pat_syntax = |pat| {
            source_map
                .pat_syntax(pat)
                .inspect_err(|_| stdx::never!("inference diagnostic in desugared pattern"))
                .ok()
        };
        let expr_or_pat_syntax = |id| match id {
            ExprOrPatId::ExprId(expr) => expr_syntax(expr),
            ExprOrPatId::PatId(pat) => pat_syntax(pat),
        };
        Some(match d {
            &InferenceDiagnostic::NoSuchField { field: expr, private, variant } => {
                let expr_or_pat = match expr {
                    ExprOrPatId::ExprId(expr) => {
                        source_map.field_syntax(expr).map(AstPtr::wrap_left)
                    }
                    ExprOrPatId::PatId(pat) => source_map.pat_field_syntax(pat),
                };
                let private = private.map(|id| Field { id, parent: variant.into() });
                NoSuchField { field: expr_or_pat, private, variant }.into()
            }
            &InferenceDiagnostic::MismatchedArgCount { call_expr, expected, found } => {
                MismatchedArgCount { call_expr: expr_syntax(call_expr)?, expected, found }.into()
            }
            &InferenceDiagnostic::PrivateField { expr, field } => {
                let expr = expr_syntax(expr)?;
                let field = field.into();
                PrivateField { expr, field }.into()
            }
            &InferenceDiagnostic::PrivateAssocItem { id, item } => {
                let expr_or_pat = expr_or_pat_syntax(id)?;
                let item = item.into();
                PrivateAssocItem { expr_or_pat, item }.into()
            }
            InferenceDiagnostic::ExpectedFunction { call_expr, found } => {
                let call_expr = expr_syntax(*call_expr)?;
                ExpectedFunction { call: call_expr, found: Type::new(db, def, found.as_ref()) }
                    .into()
            }
            InferenceDiagnostic::UnresolvedField {
                expr,
                receiver,
                name,
                method_with_same_name_exists,
            } => {
                let expr = expr_syntax(*expr)?;
                UnresolvedField {
                    expr,
                    name: name.clone(),
                    receiver: Type::new(db, def, receiver.as_ref()),
                    method_with_same_name_exists: *method_with_same_name_exists,
                }
                .into()
            }
            InferenceDiagnostic::UnresolvedMethodCall {
                expr,
                receiver,
                name,
                field_with_same_name,
                assoc_func_with_same_name,
            } => {
                let expr = expr_syntax(*expr)?;
                UnresolvedMethodCall {
                    expr,
                    name: name.clone(),
                    receiver: Type::new(db, def, receiver.as_ref()),
                    field_with_same_name: field_with_same_name
                        .as_ref()
                        .map(|ty| Type::new(db, def, ty.as_ref())),
                    assoc_func_with_same_name: assoc_func_with_same_name.map(Into::into),
                }
                .into()
            }
            &InferenceDiagnostic::UnresolvedAssocItem { id } => {
                let expr_or_pat = expr_or_pat_syntax(id)?;
                UnresolvedAssocItem { expr_or_pat }.into()
            }
            &InferenceDiagnostic::UnresolvedIdent { id } => {
                let node = match id {
                    ExprOrPatId::ExprId(id) => match source_map.expr_syntax(id) {
                        Ok(syntax) => syntax.map(|it| (it, None)),
                        Err(SyntheticSyntax) => source_map
                            .format_args_implicit_capture(id)?
                            .map(|(node, range)| (node.wrap_left(), Some(range))),
                    },
                    ExprOrPatId::PatId(id) => pat_syntax(id)?.map(|it| (it, None)),
                };
                UnresolvedIdent { node }.into()
            }
            &InferenceDiagnostic::BreakOutsideOfLoop { expr, is_break, bad_value_break } => {
                let expr = expr_syntax(expr)?;
                BreakOutsideOfLoop { expr, is_break, bad_value_break }.into()
            }
            InferenceDiagnostic::TypedHole { expr, expected } => {
                let expr = expr_syntax(*expr)?;
                TypedHole { expr, expected: Type::new(db, def, expected.as_ref()) }.into()
            }
            &InferenceDiagnostic::MismatchedTupleStructPatArgCount { pat, expected, found } => {
                let expr_or_pat = match pat {
                    ExprOrPatId::ExprId(expr) => expr_syntax(expr)?,
                    ExprOrPatId::PatId(pat) => {
                        let InFile { file_id, value } = pat_syntax(pat)?;

                        // cast from Either<Pat, SelfParam> -> Either<_, Pat>
                        let ptr = AstPtr::try_from_raw(value.syntax_node_ptr())?;
                        InFile { file_id, value: ptr }
                    }
                };
                MismatchedTupleStructPatArgCount { expr_or_pat, expected, found }.into()
            }
            InferenceDiagnostic::CastToUnsized { expr, cast_ty } => {
                let expr = expr_syntax(*expr)?;
                CastToUnsized { expr, cast_ty: Type::new(db, def, cast_ty.as_ref()) }.into()
            }
            InferenceDiagnostic::InvalidCast { expr, error, expr_ty, cast_ty } => {
                let expr = expr_syntax(*expr)?;
                let expr_ty = Type::new(db, def, expr_ty.as_ref());
                let cast_ty = Type::new(db, def, cast_ty.as_ref());
                InvalidCast { expr, error: *error, expr_ty, cast_ty }.into()
            }
            InferenceDiagnostic::TyDiagnostic { source, diag } => {
                let source_map = match source {
                    InferenceTyDiagnosticSource::Body => source_map,
                    InferenceTyDiagnosticSource::Signature => sig_map,
                };
                Self::ty_diagnostic(diag, source_map, db)?
            }
            InferenceDiagnostic::PathDiagnostic { node, diag } => {
                let source = expr_or_pat_syntax(*node)?;
                let syntax = source.value.to_node(&db.parse_or_expand(source.file_id));
                let path = match_ast! {
                    match (syntax.syntax()) {
                        ast::RecordExpr(it) => it.path()?,
                        ast::RecordPat(it) => it.path()?,
                        ast::TupleStructPat(it) => it.path()?,
                        ast::PathExpr(it) => it.path()?,
                        ast::PathPat(it) => it.path()?,
                        _ => return None,
                    }
                };
                Self::path_diagnostic(diag, source.with_value(path))?
            }
            &InferenceDiagnostic::MethodCallIncorrectGenericsLen {
                expr,
                provided_count,
                expected_count,
                kind,
                def,
            } => {
                let syntax = expr_syntax(expr)?;
                let file_id = syntax.file_id;
                let syntax =
                    syntax.with_value(syntax.value.cast::<ast::MethodCallExpr>()?).to_node(db);
                let generics_or_name = syntax
                    .generic_arg_list()
                    .map(Either::Left)
                    .or_else(|| syntax.name_ref().map(Either::Right))?;
                let generics_or_name = InFile::new(file_id, AstPtr::new(&generics_or_name));
                IncorrectGenericsLen {
                    generics_or_segment: generics_or_name,
                    kind,
                    provided: provided_count,
                    expected: expected_count,
                    def: def.into(),
                }
                .into()
            }
            &InferenceDiagnostic::MethodCallIncorrectGenericsOrder {
                expr,
                param_id,
                arg_idx,
                has_self_arg,
            } => {
                let syntax = expr_syntax(expr)?;
                let file_id = syntax.file_id;
                let syntax =
                    syntax.with_value(syntax.value.cast::<ast::MethodCallExpr>()?).to_node(db);
                let generic_args = syntax.generic_arg_list()?;
                let provided_arg = hir_generic_arg_to_ast(&generic_args, arg_idx, has_self_arg)?;
                let provided_arg = InFile::new(file_id, AstPtr::new(&provided_arg));
                let expected_kind = GenericArgKind::from_id(param_id);
                IncorrectGenericsOrder { provided_arg, expected_kind }.into()
            }
        })
    }

    fn path_diagnostic(
        diag: &PathLoweringDiagnostic,
        path: InFile<ast::Path>,
    ) -> Option<AnyDiagnostic<'db>> {
        Some(match *diag {
            PathLoweringDiagnostic::GenericArgsProhibited { segment, reason } => {
                let segment = hir_segment_to_ast_segment(&path.value, segment)?;

                if let Some(rtn) = segment.return_type_syntax() {
                    // RTN errors are emitted as `GenericArgsProhibited` or `ParenthesizedGenericArgsWithoutFnTrait`.
                    return Some(BadRtn { rtn: path.with_value(AstPtr::new(&rtn)) }.into());
                }

                let args = if let Some(generics) = segment.generic_arg_list() {
                    AstPtr::new(&generics).wrap_left()
                } else {
                    AstPtr::new(&segment.parenthesized_arg_list()?).wrap_right()
                };
                let args = path.with_value(args);
                GenericArgsProhibited { args, reason }.into()
            }
            PathLoweringDiagnostic::ParenthesizedGenericArgsWithoutFnTrait { segment } => {
                let segment = hir_segment_to_ast_segment(&path.value, segment)?;

                if let Some(rtn) = segment.return_type_syntax() {
                    // RTN errors are emitted as `GenericArgsProhibited` or `ParenthesizedGenericArgsWithoutFnTrait`.
                    return Some(BadRtn { rtn: path.with_value(AstPtr::new(&rtn)) }.into());
                }

                let args = AstPtr::new(&segment.parenthesized_arg_list()?);
                let args = path.with_value(args);
                ParenthesizedGenericArgsWithoutFnTrait { args }.into()
            }
            PathLoweringDiagnostic::IncorrectGenericsLen {
                generics_source,
                provided_count,
                expected_count,
                kind,
                def,
            } => {
                let generics_or_segment =
                    path_generics_source_to_ast(&path.value, generics_source)?;
                let generics_or_segment = path.with_value(AstPtr::new(&generics_or_segment));
                IncorrectGenericsLen {
                    generics_or_segment,
                    kind,
                    provided: provided_count,
                    expected: expected_count,
                    def: def.into(),
                }
                .into()
            }
            PathLoweringDiagnostic::IncorrectGenericsOrder {
                generics_source,
                param_id,
                arg_idx,
                has_self_arg,
            } => {
                let generic_args =
                    path_generics_source_to_ast(&path.value, generics_source)?.left()?;
                let provided_arg = hir_generic_arg_to_ast(&generic_args, arg_idx, has_self_arg)?;
                let provided_arg = path.with_value(AstPtr::new(&provided_arg));
                let expected_kind = GenericArgKind::from_id(param_id);
                IncorrectGenericsOrder { provided_arg, expected_kind }.into()
            }
            PathLoweringDiagnostic::MissingLifetime { generics_source, expected_count, def }
            | PathLoweringDiagnostic::ElisionFailure { generics_source, expected_count, def } => {
                let generics_or_segment =
                    path_generics_source_to_ast(&path.value, generics_source)?;
                let generics_or_segment = path.with_value(AstPtr::new(&generics_or_segment));
                MissingLifetime { generics_or_segment, expected: expected_count, def: def.into() }
                    .into()
            }
            PathLoweringDiagnostic::ElidedLifetimesInPath {
                generics_source,
                expected_count,
                def,
                hard_error,
            } => {
                let generics_or_segment =
                    path_generics_source_to_ast(&path.value, generics_source)?;
                let generics_or_segment = path.with_value(AstPtr::new(&generics_or_segment));
                ElidedLifetimesInPath {
                    generics_or_segment,
                    expected: expected_count,
                    def: def.into(),
                    hard_error,
                }
                .into()
            }
        })
    }

    pub(crate) fn ty_diagnostic(
        diag: &TyLoweringDiagnostic,
        source_map: &ExpressionStoreSourceMap,
        db: &'db dyn HirDatabase,
    ) -> Option<AnyDiagnostic<'db>> {
        let Ok(source) = source_map.type_syntax(diag.source) else {
            stdx::never!("error on synthetic type syntax");
            return None;
        };
        let syntax = || source.value.to_node(&db.parse_or_expand(source.file_id));
        Some(match &diag.kind {
            TyLoweringDiagnosticKind::PathDiagnostic(diag) => {
                let ast::Type::PathType(syntax) = syntax() else { return None };
                Self::path_diagnostic(diag, source.with_value(syntax.path()?))?
            }
        })
    }
}

fn path_generics_source_to_ast(
    path: &ast::Path,
    generics_source: PathGenericsSource,
) -> Option<Either<ast::GenericArgList, ast::NameRef>> {
    Some(match generics_source {
        PathGenericsSource::Segment(segment) => {
            let segment = hir_segment_to_ast_segment(path, segment)?;
            segment
                .generic_arg_list()
                .map(Either::Left)
                .or_else(|| segment.name_ref().map(Either::Right))?
        }
        PathGenericsSource::AssocType { segment, assoc_type } => {
            let segment = hir_segment_to_ast_segment(path, segment)?;
            let segment_args = segment.generic_arg_list()?;
            let assoc = hir_assoc_type_binding_to_ast(&segment_args, assoc_type)?;
            assoc
                .generic_arg_list()
                .map(Either::Left)
                .or_else(|| assoc.name_ref().map(Either::Right))?
        }
    })
}
