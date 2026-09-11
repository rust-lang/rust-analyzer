use std::iter::successors;

use hir::Semantics;
use ide_db::RootDatabase;
use syntax::{
    Direction, NodeOrToken, SyntaxElement,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, T, TextRange, TextSize, TokenAtOffset, algo,
    ast::{self, AstNode, AstToken},
};

use crate::FileRange;

// Feature: Expand and Shrink Selection
//
// Extends or shrinks the current selection to the encompassing syntactic construct
// (expression, statement, item, module, etc). It works with multiple cursors.
//
// | Editor  | Shortcut |
// |---------|----------|
// | VS Code | <kbd>Alt+Shift+→</kbd>, <kbd>Alt+Shift+←</kbd> |
//
// ![Expand and Shrink Selection](https://user-images.githubusercontent.com/48062697/113020651-b42fc800-917a-11eb-8a4f-cf1a07859fac.gif)
pub(crate) fn extend_selection(db: &RootDatabase, frange: FileRange) -> TextRange {
    let sema = Semantics::new(db);
    let src = sema.parse_guess_edition(frange.file_id);
    try_extend_selection(&sema, src.syntax(), frange).unwrap_or(frange.range)
}

fn try_extend_selection(
    sema: &Semantics<'_, RootDatabase>,
    root: &SyntaxNode,
    frange: FileRange,
) -> Option<TextRange> {
    let range = frange.range;

    let string_kinds =
        [COMMENT, INNER_DOC_COMMENT, OUTER_DOC_COMMENT, STRING, BYTE_STRING, C_STRING];
    let list_kinds = [
        RECORD_PAT_FIELD_LIST,
        MATCH_ARM_LIST,
        RECORD_FIELD_LIST,
        TUPLE_FIELD_LIST,
        RECORD_EXPR_FIELD_LIST,
        VARIANT_LIST,
        USE_TREE_LIST,
        GENERIC_PARAM_LIST,
        GENERIC_ARG_LIST,
        TYPE_BOUND_LIST,
        PARAM_LIST,
        ARG_LIST,
        ARRAY_EXPR,
        TUPLE_EXPR,
        TUPLE_TYPE,
        TUPLE_PAT,
        WHERE_CLAUSE,
    ];

    if range.is_empty() {
        let offset = range.start();
        let mut leaves = root.token_at_offset(offset);
        if leaves.clone().all(|it| matches!(it.kind(), WHITESPACE | NEWLINE)) {
            let run = syntax::algo::blank_run(&leaves.next()?);
            return Some(extend_ws(root, run.after, run.range, offset));
        }
        let leaf_range = match leaves {
            TokenAtOffset::None => return None,
            TokenAtOffset::Single(l) => {
                if string_kinds.contains(&l.kind()) {
                    extend_single_word_in_comment_or_string(&l, offset)
                        .unwrap_or_else(|| l.text_range())
                } else {
                    l.text_range()
                }
            }
            TokenAtOffset::Between(l, r) => pick_best(l, r).text_range(),
        };
        return Some(leaf_range);
    };
    let node = match root.covering_element(range) {
        NodeOrToken::Token(token) => {
            if token.text_range() != range {
                return Some(token.text_range());
            }
            if ast::AnyComment::can_cast(token.kind())
                && let Some(range) = extend_comments(&token)
            {
                return Some(range);
            }
            token.parent()?
        }
        NodeOrToken::Node(node) => node,
    };

    // if we are in single token_tree, we maybe live in macro or attr
    if node.kind() == TOKEN_TREE
        && let Some(macro_call) = node.ancestors().find_map(ast::MacroCall::cast)
        && let Some(range) = extend_tokens_from_range(sema, macro_call, range)
    {
        return Some(range);
    }

    if node.text_range_without_outer_trivia() != range {
        return Some(node.text_range_without_outer_trivia());
    }

    let node = shallowest_node(&node);

    if node.parent().is_some_and(|n| list_kinds.contains(&n.kind()))
        && let Some(range) = extend_list_item(&node)
    {
        return Some(range);
    }

    node.parent().map(|it| it.text_range_without_outer_trivia())
}

