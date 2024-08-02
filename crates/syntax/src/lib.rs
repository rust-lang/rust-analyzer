//! Syntax Tree library used throughout the rust-analyzer.
//!
//! Properties:
//!   - easy and fast incremental re-parsing
//!   - graceful handling of errors
//!   - full-fidelity representation (*any* text can be precisely represented as
//!     a syntax tree)
//!
//! For more information, see the [RFC]. Current implementation is inspired by
//! the [Swift] one.
//!
//! The most interesting modules here are `syntax_node` (which defines concrete
//! syntax tree) and `ast` (which defines abstract syntax tree on top of the
//! CST). The actual parser live in a separate `parser` crate, though the
//! lexer lives in this crate.
//!
//! See `api_walkthrough` test in this file for a quick API tour!
//!
//! [RFC]: <https://github.com/rust-lang/rfcs/pull/2256>
//! [Swift]: <https://github.com/apple/swift/blob/13d593df6f359d0cb2fc81cfaac273297c539455/lib/Syntax/README.md>

#![cfg_attr(feature = "in-rust-tree", feature(rustc_private))]
#![warn(rust_2018_idioms, unused_lifetimes)]

#[cfg(not(feature = "in-rust-tree"))]
extern crate ra_ap_rustc_lexer as rustc_lexer;
#[cfg(feature = "in-rust-tree")]
extern crate rustc_lexer;

mod parsing;
mod ptr;
mod syntax_error;
mod syntax_node;
#[cfg(test)]
mod tests;
mod token_text;
mod validation;

pub mod algo;
pub mod ast;
#[doc(hidden)]
pub mod fuzz;
pub mod hacks;
pub mod ted;
pub mod utils;

use std::marker::PhantomData;

use stdx::format_to;
use text_edit::Indel;
use triomphe::Arc;

pub use crate::{
    ast::{AstNode, AstToken},
    ptr::{AstPtr, SyntaxNodePtr},
    syntax_error::SyntaxError,
    syntax_node::{
        PreorderWithTokens, RustLanguage, SyntaxElement, SyntaxElementChildren, SyntaxNode,
        SyntaxNodeChildren, SyntaxToken, SyntaxTreeBuilder,
    },
    token_text::TokenText,
};
pub use parser::{Edition, SyntaxKind, T};
pub use rowan::{
    api::Preorder, Direction, GreenNode, NodeOrToken, SyntaxText, TextRange, TextSize,
    TokenAtOffset, WalkEvent,
};
pub use rustc_lexer::unescape;
pub use smol_str::{format_smolstr, SmolStr};

#[cfg(test)]
use ast::generated::vst_nodes;

/// `Parse` is the result of the parsing: a syntax tree and a collection of
/// errors.
///
/// Note that we always produce a syntax tree, even for completely invalid
/// files.
#[derive(Debug, PartialEq, Eq)]
pub struct Parse<T> {
    green: GreenNode,
    errors: Option<Arc<[SyntaxError]>>,
    _ty: PhantomData<fn() -> T>,
}

impl<T> Clone for Parse<T> {
    fn clone(&self) -> Parse<T> {
        Parse { green: self.green.clone(), errors: self.errors.clone(), _ty: PhantomData }
    }
}

impl<T> Parse<T> {
    fn new(green: GreenNode, errors: Vec<SyntaxError>) -> Parse<T> {
        Parse {
            green,
            errors: if errors.is_empty() { None } else { Some(errors.into()) },
            _ty: PhantomData,
        }
    }

    pub fn syntax_node(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }

    pub fn errors(&self) -> Vec<SyntaxError> {
        let mut errors = if let Some(e) = self.errors.as_deref() { e.to_vec() } else { vec![] };
        validation::validate(&self.syntax_node(), &mut errors);
        errors
    }
}

impl<T: AstNode> Parse<T> {
    /// Converts this parse result into a parse result for an untyped syntax tree.
    pub fn to_syntax(self) -> Parse<SyntaxNode> {
        Parse { green: self.green, errors: self.errors, _ty: PhantomData }
    }

    /// Gets the parsed syntax tree as a typed ast node.
    ///
    /// # Panics
    ///
    /// Panics if the root node cannot be casted into the typed ast node
    /// (e.g. if it's an `ERROR` node).
    pub fn tree(&self) -> T {
        T::cast(self.syntax_node()).unwrap()
    }

    /// Converts from `Parse<T>` to [`Result<T, Vec<SyntaxError>>`].
    pub fn ok(self) -> Result<T, Vec<SyntaxError>> {
        match self.errors() {
            errors if !errors.is_empty() => Err(errors),
            _ => Ok(self.tree()),
        }
    }
}

impl Parse<SyntaxNode> {
    pub fn cast<N: AstNode>(self) -> Option<Parse<N>> {
        if N::cast(self.syntax_node()).is_some() {
            Some(Parse { green: self.green, errors: self.errors, _ty: PhantomData })
        } else {
            None
        }
    }
}

impl Parse<SourceFile> {
    pub fn debug_dump(&self) -> String {
        let mut buf = format!("{:#?}", self.tree().syntax());
        for err in self.errors() {
            format_to!(buf, "error {:?}: {}\n", err.range(), err);
        }
        buf
    }

    pub fn reparse(&self, indel: &Indel, edition: Edition) -> Parse<SourceFile> {
        self.incremental_reparse(indel).unwrap_or_else(|| self.full_reparse(indel, edition))
    }

