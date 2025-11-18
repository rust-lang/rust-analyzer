use either::Either;
use hir::Semantics;
use ide_db::{RootDatabase, defs::Definition, search::FileReference};
use syntax::{
    AstNode, SyntaxKind, SyntaxNode, T, TextRange,
    algo::find_node_at_range,
    ast::{self, HasArgList, prec::ExprPrecedence, syntax_factory::SyntaxFactory},
    syntax_editor::{Element, Position, SyntaxEditor},
};

use crate::{AssistContext, AssistId, Assists, assist_context::SourceChangeBuilder};

// Assist: remove_reference
//
// Remove parameter reference.
//
// ```
// fn foo($0state: &bool) {
//     if *state {}
// }
// fn bar() {
//     let state = true;
//     foo(&state);
// }
// ```
// ->
// ```
// fn foo(state: bool) {
//     if state {}
// }
// fn bar() {
//     let state = true;
//     foo(state);
// }
// ```
pub(crate) fn remove_reference(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let param = ctx.find_node_at_offset::<ast::Param>()?;
    let param_list = ast::ParamList::cast(param.syntax().parent()?)?;
    let fn_ = ast::Fn::cast(param_list.syntax().parent()?)?;
    let param_nth = param_nth(&param, &param_list)?;
    let ty = param.ty()?;
    let pat = param.pat()?;
    let ast::Type::RefType(ref_ty) = ty else { return None };
    let amp_token = ref_ty.amp_token()?;
    let sema = &ctx.sema;

    if ref_ty.mut_token().is_some() || ctx.offset() > amp_token.text_range().end() {
        return None;
    }
    if type_size(sema, &ref_ty) == Some(0) {
        return None;
    }

    let fn_def = Definition::Function(sema.to_def(&fn_)?);

    acc.add(
        AssistId::refactor("remove_reference"),
        "Remove reference",
        amp_token.text_range(),
        |builder| {
            let mut edit = builder.make_editor(param.syntax());

            edit.delete(amp_token);

            let _ = remove_local_deref_usages(builder, sema, &pat);
            let _ = remove_call_usages_reference(builder, sema, &fn_def, param_nth);

            builder.add_file_edits(ctx.vfs_file_id(), edit);
        },
    )
}

fn remove_call_usages_reference(
    builder: &mut SourceChangeBuilder,
    sema: &Semantics<'_, RootDatabase>,
    fn_def: &Definition,
    param_nth: usize,
) -> Option<()> {
    let usages = fn_def.usages(sema).all();

    for (file_id, refs) in usages {
        let source_file = sema.parse(file_id);
        let file_id = file_id.file_id(sema.db);
        builder.edit_file(file_id);
        let mut edit = builder.make_editor(source_file.syntax());
        let make = SyntaxFactory::with_mappings();

        for FileReference { range, .. } in refs {
            match find_arg(source_file.syntax(), range, param_nth) {
                Some(ast::Expr::RefExpr(expr)) => {
                    let _ = delete_prefix(&mut edit, expr.amp_token());
                    let _ = delete_prefix(&mut edit, expr.mut_token());
                }
                Some(expr) => {
                    edit.insert(Position::before(expr.syntax()), make.token(T![*]));
                    if expr.precedence().needs_parentheses_in(ExprPrecedence::Prefix) {
                        edit.replace(expr.syntax().clone(), make.expr_paren(expr).syntax());
                    }
                }
                _ => (),
            }
        }

        edit.add_mappings(make.finish_with_mappings());
        builder.add_file_edits(file_id, edit);
    }

    Some(())
}

fn find_arg(syntax: &SyntaxNode, range: TextRange, param_nth: usize) -> Option<ast::Expr> {
    let call: Either<ast::CallExpr, ast::MethodCallExpr> = find_node_at_range(syntax, range)?;

    match call {
        Either::Left(call_expr) => call_expr.arg_list()?.args().nth(param_nth),
        Either::Right(method_call_expr) => {
            method_call_expr.arg_list()?.args().nth(param_nth.checked_sub(1)?)
        }
    }
}

fn param_nth(param: &ast::Param, param_list: &ast::ParamList) -> Option<usize> {
    let base = param_list.self_param().map_or(0, |_| 1);
    Some(base + param_list.params().position(|it| it == *param)?)
}

