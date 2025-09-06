use either::Either;
use hir::{GenericDef, HirDisplay, Module, TypeInfo};
use ide_db::{FileId, FxHashSet, helpers::is_editable_crate, source_change::SourceChangeBuilder};
use syntax::{
    SyntaxKind, SyntaxNode, SyntaxToken, TextRange,
    ast::{self, AstNode, HasArgList, HasModuleItem, edit::IndentLevel, make},
    syntax_editor::{Position, SyntaxEditor},
};

use crate::{AssistContext, AssistId, Assists};

// Assist: generate_struct
//
// Adds a stub struct with a definition matching the expression under the cursor.
//
// ```
// fn foo() {
//     let bar = Bar$0 { x: 2, y: 3, force: true };
// }
// ```
// ->
// ```
// fn foo() {
//     let bar = Bar { x: 2, y: 3, force: true };
// }
//
// struct Bar { x: i32, y: i32, force: bool }
// ```
pub(crate) fn generate_struct(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let path = ctx.find_node_at_offset::<ast::Path>()?;
    let name = path.segment()?.name_ref()?.ident_token()?;

    if ctx.sema.resolve_path(&path).is_some() {
        return None;
    }

    let (target, fields) = find_target(&path)?;
    let (insert, file_id, module) = target_info(ctx, &path)?;
    if let Some(module) = &module
        && !is_editable_crate(module.krate(), ctx.db())
    {
        return None;
    }
    let scope = ctx.sema.scope(path.syntax())?;

    acc.add(
        AssistId::generate("generate_struct"),
        format!("Generate {name} struct"),
        target,
        |builder| {
            builder.edit_file(file_id);

            let vis = make_vis(module, &scope);
            let mut generic_params = FxHashSet::default();

            let field_list = match fields {
                Either::Left(fields) => {
                    make::record_field_list(fields.into_iter().map(|(field_name, expr)| {
                        let ty =
                            make_type(ctx.sema.type_of_expr(&expr), &scope, &mut generic_params);
                        make::record_field(vis.clone(), field_name, ty)
                    }))
                    .into()
                }
                Either::Right(fields) => make::tuple_field_list(fields.into_iter().map(|expr| {
                    let ty = make_type(ctx.sema.type_of_expr(&expr), &scope, &mut generic_params);
                    make::tuple_field(vis.clone(), ty)
                }))
                .into(),
            };
            let generic_param_list =
                Some(make::generic_param_list(generic_params.iter().filter_map(|param| {
                    param
                        .display_source_code(ctx.db(), scope.module().into(), false)
                        .ok()
                        .map(|param| make::generic_param(&param))
                })))
                .filter(|_| !generic_params.is_empty());
            let strukt =
                make::struct_(vis, make::name(name.text()), generic_param_list, field_list);

            let edit = insert.edit(builder, &strukt);
            builder.add_file_edits(file_id, edit);
        },
    )
}

fn find_target(
    path: &ast::Path,
) -> Option<(TextRange, Either<Vec<(ast::Name, ast::Expr)>, Vec<ast::Expr>>)> {
    let (range, fields) =
        match path.syntax().parent().and_then(Either::<ast::RecordExpr, ast::PathExpr>::cast)? {
            Either::Left(record_expr) => {
                if record_expr.record_expr_field_list()?.spread().is_some() {
                    return None;
                }
                let fields = record_expr
                    .record_expr_field_list()?
                    .fields()
                    .map(|field| {
                        field
                            .field_name()
                            .and_then(|name| Some(make::name(name.ident_token()?.text())))
                            .zip(field.expr())
                    })
                    .collect::<Option<_>>()?;
                (record_expr.syntax().text_range(), Either::Left(fields))
            }
            Either::Right(path_expr) => {
                let call_expr = ast::CallExpr::cast(path_expr.syntax().parent()?)?;
                let _ = call_expr.arg_list()?.args().next()?;
                let fields = call_expr.arg_list()?.args().collect();
                (call_expr.syntax().text_range(), Either::Right(fields))
            }
        };
    Some((range, fields))
}