    fn incremental_reparse(&self, indel: &Indel) -> Option<Parse<SourceFile>> {
        // FIXME: validation errors are not handled here
        parsing::incremental_reparse(
            self.tree().syntax(),
            indel,
            self.errors.as_deref().unwrap_or_default().iter().cloned(),
        )
        .map(|(green_node, errors, _reparsed_range)| Parse {
            green: green_node,
            errors: if errors.is_empty() { None } else { Some(errors.into()) },
            _ty: PhantomData,
        })
    }

    fn full_reparse(&self, indel: &Indel, edition: Edition) -> Parse<SourceFile> {
        let mut text = self.tree().syntax().text().to_string();
        indel.apply(&mut text);
        SourceFile::parse(&text, edition)
    }
}

impl ast::Expr {
    /// Parses an `ast::Expr` from `text`.
    ///
    /// Note that if the parsed root node is not a valid expression, [`Parse::tree`] will panic.
    /// For example:
    /// ```rust,should_panic
    /// # use syntax::{ast, Edition};
    /// ast::Expr::parse("let fail = true;", Edition::CURRENT).tree();
    /// ```
    pub fn parse(text: &str, edition: Edition) -> Parse<ast::Expr> {
        let _p = tracing::info_span!("Expr::parse").entered();
        let (green, errors) = parsing::parse_text_at(text, parser::TopEntryPoint::Expr, edition);
        let root = SyntaxNode::new_root(green.clone());

        assert!(
            ast::Expr::can_cast(root.kind()) || root.kind() == SyntaxKind::ERROR,
            "{:?} isn't an expression",
            root.kind()
        );
        Parse::new(green, errors)
    }
}

/// `SourceFile` represents a parse tree for a single Rust file.
pub use crate::ast::SourceFile;

impl SourceFile {
    pub fn parse(text: &str, edition: Edition) -> Parse<SourceFile> {
        let _p = tracing::info_span!("SourceFile::parse").entered();
        let (green, errors) = parsing::parse_text(text, edition);
        let root = SyntaxNode::new_root(green.clone());

        assert_eq!(root.kind(), SyntaxKind::SOURCE_FILE);
        Parse::new(green, errors)
    }
}

impl ast::TokenTree {
    pub fn reparse_as_comma_separated_expr(
        self,
        edition: parser::Edition,
    ) -> Parse<ast::MacroEagerInput> {
        let tokens = self.syntax().descendants_with_tokens().filter_map(NodeOrToken::into_token);

        let mut parser_input = parser::Input::default();
        let mut was_joint = false;
        for t in tokens {
            let kind = t.kind();
            if kind.is_trivia() {
                was_joint = false
            } else if kind == SyntaxKind::IDENT {
                let token_text = t.text();
                let contextual_kw =
                    SyntaxKind::from_contextual_keyword(token_text).unwrap_or(SyntaxKind::IDENT);
                parser_input.push_ident(contextual_kw);
            } else {
                if was_joint {
                    parser_input.was_joint();
                }
                parser_input.push(kind);
                // Tag the token as joint if it is float with a fractional part
                // we use this jointness to inform the parser about what token split
                // event to emit when we encounter a float literal in a field access
                if kind == SyntaxKind::FLOAT_NUMBER {
                    if !t.text().ends_with('.') {
                        parser_input.was_joint();
                    } else {
                        was_joint = false;
                    }
                } else {
                    was_joint = true;
                }
            }
        }

        let parser_output = parser::TopEntryPoint::MacroEagerInput.parse(&parser_input, edition);

        let mut tokens =
            self.syntax().descendants_with_tokens().filter_map(NodeOrToken::into_token);
        let mut text = String::new();
        let mut pos = TextSize::from(0);
        let mut builder = SyntaxTreeBuilder::default();
        for event in parser_output.iter() {
            match event {
                parser::Step::Token { kind, n_input_tokens } => {
                    let mut token = tokens.next().unwrap();
                    while token.kind().is_trivia() {
                        let text = token.text();
                        pos += TextSize::from(text.len() as u32);
                        builder.token(token.kind(), text);

                        token = tokens.next().unwrap();
                    }
                    text.push_str(token.text());
                    for _ in 1..n_input_tokens {
                        let token = tokens.next().unwrap();
                        text.push_str(token.text());
                    }

                    pos += TextSize::from(text.len() as u32);
                    builder.token(kind, &text);
                    text.clear();
                }
                parser::Step::FloatSplit { ends_in_dot: has_pseudo_dot } => {
                    let token = tokens.next().unwrap();
                    let text = token.text();

                    match text.split_once('.') {
                        Some((left, right)) => {
                            assert!(!left.is_empty());
                            builder.start_node(SyntaxKind::NAME_REF);
                            builder.token(SyntaxKind::INT_NUMBER, left);
                            builder.finish_node();

                            // here we move the exit up, the original exit has been deleted in process
                            builder.finish_node();

                            builder.token(SyntaxKind::DOT, ".");

                            if has_pseudo_dot {
                                assert!(right.is_empty(), "{left}.{right}");
                            } else {
                                assert!(!right.is_empty(), "{left}.{right}");
                                builder.start_node(SyntaxKind::NAME_REF);
                                builder.token(SyntaxKind::INT_NUMBER, right);
                                builder.finish_node();

                                // the parser creates an unbalanced start node, we are required to close it here
                                builder.finish_node();
                            }
                        }
                        None => unreachable!(),
                    }
                    pos += TextSize::from(text.len() as u32);
                }
                parser::Step::Enter { kind } => builder.start_node(kind),
                parser::Step::Exit => builder.finish_node(),
                parser::Step::Error { msg } => builder.error(msg.to_owned(), pos),
            }
        }

        let (green, errors) = builder.finish_raw();
        Parse::new(green, errors)
    }
}