fn remove_local_deref_usages(
    builder: &mut SourceChangeBuilder,
    sema: &Semantics<'_, RootDatabase>,
    pat: &ast::Pat,
) -> Option<()> {
    let ast::Pat::IdentPat(ident_pat) = pat else { return None };
    let param_def = Definition::Local(sema.to_def(ident_pat)?);

    for (file_id, refs) in param_def.usages(sema).all() {
        let source_file = sema.parse(file_id);
        let file_id = file_id.file_id(sema.db);
        builder.edit_file(file_id);
        let mut edit = builder.make_editor(source_file.syntax());

        for FileReference { range, .. } in refs {
            if let Some(expr) = find_node_at_range::<ast::PrefixExpr>(source_file.syntax(), range)
                && expr.op_kind() == Some(ast::UnaryOp::Deref)
            {
                let _ = delete_prefix(&mut edit, expr.op_token());
            }
        }

        builder.add_file_edits(file_id, edit);
    }

    Some(())
}

fn delete_prefix(edit: &mut SyntaxEditor, token: Option<impl Element>) -> Option<()> {
    let op_token = token?.syntax_element();

    if let Some(next) = op_token.next_sibling_or_token()
        && next.kind() == SyntaxKind::WHITESPACE
    {
        edit.delete(next);
    }
    edit.delete(op_token);

    Some(())
}

fn type_size(sema: &Semantics<'_, RootDatabase>, ty: &ast::RefType) -> Option<u64> {
    Some(sema.resolve_type(&ty.ty()?)?.layout(sema.db).ok()?.size())
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn test_remove_reference() {
        check_assist(
            remove_reference,
            "
fn foo(n: i32, $0state: &bool) {
    if *state {}
}
fn bar() {
    let state = true;
    foo(5, &state);
}
            ",
            "
fn foo(n: i32, state: bool) {
    if state {}
}
fn bar() {
    let state = true;
    foo(5, state);
}
            ",
        );
    }

    #[test]
    fn test_remove_reference_add_deref() {
        check_assist(
            remove_reference,
            "
fn foo(n: i32, $0state: &bool) {
    if *state {}
}
fn cond() -> &'static bool { &true }
fn bar() {
    foo(5, cond());
}
            ",
            "
fn foo(n: i32, state: bool) {
    if state {}
}
fn cond() -> &'static bool { &true }
fn bar() {
    foo(5, *cond());
}
            ",
        );

        check_assist(
            remove_reference,
            "
fn foo(n: i32, $0state: &SomeType) {
    if *state {}
}
fn bar() {
    foo(5, x + y);
}
            ",
            "
fn foo(n: i32, state: SomeType) {
    if state {}
}
fn bar() {
    foo(5, *(x + y));
}
            ",
        );
    }

    #[test]
    fn test_remove_reference_with_self_param() {
        check_assist(
            remove_reference,
            "
struct Foo;
impl Foo {
    fn foo(self, n: i32, state: &$0bool; m: i32) {
        if *state {}
    }
}
fn bar() {
    let state = true;
    Foo.foo(5, &state, 6);
    Foo::foo(Foo, 5, &state, 6);
}
            ",
            "
struct Foo;
impl Foo {
    fn foo(self, n: i32, state: bool; m: i32) {
        if *state {}
    }
}
fn bar() {
    let state = true;
    Foo.foo(5, state, 6);
    Foo::foo(Foo, 5, state, 6);
}
            ",
        );
    }

    #[test]
    fn test_remove_reference_not_applicable_with_other_type() {
        check_assist_not_applicable(
            remove_reference,
            "
fn foo(n: i32, $0state: bool) {
    if *state {}
}
fn bar() {
    let state = true;
    foo(5, &state);
}
            ",
        );

        check_assist_not_applicable(
            remove_reference,
            "
fn foo(n: i32, $0state: &mut bool) {
    if *state {}
}
fn bar() {
    let mut state = true;
    foo(5, &mut state);
}
            ",
        );
    }

    #[test]
    fn test_remove_reference_not_applicable_after_amp() {
        check_assist_not_applicable(
            remove_reference,
            "
fn foo(n: i32, state: &b$0ool) {
    if *state {}
}
fn bar() {
    let state = true;
    foo(5, &state);
}
            ",
        );
    }

    #[test]
    fn test_remove_reference_not_applicable_dst() {
        check_assist_not_applicable(
            remove_reference,
            r#"
//- minicore: str
fn foo(n: i32, $0state: &str) {
    if state.is_empty() {}
}
fn bar() {
    foo(5, "x");
}
            "#,
        );
    }
}
