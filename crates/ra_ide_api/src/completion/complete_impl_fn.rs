use ra_syntax::{
    algo::{find_node_at_offset},
    ast::{self, FnDef, ImplItem, ImplItemKind, NameOwner},
    AstNode, TreeArc, SmolStr, SyntaxKind,
};
use hir::{Resolver, db::HirDatabase};

use crate::completion::{
    completion_item::{CompletionItem, CompletionKind, CompletionItemKind},
    CompletionContext, Completions,
};

pub(super) fn complete_impl_fn(acc: &mut Completions, ctx: &CompletionContext) {
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
        let trait_items = match trait_def.syntax().descendants().find_map(ast::ItemList::cast) {
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
    let mut buf = String::new();

    for child in def.syntax().children() {
        match (child.prev_sibling().map(|c| c.kind()), child.kind()) {
            (_, SyntaxKind::SEMI) => buf.push_str(" { unimplemented!() }"),
            (_, SyntaxKind::ATTR) | (_, SyntaxKind::COMMENT) => {}
            (Some(SyntaxKind::ATTR), SyntaxKind::WHITESPACE)
            | (Some(SyntaxKind::COMMENT), SyntaxKind::WHITESPACE) => {}
            _ => child.text().push_to(&mut buf),
        };
    }

    format!("fn {} {{ $0 }}", buf.trim_end())
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
            }

            impl T for () {
                <|>
            }
            ",
        );
    }
}
