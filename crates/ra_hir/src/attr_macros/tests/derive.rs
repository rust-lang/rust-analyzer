use ra_syntax::{ast, AstNode, SyntaxNode};
use crate::attr_macros::expand_attr_macro;
use crate::name::AsName;

fn expand_attr_macro_for(code: &str) -> Option<tt::Subtree> {
    let ast = ast::SourceFile::parse(code)
        .ok()
        .unwrap()
        .syntax()
        .clone();

    let attr_node = ast.descendants()
        .find_map(|item| ast::Attr::cast(item))
        .unwrap();
    let target_node = ast.children()
        .find_map(|item| ast::ModuleItem::cast(item))
        .unwrap();

    expand_attr_macro(attr_node, target_node)
}

fn has_trait_implemented(trait_name: &str, ast: &SyntaxNode) -> bool {
    let impl_blocks = ast.descendants()
        .map(|node| ast::ImplBlock::cast(node.clone()))
        .flatten();

    for impl_block in impl_blocks {
        if let Some(ast::TypeRef::PathType(path_type)) = impl_block.target_trait() {
            let name = path_type.path().unwrap()
                .segment().unwrap()
                .name_ref().unwrap()
                .as_name().to_string();

            if name == trait_name {
                return true
            }
        }
    }

    false
}

#[test]
pub fn implement_trait_simple() {
    let tt = expand_attr_macro_for("#[derive(Copy, Clone, Debug)] struct S {}").unwrap();
    let code = tt.to_string();
    let ast_struct = ast::SourceFile::parse(&code)
        .ok()
        .unwrap()
        .syntax()
        .clone();

    let tt = expand_attr_macro_for("#[derive(Copy, Clone, Debug)] enum S {}").unwrap();
    let code = tt.to_string();
    let ast_enum = ast::SourceFile::parse(&code)
        .ok()
        .unwrap()
        .syntax()
        .clone();

    for trait_name in ["Copy", "Clone", "Debug"].into_iter() {
        assert!(has_trait_implemented(trait_name, &ast_struct));
        assert!(has_trait_implemented(trait_name, &ast_enum));
    }
}
