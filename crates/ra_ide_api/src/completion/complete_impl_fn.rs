use ra_syntax::{
    algo::{find_node_at_offset},
    ast::{self, FnDef, ImplItem, ImplItemKind, NameOwner},
    AstNode, TreeArc, SmolStr,
};
use hir::{Resolver, db::HirDatabase};

use crate::completion::{
    completion_item::{CompletionItem, CompletionKind, CompletionItemKind},
    CompletionContext, Completions,
};

// TODO Avoid double `fn` in `fn <|>`
pub(super) fn complete_impl_fn(acc: &mut Completions, ctx: &CompletionContext) {
    if !ctx.is_new_item {
        return;
    }

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
        .map(|t| (def_name(t).unwrap(), build_func(t)))
        .for_each(|(name, func)| {
            CompletionItem::new(CompletionKind::Reference, ctx.source_range(), name.clone())
                .kind(CompletionItemKind::Function)
                .insert_snippet(func)
                .add_to(acc)
        });
}

fn build_func(def: &ast::FnDef) -> String {
    let header: String = def.syntax().children().map(|child| child.text().to_string()).collect();
    format!("fn {} {{ $0 }}", header)
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
    use crate::completion::{CompletionKind, completion_item::check_completion};

    fn complete(test_name: &str, code: &str) {
        check_completion(test_name, code, CompletionKind::Reference)
    }

    #[test]
    fn test_completes_trait_fn() {
        complete(
            "completes_trait_fn",
            r"
            trait T {
                fn foo(a: u32) -> bool;
                fn bar();
            }

            impl T for () {
                <|>
            }
            ",
        );
    }

    #[test]
    fn test_dont_complete_trait_fn_inside_fn() {
        complete(
            "dont_complete_trait_fn_inside_fn",
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
            ",
        );
    }

    #[ignore] // FIXME: Support associated constants
    #[test]
    fn test_completes_assoc_const() {
        complete(
            "completes_assoc_const",
            r"
            trait T {
                const C: u32;
            }

            impl T for bool {
                <|>
            }
            ",
        );
    }

    #[ignore] // FIXME: Support associated types
    #[test]
    fn test_completes_assoc_type() {
        complete(
            "completes_assoc_type",
            r"
            trait T {
                type A;
            }

            impl T for bool {
                <|>
            }
            ",
        );
    }
}
