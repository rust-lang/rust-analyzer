use crate::{
    AssistConfig,
    assist_context::{AssistContext, Assists},
};
use ide_db::assists::{AssistId, AssistKind, ExprFillDefaultMode};
use syntax::{
    AstNode,
    ast::{
        self, AssocItem, BlockExpr, GenericParam, HasGenericParams, HasName, HasTypeBounds,
        HasVisibility as astHasVisibility,
        edit::IndentLevel,
        edit_in_place::{AttrsOwnerEdit, Indent},
        make,
    },
};

// Assist: generate_blanket_trait_impl
//
// Generate blanket trait implementation.
//
// ```
// trait $0Foo<T: Send>: ToOwned
// where
//     Self::Owned: Default,
// {
//     fn foo(&self) -> T;
//
//     fn print_foo(&self) {
//         println!("{}", self.foo());
//     }
// }
// ```
// ->
// ```
// trait Foo<T: Send>: ToOwned
// where
//     Self::Owned: Default,
// {
//     fn foo(&self) -> T;
//
//     fn print_foo(&self) {
//         println!("{}", self.foo());
//     }
// }
//
// impl<T: Send, This: ToOwned> Foo<T> for This
// where
//     Self::Owned: Default,
// {
//     fn foo(&self) -> T {
//         todo!()
//     }
// }
// ```
pub(crate) fn generate_blanket_trait_impl(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
) -> Option<()> {
    let name = ctx.find_node_at_offset::<ast::Name>()?;
    let traitd = ast::Trait::cast(name.syntax().parent()?)?;
    let namety = make::ty_path(make::path_from_text(&name.text()));
    let trait_where_clause = traitd.where_clause();
    let bounds = traitd.type_bound_list();
    let is_unsafe = traitd.unsafe_token().is_some();
    let thisname = make::name("This");
    let thisty = make::ty_path(make::path_from_text(&thisname.text()));

    let gendecl = make::generic_param_list([GenericParam::TypeParam(make::type_param(
        thisname.clone(),
        bounds,
    ))]);

    let trait_gen_args = traitd.generic_param_list().map(|param_list| param_list.to_generic_args());

    let impl_ = make::impl_trait(
        is_unsafe,
        traitd.generic_param_list(),
        trait_gen_args,
        Some(gendecl),
        None,
        false,
        namety,
        thisty.clone(),
        trait_where_clause,
        None,
        None,
    )
    .clone_for_update();

    if let Some(trait_assoc_list) = traitd.assoc_item_list() {
        let new_indent = IndentLevel::from_node(impl_.syntax()) + 1;
        let assoc_item_list = impl_.get_or_create_assoc_item_list();
        for method in trait_assoc_list.assoc_items() {
            let AssocItem::Fn(f) = method else {
                continue;
            };
            if f.body().is_some() {
                continue;
            }
            let f = todo_fn(&f, ctx.config).clone_for_update();
            f.remove_attrs_and_docs();
            f.reindent_to(new_indent);
            assoc_item_list.add_item(AssocItem::Fn(f));
        }
    }

    acc.add(
        AssistId("generate_blanket_trait_impl", AssistKind::Generate, None),
        "Generate blanket trait implementation",
        name.syntax().text_range(),
        |builder| {
            builder.insert(traitd.syntax().text_range().end(), format!("\n\n{}", impl_));
        },
    );

    Some(())
}

fn todo_fn(f: &ast::Fn, config: &AssistConfig) -> ast::Fn {
    let params = f.param_list().unwrap_or_else(|| make::param_list(None, None));
    make::fn_(
        f.visibility(),
        f.name().unwrap_or_else(|| make::name("unnamed")),
        f.generic_param_list(),
        f.where_clause(),
        params,
        default_block(config),
        f.ret_type(),
        f.async_token().is_some(),
        f.const_token().is_some(),
        f.unsafe_token().is_some(),
        f.gen_token().is_some(),
    )
}

fn default_block(config: &AssistConfig) -> BlockExpr {
    let expr = match config.expr_fill_default {
        ExprFillDefaultMode::Todo => make::ext::expr_todo(),
        ExprFillDefaultMode::Underscore => make::ext::expr_underscore(),
        ExprFillDefaultMode::Default => make::ext::expr_todo(),
    };
    make::block_expr(None, Some(expr))
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::tests::{check_assist, check_assist_not_applicable};

    #[test]
    fn test_gen_blanket_works() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo<T: Send>: ToOwned
where
    Self::Owned: Default,
{
    fn foo(&self) -> T;

    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}
"#,
            r#"
trait Foo<T: Send>: ToOwned
where
    Self::Owned: Default,
{
    fn foo(&self) -> T;

    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}

impl<T: Send, This: ToOwned> Foo<T> for This
where
    Self::Owned: Default,
{
    fn foo(&self) -> T {
        todo!()
    }
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_indent() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
        trait $0Foo<T: Send>: ToOwned
        where
            Self::Owned: Default,
        {
            fn foo(&self) -> T;

            fn print_foo(&self) {
                println!("{}", self.foo());
            }
        }
        "#,
            r#"
        trait Foo<T: Send>: ToOwned
        where
            Self::Owned: Default,
        {
            fn foo(&self) -> T;

            fn print_foo(&self) {
                println!("{}", self.foo());
            }
        }

        impl<T: Send, This: ToOwned> Foo<T> for This
        where
            Self::Owned: Default,
        {
            fn foo(&self) -> T {
                todo!()
            }
        }
        "#,
        );
    }

    #[test]
    fn test_gen_blanket_remove_attribute() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo<T: Send>: ToOwned
