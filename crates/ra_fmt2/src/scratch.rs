use crate::edit_tree::EditTree;
use crate::engine::FmtDiff;
use crate::pattern::PatternSet;
use crate::fmt_diff::FmtDiff;
/// experiment purposes
///
// use crate::engine;
// use crate::dsl::SpacingDsl;
use crate::rules::{spacing, indentation};
use crate::trav_util::{walk, walk_nodes, walk_tokens};

use ra_syntax::{
    ast::{self, AstNode, AstToken},
    Parse, SmolStr, SourceFile, SyntaxKind,
    SyntaxKind::*,
    SyntaxNode, SyntaxToken, T,
};

/// will be removed
#[test]
fn show_me_the_indent_progress() {
    let rs_file = 
r#"pub(crate) struct Test{
    x: String
}  "#;

    let p = SourceFile::parse(&rs_file);
    let syn_tree = p.syntax_node();
    //println!("{:?}", syn_tree);
    let indent = indentation();

    println!();

    let fmt = EditTree::new(syn_tree);
    let orig = fmt.text().to_string();
    println!("{:#?}", fmt);
    let diff = FmtDiff::new(fmt);
    let et = diff.indent_diff(&indent);

    println!("original: {:?}\nformatted: {:#?}", orig, et.apply_edits());
    //assert_eq!(et.apply_edits(), "pub(crate) struct Test { x: String }\n")
}

fn show_me_the_progress() {
    let rs_file = "pub(crate) struct Test{x: String}  ";
    let rs_arr = "fn main() { let examp = [0,1,2]; }";

    let p = SourceFile::parse(&rs_file);
    let syn_tree = p.syntax_node();
    //println!("{:?}", syn_tree);
    let space = spacing();
    let ws_rules = PatternSet::new(space.rules.iter());

    println!();

    let fmt = EditTree::new(syn_tree);
    let orig = fmt.text().to_string();
    println!("{:#?}", fmt);
    let diff = FmtDiff::new(fmt);
    let et = diff.spacing_diff(&space);

    println!("original: {:?}\nformatted: {:#?}", orig, et.apply_edits());
    assert_eq!(et.apply_edits(), "pub(crate) struct Test { x: String }\n")
}
