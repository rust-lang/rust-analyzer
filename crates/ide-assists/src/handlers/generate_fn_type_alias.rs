use either::Either;
use hir::HirDisplay;
use ide_db::assists::{AssistId, GroupLabel};
use syntax::{
    AstNode,
    ast::{self, HasGenericParams, HasName, edit::IndentLevel, syntax_factory::SyntaxFactory},
    syntax_editor,
};

use crate::{AssistContext, Assists};

// Assist: generate_fn_type_alias_named
//
// Generate a type alias for the function with named parameters.
//
// ```
// unsafe fn fo$0o(n: i32) -> i32 { 42i32 }
// ```
// ->
// ```
// type ${0:FooFn} = unsafe fn(n: i32) -> i32;
//
// unsafe fn foo(n: i32) -> i32 { 42i32 }
// ```

// Assist: generate_fn_type_alias_unnamed
//
// Generate a type alias for the function with unnamed parameters.
//
// ```
// unsafe fn fo$0o(n: i32) -> i32 { 42i32 }
// ```
// ->
// ```
// type ${0:FooFn} = unsafe fn(i32) -> i32;
//
// unsafe fn foo(n: i32) -> i32 { 42i32 }
// ```

pub(crate) fn generate_fn_type_alias(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let name = ctx.find_node_at_offset::<ast::Name>()?;
    let func = &name.syntax().parent()?;
    let func_node = ast::Fn::cast(func.clone())?;
    let param_list = func_node.param_list()?;

    let assoc_owner = func.ancestors().nth(2).and_then(Either::<ast::Trait, ast::Impl>::cast);
    // This is where we'll insert the type alias, since type aliases in `impl`s or `trait`s are not supported
    let insertion_node = assoc_owner
        .as_ref()
        .map_or_else(|| func, |impl_| impl_.as_ref().either(AstNode::syntax, AstNode::syntax));

    for style in ParamStyle::ALL {
        acc.add_group(
            &GroupLabel("Generate a type alias for function...".into()),
            style.assist_id(),
            style.label(),
            func_node.syntax().text_range(),
            |builder| {
                let mut edit = builder.make_editor(func);
                let factory = SyntaxFactory::with_mappings();

                let alias_name = format!("{}Fn", stdx::to_camel_case(&name.to_string()));

                let mut fn_params_vec = Vec::new();

                if let Some(self_param) = param_list.self_param()
                    && let Some(self_ty) = ctx.sema.type_of_self(&self_param)
                {
                    let self_ty = ctx
                        .sema
                        .scope(self_param.syntax())
                        .and_then(|scope| {
                            self_ty.display_source_code(ctx.db(), scope.module().into(), false).ok()
                        })
                        .map(|text| factory.ty(&text))
                        .or_else(|| {
                            let is_ref = self_ty.is_reference();
                            let is_mut = self_ty.is_mutable_reference();
                            self_ty.strip_references().as_adt().map(|adt| {
                                let inner_type = factory.ty(adt.name(ctx.db()).as_str());
                                if is_ref { factory.ty_ref(inner_type, is_mut) } else { inner_type }
                            })
                        });
                    if let Some(self_ty) = self_ty {
                        fn_params_vec.push(factory.unnamed_param(self_ty));
                    }
                }

                fn_params_vec.extend(param_list.params().filter_map(|p| match style {
                    ParamStyle::Named => Some(p),
                    ParamStyle::Unnamed => p.ty().map(|ty| factory.unnamed_param(ty)),
                }));

                let mut generic_params_vec = Vec::new();
                if let Some(generic_params) = assoc_owner.as_ref().and_then(|owner| match owner {
                    Either::Left(it) => it.generic_param_list(),
                    Either::Right(it) => it.generic_param_list(),
                }) {
                    generic_params_vec.extend(generic_params.generic_params());
                }
                if let Some(generic_params) = func_node.generic_param_list() {
                    generic_params_vec.extend(generic_params.generic_params());
                }
                let generic_params = (!generic_params_vec.is_empty())
                    .then(|| factory.generic_param_list(generic_params_vec));

                let ty = build_fn_ptr_type(
                    &factory,
                    func_node.unsafe_token().is_some(),
                    func_node.abi(),
                    fn_params_vec,
                    func_node.ret_type(),
                );

                // Insert new alias
                let ty_alias =
                    factory.ty_alias([], &alias_name, generic_params, None, None, Some((ty, None)));

                let indent = IndentLevel::from_node(insertion_node);
                edit.insert_all(
                    syntax_editor::Position::before(insertion_node),
                    vec![
                        ty_alias.syntax().clone().into(),
                        factory.whitespace(&format!("\n\n{indent}")).into(),
                    ],
                );

                if let Some(cap) = ctx.config.snippet_cap
                    && let Some(name) = ty_alias.name()
                {
                    edit.add_annotation(name.syntax(), builder.make_placeholder_snippet(cap));
                }

                edit.add_mappings(factory.finish_with_mappings());
                builder.add_file_edits(ctx.vfs_file_id(), edit);
            },
        );
    }

    Some(())
}

