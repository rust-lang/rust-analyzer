use either::Either;
use hir::{CaptureKind, ClosureCapture, HirDisplay};
use ide_db::{
    FxHashSet, assists::AssistId, defs::Definition, search::FileReferenceNode,
    source_change::SourceChangeBuilder,
};
use syntax::{
    AstNode, Direction, SyntaxKind, SyntaxNode, T, ToSmolStr,
    algo::skip_trivia_token,
    ast::{
        self, HasArgList, HasGenericParams, HasName,
        edit::{AstNodeEdit, IndentLevel},
        syntax_factory::SyntaxFactory,
    },
    hacks::parse_expr_from_str,
    syntax_editor::{Position, SyntaxEditor},
};

use crate::assist_context::{AssistContext, Assists};

// Assist: convert_closure_to_fn
//
// This converts a closure to a freestanding function, changing all captures to parameters.
//
// ```
// # //- minicore: copy, fn
// # struct String;
// # impl String {
// #     fn new() -> Self {}
// #     fn push_str(&mut self, s: &str) {}
// # }
// fn main() {
//     let mut s = String::new();
//     let closure = |$0a| s.push_str(a);
//     closure("abc");
// }
// ```
// ->
// ```
// # struct String;
// # impl String {
// #     fn new() -> Self {}
// #     fn push_str(&mut self, s: &str) {}
// # }
// fn main() {
//     let mut s = String::new();
//     fn closure(a: &str, s: &mut String) {
//         s.push_str(a)
//     }
//     closure("abc", &mut s);
// }
// ```
pub(crate) fn convert_closure_to_fn(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let (editor, _) = SyntaxEditor::new(ctx.source_file().syntax().clone());
    let make = editor.make();
    let closure = ctx.find_node_at_offset::<ast::ClosureExpr>()?;
    if ctx.find_node_at_offset::<ast::Expr>() != Some(ast::Expr::ClosureExpr(closure.clone())) {
        // Not inside the parameter list.
        return None;
    }
    let closure_name = closure.syntax().parent().and_then(|parent| {
        let closure_decl = ast::LetStmt::cast(parent)?;
        match closure_decl.pat()? {
            ast::Pat::IdentPat(pat) => Some((closure_decl, pat.clone(), pat.name()?)),
            _ => None,
        }
    });
    let module = ctx.sema.scope(closure.syntax())?.module();
    let closure_ty = ctx.sema.type_of_expr(&closure.clone().into())?;
    let callable = closure_ty.original.as_callable(ctx.db())?;
    let closure_ty = closure_ty.original.as_closure()?;

    let mut ret_ty = callable.return_type();
    let mut closure_mentioned_generic_params = ret_ty.generic_params(ctx.db());

    let mut params = callable
        .params()
        .into_iter()
        .map(|param| {
            let node = ctx.sema.source(param.clone())?.value.right()?;
            let param_ty = param.ty();
            closure_mentioned_generic_params.extend(param_ty.generic_params(ctx.db()));
            match node.ty() {
                Some(_) => Some(node),
                None => {
                    let ty = param_ty
                        .display_source_code(ctx.db(), module.into(), true)
                        .unwrap_or_else(|_| "_".to_owned());
                    Some(make.param(node.pat()?, make.ty(&ty)))
                }
            }
        })
        .collect::<Option<Vec<_>>>()?;
    let capture_params_start = params.len();

    let body = closure.body()?;
    let mut is_gen = false;
    let mut is_async = closure.async_token().is_some();
    if is_async {
        ret_ty = ret_ty.future_output(ctx.db())?;
    }
    // We defer the wrapping of the body in the block, because `make::block()` will generate a new node,
    // but we need to locate `AstPtr`s inside the body.
    let (strip_async_from_body, strip_gen_from_body, wrap_body_in_block) =
        if let ast::Expr::BlockExpr(block) = &body {
            let strip_async = block.async_token().is_some() && !is_async;
            let strip_gen = block.gen_token().is_some();
            if strip_async {
                is_async = true;
                ret_ty = ret_ty.future_output(ctx.db())?;
            }
            if strip_gen {
                is_gen = true;
                ret_ty = ret_ty.iterator_item(ctx.db())?;
            }

            let has_other_modifiers = block.try_block_modifier().is_some()
                || block.unsafe_token().is_some()
                || block.label().is_some()
                || block.const_token().is_some();
            let keeps_unstripped_async_or_gen = (block.async_token().is_some() && !strip_async)
                || (block.gen_token().is_some() && !strip_gen);
            (strip_async, strip_gen, has_other_modifiers || keeps_unstripped_async_or_gen)
        } else {
            (false, false, true)
        };

    let (insert_before_stmt, insert_in_closure_body, insert_before_tail_expr) = if closure_name
        .is_none()
    {
        let top_stmt = closure.syntax().ancestors().skip(1).find_map(|ancestor| {
            ast::Stmt::cast(ancestor.clone()).map(Either::Left).or_else(|| {
                ast::ClosureExpr::cast(ancestor.clone())
                    .map(Either::Left)
                    .or_else(|| ast::BlockExpr::cast(ancestor).map(Either::Right))
                    .map(Either::Right)
            })
        })?;
        match top_stmt {
            Either::Left(stmt) => (Some(stmt), None, None),
            Either::Right(Either::Left(closure_inside_closure)) => {
                (None, Some(closure_inside_closure.body()?), None)
            }
            Either::Right(Either::Right(block_expr)) => (None, None, Some(block_expr.tail_expr()?)),
        }
    } else {
        (None, None, None)
    };

    let captures = closure_ty.captured_items(ctx.db());

    let mut captures_as_args = Vec::with_capacity(captures.len());
    let body_root = body.syntax().ancestors().last().unwrap();
    // We need to defer this work because otherwise the text range of elements is being messed up, and
    // replacements for the next captures won't work.
    let mut capture_usages_replacement_map = Vec::with_capacity(captures.len());
    for capture in &captures {
        let capture_ty = capture.captured_ty(ctx.db());
        // FIXME: Allow configuring the replacement of `self`.
        let is_self = capture.local().is_self(ctx.db()) && !capture.has_field_projections();
        let capture_name = if is_self {
            make.name("this")
        } else {
            make.name(&capture.place_to_name(ctx.db(), ctx.edition()))
        };

        closure_mentioned_generic_params.extend(capture_ty.generic_params(ctx.db()));
        let capture_ty = capture_ty
            .display_source_code(ctx.db(), module.into(), true)
            .unwrap_or_else(|_| "_".to_owned());
        let param = make.param(
            ast::Pat::IdentPat(make.ident_pat(false, false, capture_name.clone_subtree())),
            make.ty(&capture_ty),
        );
        if is_self {
            // Always put `this` first.
            params.insert(capture_params_start, param);
        } else {
            params.push(param);
        }

        for capture_usage in capture.usages().sources(ctx.db()) {
            if capture_usage.file_id() != ctx.file_id() {
                // This is from a macro, don't change it.
                continue;
            }

            let capture_usage_source = capture_usage.source();
            let capture_usage_source = capture_usage_source.to_node(&body_root);
            let mut expr = match capture_usage_source {
                Either::Left(expr) => expr,
                Either::Right(pat) => {
                    let Some(expr) = expr_of_pat(pat) else { continue };
                    expr
                }
            };

            if !capture_usage.is_ref() {
                expr = peel_ref(expr);
            }

            let replacement = wrap_capture_in_deref_if_needed(
                make,
                &expr,
                &capture_name,
                capture.kind(),
                matches!(expr, ast::Expr::RefExpr(_)) || capture_usage.is_ref(),
            );
            capture_usages_replacement_map.push((expr, replacement));
        }

        let capture_as_arg = capture_as_arg(make, ctx, capture);
        if is_self {
            captures_as_args.insert(0, capture_as_arg);
        } else {
            captures_as_args.push(capture_as_arg);
        }
    }

    let (closure_type_params, closure_where_clause) =
        compute_closure_type_params(make, ctx, closure_mentioned_generic_params, &closure);
    let body = rewrite_closure_body(
        &closure,
        &body,
        &capture_usages_replacement_map,
        strip_async_from_body,
        strip_gen_from_body,
    )?;

    let closure_name_or_default =
        closure_name.as_ref().map(|(_, _, it)| it.clone()).unwrap_or_else(|| make.name("fun_name"));

    let body = if wrap_body_in_block {
        make.block_expr([], Some(body.dedent(IndentLevel(1))))
    } else {
        ast::BlockExpr::cast(body.syntax().clone())?
    };

    acc.add(
        AssistId::refactor_rewrite("convert_closure_to_fn"),
        "Convert closure to fn",
        closure.param_list()?.syntax().text_range(),
        |builder| {
            let make = editor.make();
            let params = make.param_list(None, params);
            let ret_ty = if ret_ty.is_unit() {
                None
            } else {
                let ret_ty = ret_ty
                    .display_source_code(ctx.db(), module.into(), true)
                    .unwrap_or_else(|_| "_".to_owned());
                Some(make.ret_type(make.ty(&ret_ty)))
            };
            let mut fn_ = make.fn_(
                None,
                None,
                closure_name_or_default.clone(),
                closure_type_params,
                closure_where_clause,
                params,
                body,
                ret_ty,
                is_async,
                false,
                false,
                is_gen,
            );
            if let Some(last_token) = fn_.syntax().last_token() {
                fn_ = fn_.dedent(IndentLevel::from_token(&last_token));
            }

            match &closure_name {
                Some((closure_decl, _, _)) => {
                    fn_ = fn_.indent(closure_decl.indent_level());
                    editor.replace(closure_decl.syntax(), fn_.syntax());
                }
                None => {
                    if let Some(stmt) = &insert_before_stmt {
                        let replacement =
                            make.expr_path(make.path_from_text(&closure_name_or_default.text()));
                        editor.replace(closure.syntax(), replacement.syntax());
                        let fn_ = fn_.indent(stmt.indent_level());
                        editor.insert_all(
                            Position::before(stmt.syntax()),
                            vec![
                                fn_.syntax().clone().into(),
                                make.whitespace(&format!("\n{}", stmt.indent_level())).into(),
                            ],
                        );
                    } else if let Some(closure_body) = &insert_in_closure_body {
                        let body_with_fn = parse_expr_from_str(
                            &format!("{{ {fn_} {closure_name_or_default} }}"),
                            ctx.edition(),
                        )
                        .expect("generated closure body should parse");
                        editor.replace(closure_body.syntax(), body_with_fn.syntax());
                    } else if let Some(tail_expr) = &insert_before_tail_expr {
                        let replacement =
                            make.expr_path(make.path_from_text(&closure_name_or_default.text()));
                        editor.replace(closure.syntax(), replacement.syntax());
                        let fn_ = fn_.indent(tail_expr.indent_level());
                        editor.insert_all(
                            Position::before(tail_expr.syntax()),
                            vec![
                                fn_.syntax().clone().into(),
                                make.whitespace(&format!("\n{}", tail_expr.indent_level())).into(),
                            ],
                        );
                    }
                }
            }
            builder.add_file_edits(ctx.vfs_file_id(), editor);

            handle_calls(
                builder,
                ctx,
                closure_name.as_ref().map(|(_, it, _)| it),
                &captures_as_args,
                &closure,
            );

            // FIXME: Place the cursor at `fun_name`, like rename does.
        },
    )?;
    Some(())
}

