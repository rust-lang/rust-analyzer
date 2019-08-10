/// experiment purposes
///
// use crate::engine;
use crate::dsl::SpacingDsl;
use crate::rules::spacing;

use ra_syntax::{
    ast::{self, AstNode, AstToken},
    Parse, SmolStr, SourceFile, SyntaxKind,
    SyntaxKind::*,
    SyntaxNode, SyntaxToken, T,
};

#[test]
fn try_it() {
    let rs_file = "pub(crate)struct Test {
x: String,
}
";

    let p = SourceFile::parse(&rs_file);
    let syn_tree = p.syntax_node();
    //let rules = spacing();

    println!("{:#?}", p);
    syn_tree
        .preorder_with_tokens()
        .filter_map(|ev| match ev {
            ra_syntax::WalkEvent::Enter(el) => Some(el).filter(|t| t.kind() != WHITESPACE),
            ra_syntax::WalkEvent::Leave(_) => None,
        })
        .for_each(|n| println!("{:?}", n));

    
    //engine::format_pass(&rules, &syn_tree)
}
