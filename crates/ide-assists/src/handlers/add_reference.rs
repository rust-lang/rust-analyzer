use std::ops::ControlFlow;

use either::Either;
use ide_db::{defs::Definition, syntax_helpers::node_ext::walk_pat};
use syntax::{
    AstNode, SyntaxKind, SyntaxNode, T, TextRange,
    algo::find_node_at_range,
    ast::{self, HasArgList, prec::ExprPrecedence, syntax_factory::SyntaxFactory},
    syntax_editor::{Element, Position, SyntaxEditor},
};

use crate::{AssistContext, AssistId, Assists};

// Assist: add_reference
//
// Add reference for function parameter.
//
// ```
// fn foo(arg0: i32, $0arg1: [i32; 32]) {}
// fn bar() {
//     foo(5, [8; 32])
// }
// ```
// ->
// ```
// fn foo(arg0: i32, arg1: &[i32; 32]) {}
// fn bar() {
//     foo(5, &[8; 32])
// }
// ```
//
// ---
//
// ```
// fn foo(arg0: i32, mut $0arg1: [i32; 32]) {}
// fn bar() {
//     foo(5, [8; 32])
// }
// ```
// ->
// ```
// fn foo(arg0: i32, arg1: &mut [i32; 32]) {}
// fn bar() {
//     foo(5, &mut [8; 32])
// }
// ```
pub(crate) fn add_reference(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let param = ctx.find_node_at_offset::<ast::Param>()?;
    let pat = param.pat()?;
    let ty = param.ty()?;
    let param_list = ast::ParamList::cast(param.syntax().parent()?)?;
    let fn_ = ast::Fn::cast(param_list.syntax().parent()?)?;

    let mut param_nth = param_list.params().position(|it| it == param)?;
    if param_list.self_param().is_some() {
        param_nth += 1;
    }

    let ControlFlow::Break(name) = walk_pat(&pat, &mut |pat| {
        if let ast::Pat::IdentPat(it) = pat {
            return ControlFlow::Break(it);
        }
        ControlFlow::Continue(())
    }) else {
        return None;
    };
    let mutable = name.mut_token().is_some();

    if ctx.offset() > pat.syntax().text_range().end()
        || matches!(&ty, ast::Type::RefType(ty) if (ty.mut_token().is_some(), mutable) != (false, true))
    {
        return None;
    }

    let sema = &ctx.sema;
    let fn_def = Definition::Function(sema.to_def(&fn_)?);

    acc.add(
        AssistId::refactor("add_reference"),
        format!("Add {}reference for parameter", if mutable { "mutable " } else { "" }),
        pat.syntax().text_range(),
        |builder| {
            let mut edit = builder.make_editor(ty.syntax());
            let make = SyntaxFactory::with_mappings();

            if mutable {
                insert_mut(&ty, &mut edit, &make);
            }
            edit.insert(Position::before(ty.syntax()), make.token(T![&]));
            if let Some(mut_token) = name.mut_token() {
                delete_mut(&mut edit, mut_token);
            }

            for (file_id, refs) in fn_def.usages(sema).all() {
                let source_file = sema.parse(file_id);
                let file_id = file_id.file_id(sema.db);
                builder.edit_file(file_id);
                let mut edit = builder.make_editor(source_file.syntax());
                let make = SyntaxFactory::with_mappings();

                for reference in refs {
                    if let Some(arg) = find_arg(source_file.syntax(), reference.range, param_nth) {
                        let _ = process_usage(&mut edit, &make, arg, mutable);
                    }
                }

                edit.add_mappings(make.finish_with_mappings());
                builder.add_file_edits(file_id, edit);
            }

            edit.add_mappings(make.finish_with_mappings());
            builder.add_file_edits(ctx.vfs_file_id(), edit);
        },
    )
}

fn find_arg(node: &SyntaxNode, reference_range: TextRange, param_nth: usize) -> Option<ast::Expr> {
    let call: Either<ast::CallExpr, ast::MethodCallExpr> =
        find_node_at_range(node, reference_range)?;

    match call {
        Either::Left(call_expr) => call_expr.arg_list()?.args().nth(param_nth),
        Either::Right(method_call_expr) => {
            method_call_expr.arg_list()?.args().nth(param_nth.checked_sub(1)?)
        }
    }
}