fn compute_closure_type_params(
    make: &SyntaxFactory,
    ctx: &AssistContext<'_>,
    mentioned_generic_params: FxHashSet<hir::GenericParam>,
    closure: &ast::ClosureExpr,
) -> (Option<ast::GenericParamList>, Option<ast::WhereClause>) {
    if mentioned_generic_params.is_empty() {
        return (None, None);
    }

    let mut mentioned_names = mentioned_generic_params
        .iter()
        .filter_map(|param| match param {
            hir::GenericParam::TypeParam(param) => Some(param.name(ctx.db()).as_str().to_smolstr()),
            hir::GenericParam::ConstParam(param) => {
                Some(param.name(ctx.db()).as_str().to_smolstr())
            }
            hir::GenericParam::LifetimeParam(_) => None,
        })
        .collect::<FxHashSet<_>>();

    let Some((container_params, container_where, container)) =
        closure.syntax().ancestors().find_map(ast::AnyHasGenericParams::cast).and_then(
            |container| {
                container
                    .generic_param_list()
                    .map(|params| (params, container.where_clause(), container))
            },
        )
    else {
        return (None, None);
    };
    let containing_impl = if ast::AssocItem::can_cast(container.syntax().kind()) {
        container.syntax().ancestors().find_map(ast::Impl::cast).and_then(|impl_| {
            impl_.generic_param_list().map(|params| (params, impl_.where_clause()))
        })
    } else {
        None
    };

    let all_params = container_params
        .type_or_const_params()
        .chain(containing_impl.iter().flat_map(|(param_list, _)| param_list.type_or_const_params()))
        .filter_map(|param| param.name().map(|name| name.text().to_smolstr()))
        .collect::<FxHashSet<_>>();

    // A fixpoint algorithm to detect (very roughly) if we need to include a generic parameter
    // by checking if it is mentioned by another parameter we need to include.
    let mut reached_fixpoint = false;
    let mut container_where_bounds_indices = Vec::new();
    let mut impl_where_bounds_indices = Vec::new();
    while !reached_fixpoint {
        reached_fixpoint = true;

        let mut insert_name = |syntax: &SyntaxNode| {
            let has_name = syntax
                .descendants()
                .filter_map(ast::NameOrNameRef::cast)
                .any(|name| mentioned_names.contains(name.text().trim_start_matches("r#")));
            let mut has_new_params = false;
            if has_name {
                syntax
                    .descendants()
                    .filter_map(ast::NameOrNameRef::cast)
                    .filter_map(|name| {
                        let name = name.text();
                        let name = name.trim_start_matches("r#");
                        all_params.contains(name).then(|| name.to_smolstr())
                    })
                    .for_each(|name| {
                        if mentioned_names.insert(name) {
                            // We do this here so we don't do it if there are only matches that are not in `all_params`.
                            has_new_params = true;
                            reached_fixpoint = false;
                        }
                    });
            }
            has_new_params
        };

        for param in container_params.type_or_const_params() {
            insert_name(param.syntax());
        }
        for (pred_index, pred) in container_where.iter().flat_map(|it| it.predicates()).enumerate()
        {
            if insert_name(pred.syntax()) {
                container_where_bounds_indices.push(pred_index);
            }
        }
        if let Some((impl_params, impl_where)) = &containing_impl {
            for param in impl_params.type_or_const_params() {
                insert_name(param.syntax());
            }
            for (pred_index, pred) in impl_where.iter().flat_map(|it| it.predicates()).enumerate() {
                if insert_name(pred.syntax()) {
                    impl_where_bounds_indices.push(pred_index);
                }
            }
        }
    }

    // Order matters here (for beauty). First the outer impl parameters, then the direct container's.
    let include_params = containing_impl
        .iter()
        .flat_map(|(impl_params, _)| {
            impl_params.type_or_const_params().filter(|param| {
                param.name().is_some_and(|name| {
                    mentioned_names.contains(name.text().trim_start_matches("r#"))
                })
            })
        })
        .chain(container_params.type_or_const_params().filter(|param| {
            param
                .name()
                .is_some_and(|name| mentioned_names.contains(name.text().trim_start_matches("r#")))
        }))
        .map(ast::TypeOrConstParam::into);
    let include_where_bounds = containing_impl
        .as_ref()
        .and_then(|(_, it)| it.as_ref())
        .into_iter()
        .flat_map(|where_| {
            impl_where_bounds_indices.iter().filter_map(|&index| where_.predicates().nth(index))
        })
        .chain(container_where.iter().flat_map(|where_| {
            container_where_bounds_indices
                .iter()
                .filter_map(|&index| where_.predicates().nth(index))
        }))
        .collect::<Vec<_>>();
    let where_clause =
        (!include_where_bounds.is_empty()).then(|| make.where_clause(include_where_bounds));

    // FIXME: Consider generic parameters that do not appear in params/return type/captures but
    // written explicitly inside the closure.
    (Some(make.generic_param_list(include_params)), where_clause)
}