fn make_vis(module: Option<Module>, scope: &hir::SemanticsScope<'_>) -> Option<ast::Visibility> {
    let target_module = module?;

    if scope.module().krate() != target_module.krate() {
        Some(make::visibility_pub())
    } else if !scope.module().path_to_root(scope.db).contains(&target_module) {
        Some(make::visibility_pub_crate())
    } else {
        None
    }
}

fn make_type(
    ty: Option<TypeInfo<'_>>,
    scope: &hir::SemanticsScope<'_>,
    generic_params: &mut FxHashSet<hir::GenericParam>,
) -> ast::Type {
    let Some(ty) = ty.map(TypeInfo::original) else {
        return make::ty_placeholder();
    };
    // FIXME: reference is not adt, unable generate it lifetime parameter
    if let Some(adt) = ty.as_adt() {
        let lifetimes = GenericDef::Adt(adt).lifetime_params(scope.db);
        generic_params.extend(lifetimes.into_iter().map(hir::GenericParam::LifetimeParam));
    }
    generic_params.extend(ty.generic_params(scope.db).into_iter().filter(|it| {
        !matches!(it, hir::GenericParam::TypeParam(it)
                if it.is_implicit(scope.db))
    }));
    // FIXME: display_source_code infer lifetime cannot associated to generic_params
    match ty.display_source_code(scope.db, scope.module().into(), false) {
        Ok(ty) => make::ty(&ty),
        Err(_) => make::ty_placeholder(),
    }
}

fn target_info(
    ctx: &AssistContext<'_>,
    path: &ast::Path,
) -> Option<(InsertMethod, FileId, Option<Module>)> {
    let Some(qualifier) = path.qualifier() else {
        return Some((
            InsertMethod::AfterLast(find_after_module_item(path)?),
            ctx.vfs_file_id(),
            None,
        ));
    };
    if let Some(hir::PathResolution::Def(hir::ModuleDef::Module(module))) =
        ctx.sema.resolve_path(&qualifier)
    {
        let def_src = module.definition_source(ctx.db());
        let file_id = def_src.file_id.original_file(ctx.db()).file_id(ctx.db());
        let insert_method = match def_src.value {
            hir::ModuleSource::SourceFile(it) => InsertMethod::AfterLast(
                it.items().last().as_ref().map_or(it.syntax(), |it| it.syntax()).clone(),
            ),
            hir::ModuleSource::Module(it) => it
                .item_list()
                .and_then(|it| it.items().last())
                .map(|it| InsertMethod::AfterLast(it.syntax().clone()))
                .or_else(|| it.item_list()?.l_curly_token().map(InsertMethod::AfterFirst))?,
            hir::ModuleSource::BlockExpr(it) => it
                .statements()
                .take_while(|it| matches!(it, ast::Stmt::Item(_)))
                .last()
                .map(|it| InsertMethod::AfterLast(it.syntax().clone()))
                .or_else(|| it.stmt_list()?.l_curly_token().map(InsertMethod::AfterFirst))?,
        };
        Some((insert_method, file_id, Some(module)))
    } else {
        None
    }
}

fn find_after_module_item(node: &impl AstNode) -> Option<SyntaxNode> {
    node.syntax().ancestors().find(|it| {
        it.parent().is_some_and(|parent| match parent.kind() {
            SyntaxKind::SOURCE_FILE => true,
            SyntaxKind::ITEM_LIST => {
                parent.parent().is_some_and(|it| it.kind() == SyntaxKind::MODULE)
            }
            _ => false,
        })
    })
}

enum InsertMethod {
    AfterFirst(SyntaxToken),
    AfterLast(SyntaxNode),
}

