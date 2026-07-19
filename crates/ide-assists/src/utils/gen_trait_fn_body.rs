//! This module contains functions to generate default trait impl function bodies where possible.

use hir::TraitRef;
use syntax::ast::{
    self, AstNode, BinaryOp, CmpOp, HasName, LogicOp, edit::AstNodeEdit,
    syntax_factory::SyntaxFactory,
};
use syntax::syntax_editor::{Position, SyntaxEditor};

use crate::utils::{cfg_attrs, insert_attributes};

/// Generate custom trait bodies without default implementation where possible.
///
/// If `func` is defined within an existing impl block, pass [`TraitRef`]. Otherwise pass `None`.
///
/// Returns `Option` so that we can use `?` rather than `if let Some`. Returning
/// `None` means that generating a custom trait body failed, and the body will remain
/// as `todo!` instead.
pub(crate) fn gen_trait_fn_body(
    make: &SyntaxFactory,
    func: &ast::Fn,
    trait_path: &ast::Path,
    adt: &ast::Adt,
    trait_ref: Option<TraitRef<'_>>,
) -> Option<ast::BlockExpr> {
    let _ = func.body()?;
    match trait_path.segment()?.name_ref()?.text().as_str() {
        "Clone" => {
            stdx::always!(func.name().is_some_and(|name| name.text() == "clone"));
            gen_clone_impl(make, adt)
        }
        "Debug" => gen_debug_impl(make, adt),
        "Default" => gen_default_impl(make, adt),
        "Hash" => {
            stdx::always!(func.name().is_some_and(|name| name.text() == "hash"));
            gen_hash_impl(make, adt)
        }
        "PartialEq" => {
            stdx::always!(func.name().is_some_and(|name| name.text() == "eq"));
            gen_partial_eq(make, adt, trait_ref)
        }
        "PartialOrd" => {
            stdx::always!(func.name().is_some_and(|name| name.text() == "partial_cmp"));
            gen_partial_ord(make, adt, trait_ref)
        }
        _ => None,
    }
}

/// Generate a `Clone` impl based on the fields and members of the target type.
fn gen_clone_impl(make: &SyntaxFactory, adt: &ast::Adt) -> Option<ast::BlockExpr> {
    let gen_clone_call = |target: ast::Expr| -> ast::Expr {
        let method = make.name_ref("clone");
        make.expr_method_call(target, method, make.arg_list([])).into()
    };
    let expr = match adt {
        // `Clone` cannot be derived for unions, so no default impl can be provided.
        ast::Adt::Union(_) => return None,
        ast::Adt::Enum(enum_) => {
            let list = enum_.variant_list()?;
            let mut arms = vec![];
            for variant in list.variants() {
                let name = variant.name()?;
                let variant_name = make.path_from_idents(["Self", &format!("{name}")])?;

                match variant.field_list() {
                    // => match self { Self::Name { x } => Self::Name { x: x.clone() } }
                    Some(ast::FieldList::RecordFieldList(list)) => {
                        let mut pats = vec![];
                        let mut fields = vec![];
                        for field in list.fields() {
                            let field_name = field.name()?;
                            let pat = make.ident_pat(false, false, field_name.clone());
                            pats.push(make.record_pat_field_shorthand(pat.into()));

                            let path = make.ident_path(&field_name.to_string());
                            let method_call = gen_clone_call(make.expr_path(path));
                            let name_ref = make.name_ref(&field_name.to_string());
                            let field = make.record_expr_field(name_ref, Some(method_call));
                            fields.push(field);
                        }
                        let pat_field_list = make.record_pat_field_list(pats, None);
                        let pat = make.record_pat_with_fields(variant_name.clone(), pat_field_list);
                        let fields = make.record_expr_field_list(fields);
                        let record_expr = make.record_expr(variant_name, fields).into();
                        arms.push(make.match_arm(pat.into(), None, record_expr));
                    }

                    // => match self { Self::Name(arg1) => Self::Name(arg1.clone()) }
                    Some(ast::FieldList::TupleFieldList(list)) => {
                        let mut pats = vec![];
                        let mut fields = vec![];
                        for (i, _) in list.fields().enumerate() {
                            let field_name = format!("arg{i}");
                            let pat = make.ident_pat(false, false, make.name(&field_name));
                            pats.push(pat.into());

                            let f_path = make.expr_path(make.ident_path(&field_name));
                            fields.push(gen_clone_call(f_path));
                        }
                        let pat = make.tuple_struct_pat(variant_name.clone(), pats);
                        let struct_name = make.expr_path(variant_name);
                        let tuple_expr = make.expr_call(struct_name, make.arg_list(fields)).into();
                        arms.push(make.match_arm(pat.into(), None, tuple_expr));
                    }

                    // => match self { Self::Name => Self::Name }
                    None => {
                        let pattern = make.path_pat(variant_name.clone());
                        let variant_expr = make.expr_path(variant_name);
                        arms.push(make.match_arm(pattern, None, variant_expr));
                    }
                }
            }

            let match_target = make.expr_path(make.ident_path("self"));
            let list = make.match_arm_list(arms).indent(ast::edit::IndentLevel(1));
            make.expr_match(match_target, list).into()
        }
        ast::Adt::Struct(strukt) => {
            match strukt.field_list() {
                // => Self { name: self.name.clone() }
                Some(ast::FieldList::RecordFieldList(field_list)) => {
                    let mut fields = vec![];
                    for field in field_list.fields() {
                        let base = make.expr_path(make.ident_path("self"));
                        let target = make.expr_field(base, &field.name()?.to_string()).into();
                        let method_call = gen_clone_call(target);
                        let name_ref = make.name_ref(&field.name()?.to_string());
                        let field = make.record_expr_field(name_ref, Some(method_call));
                        fields.push(field);
                    }
                    let struct_name = make.ident_path("Self");
                    let fields = make.record_expr_field_list(fields);
                    make.record_expr(struct_name, fields).into()
                }
                // => Self(self.0.clone(), self.1.clone())
                Some(ast::FieldList::TupleFieldList(field_list)) => {
                    let mut fields = vec![];
                    for (i, _) in field_list.fields().enumerate() {
                        let f_path = make.expr_path(make.ident_path("self"));
                        let target = make.expr_field(f_path, &format!("{i}")).into();
                        fields.push(gen_clone_call(target));
                    }
                    let struct_name = make.expr_path(make.ident_path("Self"));
                    make.expr_call(struct_name, make.arg_list(fields)).into()
                }
                // => Self { }
                None => {
                    let struct_name = make.ident_path("Self");
                    let fields = make.record_expr_field_list([]);
                    make.record_expr(struct_name, fields).into()
                }
            }
        }
    };
    let body = make.block_expr(None, Some(expr)).indent(ast::edit::IndentLevel(1));
    Some(body)
}