fn peel_parens(mut expr: ast::Expr) -> ast::Expr {
    loop {
        if ast::ParenExpr::can_cast(expr.syntax().kind()) {
            let Some(parent) = expr.syntax().parent().and_then(ast::Expr::cast) else { break };
            expr = parent;
        } else {
            break;
        }
    }
    expr
}

fn peel_ref(mut expr: ast::Expr) -> ast::Expr {
    expr = peel_parens(expr);
    expr.syntax().parent().and_then(ast::RefExpr::cast).map(Into::into).unwrap_or(expr)
}

fn wrap_capture_in_deref_if_needed(
    make: &SyntaxFactory,
    expr: &ast::Expr,
    capture_name: &ast::Name,
    capture_kind: CaptureKind,
    is_ref: bool,
) -> ast::Expr {
    let capture_name = make.expr_path(make.path_from_text(&capture_name.text()));
    if capture_kind == CaptureKind::Move || is_ref {
        return capture_name;
    }
    let expr_parent = expr.syntax().parent().and_then(ast::Expr::cast);
    let expr_parent_peeled_parens = expr_parent.map(peel_parens);
    let does_autoderef = match expr_parent_peeled_parens {
        Some(
            ast::Expr::AwaitExpr(_)
            | ast::Expr::CallExpr(_)
            | ast::Expr::FieldExpr(_)
            | ast::Expr::FormatArgsExpr(_)
            | ast::Expr::MethodCallExpr(_),
        ) => true,
        Some(ast::Expr::IndexExpr(parent_expr)) if parent_expr.base().as_ref() == Some(expr) => {
            true
        }
        _ => false,
    };
    if does_autoderef {
        return capture_name;
    }
    make.expr_prefix(T![*], capture_name).into()
}

