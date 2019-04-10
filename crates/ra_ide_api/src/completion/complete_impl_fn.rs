use ra_syntax::{
    algo::{find_node_at_offset},
    ast::{self, FnDef, ImplItem, ImplItemKind, NameOwner},
    SyntaxKind::FN_DEF,
    AstNode, TreeArc, SmolStr,
};
use hir::{Resolver, db::HirDatabase};

use crate::completion::{
    completion_item::{CompletionItem, CompletionKind, CompletionItemKind},
    CompletionContext, Completions,
};

pub(super) fn complete_impl_fn(acc: &mut Completions, ctx: &CompletionContext) {
    let is_after_fn_keyword = match ctx.token.prev_sibling_or_token() {
        Some(token) if token.kind() == FN_DEF => true,
        _ if ctx.is_new_item => false,
        _ => return,
    };

    let impl_node = match find_node_at_offset::<ast::ImplBlock>(ctx.token.parent(), ctx.offset) {
        Some(impl_node) => impl_node,
        None => return,
    };
    let impl_item_list = match impl_node.item_list() {
        Some(impl_item_list) => impl_item_list,
        None => return,
    };

    let trait_def = match resolve_target_trait_def(ctx.db, &ctx.resolver, impl_node) {
        Some(trait_def) => trait_def,
        None => return,
    };

    let fn_def_opt = |kind| if let ImplItemKind::FnDef(def) = kind { Some(def) } else { None };
    let def_name = |def| -> Option<&SmolStr> { FnDef::name(def).map(ast::Name::text) };

    let trait_fns = {
        let trait_items = match trait_def.item_list() {
            Some(item_list) => item_list.impl_items(),
            None => return,
        };
        trait_items.map(ImplItem::kind).filter_map(fn_def_opt).collect::<Vec<_>>()
    };

    let impl_items = impl_item_list.impl_items();
    let impl_fns = impl_items.map(ImplItem::kind).filter_map(fn_def_opt).collect::<Vec<_>>();

    trait_fns
        .into_iter()
        .filter(|t| def_name(t).is_some())
        .filter(|t| impl_fns.iter().all(|i| def_name(i) != def_name(t)))
        .map(|t| (def_name(t).unwrap(), build_func(t, is_after_fn_keyword)))
        .for_each(|(name, func)| {
            CompletionItem::new(CompletionKind::Reference, ctx.source_range(), name.clone())
                .kind(CompletionItemKind::Function)
                .insert_snippet(func)
                .add_to(acc)
        });
}

fn build_func(def: &ast::FnDef, is_after_fn_keyword: bool) -> String {
    let header: String = def.syntax().children().map(|child| child.text().to_string()).collect();
    if is_after_fn_keyword {
        format!("{} {{ $0 }}", header)
    } else {
        format!("fn {} {{ $0 }}", header)
    }
}

// TODO de-duplicate from `add_missing_impl_members`
fn resolve_target_trait_def(
    db: &impl HirDatabase,
    resolver: &Resolver,
    impl_block: &ast::ImplBlock,
) -> Option<TreeArc<ast::TraitDef>> {
    let ast_path = impl_block.target_trait().map(AstNode::syntax).and_then(ast::PathType::cast)?;
    let hir_path = ast_path.path().and_then(hir::Path::from_ast)?;

    match resolver.resolve_path(db, &hir_path).take_types() {
        Some(hir::Resolution::Def(hir::ModuleDef::Trait(def))) => Some(def.source(db).1),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::completion::{
        completion_item::{do_completion, CompletionItem},
        CompletionKind,
};
    use insta::assert_debug_snapshot_matches;

    fn complete(code: &str) -> Vec<CompletionItem> {
        do_completion(code, CompletionKind::Reference)
    }

    #[test]
    fn test_completes_trait_fn() {
        assert_debug_snapshot_matches!(complete(
            r"
            trait T {
                fn foo(a: u32) -> bool;
                fn bar();
            }

            impl T for () {
                <|>
            }
            ",
        ), @r###"[
    CompletionItem {
        label: "bar",
        source_range: [148; 148),
        delete: [148; 148),
        insert: "fn bar() { $0 }",
        kind: Function
    },
    CompletionItem {
        label: "foo",
        source_range: [148; 148),
        delete: [148; 148),
        insert: "fn foo(a: u32)-> bool { $0 }",
        kind: Function
    }
]"###);
    }

    #[test]
    fn test_completes_trait_after_fn_keyword() {
        assert_debug_snapshot_matches!(complete(
            r"
            trait T {
                fn foo(a: u32) -> bool;
                fn bar();
            }

            impl T for () {
                fn <|>
            }
            ",
        ), @r###"[
    CompletionItem {
        label: "bar",
        source_range: [151; 151),
        delete: [151; 151),
        insert: "bar() { $0 }",
        kind: Function
    },
    CompletionItem {
        label: "foo",
        source_range: [151; 151),
        delete: [151; 151),
        insert: "foo(a: u32)-> bool { $0 }",
        kind: Function
    }
]"###);
    }

    #[test]
    fn test_dont_complete_trait_fn_inside_fn() {
        assert_debug_snapshot_matches!(complete(
            r"
            trait T {
                fn foo();
                fn bar();
            }

            impl T for () {
                fn foo() {
                    <|>
                }
            }
            "), @r###"[
    CompletionItem {
        label: "Self",
        source_range: [165; 165),
        delete: [165; 165),
        insert: "Self",
        kind: TypeParam
    },
    CompletionItem {
        label: "T",
        source_range: [165; 165),
        delete: [165; 165),
        insert: "T",
        kind: Trait
    }
]"###);
    }

    #[ignore] // FIXME: Support associated constants
    #[test]
    fn test_completes_assoc_const() {
        assert_debug_snapshot_matches!(complete(
            r"
            trait T {
                const C: u32;
            }

            impl T for bool {
                <|>
            }
            "), @r###""###);
    }

    #[ignore] // FIXME: Support associated types
    #[test]
    fn test_completes_assoc_type() {
        assert_debug_snapshot_matches!(complete(
            r"
            trait T {
                type A;
            }

            impl T for bool {
                <|>
            }
            "), @r###""###);
    }
}
