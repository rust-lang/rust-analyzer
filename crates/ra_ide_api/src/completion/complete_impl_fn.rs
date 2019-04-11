use ra_syntax::{
    algo::{find_node_at_offset},
    ast::{self, ImplItem, ImplItemKind},
    SyntaxKind::{CONST_DEF, FN_DEF, TYPE_ALIAS_DEF},
    AstNode, TreeArc, SmolStr, SyntaxKind
};
use hir::{Resolver, db::HirDatabase};

use crate::completion::{
    completion_item::{CompletionItem, CompletionKind, CompletionItemKind},
    CompletionContext, Completions,
};

pub(super) fn complete_impl_fn(acc: &mut Completions, ctx: &CompletionContext) {
    let after_keyword = match ctx.token.prev_sibling_or_token().map(|t| t.kind()) {
        k @ Some(FN_DEF) | k @ Some(TYPE_ALIAS_DEF) | k @ Some(CONST_DEF) => k,
        _ if ctx.is_new_item => None,
        _ => return,
    };

    let impl_node = match find_node_at_offset::<ast::ImplBlock>(ctx.token.parent(), ctx.offset) {
        Some(impl_node) => impl_node,
        None => return,
    };

    let found_impl_names = match impl_node.item_list() {
        Some(impl_item_list) => impl_item_list.impl_items().map(impl_item_text).collect::<Vec<_>>(),
        None => return,
    };

    let trait_def = match resolve_target_trait_def(ctx.db, &ctx.resolver, impl_node) {
        Some(trait_def) => trait_def,
        None => return,
    };
    let trait_items = match trait_def.item_list() {
        Some(item_list) => item_list,
        None => return,
    };

    for impl_item in trait_items.impl_items() {
        if let Some(name) = impl_item_text(impl_item) {
            if found_impl_names.iter().any(|n| *n == Some(name)) {
                continue;
            }

            let item = build_item(impl_item, after_keyword);
            CompletionItem::new(CompletionKind::Reference, ctx.source_range(), name.clone())
                .kind(CompletionItemKind::Function)
                .insert_snippet(item)
                .add_to(acc);
        }
    }
}

fn impl_item_text(item: &ImplItem) -> Option<&SmolStr> {
    let name = match item.kind() {
        ImplItemKind::FnDef(f) => ast::NameOwner::name(f),
        ImplItemKind::TypeAliasDef(ta) => ast::NameOwner::name(ta),
        ImplItemKind::ConstDef(c) => ast::NameOwner::name(c),
    };
    name.map(ast::Name::text)
}

fn build_item(def: &ast::ImplItem, after_keyword: Option<SyntaxKind>) -> String {
    let header: String = def.syntax().children().map(|child| child.text().to_string()).collect();

    let keyword = |kw| if after_keyword.is_some() { String::new() } else { format!("{} ", kw) };
    match def.kind() {
        ImplItemKind::FnDef(_) => format!("{}{} {{ $0 }}", keyword("fn"), header),
        ImplItemKind::TypeAliasDef(_) => format!("{}{} = $0;", keyword("type"), header),
        ImplItemKind::ConstDef(_) => format!("{}{} = $0;", keyword("const"), header),
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

    #[ignore] // TODO support associated constants
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
            "), @r###"[
    CompletionItem {
        label: "A",
        source_range: [108; 108),
        delete: [108; 108),
        insert: "type A = $0;",
        kind: Function
    }
]"###);
    }
}