fn capture_as_arg(
    make: &SyntaxFactory,
    ctx: &AssistContext<'_>,
    capture: &ClosureCapture<'_>,
) -> ast::Expr {
    let place = parse_expr_from_str(
        &capture.display_place_source_code(ctx.db(), ctx.edition()),
        ctx.edition(),
    )
    .expect("`display_place_source_code()` produced an invalid expr");
    let needs_mut = match capture.kind() {
        CaptureKind::SharedRef => false,
        CaptureKind::MutableRef | CaptureKind::UniqueSharedRef => true,
        CaptureKind::Move => return place,
    };
    if let ast::Expr::PrefixExpr(expr) = &place
        && expr.op_kind() == Some(ast::UnaryOp::Deref)
    {
        return expr.expr().expect("`display_place_source_code()` produced an invalid expr");
    }
    make.expr_ref(place, needs_mut)
}

fn handle_calls(
    builder: &mut SourceChangeBuilder,
    ctx: &AssistContext<'_>,
    closure_name: Option<&ast::IdentPat>,
    captures_as_args: &[ast::Expr],
    closure: &ast::ClosureExpr,
) {
    if captures_as_args.is_empty() {
        return;
    }

    match closure_name {
        Some(closure_name) => {
            let Some(closure_def) = ctx.sema.to_def(closure_name) else { return };
            let closure_usages = Definition::from(closure_def).usages(&ctx.sema).all();
            for (_, usages) in closure_usages {
                for usage in usages {
                    let name = match usage.name {
                        FileReferenceNode::Name(name) => name.syntax().clone(),
                        FileReferenceNode::NameRef(name_ref) => name_ref.syntax().clone(),
                        FileReferenceNode::FormatStringEntry(..) => continue,
                        FileReferenceNode::Lifetime(_) => {
                            unreachable!("impossible usage")
                        }
                    };
                    let Some(expr) = name.parent().and_then(|it| {
                        ast::Expr::cast(
                            ast::PathSegment::cast(it)?.parent_path().syntax().parent()?,
                        )
                    }) else {
                        continue;
                    };
                    handle_call(builder, ctx, expr, captures_as_args);
                }
            }
        }
        None => {
            handle_call(builder, ctx, ast::Expr::ClosureExpr(closure.clone()), captures_as_args);
        }
    }
}

