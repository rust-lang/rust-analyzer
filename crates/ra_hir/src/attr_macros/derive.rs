use ra_syntax::{ast, AstNode, SmolStr};

/// Checks whether `#[attr]` is in the `#[derive(<Traits>)]` form.
pub(crate) fn is_derive_attr(attr_node: &ast::Attr) -> bool {
    if let Some((name, _args)) = attr_node.as_call() {
        // TODO: check for empty _args tree
        name == "derive"
    } else {
        false
    }
}

pub(crate) fn expand_derive_attr(attr_node: ast::Attr, target_node: ast::ModuleItem) -> Option<tt::Subtree> {
    let traits_to_derive = collect_trait_names(&attr_node);

    // TODO:
    None
}

// TODO: try iterate without allocation
fn collect_trait_names(attr_node: &ast::Attr) -> Option<Vec<SmolStr>> {
    let items = attr_node.syntax().children().map(|c| c.to_string()).collect::<Vec<_>>();
    log::warn!("Trait names: {:?}", items);

    // TODO:
    None
}