fn process_usage(
    edit: &mut SyntaxEditor,
    make: &SyntaxFactory,
    arg: ast::Expr,
    mutable: bool,
) -> Option<()> {
    if mutable {
        insert_mut(&arg, edit, make);
    }
    edit.insert(Position::before(arg.syntax()), make.token(T![&]));
    if arg.precedence().needs_parentheses_in(ExprPrecedence::Prefix) {
        let paren_expr = make.expr_paren(arg.clone());
        edit.replace(arg.syntax(), paren_expr.syntax());
    }
    Some(())
}

fn insert_mut(node: &impl AstNode, edit: &mut SyntaxEditor, make: &SyntaxFactory) {
    edit.insert_all(
        Position::before(node.syntax()),
        vec![make.token(T![mut]).into(), make.whitespace(" ").into()],
    );
}

fn delete_mut(edit: &mut SyntaxEditor, token: impl Element) {
    let element = token.syntax_element();

    if let Some(next_token) = element.next_sibling_or_token()
        && next_token.kind() == SyntaxKind::WHITESPACE
    {
        edit.delete(next_token);
    }
    edit.delete(element);
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn test_add_reference() {
        check_assist(
            add_reference,
            "
fn foo(arg0: i32, $0arg1: [i32; 32]) {}
fn bar() {
    foo(5, [8; 32])
}
            ",
            "
fn foo(arg0: i32, arg1: &[i32; 32]) {}
fn bar() {
    foo(5, &[8; 32])
}
            ",
        );
    }

    #[test]
    fn test_add_mutable_reference() {
        check_assist(
            add_reference,
            "
fn foo(arg0: i32, $0mut arg1: [i32; 32]) {}
fn bar() {
    foo(5, [8; 32])
}
            ",
            "
fn foo(arg0: i32, arg1: &mut [i32; 32]) {}
fn bar() {
    foo(5, &mut [8; 32])
}
            ",
        );
    }

    #[test]
    fn test_add_mutable_reference_reference() {
        check_assist(
            add_reference,
            "
fn foo(arg0: i32, $0mut arg1: &[i32; 32]) {}
fn bar() {
    foo(5, &[8; 32])
}
            ",
            "
fn foo(arg0: i32, arg1: &mut &[i32; 32]) {}
fn bar() {
    foo(5, &mut &[8; 32])
}
            ",
        );
    }

    #[test]
    fn test_add_reference_with_self_param() {
        check_assist(
            add_reference,
            "
struct Foo;
impl Foo {
    fn foo(self, arg0: i32, $0arg1: [i32; 32]) {}
}
fn bar() {
    Foo.foo(5, [8; 32]);
    Foo::foo(Foo, 5, [8; 32]);
}
            ",
            "
struct Foo;
impl Foo {
    fn foo(self, arg0: i32, arg1: &[i32; 32]) {}
}
fn bar() {
    Foo.foo(5, &[8; 32]);
    Foo::foo(Foo, 5, &[8; 32]);
}
            ",
        );
    }

    #[test]
    fn test_add_reference_add_paren() {
        check_assist(
            add_reference,
            "
struct Foo;
impl Foo {
    fn foo(self, arg0: i32, $0arg1: i32) {}
}
fn bar() {
    Foo.foo(5, 8+2);
    Foo::foo(Foo, 5, 8*3);
}
            ",
            "
struct Foo;
impl Foo {
    fn foo(self, arg0: i32, arg1: &i32) {}
}
fn bar() {
    Foo.foo(5, &(8+2));
    Foo::foo(Foo, 5, &(8*3));
}
            ",
        );
    }

    #[test]
    fn test_add_reference_not_applicable_ref_type() {
        check_assist_not_applicable(
            add_reference,
            "
fn foo(arg0: i32, $0arg1: &[i32; 32]) {}
fn bar() {
    foo(5, &[8; 32])
}
            ",
        );

        check_assist_not_applicable(
            add_reference,
            "
fn foo(arg0: i32, $0arg1: &mut [i32; 32]) {}
fn bar() {
    foo(5, &mut [8; 32])
}
            ",
        );

        check_assist_not_applicable(
            add_reference,
            "
fn foo(arg0: i32, mut $0arg1: &mut [i32; 32]) {}
fn bar() {
    foo(5, &mut [8; 32])
}
            ",
        );
    }
}
