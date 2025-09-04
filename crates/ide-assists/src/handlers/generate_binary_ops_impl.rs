use syntax::{
    AstNode,
    ast::{self, ArithOp, BinaryOp, HasGenericParams, edit, edit_in_place::Indent, make},
};

use crate::{AssistContext, AssistId, Assists};

// Assist: generate_binary_ops_impl
//
// Generate binary operation from assign operation.
//
// ```
// struct Int(i32);
//
// impl<T> core::ops::AddAssign$0<i32> for Int {
//     fn add_assign(&mut self, rhs: i32) {
//         self.0 += rhs;
//         todo!()
//     }
// }
// ```
// ->
// ```
// struct Int(i32);
//
// impl<T> core::ops::Add<i32> for Int {
//     type Output = Self;
//
//     fn add(mut self, rhs: i32) -> Self::Output {
//         self += rhs;
//         self
//     }
// }
//
// impl<T> core::ops::AddAssign<i32> for Int {
//     fn add_assign(&mut self, rhs: i32) {
//         self.0 += rhs;
//         todo!()
//     }
// }
// ```
pub(crate) fn generate_binary_ops_impl(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let trait_path = ctx.find_node_at_offset::<ast::PathType>()?;
    let trait_ = ast::Type::from(trait_path.clone());
    let impl_def = ast::Impl::cast(trait_.syntax().parent()?)?;
    let trait_short_name = trait_path.path()?.segment()?.name_ref()?.to_string();
    let non_assign_name = trait_short_name.strip_suffix("Assign")?;
    let (op_func_name, trait_op) = get_ops(&trait_short_name)?;
    let ty = impl_def.self_ty()?;
    let mut non_assign_path = make::path_from_text(non_assign_name);

    if let Some(qualifier) = trait_path.path()?.qualifier() {
        non_assign_path = make::path_concat(qualifier, non_assign_path);
    }

    let target = impl_def.syntax().text_range();
    acc.add(
        AssistId::generate("generate_binary_ops_impl"),
        format!("Generate `{non_assign_name}` impl from this `{trait_short_name}` trait"),
        target,
        |builder| {
            let indent = impl_def.indent_level();
            let rhs_ty = trait_
                .generic_arg_list()
                .and_then(|list| match list.generic_args().next()? {
                    ast::GenericArg::TypeArg(type_arg) => type_arg.ty(),
                    _ => None,
                })
                .unwrap_or_else(|| make::ty("Self"));

            let self_expr = make::expr_path(make::path_from_text("self"));
            let rhs_path = make::path_from_text("rhs");
            let ty_where_clause =
                impl_def.where_clause().map(|wc| edit::AstNodeEdit::reset_indent(&wc));
            let impl_ = make::impl_trait(
                None,
                false,
                impl_def.generic_param_list(),
                trait_.generic_arg_list(),
                None,
                None,
                false,
                make::ty_path(non_assign_path),
                ty,
                None,
                ty_where_clause,
                None,
            )
            .clone_for_update();
            let body = make::block_expr(
                [make::expr_stmt(make::expr_bin_op(
                    self_expr.clone(),
                    trait_op,
                    make::expr_path(rhs_path.clone()),
                ))
                .into()],
                Some(self_expr),
            );
            let fn_ = make::fn_(
                None,
                None,
                make::name(op_func_name),
                None,
                None,
                make::param_list(
                    Some(make::owned_mut_self_param()),
                    [make::param(make::path_pat(rhs_path), rhs_ty)],
                ),
                body,
                Some(make::ret_type(make::ty("Self::Output"))),
                false,
                false,
                false,
                false,
            )
            .clone_for_update();
            fn_.indent(1.into());

            let output =
                make::ty_alias(None, "Output", None, None, None, Some((make::ty("Self"), None)))
                    .clone_for_update();
            let assoc_item_list = impl_.get_or_create_assoc_item_list();

            assoc_item_list.add_item(output.into());
            assoc_item_list.add_item(fn_.into());

            impl_.indent(indent);

            builder.insert(target.start(), format!("{impl_}\n\n{indent}"));
        },
    )
}