fn handle_call(
    builder: &mut SourceChangeBuilder,
    ctx: &AssistContext<'_>,
    closure_ref: ast::Expr,
    captures_as_args: &[ast::Expr],
) -> Option<()> {
    let call =
        ast::CallExpr::cast(peel_blocks_and_refs_and_parens(closure_ref).syntax().parent()?)?;
    let args = call.arg_list()?;
    // The really last token is `)`; we need one before that.
    let r_paren = args.syntax().last_token()?;
    let token_before_r_paren = r_paren.prev_token();
    let has_trailing_comma = token_before_r_paren.as_ref().is_some_and(|token| {
        skip_trivia_token(token.clone(), Direction::Prev).is_some_and(|token| token.kind() == T![,])
    });
    let first_arg = args.args().next();
    let has_existing_args = first_arg.is_some();
    let has_line_break_before_r_paren = token_before_r_paren
        .as_ref()
        .is_some_and(|token| token.kind() == SyntaxKind::WHITESPACE && token.text().contains('\n'));

    let file_id = ctx.sema.original_range_opt(args.syntax())?.file_id.file_id(ctx.db());
    let first_arg_indent = first_arg.map(|it| it.indent_level());
    let arg_list_indent = args.indent_level();
    let insert_newlines = first_arg_indent
        .is_some_and(|first_arg_indent| first_arg_indent != arg_list_indent)
        || (!has_existing_args && has_line_break_before_r_paren);
    let indent = first_arg_indent
        .unwrap_or_else(|| if insert_newlines { arg_list_indent + 1 } else { arg_list_indent })
        .to_string();
    let arg_separator = if insert_newlines { "\n" } else { " " };

    let editor = builder.make_editor(args.syntax());
    let make = editor.make();
    let insert_pos = match token_before_r_paren {
        Some(prev) if prev.kind() == SyntaxKind::WHITESPACE => Position::before(prev),
        _ => Position::before(r_paren),
    };
    let mut elements = Vec::new();
    if has_existing_args && !has_trailing_comma {
        elements.push(make.token(T![,]).into());
    }
    if insert_newlines || has_existing_args {
        elements.push(make.whitespace(arg_separator).into());
    }
    for (index, arg) in captures_as_args.iter().enumerate() {
        if insert_newlines {
            elements.push(make.whitespace(&indent).into());
        }
        elements.push(arg.syntax().clone().into());
        let is_last = index + 1 == captures_as_args.len();
        if !is_last || has_trailing_comma {
            elements.push(make.token(T![,]).into());
        }
        if !is_last {
            elements.push(make.whitespace(arg_separator).into());
        }
    }
    editor.insert_all(insert_pos, elements);
    builder.add_file_edits(file_id, editor);

    Some(())
}