/// Generate a `Debug` impl based on the fields and members of the target type.
fn gen_debug_impl(make: &SyntaxFactory, adt: &ast::Adt) -> Option<ast::BlockExpr> {
    let annotated_name = adt.name()?;
    match adt {
        // `Debug` cannot be derived for unions, so no default impl can be provided.
        ast::Adt::Union(_) => None,

        // => match self { Self::Variant => write!(f, "Variant") }
        ast::Adt::Enum(enum_) => {
            let list = enum_.variant_list()?;
            let mut arms = vec![];

            let mut arms_cfgs = vec![];
            let mut has_cfg = false;

            // build the arms used below
            for variant in list.variants() {
                let name = variant.name()?;
                let variant_name = make.path_from_idents(["Self", &format!("{name}")])?;
                let target = make.expr_path(make.ident_path("f"));

                let variant_cfg = cfg_attrs(&variant).collect::<Vec<_>>();
                if !variant_cfg.is_empty() {
                    has_cfg = true;
                }

                let arm = match variant.field_list() {
                    Some(ast::FieldList::RecordFieldList(list)) => {
                        let field_cfgs = list
                            .fields()
                            .map(|f| cfg_attrs(&f).collect::<Vec<_>>())
                            .collect::<Vec<_>>();
                        let field_has_cfg = field_cfgs.iter().any(|c| !c.is_empty());
                        if field_has_cfg {
                            has_cfg = true;
                        }

                        // => Self::V { a, b } => f.debug_struct("V").field("a", a)
                        let mut pats = vec![];
                        for field in list.fields() {
                            let field_name = field.name()?;

                            // create a field pattern for use in `MyStruct { a, b }`
                            let pat = make.ident_pat(false, false, field_name);
                            pats.push(make.record_pat_field_shorthand(pat.into()));
                        }

                        // => MyStruct { fields.. }
                        let pat_field_list = make.record_pat_field_list(pats, None);
                        let pat = make.record_pat_with_fields(variant_name.clone(), pat_field_list);

                        let body = if field_has_cfg {
                            gen_debug_enum_record_variant_body_with_cfg(
                                make,
                                &name,
                                &list,
                                &field_cfgs,
                            )?
                            .into()
                        } else {
                            gen_debug_enum_record_variant_body(make, &name, &list)?
                        };

                        make.match_arm(pat.into(), None, body)
                    }
                    Some(ast::FieldList::TupleFieldList(list)) => {
                        // => f.debug_tuple(name)
                        let target = make.expr_path(make.ident_path("f"));
                        let method = make.name_ref("debug_tuple");
                        let struct_name = format!("\"{name}\"");
                        let args = make.arg_list([make.expr_literal(&struct_name).into()]);
                        let mut expr = make.expr_method_call(target, method, args).into();

                        let mut pats = vec![];
                        for (i, _) in list.fields().enumerate() {
                            let name = format!("arg{i}");

                            // create a field pattern for use in `MyStruct(fields..)`
                            let field_name = make.name(&name);
                            let pat = make.ident_pat(false, false, field_name.clone());
                            pats.push(pat.into());

                            // => <expr>.field(field)
                            let method_name = make.name_ref("field");
                            let field_path = make.expr_path(make.ident_path(&name));
                            let args = make.arg_list([field_path]);
                            expr = make.expr_method_call(expr, method_name, args).into();
                        }

                        // => <expr>.finish()
                        let method = make.name_ref("finish");
                        let expr = make.expr_method_call(expr, method, make.arg_list([])).into();

                        // => MyStruct (a, b) => f.debug_tuple("MyStruct") ... .finish(),
                        let pat = make.tuple_struct_pat(variant_name.clone(), pats);
                        make.match_arm(pat.into(), None, expr)
                    }
                    None => {
                        let fmt_string = make.expr_literal(&(format!("\"{name}\""))).into();
                        let args =
                            make.token_tree_from_node(make.arg_list([target, fmt_string]).syntax());
                        let macro_name = make.ident_path("write");
                        let macro_call = make.expr_macro(macro_name, args);

                        let variant_name = make.path_pat(variant_name);
                        make.match_arm(variant_name, None, macro_call.into())
                    }
                };

                arms.push(arm);
            }

            // => match self { <arms> }
            let match_target = make.expr_path(make.ident_path("self"));
            let arms = make.match_arm_list(arms).indent(ast::edit::IndentLevel(1));
            let match_expr = make.expr_match(match_target, arms);

            let body = make.block_expr(None::<ast::Stmt>, Some(match_expr.into()));
            let body = if has_cfg {
                // => #[cfg(feature = "x")]
                //    Self::Baz { a, #[cfg(feature = "x")] b } => ...
                let (editor, body_clone) = SyntaxEditor::with_ast_node(&body);
                let match_arms = body_clone
                    .stmt_list()
                    .and_then(|stmts| stmts.tail_expr())
                    .and_then(|tail| match tail {
                        ast::Expr::MatchExpr(m) => m.match_arm_list(),
                        _ => None,
                    })
                    .map(|l| l.arms().collect::<Vec<_>>())
                    .unwrap_or_default();
                let variants = list.variants().collect::<Vec<_>>();

                stdx::always!(match_arms.len() == variants.len());
                for (arm, variant) in match_arms.iter().zip(&variants) {
                    // => #[cfg(...)]
                    let variant_cfg = cfg_attrs(variant).collect::<Vec<_>>();
                    if !variant_cfg.is_empty() {
                        insert_attributes(arm.syntax(), &editor, variant_cfg.iter().cloned());
                    }
                    // => #[cfg(...)]
                    //    (`Self::V { a, #[cfg] b }`)
                    if let Some(ast::FieldList::RecordFieldList(rfl)) = variant.field_list() {
                        let fields = arm
                            .pat()
                            .and_then(|p| match p {
                                ast::Pat::RecordPat(rp) => rp.record_pat_field_list(),
                                _ => None,
                            })
                            .map(|l| l.fields().collect::<Vec<_>>())
                            .unwrap_or_default();
                        for (rpf, field) in fields.iter().zip(rfl.fields()) {
                            let field_cfg = cfg_attrs(&field).collect::<Vec<_>>();
                            if !field_cfg.is_empty() {
                                editor.insert_all(
                                    Position::before(rpf.syntax()),
                                    intersperse_attrs(make, &field_cfg, " "),
                                );
                            }
                        }
                    }
                }
                ast::BlockExpr::cast(editor.finish().new_root().clone()).unwrap()
            } else {
                body
            };
            Some(body.indent(ast::edit::IndentLevel(1)))
        }

        ast::Adt::Struct(strukt) => {
            if has_struct_fields_with_cfg(strukt) {
                gen_debug_struct_with_fields_with_cfg(make, strukt, &annotated_name)
            } else {
                gen_debug_struct(make, strukt, &annotated_name)
            }
        }
    }
}

