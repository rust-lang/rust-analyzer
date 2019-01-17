use crate::{
    ast::{self, AstNode, AstToken,},
    string_lexing,
    yellow::{
        SyntaxError,
        SyntaxErrorKind::*,
    },
    token_set::TokenSet,
    Direction,
    SyntaxKind::*
};

pub fn validate_comment(node: &ast::Comment, mut errors: &mut Vec<SyntaxError>) {
    match node.flavor() {
        ast::CommentFlavor::Doc => validate_doc_comment(node, &mut errors),
        ast::CommentFlavor::ModuleDoc => validate_module_doc(node, &mut errors),
        _ => (),
    }
}

const COMMENT_OWNER: TokenSet =
    token_set![CONST_DEF, TYPE_DEF, STRUCT_DEF, ENUM_DEF, FN_DEF, TRAIT_DEF, MODULE];

fn validate_doc_comment(node: &ast::Comment, errors: &mut Vec<SyntaxError>) {
    if let Some(parent) = node.syntax().parent() {
        if !COMMENT_OWNER.contains(parent.kind()) {
            // errors.push();
        }
    }
}

fn validate_module_doc(node: &ast::Comment, errors: &mut Vec<SyntaxError>) {
    let mut valid_parent = false;
    if let Some(parent) = node.syntax().parent() {
        // I'm not certain if we should allow module doc comments in Trait item lists
        // My inclination is not
        if parent.kind() == ITEM_LIST {
            if let Some(parent) = parent.parent() {
                if parent.kind() == MODULE {
                    valid_parent = true;
                }
            }
        }
        if parent.kind() == SOURCE_FILE {
            valid_parent = true;
        }
    }
    if !valid_parent {
        errors.push(SyntaxError::new(
            ModuleDocInvalidParent,
            node.syntax().range(),
        ));
    } else {
        // This does not yet handle ensuring that the Attrs are module level
        if !node.syntax().siblings(Direction::Prev).any(|node| {
            !(node.kind() == WHITESPACE)
                || !(ast::Comment::cast(node)
                    .map(|inner| inner.flavor() == ast::CommentFlavor::Doc)
                    == Some(true))
                || !(node.kind() == ATTR)
        }) {
            errors.push(SyntaxError::new(
                ModuleDocNotFirstSibling,
                node.syntax().range(),
            ));
        }
    }
}