impl InsertMethod {
    fn edit(&self, builder: &mut SourceChangeBuilder, strukt: &impl AstNode) -> SyntaxEditor {
        match self {
            InsertMethod::AfterFirst(token) => {
                let node = token.parent().expect("insert token must exist parent");
                let mut edit = builder.make_editor(&node);
                let indent = IndentLevel::from_node(&node);
                edit.insert_all(
                    Position::after(token),
                    vec![
                        make::tokens::whitespace(&format!("\n{}", indent + 1)).into(),
                        strukt.syntax().clone_for_update().into(),
                        make::tokens::whitespace(&format!("\n{indent}")).into(),
                    ],
                );
                edit
            }
            InsertMethod::AfterLast(node) => {
                let mut edit = builder.make_editor(node);
                let indent = IndentLevel::from_node(node);
                edit.insert_all(
                    Position::after(node),
                    vec![
                        make::tokens::whitespace(&format!("\n\n{indent}")).into(),
                        strukt.syntax().clone_for_update().into(),
                    ],
                );
                edit
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn test_generate_struct() {
        check_assist(
            generate_struct,
            "
            fn foo() {
                let _ = $0Foo {
                    x: 2,
                    y: 3.2,
                };
            }
            ",
            "
            fn foo() {
                let _ = Foo {
                    x: 2,
                    y: 3.2,
                };
            }

            struct Foo { x: i32, y: f64 }
            ",
        );

        check_assist(
            generate_struct,
            "
            fn foo() {
                let _ = $0Foo {};
            }
            ",
            "
            fn foo() {
                let _ = Foo {};
            }

            struct Foo {  }
            ",
        );
    }

    #[test]
    fn test_generate_tuple_struct() {
        check_assist(
            generate_struct,
            "
            fn foo() {
                let _ = $0Foo(2, 3.5);
            }
            ",
            "
            fn foo() {
                let _ = Foo(2, 3.5);
            }

            struct Foo(i32, f64);
            ",
        );
    }

    #[test]
    fn test_generate_struct_generics() {
        check_assist(
            generate_struct,
            "
            //- minicore: sized
            fn foo<T, U>(x: T, _y: U) {
                let _ = $0Foo {
                    x,
                    y: 3.2,
                };
            }
            ",
            "
            fn foo<T, U>(x: T, _y: U) {
                let _ = Foo {
                    x,
                    y: 3.2,
                };
            }

            struct Foo<T> { x: T, y: f64 }
            ",
        );

        check_assist(
            generate_struct,
            "
            //- minicore: copy
            fn foo<T: Copy, U>(x: T, _y: U) {
                let _ = $0Foo {
                    x,
                    y: 3.2,
                };
            }
            ",
            "
            fn foo<T: Copy, U>(x: T, _y: U) {
                let _ = Foo {
                    x,
                    y: 3.2,
                };
            }

            struct Foo<T: Copy> { x: T, y: f64 }
            ",
        );

        check_assist(
            generate_struct,
            "
            //- minicore: copy
            fn foo(x: impl Copy) {
                let _ = $0Foo {
                    x,
                    y: 3.2,
                };
            }
            ",
            "
            fn foo(x: impl Copy) {
                let _ = Foo {
                    x,
                    y: 3.2,
                };
            }

            struct Foo { x: impl Copy, y: f64 }
            ",
        );

        check_assist(
            generate_struct,
            "
            //- minicore: copy
            struct Bar<'a, 'b>(&'a &'b ());
            fn foo(x: Bar<'_, '_>) {
                let _ = $0Foo {
                    x,
                    y: 3.2,
                };
            }
            ",
            "
            struct Bar<'a, 'b>(&'a &'b ());
            fn foo(x: Bar<'_, '_>) {
                let _ = Foo {
                    x,
                    y: 3.2,
                };
            }

            struct Foo<'b, 'a> { x: Bar<'_, '_>, y: f64 }
            ",
        );

        check_assist(
            generate_struct,
            "
            //- minicore: copy
            fn foo(x: &str) {
                let _ = $0Foo {
                    x,
                    y: 3.2,
                };
            }
            ",
            "
            fn foo(x: &str) {
                let _ = Foo {
                    x,
                    y: 3.2,
                };
            }

            struct Foo { x: &str, y: f64 }
            ",
        );
    }

    #[test]
    fn test_generate_struct_indent() {
        check_assist(
            generate_struct,
            "
            mod indent {
                fn foo() {
                    let _ = $0Foo {
                        x: 2,
                        y: 3.2,
                    };
                }
            }
            ",
            "
            mod indent {
                fn foo() {
                    let _ = Foo {
                        x: 2,
                        y: 3.2,
                    };
                }

                struct Foo { x: i32, y: f64 }
            }
            ",
        );

        check_assist(
            generate_struct,
            "
            mod indent {
                fn foo() {
                    let _ = bar::$0Foo {
                        x: 2,
                        y: 3.2,
                    };
                }

                mod bar {}
            }
            ",
            "
            mod indent {
                fn foo() {
                    let _ = bar::Foo {
                        x: 2,
                        y: 3.2,
                    };
                }

                mod bar {
                    pub(crate) struct Foo { pub(crate) x: i32, pub(crate) y: f64 }
                }
            }
            ",
        );

        check_assist(
            generate_struct,
            "
            mod indent {
                fn foo() {
                    let _ = bar::$0Foo {
                        x: 2,
                        y: 3.2,
                    };
                }

                mod bar {
                    fn _some_item() {}
                }
            }
            ",
            "
            mod indent {
                fn foo() {
                    let _ = bar::Foo {
                        x: 2,
                        y: 3.2,
                    };
                }

                mod bar {
                    fn _some_item() {}

                    pub(crate) struct Foo { pub(crate) x: i32, pub(crate) y: f64 }
                }
            }
            ",
        );
    }

    #[test]
    fn test_generate_struct_in_other_module() {
        check_assist(
            generate_struct,
            "
            fn foo() {
                let _ = foo::$0Foo {
                    x: 2,
                    y: 3.2,
                };
            }
            mod foo {
                fn _some_item() {}
            }
            ",
            "
            fn foo() {
                let _ = foo::Foo {
                    x: 2,
                    y: 3.2,
                };
            }
            mod foo {
                fn _some_item() {}

                pub(crate) struct Foo { pub(crate) x: i32, pub(crate) y: f64 }
            }
            ",
        );

        check_assist(
            generate_struct,
            "
            fn foo() {
                let _ = foo::$0Foo {
                    x: 2,
                    y: 3.2,
                };
            }
            mod foo {}
            ",
            "
            fn foo() {
                let _ = foo::Foo {
                    x: 2,
                    y: 3.2,
                };
            }
            mod foo {
                pub(crate) struct Foo { pub(crate) x: i32, pub(crate) y: f64 }
            }
            ",
        );

        check_assist(
            generate_struct,
            "
            mod foo {
                fn foo() {
                    let _ = crate::foo::$0Foo {
                        x: 2,
                        y: 3.2,
                    };
                }
            }
            ",
            "
            mod foo {
                fn foo() {
                    let _ = crate::foo::Foo {
                        x: 2,
                        y: 3.2,
                    };
                }

                struct Foo { x: i32, y: f64 }
            }
            ",
        );

        check_assist(
            generate_struct,
            "
            mod foo {
                fn foo() {
                    let _ = crate::$0Foo {
                        x: 2,
                        y: 3.2,
                    };
                }
            }
            ",
            "
            mod foo {
                fn foo() {
                    let _ = crate::Foo {
                        x: 2,
                        y: 3.2,
                    };
                }
            }

            struct Foo { x: i32, y: f64 }
            ",
        );
    }

    #[test]
    fn generate_struct_applicable_in_other_crate() {
        check_assist(
            generate_struct,
            r"
//- /lib.rs crate:lib new_source_root:local
fn dummy() {}
//- /main.rs crate:main deps:lib new_source_root:local
fn main() {
    lib::Foo$0 { x: 2 };
}
",
            r"
fn dummy() {}

pub struct Foo { pub x: i32 }
",
        );
    }

    #[test]
    fn generate_struct_not_applicable_empty_tuple_struct() {
        check_assist_not_applicable(
            generate_struct,
            "
            fn foo() {
                let _ = $0Foo();
            }
            ",
        );
    }
}