/// Will generate something like:
///
/// ```ignore
/// f.debug_struct("Foo").field("bar", &self.bar).field("baz", &self.baz).finish()
/// ```
fn gen_debug_struct(
    make: &SyntaxFactory,
    strukt: &ast::Struct,
    annotated_name: &ast::Name,
) -> Option<ast::BlockExpr> {
    let name = format!("\"{annotated_name}\"");
    let args = make.arg_list([make.expr_literal(&name).into()]);
    let target = make.expr_path(make.ident_path("f"));

    let expr = match strukt.field_list() {
        // => f.debug_struct("Name").finish()
        None => make.expr_method_call(target, make.name_ref("debug_struct"), args).into(),

        // => f.debug_struct("Name").field("foo", &self.foo).finish()
        Some(ast::FieldList::RecordFieldList(field_list)) => {
            let method = make.name_ref("debug_struct");
            let mut expr = make.expr_method_call(target, method, args).into();
            for field in field_list.fields() {
                let name = field.name()?;
                let f_name = make.expr_literal(&(format!("\"{name}\""))).into();
                let f_path = make.expr_path(make.ident_path("self"));
                let f_path = make.expr_field(f_path, &format!("{name}")).into();
                let f_path = make.expr_ref(f_path, false);
                let args = make.arg_list([f_name, f_path]);
                expr = make.expr_method_call(expr, make.name_ref("field"), args).into();
            }
            expr
        }

        // => f.debug_tuple("Name").field(&self.0).finish()
        Some(ast::FieldList::TupleFieldList(field_list)) => {
            let method = make.name_ref("debug_tuple");
            let mut expr = make.expr_method_call(target, method, args).into();
            for (i, _) in field_list.fields().enumerate() {
                let f_path = make.expr_path(make.ident_path("self"));
                let f_path = make.expr_field(f_path, &format!("{i}")).into();
                let f_path = make.expr_ref(f_path, false);
                let method = make.name_ref("field");
                expr = make.expr_method_call(expr, method, make.arg_list([f_path])).into();
            }
            expr
        }
    };

    let method = make.name_ref("finish");
    let expr = make.expr_method_call(expr, method, make.arg_list([])).into();
    let body = make.block_expr(None::<ast::Stmt>, Some(expr)).indent(ast::edit::IndentLevel(1));
    Some(body)
}