fn get_ops(name: &str) -> Option<(&'static str, ast::BinaryOp)> {
    let (func_name, op) = match name {
        "AddAssign" => ("add", ArithOp::Add),
        "SubAssign" => ("sub", ArithOp::Sub),
        "MulAssign" => ("mul", ArithOp::Mul),
        "DivAssign" => ("div", ArithOp::Div),
        "RemAssign" => ("rem", ArithOp::Rem),
        "ShlAssign" => ("shl", ArithOp::Shl),
        "ShrAssign" => ("shr", ArithOp::Shr),
        "BitAndAssign" => ("bitand", ArithOp::BitAnd),
        "BitXorAssign" => ("bitxor", ArithOp::BitXor),
        "BitOrAssign" => ("bitor", ArithOp::BitOr),
        _ => return None,
    };
    Some((func_name, BinaryOp::Assignment { op: op.into() }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{check_assist, check_assist_by_label, check_assist_not_applicable};

    #[test]
    fn it_works() {
        check_assist(
            generate_binary_ops_impl,
            r#"
            struct Foo;

            impl $0AddAssign for Foo {
                fn add_assign(&mut self, rhs: Self) {
                    todo!("...")
                }
            }
            "#,
            r#"
            struct Foo;

            impl Add for Foo {
                type Output = Self;

                fn add(mut self, rhs: Self) -> Self::Output {
                    self += rhs;
                    self
                }
            }

            impl AddAssign for Foo {
                fn add_assign(&mut self, rhs: Self) {
                    todo!("...")
                }
            }
            "#,
        );
    }

    #[test]
    fn other_operator() {
        check_assist(
            generate_binary_ops_impl,
            r#"
            struct Foo;

            impl $0SubAssign for Foo {
                fn sub_assign(&mut self, rhs: Self) {
                    todo!("...")
                }
            }
            "#,
            r#"
            struct Foo;

            impl Sub for Foo {
                type Output = Self;

                fn sub(mut self, rhs: Self) -> Self::Output {
                    self -= rhs;
                    self
                }
            }

            impl SubAssign for Foo {
                fn sub_assign(&mut self, rhs: Self) {
                    todo!("...")
                }
            }
            "#,
        );

        check_assist(
            generate_binary_ops_impl,
            r#"
            struct Foo;

            impl $0BitAndAssign for Foo {
                fn bitand_assign(&mut self, rhs: Self) {
                    todo!("...")
                }
            }
            "#,
            r#"
            struct Foo;

            impl BitAnd for Foo {
                type Output = Self;

                fn bitand(mut self, rhs: Self) -> Self::Output {
                    self &= rhs;
                    self
                }
            }

            impl BitAndAssign for Foo {
                fn bitand_assign(&mut self, rhs: Self) {
                    todo!("...")
                }
            }
            "#,
        );

        check_assist(
            generate_binary_ops_impl,
            r#"
            struct Foo;

            impl $0ShlAssign for Foo {
                fn shl_assign(&mut self, rhs: Self) {
                    todo!("...")
                }
            }
            "#,
            r#"
            struct Foo;

            impl Shl for Foo {
                type Output = Self;

                fn shl(mut self, rhs: Self) -> Self::Output {
                    self <<= rhs;
                    self
                }
            }

            impl ShlAssign for Foo {
                fn shl_assign(&mut self, rhs: Self) {
                    todo!("...")
                }
            }
            "#,
        );
    }

    #[test]
    fn full_path() {
        check_assist(
            generate_binary_ops_impl,
            r#"
            struct Foo;

            impl $0core::ops::AddAssign for Foo {
                fn add_assign(&mut self, rhs: Self) {
                    todo!("...")
                }
            }
            "#,
            r#"
            struct Foo;

            impl core::ops::Add for Foo {
                type Output = Self;

                fn add(mut self, rhs: Self) -> Self::Output {
                    self += rhs;
                    self
                }
            }

            impl core::ops::AddAssign for Foo {
                fn add_assign(&mut self, rhs: Self) {
                    todo!("...")
                }
            }
            "#,
        );
    }

    #[test]
    fn rhs_ty() {
        check_assist(
            generate_binary_ops_impl,
            r#"
            struct Foo;

            impl $0AddAssign<i32> for Foo {
                fn add_assign(&mut self, rhs: i32) {
                    todo!("...")
                }
            }
            "#,
            r#"
            struct Foo;

            impl Add<i32> for Foo {
                type Output = Self;

                fn add(mut self, rhs: i32) -> Self::Output {
                    self += rhs;
                    self
                }
            }

            impl AddAssign<i32> for Foo {
                fn add_assign(&mut self, rhs: i32) {
                    todo!("...")
                }
            }
            "#,
        );
    }

    #[test]
    fn rhs_param_name() {
        check_assist(
            generate_binary_ops_impl,
            r#"
            struct Foo;

            impl $0AddAssign<i32> for Foo {
                fn add_assign(&mut self, rhs1: i32) {
                    todo!("...")
                }
            }
            "#,
            r#"
            struct Foo;

            impl Add<i32> for Foo {
                type Output = Self;

                fn add(mut self, rhs: i32) -> Self::Output {
                    self += rhs;
                    self
                }
            }

            impl AddAssign<i32> for Foo {
                fn add_assign(&mut self, rhs1: i32) {
                    todo!("...")
                }
            }
            "#,
        );
    }

    #[test]
    fn generic_rhs_ty() {
        check_assist(
            generate_binary_ops_impl,
            r#"
            struct Foo;

            impl<T> $0AddAssign<T> for Foo {
                fn add_assign(&mut self, rhs: T) {
                    todo!("...")
                }
            }
            "#,
            r#"
            struct Foo;

            impl<T> Add<T> for Foo {
                type Output = Self;

                fn add(mut self, rhs: T) -> Self::Output {
                    self += rhs;
                    self
                }
            }

            impl<T> AddAssign<T> for Foo {
                fn add_assign(&mut self, rhs: T) {
                    todo!("...")
                }
            }
            "#,
        );
    }

    #[test]
    fn with_indent() {
        check_assist(
            generate_binary_ops_impl,
            r#"
            mod foo {
                mod bar {
                    struct Foo;

                    impl $0AddAssign for Foo {
                        fn add_assign(&mut self, rhs: Self) {
                            todo!("...")
                        }
                    }
                }
            }
            "#,
            r#"
            mod foo {
                mod bar {
                    struct Foo;

                    impl Add for Foo {
                        type Output = Self;

                        fn add(mut self, rhs: Self) -> Self::Output {
                            self += rhs;
                            self
                        }
                    }

                    impl AddAssign for Foo {
                        fn add_assign(&mut self, rhs: Self) {
                            todo!("...")
                        }
                    }
                }
            }
            "#,
        );
    }

    #[test]
    fn where_clause_with_indent() {
        check_assist(
            generate_binary_ops_impl,
            r#"
            mod foo {
                mod bar {
                    struct Foo<U>;

                    impl<T: Copy, U: Clone> $0AddAssign<T> for Foo<U>
                    where
                        T: Debug,
                        T: Sync,
                        U: Sync + Send,
                    {
                        fn add_assign(&mut self, rhs: T) {
                            todo!("...")
                        }
                    }
                }
            }
            "#,
            r#"
            mod foo {
                mod bar {
                    struct Foo<U>;

                    impl<T: Copy, U: Clone> Add<T> for Foo<U>
                    where
                        T: Debug,
                        T: Sync,
                        U: Sync + Send,
                    {
                        type Output = Self;

                        fn add(mut self, rhs: T) -> Self::Output {
                            self += rhs;
                            self
                        }
                    }

                    impl<T: Copy, U: Clone> AddAssign<T> for Foo<U>
                    where
                        T: Debug,
                        T: Sync,
                        U: Sync + Send,
                    {
                        fn add_assign(&mut self, rhs: T) {
                            todo!("...")
                        }
                    }
                }
            }
            "#,
        );
        check_assist(
            generate_binary_ops_impl,
            r#"
            mod foo {
                mod bar {
                    struct Foo<U>;

                    impl<T: Copy, U: Clone> $0AddAssign<T> for Foo<U>
                    where T: Debug,
                          T: Sync,
                          U: Sync + Send,
                    {
                        fn add_assign(&mut self, rhs: T) {
                            todo!("...")
                        }
                    }
                }
            }
            "#,
            r#"
            mod foo {
                mod bar {
                    struct Foo<U>;

                    impl<T: Copy, U: Clone> Add<T> for Foo<U>
                    where T: Debug,
                          T: Sync,
                          U: Sync + Send,
                    {
                        type Output = Self;

                        fn add(mut self, rhs: T) -> Self::Output {
                            self += rhs;
                            self
                        }
                    }

                    impl<T: Copy, U: Clone> AddAssign<T> for Foo<U>
                    where T: Debug,
                          T: Sync,
                          U: Sync + Send,
                    {
                        fn add_assign(&mut self, rhs: T) {
                            todo!("...")
                        }
                    }
                }
            }
            "#,
        );
    }

    #[test]
    fn label() {
        check_assist_by_label(
            generate_binary_ops_impl,
            r#"
            struct Foo;

            impl $0AddAssign for Foo<i32> {
                fn add_assign(&mut self, rhs: Self) {
                    todo!("...")
                }
            }
            "#,
            r#"
            struct Foo;

            impl Add for Foo<i32> {
                type Output = Self;

                fn add(mut self, rhs: Self) -> Self::Output {
                    self += rhs;
                    self
                }
            }

            impl AddAssign for Foo<i32> {
                fn add_assign(&mut self, rhs: Self) {
                    todo!("...")
                }
            }
            "#,
            "Generate `Add` impl from this `AddAssign` trait",
        );

        check_assist_by_label(
            generate_binary_ops_impl,
            r#"
            struct Foo;

            impl $0SubAssign for Foo<i32> {
                fn sub_assign(&mut self, rhs: Self) {
                    todo!("...")
                }
            }
            "#,
            r#"
            struct Foo;

            impl Sub for Foo<i32> {
                type Output = Self;

                fn sub(mut self, rhs: Self) -> Self::Output {
                    self -= rhs;
                    self
                }
            }

            impl SubAssign for Foo<i32> {
                fn sub_assign(&mut self, rhs: Self) {
                    todo!("...")
                }
            }
            "#,
            "Generate `Sub` impl from this `SubAssign` trait",
        );
    }

    #[test]
    fn label_full_path() {
        check_assist_by_label(
            generate_binary_ops_impl,
            r#"
            struct Foo;

            impl $0core::ops::AddAssign for Foo<i32> {
                fn add_assign(&mut self, rhs: Self) {
                    todo!("...")
                }
            }
            "#,
            r#"
            struct Foo;

            impl core::ops::Add for Foo<i32> {
                type Output = Self;

                fn add(mut self, rhs: Self) -> Self::Output {
                    self += rhs;
                    self
                }
            }

            impl core::ops::AddAssign for Foo<i32> {
                fn add_assign(&mut self, rhs: Self) {
                    todo!("...")
                }
            }
            "#,
            "Generate `Add` impl from this `AddAssign` trait",
        );
    }

    #[test]
    fn similar_name_not_applicable() {
        check_assist_not_applicable(
            generate_binary_ops_impl,
            r#"
            struct Foo;

            impl $0FooAssign for Foo<i32> {
                fn foo_assign(&mut self, rhs: Self) {
                    todo!("...")
                }
            }
            "#,
        );

        check_assist_not_applicable(
            generate_binary_ops_impl,
            r#"
            struct Foo;

            impl $0OtherAddAssign for Foo<i32> {
                fn other_add_assign(&mut self, rhs: Self) {
                    todo!("...")
                }
            }
            "#,
        );
    }
}
