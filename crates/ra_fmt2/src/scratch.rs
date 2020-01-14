/// FOR EXPERIMENT WILL REMOVE
// use crate::edit_tree::EditTree;
use crate::edit_tree::EditTree;
// use crate::engine::FmtDiff;
use crate::engine::{FmtDiff, format_str};
use crate::pattern::PatternSet;
use crate::rules::{indentation, spacing};

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
fn indent_method_chain() {
    let rs_file = wrap_fn!(
        r#"let t = foo()
    .bar()
        .baz()
            .foo2();"#
    );

    // let code = "fn main () {let x=0;\nlet y=1;\n}";
//     let fmted = 
// r#"fn main() {
//     let x = 0;
//     let y = 1;
// }
// "#;

    let p = SourceFile::parse(&rs_file);
    let syn_tree = p.syntax_node();
    // println!("{:#?}", syn_tree);
    let indent = indentation();
    let space = spacing();
    println!();

    let fmt = EditTree::new(syn_tree);
    let orig = fmt.text().to_string();
    // println!("{:#?}", fmt);
    let diff = FmtDiff::new(fmt);
    let et: EditTree = diff.spacing_diff(&space).indent_diff(&indent).into();
    let done = et.tokens_to_string().expect("failed");

    let rerun = format_str(&done).expect("rerun failed");

    println!(
        "original : {:?}\nformatted: {:#?}",
        orig,
        done
    );
    assert_eq!(
        done,
        wrap_fn!(
r#"let t = foo()
    .bar()
    .baz()
    .foo2();"#,
            "\n"
        )
    );
    assert_eq!(done, rerun);
}

#[test]
fn indent_function_body() {
    let code = "fn main () {let x=0;\nlet y=1;\n}";
    let fmted = 
r#"fn main() {
    let x = 0;
    let y = 1;
}
"#;

    let p = SourceFile::parse(&code);
    let syn_tree = p.syntax_node();
    // println!("{:#?}", syn_tree);
    let indent = indentation();
    let space = spacing();
    println!();

    let fmt = EditTree::new(syn_tree);
    let orig = fmt.text().to_string();
    // println!("{:#?}", fmt);
    let diff = FmtDiff::new(fmt);
    let et: EditTree = diff.spacing_diff(&space).indent_diff(&indent).into();
    let done = et.tokens_to_string().expect("failed");

    let rerun = format_str(&done).expect("rerun failed");

    println!(
        "original : {:?}\nformatted: {:#?}",
        orig,
        done
    );
    assert_eq!(done, fmted);
    assert_eq!(done, rerun);
}

#[test]
fn show_me_the_progress_space() {
    let rs_file = "pub(crate)struct Test{ x:String }";
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

    println!(
        "original : {:?}\nformatted: {:#?}",
        orig,
        et.tokens_to_string().expect("Edits failed")
    );
    assert_eq!(
        et.tokens_to_string().expect("tokens_to_string"),
        "pub(crate) struct Test { x: String }\n"
    )
}

#[test]
fn combo_test() {
    let rs_file = wrap_fn!(
        r#"let t = Test {
    x: Foo {
    y: 0,
    },
}"#
    );

    let p = SourceFile::parse(&rs_file);
    let syn_tree = p.syntax_node();

    let indent = indentation();
    let space = spacing();

    println!();

    let fmt = EditTree::new(syn_tree);
    let orig = fmt.text().to_string();
    // println!("{:#?}", fmt);
    let diffed: EditTree = FmtDiff::new(fmt).spacing_diff(&space).indent_diff(&indent).into();

    println!(
        "original : {:?}\nformatted: {:#?}",
        orig,
        diffed.tokens_to_string().expect("Edits failed")
    );
    assert_eq!(
        diffed.tokens_to_string().expect("edit failed"),
        wrap_fn!("let t = Test {\n    x: Foo {\n        y: 0,\n    },\n}", "\n")
    );
}