fn build_fn_ptr_type(
    factory: &SyntaxFactory,
    is_unsafe: bool,
    abi: Option<ast::Abi>,
    params: Vec<ast::Param>,
    ret_type: Option<ast::RetType>,
) -> ast::Type {
    let mut fn_ty = String::new();
    if is_unsafe {
        fn_ty.push_str("unsafe ");
    }
    if let Some(abi) = abi {
        fn_ty.push_str(abi.syntax().text().to_string().as_str());
        fn_ty.push(' ');
    }

    fn_ty.push_str("fn(");
    fn_ty.push_str(
        &params
            .into_iter()
            .map(|param| param.syntax().text().to_string())
            .collect::<Vec<_>>()
            .join(", "),
    );
    fn_ty.push(')');

    if let Some(ret_type) = ret_type {
        fn_ty.push(' ');
        fn_ty.push_str(ret_type.syntax().text().to_string().as_str());
    }

    factory.ty(&fn_ty)
}

enum ParamStyle {
    Named,
    Unnamed,
}

impl ParamStyle {
    const ALL: &'static [ParamStyle] = &[ParamStyle::Named, ParamStyle::Unnamed];

    fn assist_id(&self) -> AssistId {
        let s = match self {
            ParamStyle::Named => "generate_fn_type_alias_named",
            ParamStyle::Unnamed => "generate_fn_type_alias_unnamed",
        };

        AssistId::generate(s)
    }

    fn label(&self) -> &'static str {
        match self {
            ParamStyle::Named => "Generate a type alias for function with named params",
            ParamStyle::Unnamed => "Generate a type alias for function with unnamed params",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::check_assist_by_label;

    use super::*;

    #[test]
    fn generate_fn_alias_unnamed_simple() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
fn fo$0o(param: u32) -> i32 { return 42; }
"#,
            r#"
type ${0:FooFn} = fn(u32) -> i32;

fn foo(param: u32) -> i32 { return 42; }
"#,
            ParamStyle::Unnamed.label(),
        );
    }

    #[test]
    fn generate_fn_alias_unnamed_unsafe() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
unsafe fn fo$0o(param: u32) -> i32 { return 42; }
"#,
            r#"
type ${0:FooFn} = unsafe fn(u32) -> i32;

unsafe fn foo(param: u32) -> i32 { return 42; }
"#,
            ParamStyle::Unnamed.label(),
        );
    }

    #[test]
    fn generate_fn_alias_unnamed_extern() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
extern fn fo$0o(param: u32) -> i32 { return 42; }
"#,
            r#"
type ${0:FooFn} = extern fn(u32) -> i32;

extern fn foo(param: u32) -> i32 { return 42; }
"#,
            ParamStyle::Unnamed.label(),
        );
    }

    #[test]
    fn generate_fn_type_unnamed_extern_abi() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
extern "FooABI" fn fo$0o(param: u32) -> i32 { return 42; }
"#,
            r#"
type ${0:FooFn} = extern "FooABI" fn(u32) -> i32;

extern "FooABI" fn foo(param: u32) -> i32 { return 42; }
"#,
            ParamStyle::Unnamed.label(),
        );
    }

    #[test]
    fn generate_fn_alias_unnamed_unsafe_extern_abi() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
unsafe extern "FooABI" fn fo$0o(param: u32) -> i32 { return 42; }
"#,
            r#"
type ${0:FooFn} = unsafe extern "FooABI" fn(u32) -> i32;

unsafe extern "FooABI" fn foo(param: u32) -> i32 { return 42; }
"#,
            ParamStyle::Unnamed.label(),
        );
    }

    #[test]
    fn generate_fn_alias_unnamed_generics() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
fn fo$0o<A, B>(a: A, b: B) -> i32 { return 42; }
"#,
            r#"
type ${0:FooFn}<A, B> = fn(A, B) -> i32;

fn foo<A, B>(a: A, b: B) -> i32 { return 42; }
"#,
            ParamStyle::Unnamed.label(),
        );
    }

    #[test]
    fn generate_fn_alias_unnamed_generics_bounds() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
fn fo$0o<A: Trait, B: Trait>(a: A, b: B) -> i32 { return 42; }
"#,
            r#"
type ${0:FooFn}<A: Trait, B: Trait> = fn(A, B) -> i32;

fn foo<A: Trait, B: Trait>(a: A, b: B) -> i32 { return 42; }
"#,
            ParamStyle::Unnamed.label(),
        );
    }

    #[test]
    fn generate_fn_alias_unnamed_complex_types() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