fn extend_tokens_from_range(
    sema: &Semantics<'_, RootDatabase>,
    macro_call: ast::MacroCall,
    original_range: TextRange,
) -> Option<TextRange> {
    let src = macro_call.syntax().covering_element(original_range);
    let (first_token, last_token) = match src {
        NodeOrToken::Node(it) => (it.first_non_trivia_token()?, it.last_non_trivia_token()?),
        NodeOrToken::Token(it) => (it.clone(), it),
    };

    let mut first_token =
        if !first_token.is_trivia() { first_token } else { first_token.next_non_trivia_token()? };
    let mut last_token =
        if !last_token.is_trivia() { last_token } else { last_token.prev_non_trivia_token()? };

    while !original_range.contains_range(first_token.text_range()) {
        first_token = first_token.next_non_trivia_token()?;
    }
    while !original_range.contains_range(last_token.text_range()) {
        last_token = last_token.prev_non_trivia_token()?;
    }

    // compute original mapped token range
    let extended = {
        let fst_expanded = sema.descend_into_macros_single_exact(first_token.clone());
        let lst_expanded = sema.descend_into_macros_single_exact(last_token.clone());
        let mut lca =
            algo::least_common_ancestor(&fst_expanded.parent()?, &lst_expanded.parent()?)?;
        lca = shallowest_node(&lca);
        if lca.first_non_trivia_token() == Some(fst_expanded)
            && lca.last_non_trivia_token() == Some(lst_expanded)
        {
            lca = lca.parent()?;
        }
        lca
    };

    // Compute parent node range
    let validate = || {
        let extended = &extended;
        move |token: &SyntaxToken| -> bool {
            let expanded = sema.descend_into_macros_single_exact(token.clone());
            let parent = match expanded.parent() {
                Some(it) => it,
                None => return false,
            };
            algo::least_common_ancestor(extended, &parent).as_ref() == Some(extended)
        }
    };

    // Find the first and last text range under expanded parent
    let first = successors(Some(first_token), |token| token.prev_non_trivia_token())
        .take_while(validate())
        .last()?;

    let last = successors(Some(last_token), |token| token.next_non_trivia_token())
        .take_while(validate())
        .last()?;

    let range = first.text_range().cover(last.text_range());
    if range.contains_range(original_range) && original_range != range { Some(range) } else { None }
}

/// Find the shallowest node with same range, which allows us to traverse siblings.
fn shallowest_node(node: &SyntaxNode) -> SyntaxNode {
    node.ancestors()
        .take_while(|n| {
            n.text_range_without_outer_trivia() == node.text_range_without_outer_trivia()
        })
        .last()
        .unwrap()
}

fn extend_single_word_in_comment_or_string(
    leaf: &SyntaxToken,
    offset: TextSize,
) -> Option<TextRange> {
    let text: &str = leaf.text();
    let cursor_position: u32 = (offset - leaf.text_range().start()).into();

    let (before, after) = text.split_at(cursor_position as usize);

    fn non_word_char(c: char) -> bool {
        !(c.is_alphanumeric() || c == '_')
    }

    let start_idx = before.rfind(non_word_char)? as u32;
    let end_idx = after.find(non_word_char).unwrap_or(after.len()) as u32;

    // FIXME: use `ceil_char_boundary` from `std::str` when it gets stable
    // https://github.com/rust-lang/rust/issues/93743
    fn ceil_char_boundary(text: &str, index: u32) -> u32 {
        (index..).find(|&index| text.is_char_boundary(index as usize)).unwrap_or(text.len() as u32)
    }

    let from: TextSize = ceil_char_boundary(text, start_idx + 1).into();
    let to: TextSize = (cursor_position + end_idx).into();

    let range = TextRange::new(from, to);
    if range.is_empty() { None } else { Some(range + leaf.text_range().start()) }
}

fn extend_ws(
    root: &SyntaxNode,
    after: Option<SyntaxToken>,
    ws: TextRange,
    offset: TextSize,
) -> TextRange {
    let ws_text = root.text().slice(ws - root.text_range().start()).to_string();
    let ws_text = ws_text.as_str();
    let suffix = TextRange::new(offset, ws.end()) - ws.start();
    let prefix = TextRange::new(ws.start(), offset) - ws.start();
    let ws_suffix = &ws_text[suffix];
    let ws_prefix = &ws_text[prefix];
    if ws_text.contains('\n')
        && !ws_suffix.contains('\n')
        && let Some(after) = after
    {
        let node: SyntaxElement = match after.index() {
            None => after.into(),
            Some(_) => {
                let owner = after.prev_token().and_then(|trivia_token| trivia_token.owning_node());
                after
                    .parent_ancestors()
                    .take_while(|it| Some(it) != owner.as_ref())
                    .last()
                    .map_or(after.into(), Into::into)
            }
        };
        let start = match ws_prefix.rfind('\n') {
            Some(idx) => ws.start() + TextSize::from((idx + 1) as u32),
            None => node.text_range_without_outer_trivia().start(),
        };
        let end = if root.text().char_at(node.text_range_without_outer_trivia().end()) == Some('\n')
        {
            node.text_range_without_outer_trivia().end() + TextSize::of('\n')
        } else {
            node.text_range_without_outer_trivia().end()
        };
        return TextRange::new(start, end);
    }
    ws
}