/// Whether a struct has at least one field carrying a `#[cfg(...)]` attribute.
fn has_struct_fields_with_cfg(strukt: &ast::Struct) -> bool {
    strukt.field_list().is_some_and(|field_list| match field_list {
        ast::FieldList::RecordFieldList(list) => {
            list.fields().any(|field| cfg_attrs(&field).next().is_some())
        }
        ast::FieldList::TupleFieldList(list) => {
            list.fields().any(|field| cfg_attrs(&field).next().is_some())
        }
    })
}

/// Intersperse `sep` between the given `attrs`. A trailing `sep` will be added.
fn intersperse_attrs(
    make: &SyntaxFactory,
    attrs: &[ast::Attr],
    sep: &str,
) -> Vec<syntax::SyntaxElement> {
    let mut elements = Vec::with_capacity(attrs.len() * 2);
    for attr in attrs {
        elements.push(attr.syntax().clone().into());
        elements.push(make.whitespace(sep).into());
    }
    elements
}

/// Builds `{ <let_stmt>; <stmts_with_attrs>; <tail> }` block
fn build_let_stmts_tail_block(
    make: &SyntaxFactory,
    let_stmt: ast::Stmt,
    stmts_with_attrs: Vec<(ast::Stmt, Vec<ast::Attr>)>,
    tail: ast::Expr,
) -> Option<ast::BlockExpr> {
    let block = make.block_expr(vec![let_stmt], Some(tail));
    let (editor, block_clone) = SyntaxEditor::with_ast_node(&block);
    let tail_expr = block_clone.stmt_list()?.tail_expr()?;

    let stmt_sep = format!("\n{}", ast::edit::IndentLevel(1));
    let mut elements = Vec::with_capacity(stmts_with_attrs.len());
    for (stmt, attrs) in stmts_with_attrs {
        elements.extend(intersperse_attrs(make, &attrs, &stmt_sep));
        elements.push(stmt.syntax().clone().into());
        elements.push(make.whitespace(&stmt_sep).into());
    }

    editor.insert_all(Position::before(tail_expr.syntax()), elements);
    let block = ast::BlockExpr::cast(editor.finish().new_root().clone())?;
    Some(block.indent(ast::edit::IndentLevel(1)))
}

/// Will generate something like:
///
/// ```ignore
/// let mut s = f.debug_struct("Foo");
/// s.field("bar", &self.bar);
/// #[cfg(feature = "baz")]
/// s.field("baz", &self.baz);
/// s.finish()
/// ```
fn gen_debug_struct_with_fields_with_cfg(
    make: &SyntaxFactory,
    strukt: &ast::Struct,
    annotated_name: &ast::Name,
) -> Option<ast::BlockExpr> {
    let field_list = strukt.field_list()?;
    let builder_method = match field_list {
        ast::FieldList::RecordFieldList(_) => "debug_struct",
        ast::FieldList::TupleFieldList(_) => "debug_tuple",
    };

    // => let mut s = f.debug_struct("Name");
    let struct_name = format!("\"{annotated_name}\"");
    let name_arg = make.arg_list([make.expr_literal(&struct_name).into()]);
    let init = make.expr_method_call(
        make.expr_path(make.ident_path("f")),
        make.name_ref(builder_method),
        name_arg,
    );
    let builder_pat = make.ident_pat(false, true, make.name("s"));
    let let_stmt: ast::Stmt = make.let_stmt(builder_pat.into(), None, Some(init.into())).into();

    // => s.finish()
    let s_path = || make.expr_path(make.ident_path("s"));
    let tail = make.expr_method_call(s_path(), make.name_ref("finish"), make.arg_list([]));

    let mut field_stmts = Vec::new();
    match field_list {
        // => s.field("name", &self.name);
        ast::FieldList::RecordFieldList(field_list) => {
            for field in field_list.fields() {
                let name = field.name()?;

                let f_name = make.expr_literal(&(format!("\"{name}\""))).into();
                let f_path = make.expr_path(make.ident_path("self"));
                let f_path = make.expr_field(f_path, &format!("{name}")).into();
                let f_path = make.expr_ref(f_path, false);
                let call = make.expr_method_call(
                    s_path(),
                    make.name_ref("field"),
                    make.arg_list([f_name, f_path]),
                );

                let stmt: ast::Stmt = make.expr_stmt(call.into()).into();
                field_stmts.push((stmt, cfg_attrs(&field).collect()));
            }
        }
        // => s.field(&self.0);
        ast::FieldList::TupleFieldList(field_list) => {
            for (i, field) in field_list.fields().enumerate() {
                let f_path = make.expr_path(make.ident_path("self"));
                let f_path = make.expr_field(f_path, &format!("{i}")).into();
                let f_path = make.expr_ref(f_path, false);
                let call = make.expr_method_call(
                    s_path(),
                    make.name_ref("field"),
                    make.arg_list([f_path]),
                );

                let stmt: ast::Stmt = make.expr_stmt(call.into()).into();
                field_stmts.push((stmt, cfg_attrs(&field).collect()));
            }
        }
    }

    build_let_stmts_tail_block(make, let_stmt, field_stmts, tail.into())
}