where
    Self::Owned: Default,
{
    #[doc(hidden)]
    fn foo(&self) -> T;

    /// foo
    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}
"#,
            r#"
trait Foo<T: Send>: ToOwned
where
    Self::Owned: Default,
{
    #[doc(hidden)]
    fn foo(&self) -> T;

    /// foo
    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}

impl<T: Send, This: ToOwned> Foo<T> for This
where
    Self::Owned: Default,
{
    fn foo(&self) -> T {
        todo!()
    }
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_not_gen_type_alias() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo<T: Send>: ToOwned
where
    Self::Owned: Default,
{
    type X: Sync;

    fn foo(&self, x: Self::X) -> T;

    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}
"#,
            r#"
trait Foo<T: Send>: ToOwned
where
    Self::Owned: Default,
{
    type X: Sync;

    fn foo(&self, x: Self::X) -> T;

    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}

impl<T: Send, This: ToOwned> Foo<T> for This
where
    Self::Owned: Default,
{
    fn foo(&self, x: Self::X) -> T {
        todo!()
    }
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_no_quick_bound() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo<T: Send>
where
    Self: ToOwned,
    Self::Owned: Default,
{
    type X: Sync;

    fn foo(&self, x: Self::X) -> T;

    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}
"#,
            r#"
trait Foo<T: Send>
where
    Self: ToOwned,
    Self::Owned: Default,
{
    type X: Sync;

    fn foo(&self, x: Self::X) -> T;

    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}

impl<T: Send, This> Foo<T> for This
where
    Self: ToOwned,
    Self::Owned: Default,
{
    fn foo(&self, x: Self::X) -> T {
        todo!()
    }
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_no_where_clause() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo<T: Send> {
    type X: Sync;

    fn foo(&self, x: Self::X) -> T;

    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}
"#,
            r#"
trait Foo<T: Send> {
    type X: Sync;

    fn foo(&self, x: Self::X) -> T;

    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}

impl<T: Send, This> Foo<T> for This {
    fn foo(&self, x: Self::X) -> T {
        todo!()
    }
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_basic() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo {
    type X: Sync;

    fn foo(&self, x: Self::X) -> i32;

    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}
"#,
            r#"
trait Foo {
    type X: Sync;

    fn foo(&self, x: Self::X) -> i32;

    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}

impl<This> Foo for This {
    fn foo(&self, x: Self::X) -> i32 {
        todo!()
    }
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_empty_trait() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo {}
"#,
            r#"
trait Foo {}

impl<This> Foo for This {}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_empty_trait_with_quick_bounds() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo: Copy {}
"#,
            r#"
trait Foo: Copy {}

impl<This: Copy> Foo for This {}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_empty_trait_with_where_clause() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo where Self: Copy {}
"#,
            r#"
trait Foo where Self: Copy {}

impl<This> Foo for This
where Self: Copy
{
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_empty_trait_with_where_clause_comma() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo where Self: Copy, {}
"#,
            r#"
trait Foo where Self: Copy, {}

impl<This> Foo for This
where Self: Copy,
{
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_empty_trait_with_where_clause_newline() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo
where Self: Copy
{}
"#,
            r#"
trait Foo
where Self: Copy
{}

impl<This> Foo for This
where Self: Copy
{
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_empty_trait_with_where_clause_newline_newline() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo
where
    Self: Copy
{}
"#,
            r#"
trait Foo
where
    Self: Copy
{}

impl<This> Foo for This
where
    Self: Copy
{
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_empty_trait_with_where_clause_newline_newline_comma() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo
where
    Self: Copy,
{}
"#,
            r#"
trait Foo
where
    Self: Copy,
{}

impl<This> Foo for This
where
    Self: Copy,
{
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_empty_trait_with_multiple_where_clause() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo
where
    Self: Copy,
    (): Into<Self>,
{}
"#,
            r#"
trait Foo
where
    Self: Copy,
    (): Into<Self>,
{}

impl<This> Foo for This
where
    Self: Copy,
    (): Into<Self>,
{
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_empty_trait_with_multiple_bounds_where_clause() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo
where
    Self: Copy + Sync,
    (): Into<Self>,
{}
"#,
            r#"
trait Foo
where
    Self: Copy + Sync,
    (): Into<Self>,
{}

impl<This> Foo for This
where
    Self: Copy + Sync,
    (): Into<Self>,
{
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_empty_generate() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo {
    fn foo(&self) {}
}
"#,
            r#"
trait Foo {
    fn foo(&self) {}
}

impl<This> Foo for This {}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_trait_with_doc() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
/// some docs
trait $0Foo {}
"#,
            r#"
/// some docs
trait Foo {}

impl<This> Foo for This {}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_multiple_method() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo {
    fn foo(&self);
    fn bar(&self);
}
"#,
            r#"
trait Foo {
    fn foo(&self);
    fn bar(&self);
}

impl<This> Foo for This {
    fn foo(&self) {
        todo!()
    }

    fn bar(&self) {
        todo!()
    }
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_method_with_generic() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo {
    fn foo<T>(&self, value: T) -> T;
    fn bar<T>(&self, value: T) -> T { todo!() }
}
"#,
            r#"
trait Foo {
    fn foo<T>(&self, value: T) -> T;
    fn bar<T>(&self, value: T) -> T { todo!() }
}

impl<This> Foo for This {
    fn foo<T>(&self, value: T) -> T {
        todo!()
    }
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_method_with_lifetimes() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo<'a> {
    fn foo(&self) -> &'a str;
}
"#,
            r#"
trait Foo<'a> {
    fn foo(&self) -> &'a str;
}

impl<'a, This> Foo<'a> for This {
    fn foo(&self) -> &'a str {
        todo!()
    }
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_method_with_lifetime_bounds() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo<'a: 'static> {
    fn foo(&self) -> &'a str;
}
"#,
            r#"
trait Foo<'a: 'static> {
    fn foo(&self) -> &'a str;
}

impl<'a: 'static, This> Foo<'a> for This {
    fn foo(&self) -> &'a str {
        todo!()
    }
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_method_with_lifetime_quick_bounds() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo<'a>: 'a {
    fn foo(&self) -> &'a str;
}
"#,
            r#"
trait Foo<'a>: 'a {
    fn foo(&self) -> &'a str;
}

impl<'a, This: 'a> Foo<'a> for This {
    fn foo(&self) -> &'a str {
        todo!()
    }
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_method_with_multiple_lifetimes() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo<'a, 'b> {
    fn foo(&self) -> &'a &'b str;
}
"#,
            r#"
trait Foo<'a, 'b> {
    fn foo(&self) -> &'a &'b str;
}

impl<'a, 'b, This> Foo<'a, 'b> for This {
    fn foo(&self) -> &'a &'b str {
        todo!()
    }
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_method_with_lifetime_bounds_at_where_clause() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo<'a>
where 'a: 'static,
{
    fn foo(&self) -> &'a str;
}
"#,
            r#"
trait Foo<'a>
where 'a: 'static,
{
    fn foo(&self) -> &'a str;
}

impl<'a, This> Foo<'a> for This
where 'a: 'static,
{
    fn foo(&self) -> &'a str {
        todo!()
    }
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_not_on_name() {
        check_assist_not_applicable(
            generate_blanket_trait_impl,
            r#"
trait Foo<T: Send>: $0ToOwned
where
    Self::Owned: Default,
{
    fn foo(&self) -> T;

    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}
"#,
        );

        check_assist_not_applicable(
            generate_blanket_trait_impl,
            r#"
trait Foo<T: Send>: ToOwned
where
    Self::Owned: Default,
{
    $0fn foo(&self) -> T;

    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}
"#,
        );

        check_assist_not_applicable(
            generate_blanket_trait_impl,
            r#"
trait Foo<T: Send>: ToOwned
where
    Self::Owned: Default,
{
    fn $0foo(&self) -> T;

    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_apply_on_other_impl_block() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo {
    fn foo(&self) -> i32;

    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}

trait Bar {}
impl Bar for i32 {}
"#,
            r#"
trait Foo {
    fn foo(&self) -> i32;

    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}

impl<This> Foo for This {
    fn foo(&self) -> i32 {
        todo!()
    }
}

trait Bar {}
impl Bar for i32 {}
"#,
        );
    }

    #[test]
    fn test_gen_blanket_apply_on_other_blanket_impl_block() {
        check_assist(
            generate_blanket_trait_impl,
            r#"
trait $0Foo {
    fn foo(&self) -> i32;

    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}

trait Bar {}
impl<This> Bar for This {}
"#,
            r#"
trait Foo {
    fn foo(&self) -> i32;

    fn print_foo(&self) {
        println!("{}", self.foo());
    }
}

impl<This> Foo for This {
    fn foo(&self) -> i32 {
        todo!()
    }
}

trait Bar {}
impl<This> Bar for This {}
"#,
        );
    }
}