/// Matches a `SyntaxNode` against an `ast` type.
///
/// # Example:
///
/// ```ignore
/// match_ast! {
///     match node {
///         ast::CallExpr(it) => { ... },
///         ast::MethodCallExpr(it) => { ... },
///         ast::MacroCall(it) => { ... },
///         _ => None,
///     }
/// }
/// ```
#[macro_export]
macro_rules! match_ast {
    (match $node:ident { $($tt:tt)* }) => { $crate::match_ast!(match ($node) { $($tt)* }) };

    (match ($node:expr) {
        $( $( $path:ident )::+ ($it:pat) => $res:expr, )*
        _ => $catch_all:expr $(,)?
    }) => {{
        $( if let Some($it) = $($path::)+cast($node.clone()) { $res } else )*
        { $catch_all }
    }};
}

/// This test does not assert anything and instead just shows off the crate's
/// API.
#[test]
fn api_walkthrough() {
    use ast::{HasModuleItem, HasName};

    let source_code = "
        fn foo() {
            1 + 1
        }
    ";
    // `SourceFile` is the main entry point.
    //
    // The `parse` method returns a `Parse` -- a pair of syntax tree and a list
    // of errors. That is, syntax tree is constructed even in presence of errors.
    let parse = SourceFile::parse(source_code, parser::Edition::CURRENT);
    assert!(parse.errors().is_empty());

    // The `tree` method returns an owned syntax node of type `SourceFile`.
    // Owned nodes are cheap: inside, they are `Rc` handles to the underling data.
    let file: SourceFile = parse.tree();

    // `SourceFile` is the root of the syntax tree. We can iterate file's items.
    // Let's fetch the `foo` function.
    let mut func = None;
    for item in file.items() {
        match item {
            ast::Item::Fn(f) => func = Some(f),
            _ => unreachable!(),
        }
    }
    let func: ast::Fn = func.unwrap();

    // Each AST node has a bunch of getters for children. All getters return
    // `Option`s though, to account for incomplete code. Some getters are common
    // for several kinds of node. In this case, a trait like `ast::NameOwner`
    // usually exists. By convention, all ast types should be used with `ast::`
    // qualifier.
    let name: Option<ast::Name> = func.name();
    let name = name.unwrap();
    assert_eq!(name.text(), "foo");

    // Let's get the `1 + 1` expression!
    let body: ast::BlockExpr = func.body().unwrap();
    let stmt_list: ast::StmtList = body.stmt_list().unwrap();
    let expr: ast::Expr = stmt_list.tail_expr().unwrap();

    // Enums are used to group related ast nodes together, and can be used for
    // matching. However, because there are no public fields, it's possible to
    // match only the top level enum: that is the price we pay for increased API
    // flexibility
    let bin_expr: &ast::BinExpr = match &expr {
        ast::Expr::BinExpr(e) => e,
        _ => unreachable!(),
    };

    // Besides the "typed" AST API, there's an untyped CST one as well.
    // To switch from AST to CST, call `.syntax()` method:
    let expr_syntax: &SyntaxNode = expr.syntax();

    // Note how `expr` and `bin_expr` are in fact the same node underneath:
    assert!(expr_syntax == bin_expr.syntax());

    // To go from CST to AST, `AstNode::cast` function is used:
    let _expr: ast::Expr = match ast::Expr::cast(expr_syntax.clone()) {
        Some(e) => e,
        None => unreachable!(),
    };

    // The two properties each syntax node has is a `SyntaxKind`:
    assert_eq!(expr_syntax.kind(), SyntaxKind::BIN_EXPR);

    // And text range:
    assert_eq!(expr_syntax.text_range(), TextRange::new(32.into(), 37.into()));

    // You can get node's text as a `SyntaxText` object, which will traverse the
    // tree collecting token's text:
    let text: SyntaxText = expr_syntax.text();
    assert_eq!(text.to_string(), "1 + 1");

    // There's a bunch of traversal methods on `SyntaxNode`:
    assert_eq!(expr_syntax.parent().as_ref(), Some(stmt_list.syntax()));
    assert_eq!(stmt_list.syntax().first_child_or_token().map(|it| it.kind()), Some(T!['{']));
    assert_eq!(
        expr_syntax.next_sibling_or_token().map(|it| it.kind()),
        Some(SyntaxKind::WHITESPACE)
    );

    // As well as some iterator helpers:
    let f = expr_syntax.ancestors().find_map(ast::Fn::cast);
    assert_eq!(f, Some(func));
    assert!(expr_syntax.siblings_with_tokens(Direction::Next).any(|it| it.kind() == T!['}']));
    assert_eq!(
        expr_syntax.descendants_with_tokens().count(),
        8, // 5 tokens `1`, ` `, `+`, ` `, `1`
           // 2 child literal expressions: `1`, `1`
           // 1 the node itself: `1 + 1`
    );

    // There's also a `preorder` method with a more fine-grained iteration control:
    let mut buf = String::new();
    let mut indent = 0;
    for event in expr_syntax.preorder_with_tokens() {
        match event {
            WalkEvent::Enter(node) => {
                let text = match &node {
                    NodeOrToken::Node(it) => it.text().to_string(),
                    NodeOrToken::Token(it) => it.text().to_owned(),
                };
                format_to!(buf, "{:indent$}{:?} {:?}\n", " ", text, node.kind(), indent = indent);
                indent += 2;
            }
            WalkEvent::Leave(_) => indent -= 2,
        }
    }
    assert_eq!(indent, 0);
    assert_eq!(
        buf.trim(),
        r#"
"1 + 1" BIN_EXPR
  "1" LITERAL
    "1" INT_NUMBER
  " " WHITESPACE
  "+" PLUS
  " " WHITESPACE
  "1" LITERAL
    "1" INT_NUMBER
"#
        .trim()
    );

    // To recursively process the tree, there are three approaches:
    // 1. explicitly call getter methods on AST nodes.
    // 2. use descendants and `AstNode::cast`.
    // 3. use descendants and `match_ast!`.
    //
    // Here's how the first one looks like:
    let exprs_cast: Vec<String> = file
        .syntax()
        .descendants()
        .filter_map(ast::Expr::cast)
        .map(|expr| expr.syntax().text().to_string())
        .collect();

    // An alternative is to use a macro.
    let mut exprs_visit = Vec::new();
    for node in file.syntax().descendants() {
        match_ast! {
            match node {
                ast::Expr(it) => {
                    let res = it.syntax().text().to_string();
                    exprs_visit.push(res);
                },
                _ => (),
            }
        }
    }
    assert_eq!(exprs_cast, exprs_visit);
}