/// Will generate something like:
///
/// ```ignore
/// f.debug_struct("V").field("a", a).field("b", b).finish()
/// ```
fn gen_debug_enum_record_variant_body(
    make: &SyntaxFactory,
    name: &ast::Name,
    list: &ast::RecordFieldList,
) -> Option<ast::Expr> {
    // => f.debug_struct("V")
    let target = make.expr_path(make.ident_path("f"));
    let method = make.name_ref("debug_struct");
    let struct_name = format!("\"{name}\"");
    let args = make.arg_list([make.expr_literal(&struct_name).into()]);
    let mut expr = make.expr_method_call(target, method, args).into();

    // => .field("a", a)
    for field in list.fields() {
        let field_name = field.name()?;
        let method_name = make.name_ref("field");
        let lit = make.expr_literal(&(format!("\"{field_name}\""))).into();
        let path = make.expr_path(make.ident_path(&format!("{field_name}")));
        let args = make.arg_list([lit, path]);
        expr = make.expr_method_call(expr, method_name, args).into();
    }

    // => .finish()
    let method = make.name_ref("finish");
    Some(make.expr_method_call(expr, method, make.arg_list([])).into())
}

/// Will generate something like:
///
/// ```ignore
/// {
///     let mut s = f.debug_struct("V");
///     s.field("a", a);
///     #[cfg(feature = "x")]
///     s.field("b", b);
///     s.finish()
/// }
/// ```
fn gen_debug_enum_record_variant_body_with_cfg(
    make: &SyntaxFactory,
    name: &ast::Name,
    list: &ast::RecordFieldList,
    field_cfgs: &[Vec<ast::Attr>],
) -> Option<ast::BlockExpr> {
    // => let mut s = f.debug_struct("V");
    let struct_name = format!("\"{name}\"");
    let name_arg = make.arg_list([make.expr_literal(&struct_name).into()]);
    let init = make.expr_method_call(
        make.expr_path(make.ident_path("f")),
        make.name_ref("debug_struct"),
        name_arg,
    );
    let builder_pat = make.ident_pat(false, true, make.name("s"));
    let let_stmt = make.let_stmt(builder_pat.into(), None, Some(init.into())).into();

    // => s.finish()
    let s = make.expr_path(make.ident_path("s"));
    let tail = make.expr_method_call(s.clone(), make.name_ref("finish"), make.arg_list([]));

    // => s.field("name", name);
    let mut field_stmts = Vec::new();
    for (field, cfg) in list.fields().zip(field_cfgs) {
        let field_name = field.name()?;
        let lit = make.expr_literal(&(format!("\"{field_name}\""))).into();
        let path = make.expr_path(make.ident_path(&format!("{field_name}")));
        let call =
            make.expr_method_call(s.clone(), make.name_ref("field"), make.arg_list([lit, path]));
        let stmt = make.expr_stmt(call.into()).into();
        field_stmts.push((stmt, cfg.clone()));
    }

    build_let_stmts_tail_block(make, let_stmt, field_stmts, tail.into())
}

/// Generate a `Default` impl based on the fields and members of the target type.
fn gen_default_impl(make: &SyntaxFactory, adt: &ast::Adt) -> Option<ast::BlockExpr> {
    let gen_default_call = || -> Option<ast::Expr> {
        let fn_name = make.path_from_idents(["Default", "default"])?;
        Some(make.expr_call(make.expr_path(fn_name), make.arg_list([])).into())
    };
    match adt {
        // `Debug` cannot be derived for unions, so no default impl can be provided.
        ast::Adt::Union(_) => None,
        // Deriving `Debug` for enums is not stable yet.
        ast::Adt::Enum(_) => None,
        ast::Adt::Struct(strukt) => {
            let expr = match strukt.field_list() {
                Some(ast::FieldList::RecordFieldList(field_list)) => {
                    let mut fields = vec![];
                    for field in field_list.fields() {
                        let method_call = gen_default_call()?;
                        let name_ref = make.name_ref(&field.name()?.to_string());
                        let field = make.record_expr_field(name_ref, Some(method_call));
                        fields.push(field);
                    }
                    let struct_name = make.ident_path("Self");
                    let fields = make.record_expr_field_list(fields);
                    make.record_expr(struct_name, fields).into()
                }
                Some(ast::FieldList::TupleFieldList(field_list)) => {
                    let struct_name = make.expr_path(make.ident_path("Self"));
                    let fields = field_list
                        .fields()
                        .map(|_| gen_default_call())
                        .collect::<Option<Vec<ast::Expr>>>()?;
                    make.expr_call(struct_name, make.arg_list(fields)).into()
                }
                None => {
                    let struct_name = make.ident_path("Self");
                    let fields = make.record_expr_field_list([]);
                    make.record_expr(struct_name, fields).into()
                }
            };
            let body =
                make.block_expr(None::<ast::Stmt>, Some(expr)).indent(ast::edit::IndentLevel(1));
            Some(body)
        }
    }
}

