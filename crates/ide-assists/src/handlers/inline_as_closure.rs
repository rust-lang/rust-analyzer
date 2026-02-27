use ast::make;
use hir::{EditionedFileId, PathResolution, Semantics};
use ide_db::{
    RootDatabase,
    defs::Definition,
    path_transform::PathTransform,
    search::{FileReference, FileReferenceNode},
};
use syntax::{
    AstNode,
    ast::{self, HasGenericArgs, edit::IndentLevel, edit_in_place::Indent},
    ted,
};

use crate::{
    AssistId,
    assist_context::{AssistContext, Assists},
};

// Assist: inline_as_closure
//
// Inline non calling function as closure.
//
// ```
// fn foo() { println!("Hello, World!"); }
// fn main() {
//     let _ = foo$0;
// }
// ```
// ->
// ```
// fn foo() { println!("Hello, World!"); }
// fn main() {
//     let _ = || { println!("Hello, World!"); };
// }
// ```
pub(crate) fn inline_as_closure(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let path: ast::PathExpr = ctx.find_node_at_offset()?;
    let indent_level = path.indent_level();

    if ast::CallExpr::can_cast(path.syntax().parent()?.kind()) {
        cov_mark::hit!(inline_as_closure_call_expr);
        return None;
    }

    let sema = &ctx.sema;
    let PathResolution::Def(hir::ModuleDef::Function(func)) = sema.resolve_path(&path.path()?)?
    else {
        return None;
    };

    let source = sema.source(func)?;
    let param_list = source.value.param_list()?;
    let body = source.value.body()?.clone_for_update();
    let body = rename_usages(func, body, &path, ctx.file_id(), sema)?;

    let target = path.syntax().text_range();
    acc.add(
        AssistId::refactor_inline("inline_as_closure"),
        format!("Inline `{path}` to closure"),
        target,
        |builder| {
            let mut edit = builder.make_editor(path.syntax());

            let param = process_params(param_list);

            let closure = make::expr_closure(param, process_body(body, indent_level));
            edit.replace(path.syntax(), closure.syntax().clone_for_update());

            builder.add_file_edits(ctx.vfs_file_id(), edit);
        },
    )
}

fn process_params(param_list: ast::ParamList) -> impl Iterator<Item = ast::Param> {
    let this = make::untyped_param(make::path_pat(make::path_from_text("this")));
    let wildcard = make::wildcard_pat();

    param_list.self_param().map(|_| this).into_iter().chain(
        param_list
            .params()
            .map(move |param| param.pat().unwrap_or_else(|| wildcard.clone().into()))
            .map(make::untyped_param),
    )
}

fn rename_usages(
    func: hir::Function,
    body: ast::BlockExpr,
    path: &ast::PathExpr,
    file_id: EditionedFileId,
    sema: &Semantics<'_, RootDatabase>,
) -> Option<ast::BlockExpr> {
    if func.self_param(sema.db).is_some() {
        let self_param = func.assoc_fn_params(sema.db)[0].as_local(sema.db)?;
        let this = make::name_ref("this");

        Definition::Local(self_param)
            .usages(sema)
            .all()
            .references
            .remove(&file_id)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|FileReference { name, range, .. }| match name {
                FileReferenceNode::NameRef(_) => Some(body.syntax().covering_element(range)),
                _ => None,
            })
            .for_each(|usage| ted::replace(usage, this.syntax().clone_for_update()));
    }

    if let Some(generic_arg_list) = path.path()?.segment()?.generic_arg_list()
        && let Some((target, source)) = sema.scope(path.syntax()).zip(sema.scope(body.syntax()))
    {
        let new_node = PathTransform::function_call(&target, &source, func, generic_arg_list)
            .apply(body.syntax());
        return AstNode::cast(new_node);
    }

    Some(body)
}