fn rewrite_closure_body(
    closure: &ast::ClosureExpr,
    original_body: &ast::Expr,
    capture_usages_replacement_map: &[(ast::Expr, ast::Expr)],
    strip_async_from_body: bool,
    strip_gen_from_body: bool,
) -> Option<ast::Expr> {
    if capture_usages_replacement_map.is_empty() && !strip_async_from_body && !strip_gen_from_body {
        return Some(original_body.clone());
    }

    let closure_start = closure.syntax().text_range().start();
    let (editor, _) = SyntaxEditor::new(closure.syntax().ancestors().last()?.clone());
    for (old_expr, new_expr) in capture_usages_replacement_map {
        editor.replace(old_expr.syntax(), new_expr.syntax());
    }

    if strip_async_from_body || strip_gen_from_body {
        let body = closure.body().unwrap_or_else(|| original_body.clone());
        if let ast::Expr::BlockExpr(block) = body {
            let delete_with_following_whitespace = |token: syntax::SyntaxToken| {
                let trailing_ws: Vec<_> = token
                    .siblings_with_tokens(Direction::Next)
                    .skip(1)
                    .take_while(|t| {
                        t.as_token().is_some_and(|t| t.kind() == SyntaxKind::WHITESPACE)
                    })
                    .filter_map(|t| t.into_token())
                    .collect();
                editor.delete(token);
                for ws in trailing_ws {
                    editor.delete(ws);
                }
            };

            if strip_async_from_body && let Some(async_tok) = block.async_token() {
                delete_with_following_whitespace(async_tok);
            }
            if strip_gen_from_body && let Some(gen_tok) = block.gen_token() {
                delete_with_following_whitespace(gen_tok);
            }
        }
    }

    editor.finish().new_root().descendants().find_map(|node| {
        let closure = ast::ClosureExpr::cast(node)?;
        (closure.syntax().text_range().start() == closure_start).then(|| closure.body()).flatten()
    })
}

fn peel_blocks_and_refs_and_parens(mut expr: ast::Expr) -> ast::Expr {
    loop {
        let Some(parent) = expr.syntax().parent() else { break };
        if matches!(parent.kind(), SyntaxKind::PAREN_EXPR | SyntaxKind::REF_EXPR) {
            expr = ast::Expr::cast(parent).unwrap();
            continue;
        }
        if let Some(stmt_list) = ast::StmtList::cast(parent)
            && let Some(block) = stmt_list.syntax().parent().and_then(ast::BlockExpr::cast)
        {
            expr = ast::Expr::BlockExpr(block);
            continue;
        }
        break;
    }
    expr
}