fn fo$0o(x: Vec<i32>) {}
"#,
            r#"
type ${0:FooFn} = fn(Vec<i32>);

fn foo(x: Vec<i32>) {}
"#,
            ParamStyle::Unnamed.label(),
        );
    }

    #[test]
    fn generate_fn_alias_unnamed_self() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
struct S;

impl S {
    fn fo$0o(&mut self, param: u32) -> i32 { return 42; }
}
"#,
            r#"
struct S;

type ${0:FooFn} = fn(&mut S, u32) -> i32;

impl S {
    fn foo(&mut self, param: u32) -> i32 { return 42; }
}
"#,
            ParamStyle::Unnamed.label(),
        );
    }

    #[test]
    fn generate_fn_alias_named_simple() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
fn fo$0o(param: u32) -> i32 { return 42; }
"#,
            r#"
type ${0:FooFn} = fn(param: u32) -> i32;

fn foo(param: u32) -> i32 { return 42; }
"#,
            ParamStyle::Named.label(),
        );
    }

    #[test]
    fn generate_fn_alias_named_unsafe() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
unsafe fn fo$0o(param: u32) -> i32 { return 42; }
"#,
            r#"
type ${0:FooFn} = unsafe fn(param: u32) -> i32;

unsafe fn foo(param: u32) -> i32 { return 42; }
"#,
            ParamStyle::Named.label(),
        );
    }

    #[test]
    fn generate_fn_alias_named_extern() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
extern fn fo$0o(param: u32) -> i32 { return 42; }
"#,
            r#"
type ${0:FooFn} = extern fn(param: u32) -> i32;

extern fn foo(param: u32) -> i32 { return 42; }
"#,
            ParamStyle::Named.label(),
        );
    }

    #[test]
    fn generate_fn_type_named_extern_abi() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
extern "FooABI" fn fo$0o(param: u32) -> i32 { return 42; }
"#,
            r#"
type ${0:FooFn} = extern "FooABI" fn(param: u32) -> i32;

extern "FooABI" fn foo(param: u32) -> i32 { return 42; }
"#,
            ParamStyle::Named.label(),
        );
    }

    #[test]
    fn generate_fn_alias_named_unsafe_extern_abi() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
unsafe extern "FooABI" fn fo$0o(param: u32) -> i32 { return 42; }
"#,
            r#"
type ${0:FooFn} = unsafe extern "FooABI" fn(param: u32) -> i32;

unsafe extern "FooABI" fn foo(param: u32) -> i32 { return 42; }
"#,
            ParamStyle::Named.label(),
        );
    }

    #[test]
    fn generate_fn_alias_named_generics() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
fn fo$0o<A, B>(a: A, b: B) -> i32 { return 42; }
"#,
            r#"
type ${0:FooFn}<A, B> = fn(a: A, b: B) -> i32;

fn foo<A, B>(a: A, b: B) -> i32 { return 42; }
"#,
            ParamStyle::Named.label(),
        );
    }

    #[test]
    fn generate_fn_alias_named_generics_bounds() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
fn fo$0o<A: Trait, B: Trait>(a: A, b: B) -> i32 { return 42; }
"#,
            r#"
type ${0:FooFn}<A: Trait, B: Trait> = fn(a: A, b: B) -> i32;

fn foo<A: Trait, B: Trait>(a: A, b: B) -> i32 { return 42; }
"#,
            ParamStyle::Named.label(),
        );
    }

    #[test]
    fn generate_fn_alias_named_complex_types() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
fn fo$0o(x: Vec<i32>) {}
"#,
            r#"
type ${0:FooFn} = fn(x: Vec<i32>);

fn foo(x: Vec<i32>) {}
"#,
            ParamStyle::Named.label(),
        );
    }

    #[test]
    fn generate_fn_alias_named_self() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
struct S;

impl S {
    fn fo$0o(&mut self, param: u32) -> i32 { return 42; }
}
"#,
            r#"
struct S;

type ${0:FooFn} = fn(&mut S, param: u32) -> i32;

impl S {
    fn foo(&mut self, param: u32) -> i32 { return 42; }
}
"#,
            ParamStyle::Named.label(),
        );
    }

    #[test]
    fn generate_fn_alias_unnamed_impl_generics_on_self() {
        check_assist_by_label(
            generate_fn_type_alias,
            r#"
struct S<T>(T);

impl<T> S<T> {
    fn fo$0o(&self, param: u32) -> i32 { return 42; }
}
"#,
            r#"
struct S<T>(T);

type ${0:FooFn}<T> = fn(&S<T>, u32) -> i32;

impl<T> S<T> {
    fn foo(&self, param: u32) -> i32 { return 42; }
}
"#,
            ParamStyle::Unnamed.label(),
        );
    }
}
