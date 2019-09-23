use ra_syntax::{
    ast::{self, NameOwner},
    AstNode, SyntaxKind,
};

/// Checks whether `#[attr]` is in the `#[derive(<Traits>)]` form.
pub(crate) fn is_derive_attr(attr_node: &ast::Attr) -> bool {
    if let Some((name, _args)) = attr_node.as_call() {
        // FIXME: check for empty _args tree
        name == "derive"
    } else {
        false
    }
}

pub(crate) fn expand_derive_attr(
    attr_node: ast::Attr,
    target_node: ast::ModuleItem,
) -> Option<tt::Subtree> {
    let traits_to_derive = collect_trait_names(&attr_node);

    let tts = traits_to_derive
        .into_iter()
        .flatten()
        .map(|tr| implement_trait_simple(&tr, &target_node))
        .flatten()
        .map(|subtree| tt::TokenTree::Subtree(subtree))
        .collect::<Vec<_>>();

    if !tts.is_empty() {
        let tt = tt::Subtree { delimiter: tt::Delimiter::None, token_trees: tts };
        Some(tt)
    } else {
        None
    }
}

fn collect_trait_names(attr_node: &ast::Attr) -> Option<Vec<String>> {
    if let Some((_, tt)) = attr_node.as_call() {
        let items = tt
            .syntax()
            .children_with_tokens()
            .into_iter()
            .filter(|token| token.kind() == SyntaxKind::IDENT)
            .map(|c| c.to_string())
            .collect::<Vec<_>>();

        Some(items)
    } else {
        None
    }
}

fn item_name(item: &ast::ModuleItem) -> Option<String> {
    if let Some(s) = ast::StructDef::cast(item.syntax().clone()) {
        s.name().and_then(|n| Some(n.text().to_string()))
    } else if let Some(e) = ast::EnumDef::cast(item.syntax().clone()) {
        e.name().and_then(|n| Some(n.text().to_string()))
    } else {
        None
    }
}

fn implement_trait_simple(trait_name: &str, target: &ast::ModuleItem) -> Option<tt::Subtree> {
    if let Some(name) = item_name(target) {
        let impl_code = format!("impl {} for {} {{}}", trait_name, name);
        let tt = mbe::text_to_tokentree(&impl_code);

        return Some(tt);
    }

    None
}
