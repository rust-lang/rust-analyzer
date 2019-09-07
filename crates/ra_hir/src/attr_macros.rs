use ra_syntax::ast::{self, AstNode};

#[cfg(test)]
mod tests;

mod derive;

pub fn expand_attr_macro(
    attr_node: ast::Attr,
    target_node: ast::ModuleItem,
) -> Option<tt::Subtree> {
    if derive::is_derive_attr(&attr_node) {
        return derive::expand_derive_attr(attr_node, target_node);
    }

    log::debug!("Unimplemented macro attr: {}", attr_node.syntax().to_string());
    None
}