/// Generate a `Hash` impl based on the fields and members of the target type.
fn gen_hash_impl(make: &SyntaxFactory, adt: &ast::Adt) -> Option<ast::BlockExpr> {
    let gen_hash_call = |target: ast::Expr| -> ast::Stmt {
        let method = make.name_ref("hash");
        let arg = make.expr_path(make.ident_path("state"));
        let expr = make.expr_method_call(target, method, make.arg_list([arg])).into();
        make.expr_stmt(expr).into()
    };

    let body = match adt {
        // `Hash` cannot be derived for unions, so no default impl can be provided.
        ast::Adt::Union(_) => return None,

        // => std::mem::discriminant(self).hash(state);
        ast::Adt::Enum(_) => {
            let fn_name = make_discriminant(make)?;

            let arg = make.expr_path(make.ident_path("self"));
            let fn_call: ast::Expr = make.expr_call(fn_name, make.arg_list([arg])).into();
            let stmt = gen_hash_call(fn_call);

            make.block_expr([stmt], None).indent(ast::edit::IndentLevel(1))
        }
        ast::Adt::Struct(strukt) => match strukt.field_list() {
            // => self.<field>.hash(state);
            Some(ast::FieldList::RecordFieldList(field_list)) => {
                let mut stmts = vec![];
                for field in field_list.fields() {
                    let base = make.expr_path(make.ident_path("self"));
                    let target = make.expr_field(base, &field.name()?.to_string()).into();
                    stmts.push(gen_hash_call(target));
                }
                make.block_expr(stmts, None).indent(ast::edit::IndentLevel(1))
            }

            // => self.<field_index>.hash(state);
            Some(ast::FieldList::TupleFieldList(field_list)) => {
                let mut stmts = vec![];
                for (i, _) in field_list.fields().enumerate() {
                    let base = make.expr_path(make.ident_path("self"));
                    let target = make.expr_field(base, &format!("{i}")).into();
                    stmts.push(gen_hash_call(target));
                }
                make.block_expr(stmts, None).indent(ast::edit::IndentLevel(1))
            }

            // No fields in the body means there's nothing to hash.
            None => return None,
        },
    };

    Some(body)
}

