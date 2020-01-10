/// FOR EXPERIMENT WILL REMOVE

// use crate::edit_tree::EditTree;
use crate::et2::EditTree;
// use crate::engine::FmtDiff;
use crate::eng2::FmtDiff;
use crate::pattern::PatternSet;
use crate::rules::{spacing, indentation};
use crate::trav_util::{walk, walk_nodes, walk_tokens};

use ra_syntax::{
    ast::{self, AstNode, AstToken},
    Parse, SmolStr, SourceFile, SyntaxKind,
    SyntaxKind::*,
    SyntaxNode, SyntaxToken, T,
};

///
/// 
/// WILL BE REMOVED, or moved into actual tests
/// 
/// 

#[test]
fn show_me_the_indent_progress() {
    let rs_file = wrap_fn!(r#"let t = foo()
    .bar()
        .baz()
            .foo2();"#);

    let p = SourceFile::parse(&rs_file);
    let syn_tree = p.syntax_node();
    // println!("{:#?}", syn_tree);
    let indent = indentation();

    println!();

    let fmt = EditTree::new(syn_tree);
    let orig = fmt.text().to_string();
    // println!("{:#?}", fmt);
    let diff = FmtDiff::new(fmt);
    let et: EditTree = diff.indent_diff(&indent).into();


    println!("original: {:?}\nformatted: {:#?}", orig, et.tokens_to_string().expect("Edits failed"));
    assert_eq!(et.tokens_to_string().expect("to string failed"), wrap_fn!(
r#"let t = foo()
    .bar()
    .baz()
    .foo2();"#, "\n"));

}

#[test]
fn show_me_the_progress_space() {
    let rs_file = "pub(crate)struct Test    {x:String    }  ";
    let _rs_if = wrap_fn!("let x = if y {0} else {1};");

    let p = SourceFile::parse(&rs_file);
    let syn_tree = p.syntax_node();
    println!("{:#?}", syn_tree);
    let space = spacing();
    println!();

    let fmt = EditTree::new(syn_tree);
    let orig = fmt.text().to_string();
    //println!("{:#?}", fmt);
    let diff = FmtDiff::new(fmt);
    let et: EditTree = diff.spacing_diff(&space).into();

    println!("original: {:?}\nformatted: {:#?}", orig, et.tokens_to_string().expect("Edits failed"));
    assert_eq!(et.tokens_to_string().expect("tokens_to_string"), "pub(crate) struct Test { x: String }\n")
}

#[test]
fn combo_test() {
    let rs_file = wrap_fn!(
r#"let t = Test {
    x: Foo {
    y: 0,
    },
}"#);

    let p = SourceFile::parse(&rs_file);
    let syn_tree = p.syntax_node();

    let indent = indentation();
    let space = spacing();

    println!();

    let fmt = EditTree::new(syn_tree);
    let orig = fmt.text().to_string();
    // println!("{:#?}", fmt);
    let diffed: EditTree = FmtDiff::new(fmt)
        .spacing_diff(&space)
        .indent_diff(&indent)
        .into();
    
    println!("original: {:?}\nformatted: {:#?}", orig, diffed.tokens_to_string().expect("Edits failed"));
}