fn pick_best(l: SyntaxToken, r: SyntaxToken) -> SyntaxToken {
    return if priority(&r) > priority(&l) { r } else { l };
    fn priority(n: &SyntaxToken) -> usize {
        match n.kind() {
            WHITESPACE | NEWLINE => 0,
            IDENT | T![self] | T![super] | T![crate] | T![Self] | LIFETIME_IDENT => 2,
            _ => 1,
        }
    }
}

/// Extend list item selection to include nearby delimiter and whitespace.
fn extend_list_item(node: &SyntaxNode) -> Option<TextRange> {
    fn nearby_delimiter(
        delimiter_kind: SyntaxKind,
        node: &SyntaxNode,
        dir: Direction,
    ) -> Option<SyntaxToken> {
        node.siblings_with_tokens(dir)
            .nth(1)
            .and_then(|it| it.into_token())
            .filter(|node| node.kind() == delimiter_kind)
    }

    let delimiter = match node.kind() {
        TYPE_BOUND => T![+],
        _ => T![,],
    };

    if let Some(delimiter_node) = nearby_delimiter(delimiter, node, Direction::Next) {
        // Include any following whitespace when delimiter is after list item.
        let mut end = delimiter_node.text_range().end();
        for trivia_token in delimiter_node.trailing_trivia() {
            if trivia_token.kind() != WHITESPACE {
                break;
            }
            end = trivia_token.text_range().end();
        }

        return Some(TextRange::new(node.text_range_without_outer_trivia().start(), end));
    }
    if let Some(delimiter_node) = nearby_delimiter(delimiter, node, Direction::Prev) {
        return Some(TextRange::new(
            delimiter_node.text_range().start(),
            node.text_range_without_outer_trivia().end(),
        ));
    }

    None
}

fn extend_comments(comment: &SyntaxToken) -> Option<TextRange> {
    let prev =
        successors(Some(comment.clone()), |it| syntax::algo::adjacent_comment(it, Direction::Prev))
            .last()?;
    let next =
        successors(Some(comment.clone()), |it| syntax::algo::adjacent_comment(it, Direction::Next))
            .last()?;
    let range = TextRange::new(prev.text_range().start(), next.text_range().end());
    (range != comment.text_range()).then_some(range)
}

#[cfg(test)]
mod tests {
    use crate::fixture;

    use super::*;

    fn do_check(before: &str, afters: &[&str]) {
        let (analysis, position) = fixture::position(before);
        let before = analysis.file_text(position.file_id).unwrap();
        let range = TextRange::empty(position.offset);
        let mut frange = FileRange { file_id: position.file_id, range };

        for &after in afters {
            frange.range = analysis.extend_selection(frange).unwrap();
            let actual = &before[frange.range];
            assert_eq!(after, actual);
        }
    }