/// Generate a `PartialEq` impl based on the fields and members of the target type.
fn gen_partial_eq(
    make: &SyntaxFactory,
    adt: &ast::Adt,
    trait_ref: Option<TraitRef<'_>>,
) -> Option<ast::BlockExpr> {
    let gen_eq_chain = |expr: Option<ast::Expr>, cmp: ast::Expr| -> Option<ast::Expr> {
        match expr {
            Some(expr) => Some(make.expr_bin_op(expr, BinaryOp::LogicOp(LogicOp::And), cmp)),
            None => Some(cmp),
        }
    };

    let gen_record_pat_field = |field_name: &str, pat_name: &str| -> ast::RecordPatField {
        let pat = make.ident_pat(false, false, make.name(pat_name));
        let name_ref = make.name_ref(field_name);
        make.record_pat_field(name_ref, pat.into())
    };

    let gen_record_pat =
        |record_name: ast::Path, fields: Vec<ast::RecordPatField>| -> ast::RecordPat {
            let list = make.record_pat_field_list(fields, None);
            make.record_pat_with_fields(record_name, list)
        };

    let gen_variant_path = |variant: &ast::Variant| -> Option<ast::Path> {
        make.path_from_idents(["Self", &variant.name()?.to_string()])
    };

    let gen_tuple_field = |field_name: &str| -> ast::Pat {
        ast::Pat::IdentPat(make.ident_pat(false, false, make.name(field_name)))
    };

    // Check that self type and rhs type match. We don't know how to implement the method
    // automatically otherwise.
    if let Some(trait_ref) = trait_ref {
        let self_ty = trait_ref.self_ty();
        let rhs_ty = trait_ref.get_type_argument(1)?;
        if self_ty != rhs_ty {
            return None;
        }
    }

    let body = match adt {
        // `PartialEq` cannot be derived for unions, so no default impl can be provided.
        ast::Adt::Union(_) => return None,

        ast::Adt::Enum(enum_) => {
            // => std::mem::discriminant(self) == std::mem::discriminant(other)
            let lhs_name = make.expr_path(make.ident_path("self"));
            let lhs =
                make.expr_call(make_discriminant(make)?, make.arg_list([lhs_name.clone()])).into();
            let rhs_name = make.expr_path(make.ident_path("other"));
            let rhs =
                make.expr_call(make_discriminant(make)?, make.arg_list([rhs_name.clone()])).into();
            let eq_check =
                make.expr_bin_op(lhs, BinaryOp::CmpOp(CmpOp::Eq { negated: false }), rhs);

            let mut n_cases = 0;
            let mut arms = vec![];
            for variant in enum_.variant_list()?.variants() {
                n_cases += 1;
                match variant.field_list() {
                    // => (Self::Bar { bin: l_bin }, Self::Bar { bin: r_bin }) => l_bin == r_bin,
                    Some(ast::FieldList::RecordFieldList(list)) => {
                        let mut expr = None;
                        let mut l_fields = vec![];
                        let mut r_fields = vec![];

                        for field in list.fields() {
                            let field_name = field.name()?.to_string();

                            let l_name = &format!("l_{field_name}");
                            l_fields.push(gen_record_pat_field(&field_name, l_name));

                            let r_name = &format!("r_{field_name}");
                            r_fields.push(gen_record_pat_field(&field_name, r_name));

                            let lhs = make.expr_path(make.ident_path(l_name));
                            let rhs = make.expr_path(make.ident_path(r_name));
                            let cmp = make.expr_bin_op(
                                lhs,
                                BinaryOp::CmpOp(CmpOp::Eq { negated: false }),
                                rhs,
                            );
                            expr = gen_eq_chain(expr, cmp);
                        }

                        let left = gen_record_pat(gen_variant_path(&variant)?, l_fields);
                        let right = gen_record_pat(gen_variant_path(&variant)?, r_fields);
                        let tuple = make.tuple_pat(vec![left.into(), right.into()]);

                        if let Some(expr) = expr {
                            arms.push(make.match_arm(tuple.into(), None, expr));
                        }
                    }

                    Some(ast::FieldList::TupleFieldList(list)) => {
                        let mut expr = None;
                        let mut l_fields = vec![];
                        let mut r_fields = vec![];

                        for (i, _) in list.fields().enumerate() {
                            let field_name = format!("{i}");

                            let l_name = format!("l{field_name}");
                            l_fields.push(gen_tuple_field(&l_name));

                            let r_name = format!("r{field_name}");
                            r_fields.push(gen_tuple_field(&r_name));

                            let lhs = make.expr_path(make.ident_path(&l_name));
                            let rhs = make.expr_path(make.ident_path(&r_name));
                            let cmp = make.expr_bin_op(
                                lhs,
                                BinaryOp::CmpOp(CmpOp::Eq { negated: false }),
                                rhs,
                            );
                            expr = gen_eq_chain(expr, cmp);
                        }

                        let left = make.tuple_struct_pat(gen_variant_path(&variant)?, l_fields);
                        let right = make.tuple_struct_pat(gen_variant_path(&variant)?, r_fields);
                        let tuple = make.tuple_pat(vec![left.into(), right.into()]);

                        if let Some(expr) = expr {
                            arms.push(make.match_arm(tuple.into(), None, expr));
                        }
                    }
                    None => continue,
                }
            }

            let expr = match arms.len() {
                0 => eq_check,
                arms_len => {
                    // Generate the fallback arm when this enum has >1 variants.
                    // The fallback arm will be `_ => false,` if we've already gone through every case where the variants of self and other match,
                    // and `_ => std::mem::discriminant(self) == std::mem::discriminant(other),` otherwise.
                    if n_cases > 1 {
                        let lhs = make.wildcard_pat().into();
                        let rhs = if arms_len == n_cases {
                            make.expr_literal("false").into()
                        } else {
                            eq_check
                        };
                        arms.push(make.match_arm(lhs, None, rhs));
                    }

                    let match_target = make.expr_tuple([lhs_name, rhs_name]).into();
                    let list = make.match_arm_list(arms).indent(ast::edit::IndentLevel(1));
                    make.expr_match(match_target, list).into()
                }
            };

            make.block_expr(None::<ast::Stmt>, Some(expr)).indent(ast::edit::IndentLevel(1))
        }
        ast::Adt::Struct(strukt) => match strukt.field_list() {
            Some(ast::FieldList::RecordFieldList(field_list)) => {
                let mut expr = None;
                for field in field_list.fields() {
                    let lhs = make.expr_path(make.ident_path("self"));
                    let lhs = make.expr_field(lhs, &field.name()?.to_string()).into();
                    let rhs = make.expr_path(make.ident_path("other"));
                    let rhs = make.expr_field(rhs, &field.name()?.to_string()).into();
                    let cmp =
                        make.expr_bin_op(lhs, BinaryOp::CmpOp(CmpOp::Eq { negated: false }), rhs);
                    expr = gen_eq_chain(expr, cmp);
                }
                make.block_expr(None, expr).indent(ast::edit::IndentLevel(1))
            }

            Some(ast::FieldList::TupleFieldList(field_list)) => {
                let mut expr = None;
                for (i, _) in field_list.fields().enumerate() {
                    let idx = format!("{i}");
                    let lhs = make.expr_path(make.ident_path("self"));
                    let lhs = make.expr_field(lhs, &idx).into();
                    let rhs = make.expr_path(make.ident_path("other"));
                    let rhs = make.expr_field(rhs, &idx).into();
                    let cmp =
                        make.expr_bin_op(lhs, BinaryOp::CmpOp(CmpOp::Eq { negated: false }), rhs);
                    expr = gen_eq_chain(expr, cmp);
                }
                make.block_expr(None::<ast::Stmt>, expr).indent(ast::edit::IndentLevel(1))
            }

            // No fields in the body means there's nothing to compare.
            None => {
                let expr = make.expr_literal("true").into();
                make.block_expr(None, Some(expr)).indent(ast::edit::IndentLevel(1))
            }
        },
    };

    Some(body)
}

