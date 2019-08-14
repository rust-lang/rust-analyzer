use crate::edit_tree::EditTree;
use crate::pattern::PatternSet;
/// experiment purposes
///
// use crate::engine;
// use crate::dsl::SpacingDsl;
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};

use ra_syntax::{
    ast::{self, AstNode, AstToken},
    Parse, SmolStr, SourceFile, SyntaxKind,
    SyntaxKind::*,
    SyntaxNode, SyntaxToken, T,
};

// will be removed
#[test]
fn show_me_the_progress() {
    let rs_file = "pub(crate)struct Test {x: String}";

    let p = SourceFile::parse(&rs_file);
    let syn_tree = p.syntax_node();
    // fix this, this call is not great do some other way
    let space = spacing();
    let ws_rules = PatternSet::new(space.rules.iter());

    println!();

    let fmt = EditTree::new(&syn_tree);
    println!("{:#?}", fmt);
    let x = fmt.to_string();
    println!("{:#?}", x);
}