    #[test]
    fn test_extend_selection_arith() {
        do_check(r#"fn foo() { $01 + 1 }"#, &["1", "1 + 1", "{ 1 + 1 }"]);
    }

    #[test]
    fn test_extend_selection_list() {
        do_check(r#"fn foo($0x: i32) {}"#, &["x", "x: i32"]);
        do_check(r#"fn foo($0x: i32, y: i32) {}"#, &["x", "x: i32", "x: i32, "]);
        do_check(r#"fn foo($0x: i32,y: i32) {}"#, &["x", "x: i32", "x: i32,", "(x: i32,y: i32)"]);
        do_check(r#"fn foo(x: i32, $0y: i32) {}"#, &["y", "y: i32", ", y: i32"]);
        do_check(r#"fn foo(x: i32, $0y: i32, ) {}"#, &["y", "y: i32", "y: i32, "]);
        do_check(r#"fn foo(x: i32,$0y: i32) {}"#, &["y", "y: i32", ",y: i32"]);

        do_check(r#"const FOO: [usize; 2] = [ 22$0 , 33];"#, &["22", "22 , "]);
        do_check(r#"const FOO: [usize; 2] = [ 22 , 33$0];"#, &["33", ", 33"]);
        do_check(r#"const FOO: [usize; 2] = [ 22 , 33$0 ,];"#, &["33", "33 ,", "[ 22 , 33 ,]"]);

        do_check(r#"fn main() { (1, 2$0) }"#, &["2", ", 2", "(1, 2)"]);

        do_check(
            r#"
const FOO: [usize; 2] = [
    22,
    $033,
]"#,
            &["33", "33,"],
        );

        do_check(
            r#"
const FOO: [usize; 2] = [
    22
    , 33$0,
]"#,
            &["33", "33,"],
        );
    }

    #[test]
    fn test_extend_selection_start_of_the_line() {
        do_check(
            r#"
impl S {
$0    fn foo() {

    }
}"#,
            &["    fn foo() {\n\n    }\n"],
        );
    }

    #[test]
    fn test_extend_selection_doc_comments() {
        do_check(
            r#"
struct A;

/// bla
/// bla
struct B {
    $0
}
            "#,
            &["\n    \n", "{\n    \n}", "/// bla\n/// bla\nstruct B {\n    \n}"],
        )
    }

    #[test]
    fn test_extend_selection_comments() {
        do_check(
            r#"
fn bar(){}

// fn foo() {
// 1 + $01
// }

// fn foo(){}
    "#,
            &["1", "// 1 + 1", "// fn foo() {\n// 1 + 1\n// }"],
        );

        do_check(
            r#"
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum Direction {
//  $0   Next,
//     Prev
// }
"#,
            &[
                "//     Next,",
                "// #[derive(Debug, Clone, Copy, PartialEq, Eq)]\n// pub enum Direction {\n//     Next,\n//     Prev\n// }",
            ],
        );

        do_check(
            r#"
/*
foo
_bar1$0*/
"#,
            &["_bar1", "/*\nfoo\n_bar1*/"],
        );

        do_check(r#"//!$0foo_2 bar"#, &["foo_2", "//!foo_2 bar"]);

        do_check(r#"/$0/foo bar"#, &["//foo bar"]);
    }

    #[test]
    fn test_extend_selection_prefer_idents() {
        do_check(
            r#"
fn main() { foo$0+bar;}
"#,
            &["foo", "foo+bar"],
        );
        do_check(
            r#"
fn main() { foo+$0bar;}
"#,
            &["bar", "foo+bar"],
        );
    }

    #[test]
    fn test_extend_selection_prefer_lifetimes() {
        do_check(r#"fn foo<$0'a>() {}"#, &["'a", "<'a>"]);
        do_check(r#"fn foo<'a$0>() {}"#, &["'a", "<'a>"]);
    }

    #[test]
    fn test_extend_selection_select_first_word() {
        do_check(r#"// foo bar b$0az quxx"#, &["baz", "// foo bar baz quxx"]);
        do_check(
            r#"
impl S {
fn foo() {
// hel$0lo world
}
}
"#,
            &["hello", "// hello world"],
        );
    }

    #[test]
    fn test_extend_selection_string() {
        do_check(
            r#"
fn bar(){}

" fn f$0oo() {"
"#,
            &["foo", "\" fn foo() {\""],
        );
    }

    #[test]
    fn test_extend_trait_bounds_list_in_where_clause() {
        do_check(
            r#"
fn foo<R>()
    where
        R: req::Request + 'static,
        R::Params: DeserializeOwned$0 + panic::UnwindSafe + 'static,
        R::Result: Serialize + 'static,
"#,
            &[
                "DeserializeOwned",
                "DeserializeOwned + ",
                "DeserializeOwned + panic::UnwindSafe + 'static",
                "R::Params: DeserializeOwned + panic::UnwindSafe + 'static",
                "R::Params: DeserializeOwned + panic::UnwindSafe + 'static,",
            ],
        );
        do_check(r#"fn foo<T>() where T: $0Copy"#, &["Copy"]);
        do_check(r#"fn foo<T>() where T: $0Copy + Display"#, &["Copy", "Copy + "]);
        do_check(r#"fn foo<T>() where T: $0Copy +Display"#, &["Copy", "Copy +"]);
        do_check(r#"fn foo<T>() where T: $0Copy+Display"#, &["Copy", "Copy+"]);
        do_check(r#"fn foo<T>() where T: Copy + $0Display"#, &["Display", "+ Display"]);
        do_check(r#"fn foo<T>() where T: Copy + $0Display + Sync"#, &["Display", "Display + "]);
        do_check(r#"fn foo<T>() where T: Copy +$0Display"#, &["Display", "+Display"]);
    }

    #[test]
    fn test_extend_trait_bounds_list_inline() {
        do_check(r#"fn foo<T: $0Copy>() {}"#, &["Copy"]);
        do_check(r#"fn foo<T: $0Copy + Display>() {}"#, &["Copy", "Copy + "]);
        do_check(r#"fn foo<T: $0Copy +Display>() {}"#, &["Copy", "Copy +"]);
        do_check(r#"fn foo<T: $0Copy+Display>() {}"#, &["Copy", "Copy+"]);
        do_check(r#"fn foo<T: Copy + $0Display>() {}"#, &["Display", "+ Display"]);
        do_check(r#"fn foo<T: Copy + $0Display + Sync>() {}"#, &["Display", "Display + "]);
        do_check(r#"fn foo<T: Copy +$0Display>() {}"#, &["Display", "+Display"]);
        do_check(
            r#"fn foo<T: Copy$0 + Display, U: Copy>() {}"#,
            &[
                "Copy",
                "Copy + ",
                "Copy + Display",
                "T: Copy + Display",
                "T: Copy + Display, ",
                "<T: Copy + Display, U: Copy>",
            ],
        );
    }

    #[test]
    fn test_extend_selection_on_tuple_in_type() {
        do_check(
            r#"fn main() { let _: (krate, $0_crate_def_map, module_id) = (); }"#,
            &["_crate_def_map", "_crate_def_map, ", "(krate, _crate_def_map, module_id)"],
        );
        // white space variations
        do_check(
            r#"fn main() { let _: (krate,$0_crate_def_map,module_id) = (); }"#,
            &["_crate_def_map", "_crate_def_map,", "(krate,_crate_def_map,module_id)"],
        );
        do_check(
            r#"
fn main() { let _: (
    krate,
    _crate$0_def_map,
    module_id
) = (); }"#,
            &[
                "_crate_def_map",
                "_crate_def_map,",
                "(\n    krate,\n    _crate_def_map,\n    module_id\n)",
            ],
        );
    }

    #[test]
    fn test_extend_selection_on_tuple_in_rvalue() {
        do_check(
            r#"fn main() { let var = (krate, _crate_def_map$0, module_id); }"#,
            &["_crate_def_map", "_crate_def_map, ", "(krate, _crate_def_map, module_id)"],
        );
        // white space variations
        do_check(
            r#"fn main() { let var = (krate,_crate$0_def_map,module_id); }"#,
            &["_crate_def_map", "_crate_def_map,", "(krate,_crate_def_map,module_id)"],
        );
        do_check(
            r#"
fn main() { let var = (
    krate,
    _crate_def_map$0,
    module_id
); }"#,
            &[
                "_crate_def_map",
                "_crate_def_map,",
                "(\n    krate,\n    _crate_def_map,\n    module_id\n)",
            ],
        );
    }

    #[test]
    fn test_extend_selection_on_tuple_pat() {
        do_check(
            r#"fn main() { let (krate, _crate_def_map$0, module_id) = var; }"#,
            &["_crate_def_map", "_crate_def_map, ", "(krate, _crate_def_map, module_id)"],
        );
        // white space variations
        do_check(
            r#"fn main() { let (krate,_crate$0_def_map,module_id) = var; }"#,
            &["_crate_def_map", "_crate_def_map,", "(krate,_crate_def_map,module_id)"],
        );
        do_check(
            r#"
fn main() { let (
    krate,
    _crate_def_map$0,
    module_id
) = var; }"#,
            &[
                "_crate_def_map",
                "_crate_def_map,",
                "(\n    krate,\n    _crate_def_map,\n    module_id\n)",
            ],
        );
    }

    #[test]
    fn extend_selection_inside_macros() {
        do_check(
            r#"macro_rules! foo { ($item:item) => {$item} }
                foo!{fn hello(na$0me:usize){}}"#,
            &[
                "name",
                "name:usize",
                "(name:usize)",
                "fn hello(name:usize){}",
                "{fn hello(name:usize){}}",
                "foo!{fn hello(name:usize){}}",
            ],
        );
    }

    #[test]
    fn extend_selection_inside_recur_macros() {
        do_check(
            r#" macro_rules! foo2 { ($item:item) => {$item} }
                macro_rules! foo { ($item:item) => {foo2!($item);} }
                foo!{fn hello(na$0me:usize){}}"#,
            &[
                "name",
                "name:usize",
                "(name:usize)",
                "fn hello(name:usize){}",
                "{fn hello(name:usize){}}",
                "foo!{fn hello(name:usize){}}",
            ],
        );
    }

    #[test]
    fn extend_selection_inside_str_with_wide_char() {
        // should not panic
        do_check(
            r#"fn main() { let x = "═$0═══════"; }"#,
            &[
                r#""════════""#,
                r#"let x = "════════";"#,
                r#"{ let x = "════════"; }"#,
                r#"fn main() { let x = "════════"; }"#,
            ],
        );
    }
}