fn gen_partial_ord(
    make: &SyntaxFactory,
    adt: &ast::Adt,
    trait_ref: Option<TraitRef<'_>>,
) -> Option<ast::BlockExpr> {
    let gen_partial_eq_match = |match_target: ast::Expr| -> Option<ast::Stmt> {
        let mut arms = vec![];

        let variant_name =
            make.path_pat(make.path_from_idents(["core", "cmp", "Ordering", "Equal"])?);
        let lhs = make.tuple_struct_pat(make.path_from_idents(["Some"])?, [variant_name]);
        arms.push(make.match_arm(lhs.into(), None, make.expr_empty_block().into()));

        arms.push(make.match_arm(
            make.ident_pat(false, false, make.name("ord")).into(),
            None,
            make.expr_return(Some(make.expr_path(make.ident_path("ord")))).into(),
        ));
        let list = make.match_arm_list(arms).indent(ast::edit::IndentLevel(1));
        Some(make.expr_stmt(make.expr_match(match_target, list).into()).into())
    };

    let gen_partial_cmp_call = |lhs: ast::Expr, rhs: ast::Expr| -> ast::Expr {
        let rhs = make.expr_ref(rhs, false);
        let method = make.name_ref("partial_cmp");
        make.expr_method_call(lhs, method, make.arg_list([rhs])).into()
    };

    // Check that self type and rhs type match. We don't know how to implement the method
    // automatically otherwise.
    if let Some(trait_ref) = trait_ref {
        let self_ty = trait_ref.self_ty();
        let rhs_ty = trait_ref.get_type_argument(1)?;
        if self_ty != rhs_ty {
            return None;
        }
    }

    let body = match adt {
        // `PartialOrd` cannot be derived for unions, so no default impl can be provided.
        ast::Adt::Union(_) => return None,
        // `core::mem::Discriminant` does not implement `PartialOrd` in stable Rust today.
        ast::Adt::Enum(_) => return None,
        ast::Adt::Struct(strukt) => match strukt.field_list() {
            Some(ast::FieldList::RecordFieldList(field_list)) => {
                let mut exprs = vec![];
                for field in field_list.fields() {
                    let lhs = make.expr_path(make.ident_path("self"));
                    let lhs = make.expr_field(lhs, &field.name()?.to_string()).into();
                    let rhs = make.expr_path(make.ident_path("other"));
                    let rhs = make.expr_field(rhs, &field.name()?.to_string()).into();
                    let ord = gen_partial_cmp_call(lhs, rhs);
                    exprs.push(ord);
                }

                let tail = exprs.pop();
                let stmts = exprs
                    .into_iter()
                    .map(gen_partial_eq_match)
                    .collect::<Option<Vec<ast::Stmt>>>()?;
                make.block_expr(stmts, tail).indent(ast::edit::IndentLevel(1))
            }

            Some(ast::FieldList::TupleFieldList(field_list)) => {
                let mut exprs = vec![];
                for (i, _) in field_list.fields().enumerate() {
                    let idx = format!("{i}");
                    let lhs = make.expr_path(make.ident_path("self"));
                    let lhs = make.expr_field(lhs, &idx).into();
                    let rhs = make.expr_path(make.ident_path("other"));
                    let rhs = make.expr_field(rhs, &idx).into();
                    let ord = gen_partial_cmp_call(lhs, rhs);
                    exprs.push(ord);
                }
                let tail = exprs.pop();
                let stmts = exprs
                    .into_iter()
                    .map(gen_partial_eq_match)
                    .collect::<Option<Vec<ast::Stmt>>>()?;
                make.block_expr(stmts, tail).indent(ast::edit::IndentLevel(1))
            }

            // No fields in the body means there's nothing to compare.
            None => {
                let expr = make.expr_literal("true").into();
                make.block_expr(None, Some(expr)).indent(ast::edit::IndentLevel(1))
            }
        },
    };

    Some(body)
}

fn make_discriminant(make: &SyntaxFactory) -> Option<ast::Expr> {
    Some(make.expr_path(make.path_from_idents(["core", "mem", "discriminant"])?))
}
