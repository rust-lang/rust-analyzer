use ra_syntax::{
    algo::{find_node_at_offset},
    ast::{self, FnDef, ImplItem, ImplItemKind, NameOwner},
    AstNode, TreeArc, SmolStr
};
use hir::{Resolver, db::HirDatabase};

use crate::completion::{CompletionContext, Completions, completion_item::{CompletionItem, CompletionKind}};

pub(super) fn complete_impl_fn(acc: &mut Completions, ctx: &CompletionContext) {
    let impl_node = match find_node_at_offset::<ast::ImplBlock>(ctx.leaf, ctx.offset) {
        Some(impl_node) => impl_node,
        None => return,
    };
    let impl_item_list = impl_node.item_list().unwrap();

    // TODO remove unwraps
    let trait_def = resolve_target_trait_def(ctx.db, &ctx.resolver, impl_node).unwrap();

    let trait_items =
        trait_def.syntax().descendants().find_map(ast::ItemList::cast).unwrap().impl_items();
    let fn_def_opt = |kind| if let ImplItemKind::FnDef(def) = kind { Some(def) } else { None };
    let def_name = |def| -> Option<&SmolStr> { FnDef::name(def).map(ast::Name::text) };

    let trait_fns = trait_items.map(ImplItem::kind).filter_map(fn_def_opt).collect::<Vec<_>>();

    let impl_items = impl_item_list.impl_items();
    let impl_fns = impl_items.map(ImplItem::kind).filter_map(fn_def_opt).collect::<Vec<_>>();

    trait_fns
        .into_iter()
        .filter_map(|t| def_name(t))
        .filter(|name| impl_fns.iter().all(|i| def_name(i) != Some(name)))
        .for_each(|name| {
            CompletionItem::new(CompletionKind::Reference, ctx.source_range(), name.to_string())
                .add_to(acc)
        });
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
                fn foo();
            }

            impl T for () {
                <|>
            }
            ",
        );
    }
}