// FIXME:
// Somehow handle the case of `let Struct { field, .. } = capture`.
// Replacing `capture` with `capture_field` won't work.
fn expr_of_pat(pat: ast::Pat) -> Option<ast::Expr> {
    'find_expr: {
        for ancestor in pat.syntax().ancestors() {
            if let Some(let_stmt) = ast::LetStmt::cast(ancestor.clone()) {
                break 'find_expr let_stmt.initializer();
            }
            if ast::MatchArm::can_cast(ancestor.kind())
                && let Some(match_) =
                    ancestor.parent().and_then(|it| it.parent()).and_then(ast::MatchExpr::cast)
            {
                break 'find_expr match_.expr();
            }
            if ast::ExprStmt::can_cast(ancestor.kind()) {
                break;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn handles_unique_captures() {
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, fn
fn main() {
    let s = &mut true;
    let closure = |$0| { *s = false; };
    closure();
}
"#,
            r#"
fn main() {
    let s = &mut true;
    fn closure(s: &mut bool) { *s = false; }
    closure(s);
}
"#,
        );
    }

    #[test]
    fn multiple_capture_usages() {
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, fn
struct A { a: i32, b: bool }
fn main() {
    let mut a = A { a: 123, b: false };
    let closure = |$0| {
        let b = a.b;
        a = A { a: 456, b: true };
    };
    closure();
}
"#,
            r#"
struct A { a: i32, b: bool }
fn main() {
    let mut a = A { a: 123, b: false };
    fn closure(a: &mut A) {
        let b = a.b;
        *a = A { a: 456, b: true };
    }
    closure(&mut a);
}
"#,
        );
    }

    #[test]
    fn changes_names_of_place() {
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, fn
struct A { b: &'static mut B, c: i32 }
struct B(bool, i32);
struct C;
impl C {
    fn foo(&self) {
        let a = A { b: &B(false, 0), c: 123 };
        let closure = |$0| {
            let b = a.b.1;
            let c = &*self;
        };
        closure();
    }
}
"#,
            r#"
struct A { b: &'static mut B, c: i32 }
struct B(bool, i32);
struct C;
impl C {
    fn foo(&self) {
        let a = A { b: &B(false, 0), c: 123 };
        fn closure(this: &C, a_b_1: &i32) {
            let b = *a_b_1;
            let c = this;
        }
        closure(self, &a.b.1);
    }
}
"#,
        );
    }

    #[test]
    fn self_with_fields_does_not_change_to_this() {
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, fn
struct A { b: &'static B, c: i32 }
struct B(bool, i32);
impl A {
    fn foo(&self) {
        let closure = |$0| {
            let b = self.b.1;
        };
        closure();
    }
}
"#,
            r#"
struct A { b: &'static B, c: i32 }
struct B(bool, i32);
impl A {
    fn foo(&self) {
        fn closure(self_b: &B) {
            let b = self_b.1;
        }
        closure(self.b);
    }
}
"#,
        );
    }

    #[test]
    fn replaces_async_closure_with_async_fn() {
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, future, async_fn
fn foo(&self) {
    let closure = async |$0| 1;
    closure();
}
"#,
            r#"
fn foo(&self) {
    async fn closure() -> i32 {
        1
    }
    closure();
}
"#,
        );
    }

    #[test]
    fn replaces_async_block_with_async_fn() {
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, future, fn
fn foo() {
    let closure = |$0| async { 1 };
    closure();
}
"#,
            r#"
fn foo() {
    async fn closure() -> i32 { 1 }
    closure();
}
"#,
        );
    }

    #[test]
    #[ignore = "FIXME: we do not do type inference for gen blocks yet"]
    fn replaces_gen_block_with_gen_fn() {
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, iterator
//- /lib.rs edition:2024
fn foo() {
    let closure = |$0| gen {
        yield 1;
    };
    closure();
}
"#,
            r#"
fn foo() {
    gen fn closure() -> i32 {
        yield 1;
    }
    closure();
}
"#,
        );
    }

    #[test]
    fn leaves_block_in_place() {
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, fn
fn foo() {
    let closure = |$0| {};
    closure();
}
"#,
            r#"
fn foo() {
    fn closure() {}
    closure();
}
"#,
        );
    }

    #[test]
    fn wraps_in_block_if_needed() {
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, fn
fn foo() {
    let a = 1;
    let closure = |$0| a;
    closure();
}
"#,
            r#"
fn foo() {
    let a = 1;
    fn closure(a: &i32) -> i32 {
        *a
    }
    closure(&a);
}
"#,
        );
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, fn
fn foo() {
    let closure = |$0| 'label: {};
    closure();
}
"#,
            r#"
fn foo() {
    fn closure() {
        'label: {}
    }
    closure();
}
"#,
        );
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, fn
fn foo() {
    let closure = |$0| {
        const { () }
    };
    closure();
}
"#,
            r#"
fn foo() {
    fn closure() {
        const { () }
    }
    closure();
}
"#,
        );
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, fn
fn foo() {
    let closure = |$0| unsafe { };
    closure();
}
"#,
            r#"
fn foo() {
    fn closure() {
        unsafe { }
    }
    closure();
}
"#,
        );
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, fn
fn foo() {
    {
        let closure = |$0| match () {
            () => {},
        };
        closure();
    }
}
"#,
            r#"