// Verus tests
// Do "cargo test --package syntax --lib -- tests"

#[test]
fn verus_walkthrough0() {
    use ast::HasModuleItem;

    let source_code = "verus!{
        proof fn my_proof_fun(x: int, y: int)
            {
                let z = 1;
            }

        spec fn identity(x: u32) -> u32 {
            x
        }

        proof fn sq(x: nat) -> (squared: nat) {
            x
        }
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();

    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough1() {
    use ast::HasModuleItem;

    let source_code = "verus!{
        proof fn my_proof_fun(x: int, y: int)
            requires
                x < 100,
                y < 100,
            ensures
                x + y < 200,
            {
                assert(x + y < 200);
            }
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();

    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough1_1() {
    use ast::HasModuleItem;

    let source_code = "
verus! {
    spec fn identity(x: u32) -> u32 {
        x
    }
    proof fn proof_index(a: u32, offset: u32)
    requires
        offset < 1000,
    ensures
        offset < 1000,
    {
        let mut x:u32 = 10;
        x = identity(x);
        assert(offset < 100);
    }
}
";

    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();

    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        // match item {
        //     ast::Item::Fn(f) => func = Some(f),
        //     _ => unreachable!(),
        // }
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough2() {
    use ast::HasModuleItem;

    let source_code = "verus!{
        proof fn my_proof_fun(x: int, y: int) -> (sum: int)
            requires
                x < 100,
                y < 100,
            ensures
                sum < 200,
        {
            x + y
        }
        spec fn my_spec_fun(x: int, y: int) -> int
            recommends
                x < 100,
                y < 100,
        {
            x + y
        }
        pub(crate) open spec fn my_pub_spec_fun3(x: int, y: int) -> int {
            // function and body visible to crate
            x / 2 + y / 2
        }
        pub closed spec fn my_pub_spec_fun4(x: int, y: int) -> int {
            // function visible to all, body visible to module
            x / 2 + y / 2
        }
        pub(crate) closed spec fn my_pub_spec_fun5(x: int, y: int) -> int {
            // function visible to crate, body visible to module
            x / 2 + y / 2
        }
    }";

    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    // dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough3() {
    use ast::HasModuleItem;
    let source_code = "verus!{
        proof fn test5_bound_checking(x: u32, y: u32, z: u32)
            requires
                x <= 0xffff,
                y <= 0xffff,
                z <= 0xffff,
        {
            assert(x * z == mul(x, z)) by(nonlinear_arith)
                requires
                    x <= 0xffff,
                    z <= 0xffff,
            {
                assert(0 <= x * z);
                assert(x * z <= 0xffff * 0xffff);
            }
            assert(0 <= y < 100 ==> my_spec_fun(x, y) >= x);
            assert(forall|x: int, y: int| 0 <= x < 100 && 0 <= y < 100 ==> my_spec_fun(x, y) >= x);
        }
        fn test_quantifier() {
            assert(forall|x: int, y: int| 0 <= x < 100 && 0 <= y < 100 ==> my_spec_fun(x, y) >= x);
            assert(my_spec_fun(10, 20) == 30);
            assert(exists|x: int, y: int| my_spec_fun(x, y) == 30);
        }
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    for item in file.items() {
        dbg!(&item);
        // let v_item: vst_nodes::Item = item.try_into().unwrap();
        // dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough4() {
    use ast::HasModuleItem;
    let source_code = "verus!{
        fn test_assert_forall_by() {
            assert forall|x: int, y: int| f1(x) + f1(y) == x + y + 2 by {
                reveal(f1);
            }
            assert(f1(1) + f1(2) == 5);
            assert(f1(3) + f1(4) == 9);
            // to prove forall|...| P ==> Q, write assert forall|...| P implies Q by {...}
            assert forall|x: int| x < 10 implies f1(x) < 11 by {
                assert(x < 10);
                reveal(f1);
                assert(f1(x) < 11);
            }
            assert(f1(3) < 11);
        }
        fn test_choose() {
            assume(exists|x: int| f1(x) == 10);
            proof {
                let x_witness = choose|x: int| f1(x) == 10;
                assert(f1(x_witness) == 10);
            }

            assume(exists|x: int, y: int| f1(x) + f1(y) == 30);
            proof {
                let (x_witness, y_witness): (int, int) = choose|x: int, y: int| f1(x) + f1(y) == 30;
                assert(f1(x_witness) + f1(y_witness) == 30);
            }
        }
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    // dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough5() {
    use ast::HasModuleItem;
    let source_code =
    "verus!{
        fn test_single_trigger1() {
            assume(forall|x: int, y: int| f1(x) < 100 && f1(y) < 100 ==> #[trigger] my_spec_fun(x, y) >= x);
        }

        fn foo(x:int) -> int {
            if x>0 {1} else {-1}
        }
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        // let v_item: vst_nodes::Item = item.try_into().unwrap();
        // dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough6() {
    use ast::HasModuleItem;
    let source_code = "verus!{
        proof fn my_proof_fun(x: int, y: int) -> (sum: int)
            requires
                x < 100,
                y < 100,
            ensures
                sum < 200,
        {
            x + y
        }
        spec fn sum2(i: int, j: int) -> int
            recommends
                0 <= i < 10,
                0 <= j < 10,
        {
            i + j
        }
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough7() {
    use ast::HasModuleItem;
    let source_code =
    "verus!{
        fn test_single_trigger2() {
            // Use [f1(x), f1(y)] as the trigger
            assume(forall|x: int, y: int| #[trigger] f1(x) < 100 && #[trigger] f1(y) < 100 ==> my_spec_fun(x, y) >= x);
        }
        /// To manually specify multiple triggers, use #![trigger]:
        fn test_multiple_triggers() {
            // Use both [my_spec_fun(x, y)] and [f1(x), f1(y)] as triggers
            assume(forall|x: int, y: int|
                #![trigger my_spec_fun(x, y)]
                #![trigger f1(x), f1(y)]
                f1(x) < 100 && f1(y) < 100 ==> my_spec_fun(x, y) >= x
            );
        }
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        // let v_item: vst_nodes::Item = item.try_into().unwrap();
        // dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough8() {
    use ast::HasModuleItem;
    let source_code = "verus!{
    fn test_my_funs2(
        a: u32, // exec variable
        b: u32, // exec variable
    )
        requires
            a < 100,
            b < 100,
    {
        let s = a + b; // s is an exec variable
        proof {
            let u = a + b; // u is a ghost variable
            my_proof_fun(u / 2, b as int); // my_proof_fun(x, y) takes ghost parameters x and y
        }
    }
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough9_0() {
    use ast::HasModuleItem;
    let source_code = "verus!{
    fn test_is_variant_1(v: Vehicle2<u64>) {
        match v {
            Vehicle2::Car(_) => assert(v.is_Car()),
            Vehicle2::Train(_) => assert(v.is_Train()),
        };
    }
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough9() {
    use ast::HasModuleItem;
    let source_code = "verus!{
    proof fn test_tracked(
        tracked w: int,
        tracked x: int,
        tracked y: int,
        z: int,
      ) -> tracked TrackedAndGhost<(int, int), int> {

    }
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough10_0() {
    use ast::HasModuleItem;
    let source_code = "verus!{
    pub(crate) proof fn binary_ops<A>(a: A, x: int) {
        assert(2 + 2 !== 3);
        assert(a === a);

        assert(false <==> true && false);
    }

    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        // let v_item: vst_nodes::Item = item.try_into().unwrap();
        // dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough10_1() {
    use ast::HasModuleItem;
    let source_code = "verus!{
    spec fn ccc(x: int, y: int) -> bool {
        &&& if false {
                true
            } else {
                &&& b ==> b
                &&& !b
            }
        &&& true
    }
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    for item in file.items() {
        dbg!(&item);
        // let v_item: vst_nodes::Item = item.try_into().unwrap();
        // dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough10_2() {
    use ast::HasModuleItem;
    let source_code = "verus!{
    spec fn complex_conjuncts(x: int, y: int) -> bool {
        let b = x < y;
        &&& b
        &&& if false {
                &&& b ==> b
                &&& !b ==> !b
            } else {
                ||| b ==> b
                ||| !b
            }
        &&& false ==> true
    }

    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        // let v_item: vst_nodes::Item = item.try_into().unwrap();
        // dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough10() {
    use ast::HasModuleItem;
    let source_code = "verus!{
    fn test_views() {
        let mut v: Vec<u8> = Vec::new();
        v.push(10);
        v.push(20);
        proof {
            let s: Seq<u8> = v@; // v@ is equivalent to v.view()
            assert(s[0] == 10);
            assert(s[1] == 20);
        }
    }
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough11() {
    use ast::HasModuleItem;
    let source_code = "verus!{
fn binary_search(v: &Vec<u64>, k: u64) -> (r: usize)
    requires
        forall|i:int, j:int| 0 <= i <= j < v.len() ==> v[i] <= v[j],
        exists|i:int| 0 <= i < v.len() && k == v[i],
    ensures
        r < v.len(),
        k == v[r as int],
{
    let mut i1: usize = 0;
    let mut i2: usize = v.len() - 1;
    while i1 != i2
        invariant
            i2 < v.len(),
            exists|i:int| i1 <= i <= i2 && k == v[i],
            forall|i:int, j:int| 0 <= i <= j < v.len() ==> v[i] <= v[j],
    {
        //let d: Ghost<int> = ghost(i2 - i1);
        let ix = i1 + (i2 - i1) / 2;
        if *v.index(ix) < k {
            i1 = ix + 1;
        } else {
            i2 = ix;
        }
        assert(i2 - i1 < d@);
    }
    i1
}
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        // let v_item: vst_nodes::Item = item.try_into().unwrap();
        // dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough12() {
    use ast::HasModuleItem;
    let source_code = "verus!{
fn pop_test(t: Vec<u64>)
requires
    t.len() > 0,
    forall|i: int| #![auto] 0 <= i < t.len() ==> uninterp_fn(t[i]),
{
let mut t = t;
let x = t.pop();
assert(uninterp_fn(x));
assert(forall|i: int| #![auto] 0 <= i < t.len() ==> uninterp_fn(t[i]));
}
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        // let v_item: vst_nodes::Item = item.try_into().unwrap();
        // dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough13() {
    use ast::HasModuleItem;
    let source_code = "verus!{
    proof fn arith_sum_int_nonneg(i: nat)
        ensures
            arith_sum_int(i as int) >= 0,
        decreases
            i,
    {
        if i > 0 {
            arith_sum_int_nonneg((i - 1) as nat);
        }
    }

    spec fn arith_sum_int(i: int) -> int
    decreases i
{
    if i <= 0 { 0 } else { i + arith_sum_int(i - 1) }
}
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough14() {
    use ast::HasModuleItem;
    let source_code = "verus!{
fn exec_with_decreases(n: u64) -> u64
    decreases 100 - n,
{
    if n < 100 {
        exec_with_decreases(n + 1)
    } else {
        n
    }
}
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough15() {
    use ast::HasModuleItem;
    let source_code =
    "verus!{
spec(checked) fn my_spec_fun2(x: int, y: int) -> int
    recommends
        x < 100,
        y < 100,
{
    // Because of spec(checked), Verus checks that my_spec_fun's recommends clauses are satisfied here:
    my_spec_fun(x, y)
}
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough16() {
    use ast::HasModuleItem;
    let source_code = "verus!{
proof fn test_even_f()
    ensures
        forall|i: int| is_even(i) ==> f(i),
{
    assert forall|i: int| is_even(i) implies f(i) by {
        // First, i is in scope here
        // Second, we assume is_even(i) here
        lemma_even_f(i);
        // Finally, we have to prove f(i) here
    }
}
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        // let v_item: vst_nodes::Item = item.try_into().unwrap();
        // dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough17() {
    use ast::HasModuleItem;
    //from: https://github.com/verus-lang/verus/wiki/Doc%3A-Deprecated-and-recommended-syntax%2C-and-upcoming-changes
    let source_code = "verus!{
proof fn lemma1(i: int, tracked t: S) {
}
fn f(i: u32, Ghost(j): Ghost<int>, Tracked(t): Tracked<S>) -> (k: u32)
    // Note: Ghost(j) unwraps the Ghost<int> value so that j has type int
    // Note: Tracked(t) unwraps the Tracked<S> value so that t has type S
    requires
        i != j,
        i < 10,
    ensures
        k == i + 1,
{
    let ghost i_plus_j = i + j;
    let ghost t_ghost_copy = t;
    let tracked t_moved = t;
    proof {
        lemma1(i as int, t_moved);
    }
    assert(t_moved == t_ghost_copy);
    assert(i_plus_j == i + j);
    i + 1
}
fn g(Tracked(t): Tracked<S>) -> u32 {
    f(5, Ghost(6), Tracked(t))
}
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough18() {
    use ast::HasModuleItem;
    //from: https://github.com/verus-lang/verus/blob/main/source/rust_verify/example/syntax.rs
    let source_code = "
verus!{
spec fn add0(a: nat, b: nat) -> nat
    recommends a > 0,
    via add0_recommends
{
    a + b
}

spec fn dec0(a: int) -> int
    decreases a
    when a > 0
    via dec0_decreases
{
    if a > 0 {
        dec0(a - 1)
    } else {
        0
    }
}

#[via_fn]
proof fn add0_recommends(a: nat, b: nat) {
    // proof
}

#[via_fn]
proof fn dec0_decreases(a: int) {
    // proof
}
} // verus!";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough19() {
    use ast::HasModuleItem;
    //from: https://github.com/verus-lang/verus/blob/main/source/rust_verify/example/syntax.rs
    let source_code = "
verus!{
tracked struct TrackedAndGhost<T, G>(
    tracked T,
    ghost G,
);
} // verus!";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough20() {
    use ast::HasModuleItem;
    //from: https://github.com/verus-lang/verus/blob/main/source/rust_verify/example/syntax.rs
    let source_code = "
verus!{
proof fn lemma_mul_upper_bound(x: int, x_bound: int, y: int, y_bound: int)
    by (nonlinear_arith)
    requires x <= x_bound, y <= y_bound, 0 <= x, 0 <= y,
    ensures x * y <= x_bound * y_bound,
{
}
} // verus!";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough21() {
    use ast::HasModuleItem;
    //from: https://github.com/verus-lang/verus/blob/main/source/rust_verify/example/syntax.rs
    let source_code = "
verus!{
    spec fn add0(a: nat, b: nat) -> nat
    recommends a > 0,
    via add0_recommends
{
    a + b
}

#[via_fn]
proof fn add0_recommends(a: nat, b: nat) {
    // proof
}

} // verus!";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough22() {
    use ast::HasModuleItem;
    //from: https://github.com/verus-lang/verus/blob/main/source/rust_verify/example/syntax.rs
    let source_code = "
verus!{
    spec fn add0(a: nat, b: nat) -> nat
    recommends a > 0,
    via add0_recommends
{
    a + b
}

spec fn dec0(a: int) -> int
    decreases a,
    when a > 0
    via dec0_decreases
{
    if a > 0 {
        dec0(a - 1)
    } else {
        0
    }
}

#[via_fn]
proof fn add0_recommends(a: nat, b: nat) {
    // proof
}

#[via_fn]
proof fn dec0_decreases(a: int) {
    // proof
}
} // verus!";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

// verus trigger attribute is custom syntax
// Need to extend Rust attribute
// or make a syntax kind for it (e.g. TriggerAttribute)
// Reference https://github.com/verus-lang/verus/blob/4ef61030aadc4fd66b62f3614f36e3b64e89b855/source/builtin_macros/src/syntax.rs#L1808
// Note that verus! macro compares the attributes name with "trigger", and process it
// However, the rust-analyzer parser does not have access to string's content.
// Therefore, to make a new syntax kind (e.g. TriggerAttribute),
// we need to register `trigger` as a reserved keyword.
// however, by registering `trigger` as a reserved keyword,
// single trigger attribute, which is `#[trigger]` becomes invalid syntax.
// therefore, we just special-case `#[trigger]`
#[test]
fn verus_walkthrough23() {
    use ast::HasModuleItem;
    //from: https://github.com/verus-lang/verus/blob/main/source/rust_verify/example/syntax.rs
    let source_code = "
verus!{
    fn test_multiple_triggers() {
        assume(forall|x: int, y: int|
            #![trigger my_spec_fun(x, y)]
            #![trigger f1(x), f1(y)]
            f1(x) < 100 && f1(y) < 100 ==> my_spec_fun(x, y) >= x
        );
    }
} // verus!";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        // let v_item: vst_nodes::Item = item.try_into().unwrap();
        // dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough24() {
    use ast::HasModuleItem;
    //from: https://github.com/verus-lang/verus/blob/main/source/rust_verify/example/syntax.rs
    let source_code = "
verus!{
spec fn test_rec2(x: int, y: int) -> int
    decreases x, y
{
    if y > 0 {
        1 + test_rec2(x, y - 1)
    } else if x > 0 {
        2 + test_rec2(x - 1, 100)
    } else {
        3
    }
}
} // verus!";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(v_item);
    }
}

#[test]
fn verus_walkthrough25() {
    use ast::HasModuleItem;
    // https://github.com/verus-lang/verus/blob/ed95a417a236707fbb50efd96c91cb217ed2b22a/source/rust_verify/example/vectors.rs
    let source_code = "
verus!{
    fn pusher() -> Vec<u64> {
        let mut v = Vec::new();
        v.push(0);
        v.push(1);
        v.push(2);
        v.push(3);
        v.push(4);
        let ghost goal = Seq::new(5, |i: int| i as u64);
        assert(v@ =~= goal);
        assert(v[2] == 2);

        v.pop();
        v.push(4);
        assert(v@ =~= goal);

        v
    }
} // verus!";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        // let v_item: vst_nodes::Item = item.try_into().unwrap();
        // dbg!(&v_item);
        // println!("{}", &v_item);
    }
}

#[test]
fn verus_anonymous_return_types() {
    use ast::HasModuleItem;
    let source_code = "verus!{
fn foo() -> (u32, u32)
{
    (1, 2)
}
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        // let v_item: vst_nodes::Item = item.try_into().unwrap();
        // dbg!(v_item);
    }
}

#[test]
fn verus_struct_syntax() {
    use ast::HasModuleItem;
    let source_code = "verus!{
proof fn sufficiently_creamy() -> bool
    requires 
        bev is Coffee,
{
   bev->creamers
}

spec fn is_insect(l: Life) -> bool {
    l is Arthropod && l->Arthropod_legs == 6
}

spec fn rect_height(s: Shape) -> int
    recommends s is Rect
{
    s->1
}

spec fn cuddly(l: Life) -> bool
{
    ||| l matches Mammal{legs, ..} && legs == 4
    ||| l matches Arthropod{legs, wings} && legs == 8 && wings == 0
}

spec fn is_kangaroo(l: Life) -> bool
{
    &&& l matches Life::Mammal{legs, has_pocket}
    &&& legs == 2
    &&& has_pocket
}

spec fn walks_upright(l: Life) -> bool
{
    l matches Life::Mammal{legs, ..} ==> legs==2
}
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        // let v_item: vst_nodes::Item = item.try_into().unwrap();
        // dbg!(v_item);
    }
}

#[test]
fn verus_while_loops() {
    use ast::HasModuleItem;
    let source_code = "verus!{
pub fn clone_vec_u8() {
    let i = 0;
    while i < v.len()
        invariant_except_break
            i <= v.len(),
        invariant
            i <= v.len(),
            i == out.len(),
            forall |j| #![auto] 0 <= j < i  ==> out@[j] == v@[j],
        ensures
            i > 0,
        decreases
            72,
    {
        i = i + 1;
    }
}
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        // let v_item: vst_nodes::Item = item.try_into().unwrap();
        // dbg!(v_item);
    }
}

#[test]
fn verus_loops() {
    use ast::HasModuleItem;
    let source_code = "verus!{
fn test() {
    loop
        invariant
            x > 0,
    {
        x += 1;
    }
}

fn test() {
    loop
        invariant
            false,
        ensures
            next_idx + count <= 512,
    {
        x
    }
}
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        // let v_item: vst_nodes::Item = item.try_into().unwrap();
        // dbg!(v_item);
    }
}

// TODO: Restore once we have while loops in a better state
#[test]
fn verus_for_loops() {
    use ast::HasModuleItem;
    let source_code = "verus!{
fn reverse(v: &mut Vec<u64>)
    ensures
        v.len() == old(v).len(),
        forall|i: int| 0 <= i < old(v).len() ==> v[i] == old(v)[old(v).len() - i - 1],
{
    let length = v.len();
    let ghost v1 = v@;
    for n in 0..(length / 2)
        invariant
            length == v.len(),
            forall|i: int| 0 <= i < n ==> v[i] == v1[length - i - 1],
            forall|i: int| 0 <= i < n ==> v1[i] == v[length - i - 1],
            forall|i: int| n <= i && i + n < length ==> #[trigger] v[i] == v1[i],
    {
        let x = v[n];
        let y = v[length - 1 - n];
        v.set(n, y);
        v.set(length - 1 - n, x);
    }
}

fn test() {
    for x in iter: 0..end
        invariant
            end == 10,
    {
        n += 3;
    }
    let x = 2;
    for x in iter: vec_iter_copy(v)
        invariant
            b <==> (forall|i: int| 0 <= i < iter.cur ==> v[i] > 0),
    {
        b = b && x > 0;
    }
    let y = 3;
    for x in iter: 0..({
        let z = end;
        non_spec();
        z
    })
        invariant
            n == iter.cur * 3,
            end == 10,
    {
        n += 3;
        end = end + 0;  // causes end to be non-constant
    }
}
    }";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        // let v_item: vst_nodes::Item = item.try_into().unwrap();
        // dbg!(v_item);
    }
}

#[test]
fn verus_broadcast() {
    use ast::HasModuleItem;
    let source_code = "verus!{
mod ring {
    use builtin::*;

    pub struct Ring {
        pub i: u64,
    }

    impl Ring {
        pub closed spec fn inv(&self) -> bool {
            self.i < 10
        }

        pub closed spec fn spec_succ(&self) -> Ring {
            Ring { i: if self.i == 9 { 0 } else { (self.i + 1) as u64 } }
        }

        pub closed spec fn spec_prev(&self) -> Ring {
            Ring { i: if self.i == 0 { 9 } else { (self.i - 1) as u64 } }
        }

        pub broadcast proof fn spec_succ_ensures(p: Ring)
            requires p.inv()
            ensures p.inv() && (#[trigger] p.spec_succ()).spec_prev() == p
        { }

        pub broadcast proof fn spec_prev_ensures(p: Ring)
            requires p.inv()
            ensures p.inv() && (#[trigger] p.spec_prev()).spec_succ() == p
        { }

        pub    broadcast    group    properties {
        Ring::spec_succ_ensures,
                Ring::spec_prev_ensures,
        }
    }

    #[verifier::prune_unless_this_module_is_used]
    pub    broadcast    group    properties {
    Ring::spec_succ_ensures,
            Ring::spec_prev_ensures,
    }
}

mod m2 {
    use builtin::*;
    use crate::ring::*;

    fn t2(p: Ring) requires p.inv() {
           broadcast    use     Ring::properties;
        assert(p.spec_succ().spec_prev() == p);
        assert(p.spec_prev().spec_succ() == p);
    }
}

mod m3 {
    use builtin::*;
    use crate::ring::*;

        broadcast   use    Ring::properties;
        
        fn a() { }
}

mod m4 {
    use builtin::*;
    use crate::ring::*;

        broadcast   use    
                    Ring::spec_succ_ensures,
            Ring::spec_prev_ensures;
}
}";

    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        // let v_item: vst_nodes::Item = item.try_into().unwrap();
        // dbg!(v_item);
    }
}

#[test]
fn verus_broadcast_regression() {
    use ast::HasModuleItem;
    let source_code = "verus!{
fn f() { let group = Group::new(Delimiter::Bracket, bracketed.build()); let mut group = crate::Group::_new_fallback(group); group.set_span(span); trees.push_token_from_parser(TokenTree::Group(group)); }
}";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
    }
}


#[test]
fn cst_to_vst1() {
    use ast::HasModuleItem;
    let source_code = "
verus!{
spec fn sum(x: int, y: int) -> int
{
    x + y
}
} // verus!";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(&v_item);
        println!("{}", &v_item);
    }
}

#[test]
fn cst_to_vst2() {
    use ast::HasModuleItem;
    let source_code = "
verus!{
spec fn test_rec2(x: int, y: int) -> int
    decreases x, y
{
    if y > 0 {
        1 + test_rec2(x, y - 1)
    } else {
        3
    }
}
} // verus!";
    let parse = SourceFile::parse(source_code, Edition::Edition2024);
    dbg!(&parse.errors);
    assert!(parse.errors().is_empty());
    let file: SourceFile = parse.tree();
    dbg!(&file);
    for item in file.items() {
        dbg!(&item);
        let v_item: vst_nodes::Item = item.try_into().unwrap();
        dbg!(&v_item);
        println!("{}", &v_item);
    }
}

/*
TODO: cst_to_vst, "op_details"
failures:
    verus_walkthrough10_0
    verus_walkthrough10_1
    verus_walkthrough10_2
    verus_walkthrough11
    verus_walkthrough12
    verus_walkthrough16
    verus_walkthrough23
    verus_walkthrough25
    verus_walkthrough3
    verus_walkthrough5
    verus_walkthrough7
 */