fn process_body(body: ast::BlockExpr, indent_level: IndentLevel) -> ast::Expr {
    match body.tail_expr() {
        Some(tail_expr) if body.statements().next().is_none() => {
            tail_expr.reindent_to(indent_level);
            tail_expr
        }
        _ => {
            body.reindent_to(indent_level);
            body.into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{check_assist, check_assist_not_applicable};

    #[test]
    fn inline_as_closure_basic() {
        check_assist(
            inline_as_closure,
            r#"
fn foo() { println!("Hello, World!"); }
fn main() {
    let _ = foo$0;
}
        "#,
            r#"
fn foo() { println!("Hello, World!"); }
fn main() {
    let _ = || { println!("Hello, World!"); };
}
        "#,
        );
    }

    #[test]
    fn inline_as_closure_tail_expr() {
        check_assist(
            inline_as_closure,
            r#"
fn foo() -> i32 { 2 }
fn main() {
    let _ = foo$0;
}
        "#,
            r#"
fn foo() -> i32 { 2 }
fn main() {
    let _ = || 2;
}
        "#,
        );
    }

    #[test]
    fn inline_as_closure_params() {
        check_assist(
            inline_as_closure,
            r#"
fn foo(x: i32) -> i32 { x+2 }
fn main() {
    let _ = foo$0;
}
        "#,
            r#"
fn foo(x: i32) -> i32 { x+2 }
fn main() {
    let _ = |x| x+2;
}
        "#,
        );
    }

    #[test]
    fn inline_as_closure_self_params() {
        check_assist(
            inline_as_closure,
            r#"
struct Foo(i32);
impl Foo {
    fn foo(&self, x: i32) -> i32 { x+self.0 }
}
fn main() {
    let _ = Foo::foo$0;
}
        "#,
            r#"
struct Foo(i32);
impl Foo {
    fn foo(&self, x: i32) -> i32 { x+self.0 }
}
fn main() {
    let _ = |this, x| x+this.0;
}
        "#,
        );
    }

    #[test]
    fn inline_as_closure_with_indent() {
        check_assist(
            inline_as_closure,
            r#"
fn foo() -> i32 {
    println!("foo");
    2
}
fn main() {
    {
        let _ = foo$0;
    }
}
        "#,
            r#"
fn foo() -> i32 {
    println!("foo");
    2
}
fn main() {
    {
        let _ = || {
            println!("foo");
            2
        };
    }
}
        "#,
        );

        check_assist(
            inline_as_closure,
            r#"
mod a {
    pub mod b {
        pub fn foo() -> i32 {
            println!("foo");
            2
        }
    }
}
use a::b::foo;
fn main() {
    {
        let _ = foo$0;
    }
}
        "#,
            r#"
mod a {
    pub mod b {
        pub fn foo() -> i32 {
            println!("foo");
            2
        }
    }
}
use a::b::foo;
fn main() {
    {
        let _ = || {
            println!("foo");
            2
        };
    }
}
        "#,
        );

        check_assist(
            inline_as_closure,
            r#"
mod a {
    pub mod b {
        pub fn foo() -> i32 {
            {
                2
            }
        }
    }
}
fn main() {
    {
        let _ = a::b::foo$0;
    }
}
        "#,
            r#"
mod a {
    pub mod b {
        pub fn foo() -> i32 {
            {
                2
            }
        }
    }
}
fn main() {
    {
        let _ = || {
            2
        };
    }
}
        "#,
        );
    }

    #[test]
    fn inline_as_closure_generic_args() {
        check_assist(
            inline_as_closure,
            r#"
fn foo<const N: i32>() -> i32 { N }
fn main() {
    let _ = foo$0;
}
        "#,
            r#"
fn foo<const N: i32>() -> i32 { N }
fn main() {
    let _ = || N;
}
        "#,
        );

        check_assist(
            inline_as_closure,
            r#"
fn foo<const N: usize>() -> usize { N }
fn main() {
    let _ = foo::<2>$0;
}
        "#,
            r#"
fn foo<const N: usize>() -> usize { N }
fn main() {
    let _ = || 2;
}
        "#,
        );

        check_assist(
            inline_as_closure,
            r#"
fn bar<T, const N: usize>() -> usize { N }
fn foo<T, const N: usize>() -> usize { bar::<T, N>() }
fn main() {
    let _ = foo::<usize, 2>$0;
}
        "#,
            r#"
fn bar<T, const N: usize>() -> usize { N }
fn foo<T, const N: usize>() -> usize { bar::<T, N>() }
fn main() {
    let _ = || bar::<usize, 2>();
}
        "#,
        );
    }

    #[test]
    fn inline_as_closure_not_applicate_call() {
        cov_mark::check!(inline_as_closure_call_expr);
        check_assist_not_applicable(
            inline_as_closure,
            r#"
fn foo() -> i32 { 2 }
fn main() {
    let _ = foo$0();
}
        "#,
        );
    }
}
