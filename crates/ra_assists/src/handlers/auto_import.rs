use crate::{
    assist_ctx::{Assist, AssistCtx},
    utils::insert_use_statement,
    AssistId, UnresolvedElement,
};
use ra_syntax::ast;

// Assist: auto_import
//
// If the name is unresolved, provides all possible imports for it.
//
// ```
// fn main() {
//     let map = HashMap<|>::new();
// }
// # pub mod std { pub mod collections { pub struct HashMap { } } }
// ```
// ->
// ```
// use std::collections::HashMap;
//
// fn main() {
//     let map = HashMap::new();
// }
// # pub mod std { pub mod collections { pub struct HashMap { } } }
// ```
pub(crate) fn auto_import(ctx: AssistCtx) -> Option<Assist> {
    let unresolved_element = if let Some(path_under_caret) = ctx.find_node_at_offset::<ast::Path>()
    {
        UnresolvedElement::for_path(path_under_caret, &ctx.sema)
    } else {
        UnresolvedElement::for_method_call(ctx.find_node_at_offset()?, &ctx.sema)
    }?;
    let proposed_imports = unresolved_element.search_for_imports(ctx.db);
    if proposed_imports.is_empty() {
        return None;
    }

    let mut group = ctx.add_assist_group(unresolved_element.get_import_group_message());
    for import in proposed_imports {
        group.add_assist(AssistId("auto_import"), format!("Import `{}`", &import), |edit| {
            edit.target(unresolved_element.element_syntax.text_range());
            insert_use_statement(
                &unresolved_element.element_syntax,
                &import,
                edit.text_edit_builder(),
            );
        });
    }
    group.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::{check_assist, check_assist_not_applicable, check_assist_target};

    #[test]
    fn applicable_when_found_an_import() {
        check_assist(
            auto_import,
            r"
            <|>PubStruct

            pub mod PubMod {
                pub struct PubStruct;
            }
            ",
            r"
            <|>use PubMod::PubStruct;

            PubStruct

            pub mod PubMod {
                pub struct PubStruct;
            }
            ",
        );
    }

    #[test]
    fn auto_imports_are_merged() {
        check_assist(
            auto_import,
            r"
            use PubMod::PubStruct1;

            struct Test {
                test: Pub<|>Struct2<u8>,
            }

            pub mod PubMod {
                pub struct PubStruct1;
                pub struct PubStruct2<T> {
                    _t: T,
                }
            }
            ",
            r"
            use PubMod::{PubStruct2, PubStruct1};

            struct Test {
                test: Pub<|>Struct2<u8>,
            }

            pub mod PubMod {
                pub struct PubStruct1;
                pub struct PubStruct2<T> {
                    _t: T,
                }
            }
            ",
        );
    }

    #[test]
    fn applicable_when_found_multiple_imports() {
        check_assist(
            auto_import,
            r"
            PubSt<|>ruct

            pub mod PubMod1 {
                pub struct PubStruct;
            }
            pub mod PubMod2 {
                pub struct PubStruct;
            }
            pub mod PubMod3 {
                pub struct PubStruct;
            }
            ",
            r"
            use PubMod1::PubStruct;

            PubSt<|>ruct

            pub mod PubMod1 {
                pub struct PubStruct;
            }
            pub mod PubMod2 {
                pub struct PubStruct;
            }
            pub mod PubMod3 {
                pub struct PubStruct;
            }
            ",
        );
    }

    #[test]
    fn not_applicable_for_already_imported_types() {
        check_assist_not_applicable(
            auto_import,
            r"
            use PubMod::PubStruct;

            PubStruct<|>

            pub mod PubMod {
                pub struct PubStruct;
            }
            ",
        );
    }

    #[test]
    fn not_applicable_for_types_with_private_paths() {
        check_assist_not_applicable(
            auto_import,
            r"
            PrivateStruct<|>

            pub mod PubMod {
                struct PrivateStruct;
            }
            ",
        );
    }

    #[test]
    fn not_applicable_when_no_imports_found() {
        check_assist_not_applicable(
            auto_import,
            "
            PubStruct<|>",
        );
    }

    #[test]
    fn not_applicable_in_import_statements() {
        check_assist_not_applicable(
            auto_import,
            r"
            use PubStruct<|>;

            pub mod PubMod {
                pub struct PubStruct;
            }",
        );
    }

    #[test]
    fn function_import() {
        check_assist(
            auto_import,
            r"
            test_function<|>

            pub mod PubMod {
                pub fn test_function() {};
            }
            ",
            r"
            use PubMod::test_function;

            test_function<|>

            pub mod PubMod {
                pub fn test_function() {};
            }
            ",
        );
    }

    #[test]
    fn auto_import_target() {
        check_assist_target(
            auto_import,
            r"
            struct AssistInfo {
                group_label: Option<<|>GroupLabel>,
            }

            mod m { pub struct GroupLabel; }
            ",
            "GroupLabel",
        )
    }

    #[test]
    fn not_applicable_when_path_start_is_imported() {
        check_assist_not_applicable(
            auto_import,
            r"
            pub mod mod1 {
                pub mod mod2 {
                    pub mod mod3 {
                        pub struct TestStruct;
                    }
                }
            }

            use mod1::mod2;
            fn main() {
                mod2::mod3::TestStruct<|>
            }
            ",
        );
    }

    #[test]
    fn not_applicable_for_imported_function() {
        check_assist_not_applicable(
            auto_import,
            r"
            pub mod test_mod {
                pub fn test_function() {}
            }

            use test_mod::test_function;
            fn main() {
                test_function<|>
            }
            ",
        );
    }

    #[test]
    fn associated_struct_function() {
        check_assist(
            auto_import,
            r"
            mod test_mod {
                pub struct TestStruct {}
                impl TestStruct {
                    pub fn test_function() {}
                }
            }

            fn main() {
                TestStruct::test_function<|>
            }
            ",
            r"
            use test_mod::TestStruct;

            mod test_mod {
                pub struct TestStruct {}
                impl TestStruct {
                    pub fn test_function() {}
                }
            }

            fn main() {
                TestStruct::test_function<|>
            }
            ",
        );
    }

    #[test]
    fn associated_struct_const() {
        check_assist(
            auto_import,
            r"
            mod test_mod {
                pub struct TestStruct {}
                impl TestStruct {
                    const TEST_CONST: u8 = 42;
                }
            }

            fn main() {
                TestStruct::TEST_CONST<|>
            }
            ",
            r"
            use test_mod::TestStruct;

            mod test_mod {
                pub struct TestStruct {}
                impl TestStruct {
                    const TEST_CONST: u8 = 42;
                }
            }

            fn main() {
                TestStruct::TEST_CONST<|>
            }
            ",
        );
    }

    #[test]
    fn associated_trait_function() {
        check_assist(
            auto_import,
            r"
            mod test_mod {
                pub trait TestTrait {
                    fn test_function();
                }
                pub struct TestStruct {}
                impl TestTrait for TestStruct {
                    fn test_function() {}
                }
            }

            fn main() {
                test_mod::TestStruct::test_function<|>
            }
            ",
            r"
            use test_mod::TestTrait;

            mod test_mod {
                pub trait TestTrait {
                    fn test_function();
                }
                pub struct TestStruct {}
                impl TestTrait for TestStruct {
                    fn test_function() {}
                }
            }

            fn main() {
                test_mod::TestStruct::test_function<|>
            }
            ",
        );
    }

    #[test]
    fn not_applicable_for_imported_trait_for_function() {
        check_assist_not_applicable(
            auto_import,
            r"
            mod test_mod {
                pub trait TestTrait {
                    fn test_function();
                }
                pub trait TestTrait2 {
                    fn test_function();
                }
                pub enum TestEnum {
                    One,
                    Two,
                }
                impl TestTrait2 for TestEnum {
                    fn test_function() {}
                }
                impl TestTrait for TestEnum {
                    fn test_function() {}
                }
            }

            use test_mod::TestTrait2;
            fn main() {
                test_mod::TestEnum::test_function<|>;
            }
            ",
        )
    }

    #[test]
    fn associated_trait_const() {
        check_assist(
            auto_import,
            r"
            mod test_mod {
                pub trait TestTrait {
                    const TEST_CONST: u8;
                }
                pub struct TestStruct {}
                impl TestTrait for TestStruct {
                    const TEST_CONST: u8 = 42;
                }
            }

            fn main() {
                test_mod::TestStruct::TEST_CONST<|>
            }
            ",
            r"
            use test_mod::TestTrait;

            mod test_mod {
                pub trait TestTrait {
                    const TEST_CONST: u8;
                }
                pub struct TestStruct {}
                impl TestTrait for TestStruct {
                    const TEST_CONST: u8 = 42;
                }
            }

            fn main() {
                test_mod::TestStruct::TEST_CONST<|>
            }
            ",
        );
    }

    #[test]
    fn not_applicable_for_imported_trait_for_const() {
        check_assist_not_applicable(
            auto_import,
            r"
            mod test_mod {
                pub trait TestTrait {
                    const TEST_CONST: u8;
                }
                pub trait TestTrait2 {
                    const TEST_CONST: f64;
                }
                pub enum TestEnum {
                    One,
                    Two,
                }
                impl TestTrait2 for TestEnum {
                    const TEST_CONST: f64 = 42.0;
                }
                impl TestTrait for TestEnum {
                    const TEST_CONST: u8 = 42;
                }
            }

            use test_mod::TestTrait2;
            fn main() {
                test_mod::TestEnum::TEST_CONST<|>;
            }
            ",
        )
    }

    #[test]
    fn trait_method() {
        check_assist(
            auto_import,
            r"
            mod test_mod {
                pub trait TestTrait {
                    fn test_method(&self);
                }
                pub struct TestStruct {}
                impl TestTrait for TestStruct {
                    fn test_method(&self) {}
                }
            }

            fn main() {
                let test_struct = test_mod::TestStruct {};
                test_struct.test_meth<|>od()
            }
            ",
            r"
            use test_mod::TestTrait;

            mod test_mod {
                pub trait TestTrait {
                    fn test_method(&self);
                }
                pub struct TestStruct {}
                impl TestTrait for TestStruct {
                    fn test_method(&self) {}
                }
            }

            fn main() {
                let test_struct = test_mod::TestStruct {};
                test_struct.test_meth<|>od()
            }
            ",
        );
    }

    #[test]
    fn not_applicable_for_imported_trait_for_method() {
        check_assist_not_applicable(
            auto_import,
            r"
            mod test_mod {
                pub trait TestTrait {
                    fn test_method(&self);
                }
                pub trait TestTrait2 {
                    fn test_method(&self);
                }
                pub enum TestEnum {
                    One,
                    Two,
                }
                impl TestTrait2 for TestEnum {
                    fn test_method(&self) {}
                }
                impl TestTrait for TestEnum {
                    fn test_method(&self) {}
                }
            }

            use test_mod::TestTrait2;
            fn main() {
                let one = test_mod::TestEnum::One;
                one.test<|>_method();
            }
            ",
        )
    }
}