fn foo() {
    {
        fn closure() {
            match () {
                () => {},
            }
        }
        closure();
    }
}
"#,
        );
    }

    #[test]
    fn closure_in_closure() {
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy
fn foo() {
    let a = 1;
    || |$0| { let b = &a; };
}
"#,
            r#"
fn foo() {
    let a = 1;
    || { fn fun_name(a: &i32) { let b = a; } fun_name };
}
"#,
        );
    }

    #[test]
    fn closure_in_block() {
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy
fn foo() {
    {
        let a = 1;
        |$0| { let b = &a; }
    };
}
"#,
            r#"
fn foo() {
    {
        let a = 1;
        fn fun_name(a: &i32) { let b = a; }
        fun_name
    };
}
"#,
        );
    }

    #[test]
    fn finds_pat_for_expr() {
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, fn
struct A { b: B }
struct B(bool, i32);
fn foo() {
    let mut a = A { b: B(true, 0) };
    let closure = |$0| {
        let A { b: B(_, ref mut c) } = a;
    };
    closure();
}
"#,
            r#"
struct A { b: B }
struct B(bool, i32);
fn foo() {
    let mut a = A { b: B(true, 0) };
    fn closure(a_b_1: &mut i32) {
        let A { b: B(_, ref mut c) } = *a_b_1;
    }
    closure(&mut a.b.1);
}
"#,
        );
    }

    #[test]
    fn with_existing_params() {
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, fn
fn foo() {
    let (mut a, b) = (0.1, "abc");
    let closure = |$0p1: i32, p2: &mut bool| {
        a = 1.2;
        let c = b;
    };
    closure(0, &mut false);
}
"#,
            r#"
fn foo() {
    let (mut a, b) = (0.1, "abc");
    fn closure(p1: i32, p2: &mut bool, a: &mut f64, b: &&str) {
        *a = 1.2;
        let c = *b;
    }
    closure(0, &mut false, &mut a, &b);
}
"#,
        );
    }

    #[test]
    fn with_existing_params_newlines() {
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, fn
fn foo() {
    let (mut a, b) = (0.1, "abc");
    let closure = |$0p1: i32, p2| {
        let _: &mut bool = p2;
        a = 1.2;
        let c = b;
    };
    closure(
        0,
        &mut false
    );
}
"#,
            r#"
fn foo() {
    let (mut a, b) = (0.1, "abc");
    fn closure(p1: i32, p2: &mut bool, a: &mut f64, b: &&str) {
        let _: &mut bool = p2;
        *a = 1.2;
        let c = *b;
    }
    closure(
        0,
        &mut false,
        &mut a,
        &b
    );
}
"#,
        );
    }

    #[test]
    fn with_existing_params_trailing_comma() {
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, fn
fn foo() {
    let (mut a, b) = (0.1, "abc");
    let closure = |$0p1: i32, p2| {
        let _: &mut bool = p2;
        a = 1.2;
        let c = b;
    };
    closure(
        0,
        &mut false,
    );
}
"#,
            r#"
fn foo() {
    let (mut a, b) = (0.1, "abc");
    fn closure(p1: i32, p2: &mut bool, a: &mut f64, b: &&str) {
        let _: &mut bool = p2;
        *a = 1.2;
        let c = *b;
    }
    closure(
        0,
        &mut false,
        &mut a,
        &b,
    );
}
"#,
        );
    }

    #[test]
    fn closure_using_generic_params() {
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, from
struct Foo<A, B, const C: usize>(A, B);
impl<A, B: From<A>, const C: usize> Foo<A, B, C> {
    fn foo<D, E, F, G>(a: A, b: D)
    where
        E: From<D>,
    {
        let closure = |$0c: F| {
            let a = B::from(a);
            let b = E::from(b);
        };
    }
}
"#,
            r#"
struct Foo<A, B, const C: usize>(A, B);
impl<A, B: From<A>, const C: usize> Foo<A, B, C> {
    fn foo<D, E, F, G>(a: A, b: D)
    where
        E: From<D>,
    {
        fn closure<A, B: From<A>, D, E, F>(c: F, a: A, b: D) where E: From<D> {
            let a = B::from(a);
            let b = E::from(b);
        }
    }
}
"#,
        );
    }

    #[test]
    fn closure_in_stmt() {
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy
fn bar(_: impl FnOnce() -> i32) {}
fn foo() {
    let a = 123;
    bar(|$0| a);
}
"#,
            r#"
fn bar(_: impl FnOnce() -> i32) {}
fn foo() {
    let a = 123;
    fn fun_name(a: &i32) -> i32 {
        *a
    }
    bar(fun_name);
}
"#,
        );
    }

    #[test]
    fn unique_and_imm() {
        check_assist(
            convert_closure_to_fn,
            r#"
//- minicore: copy, fn
fn main() {
    let a = &mut true;
    let closure = |$0| {
        let b = &a;
        *a = false;
    };
    closure();
}
"#,
            r#"
fn main() {
    let a = &mut true;
    fn closure(a: &mut &mut bool) {
        let b = a;
        **a = false;
    }
    closure(&mut a);
}
"#,
        );
    }

    #[test]
    fn only_applicable_in_param_list() {
        check_assist_not_applicable(
            convert_closure_to_fn,
            r#"
//- minicore:copy
fn main() {
    let closure = || { $0 };
}
"#,
        );
        check_assist_not_applicable(
            convert_closure_to_fn,
            r#"
//- minicore:copy
fn main() {
    let $0closure = || { };
}
"#,
        );
    }
}
