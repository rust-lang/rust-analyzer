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

#[test]
fn try_it() {
    let rs_file = "pub(crate)struct Test {x: String}";

    let p = SourceFile::parse(&rs_file);
    let syn_tree = p.syntax_node();
    // fix this, this call is not great do some other way
    let space = spacing();
    let ws_rules = PatternSet::new(space.rules.iter());
    // println!("{:#?}", p);
    syn_tree
        .preorder_with_tokens()
        .filter_map(|ev| match ev {
            ra_syntax::WalkEvent::Enter(el) => Some(el).filter(|t| t.kind() != WHITESPACE),
            ra_syntax::WalkEvent::Leave(_) => None,
        })
        .for_each(|n| println!("{:?}", n));
    println!();
    walk_nodes(&syn_tree).for_each(|n| {
        println!("Node {:?}", n);
        walk_tokens(&n).for_each(|t| println!("    TOK {:?}", t));
    });
    let fmt = EditTree::from_root(&syn_tree, ws_rules);
    println!("{:#?}", fmt);
    fmt.to_string();
}
