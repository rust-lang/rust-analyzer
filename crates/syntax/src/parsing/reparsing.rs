//! Implementation of incremental re-parsing.
//!
//! We use two simple strategies for this:
//!   - if the edit modifies only a single token (like changing an identifier's
//!     letter), we replace only this token.
//!   - otherwise, we search for the nearest `{}` block which contains the edit
//!     and try to parse only this block.

use std::ops::Range;

use parser::{Edition, Reparser, SyntaxKind};
use rowan::Language;

use crate::{
    SyntaxError,
    SyntaxKind::*,
    T, TextRange, TextSize,
    parsing::build_tree,
    syntax_node::{
        GreenNode, GreenToken, NodeOrToken, RustLanguage, SyntaxElement, SyntaxNode, SyntaxToken,
        trivia_pieces,
    },
};

pub(crate) fn incremental_reparse(
    node: &SyntaxNode,
    delete: TextRange,
    insert: &str,
    errors: impl IntoIterator<Item = SyntaxError>,
    edition: Edition,
) -> Option<(GreenNode, Vec<SyntaxError>, TextRange)> {
    if let Some((green, new_errors, old_range)) = reparse_token(node, delete, insert, edition) {
        return Some((
            green,
            merge_errors(errors, new_errors, old_range, delete, insert),
            old_range,
        ));
    }

    if let Some((green, new_errors, old_range)) = reparse_block(node, delete, insert, edition) {
        return Some((
            green,
            merge_errors(errors, new_errors, old_range, delete, insert),
            old_range,
        ));
    }
    None
}

fn reparse_token(
    root: &SyntaxNode,
    delete: TextRange,
    insert: &str,
    edition: Edition,
) -> Option<(GreenNode, Vec<SyntaxError>, TextRange)> {
    let prev_token = root.covering_element(delete).as_token()?.clone();

    if !prev_token.text_range().contains_range(delete) {
        return reparse_trivia(&prev_token, delete, insert, edition);
    }

    let prev_token_kind = prev_token.kind();
    match prev_token_kind {
        IDENT | STRING | BYTE_STRING | C_STRING => {
            let mut new_text = get_text_after_edit(prev_token.clone().into(), delete, insert);
            let (new_token_kind, new_err) = parser::LexedStr::single_token(edition, &new_text)?;

            if new_token_kind != prev_token_kind
                || (new_token_kind == IDENT && is_contextual_kw(&new_text))
            {
                return None;
            }

            // Check that edited token is not a part of the bigger token.
            // E.g. if for source code `bruh"str"` the user removed `ruh`, then
            // `b` no longer remains an identifier, but becomes a part of byte string literal
            if let Some(next_char) = root.text().char_at(prev_token.text_range().end()) {
                new_text.push(next_char);
                let token_with_next_char = parser::LexedStr::single_token(edition, &new_text);
                if let Some((_kind, _error)) = token_with_next_char {
                    return None;
                }
                new_text.pop();
            }

            let new_token = GreenToken::with_trivia(
                rowan::SyntaxKind(prev_token_kind.into()),
                &new_text,
                prev_token.green().leading_trivia().to_vec(),
                prev_token.green().trailing_trivia().to_vec(),
            );
            let range = TextRange::up_to(TextSize::of(&new_text));
            Some((
                prev_token.replace_with(new_token),
                new_err.into_iter().map(|msg| SyntaxError::new(msg, range)).collect(),
                prev_token.text_range(),
            ))
        }
        _ => None,
    }
}

fn reparse_trivia(
    token: &SyntaxToken,
    delete: TextRange,
    insert: &str,
    edition: Edition,
) -> Option<(GreenNode, Vec<SyntaxError>, TextRange)> {
    let green = token.green();
    let full_start = token.text_range_including_trivia().start();

    let mut leading = green.leading_trivia().to_vec();
    let mut trailing = green.trailing_trivia().to_vec();

    let mut offset = full_start;
    let mut target = None;
    for (is_leading, index) in
        (0..leading.len()).map(|it| (true, it)).chain((0..trailing.len()).map(|it| (false, it)))
    {
        if !is_leading && index == 0 {
            offset =
                leading.iter().fold(full_start, |acc, it| acc + it.text_len()) + green.text_len();
        }
        let piece = if is_leading { &leading[index] } else { &trailing[index] };
        let range = TextRange::at(offset, piece.text_len());
        if range.contains_range(delete) {
            target = Some((is_leading, index, range));
            break;
        }
        offset = range.end();
    }
    let (is_leading, index, range) = target?;

    let piece = if is_leading { &leading[index] } else { &trailing[index] };
    let old_text = piece.text();

    let deleted = delete - range.start();
    if old_text[Range::<usize>::from(deleted)].contains('\n') {
        return None;
    }

    let mut new_text = old_text.to_owned();
    new_text.replace_range(Range::<usize>::from(deleted), insert);

    let (new_kind, new_err) = parser::LexedStr::single_token(edition, &new_text)?;
    if !new_kind.is_trivia() {
        return None;
    }
    let piece_kind = RustLanguage::kind_from_raw(piece.kind());
    let is_whitespace = matches!(piece_kind, SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE);
    if is_whitespace != (new_kind == SyntaxKind::WHITESPACE) {
        return None;
    }
    if !is_whitespace && piece_kind != new_kind {
        return None;
    }
    if !is_leading && new_text.contains('\n') {
        return None;
    }

    let new_pieces: Vec<GreenToken> = match is_whitespace {
        true => trivia_pieces(&new_text)
            .into_iter()
            .map(|(kind, text)| GreenToken::new(kind, text))
            .collect(),
        false => vec![GreenToken::new(piece.kind(), &new_text)],
    };
    if is_leading {
        leading.splice(index..index + 1, new_pieces);
    } else {
        trailing.splice(index..index + 1, new_pieces);
    }

    let new_token = GreenToken::with_trivia(green.kind(), green.text(), leading, trailing);
    let err_range = TextRange::up_to(TextSize::of(&new_text));
    Some((
        token.replace_with(new_token),
        new_err.into_iter().map(|msg| SyntaxError::new(msg, err_range)).collect(),
        range,
    ))
}

fn reparse_block(
    root: &SyntaxNode,
    delete: TextRange,
    insert: &str,
    edition: parser::Edition,
) -> Option<(GreenNode, Vec<SyntaxError>, TextRange)> {
    let (node, reparser) = find_reparsable_node(root, delete)?;
    let text = get_text_after_edit(node.clone().into(), delete, insert);

    let lexed = parser::LexedStr::new(edition, text.as_str());
    let parser_input = lexed.to_input(edition);
    if !is_balanced(&lexed) {
        return None;
    }

    let tree_traversal = reparser.parse(&parser_input);

    let (green, new_parser_errors, _eof) = build_tree(lexed, tree_traversal);
    let green = strip_eof(green);

    Some((node.replace_with(green), new_parser_errors, node.text_range()))
}

fn strip_eof(green: GreenNode) -> GreenNode {
    let is_empty_eof = |child: &rowan::NodeOrToken<_, &rowan::GreenTokenData>| match child {
        NodeOrToken::Token(token) => {
            RustLanguage::kind_from_raw(token.kind()) == EOF
                && token.text().is_empty()
                && token.leading_trivia().is_empty()
                && token.trailing_trivia().is_empty()
        }
        NodeOrToken::Node(_) => false,
    };
    if !green.children().next_back().is_some_and(|it| is_empty_eof(&it)) {
        return green;
    }
    let children = green.children().collect::<Vec<_>>();
    GreenNode::new(
        green.kind(),
        children[..children.len() - 1].iter().map(|child| match child {
            NodeOrToken::Node(node) => NodeOrToken::Node((*node).to_owned()),
            NodeOrToken::Token(token) => NodeOrToken::Token((*token).to_owned()),
        }),
    )
}

fn get_text_after_edit(element: SyntaxElement, mut delete: TextRange, insert: &str) -> String {
    let (start, mut text) = match &element {
        NodeOrToken::Token(token) => (token.text_range().start(), token.text().to_owned()),
        NodeOrToken::Node(node) => (node.text_range().start(), node.text().to_string()),
    };
    delete -= start;
    text.replace_range(Range::<usize>::from(delete), insert);
    text
}

fn is_contextual_kw(text: &str) -> bool {
    matches!(text, "auto" | "default" | "union")
}

fn find_reparsable_node(node: &SyntaxNode, range: TextRange) -> Option<(SyntaxNode, Reparser)> {
    let node = node.covering_element(range);

    node.ancestors().find_map(|node| {
        let first_child = node.first_child_or_token().map(|it| it.kind());
        let parent = node.parent().map(|it| it.kind());
        Reparser::for_node(node.kind(), first_child, parent).map(|r| (node, r))
    })
}

fn is_balanced(lexed: &parser::LexedStr<'_>) -> bool {
    let significant =
        (0..lexed.len()).filter(|&it| !lexed.kind(it).is_trivia()).collect::<Vec<_>>();
    let (Some(&first), Some(&last)) = (significant.first(), significant.last()) else {
        return false;
    };
    if first == last || lexed.kind(first) != T!['{'] || lexed.kind(last) != T!['}'] {
        return false;
    }
    let mut balance = 0usize;
    for i in first + 1..last {
        match lexed.kind(i) {
            T!['{'] => balance += 1,
            T!['}'] => {
                balance = match balance.checked_sub(1) {
                    Some(b) => b,
                    None => return false,
                }
            }
            _ => (),
        }
    }
    balance == 0
}

fn merge_errors(
    old_errors: impl IntoIterator<Item = SyntaxError>,
    new_errors: Vec<SyntaxError>,
    range_before_reparse: TextRange,
    delete: TextRange,
    insert: &str,
) -> Vec<SyntaxError> {
    let mut res = Vec::new();

    for old_err in old_errors {
        let old_err_range = old_err.range();
        if old_err_range.end() <= range_before_reparse.start() {
            res.push(old_err);
        } else if old_err_range.start() >= range_before_reparse.end() {
            let inserted_len = TextSize::of(insert);
            res.push(old_err.with_range((old_err_range + inserted_len) - delete.len()));
            // Note: extra parens are intentional to prevent uint underflow, HWAB (here was a bug)
        }
    }
    res.extend(new_errors.into_iter().map(|new_err| {
        // fighting borrow checker with a variable ;)
        let offsetted_range = new_err.range() + range_before_reparse.start();
        new_err.with_range(offsetted_range)
    }));
    res
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use parser::Edition;
    use test_utils::{assert_eq_text, extract_range};

    use super::*;
    use crate::{AstNode, Parse, SourceFile};

    fn do_check_fallback(before: &str, replace_with: &str) {
        let (range, before) = extract_range(before);
        let parse = SourceFile::parse(&before, Edition::CURRENT);
        let reparsed = incremental_reparse(
            parse.tree().syntax(),
            range,
            replace_with,
            parse.errors.as_deref().unwrap_or_default().iter().cloned(),
            Edition::CURRENT,
        );
        assert!(reparsed.is_none(), "expected this edit to need a full reparse");
    }

    fn do_check(before: &str, replace_with: &str, reparsed_len: u32) {
        let (range, before) = extract_range(before);
        let after = {
            let mut after = before.clone();
            after.replace_range(Range::<usize>::from(range), replace_with);
            after
        };

        let fully_reparsed = SourceFile::parse(&after, Edition::CURRENT);
        let incrementally_reparsed: Parse<SourceFile> = {
            let before = SourceFile::parse(&before, Edition::CURRENT);
            let (green, new_errors, range) = incremental_reparse(
                before.tree().syntax(),
                range,
                replace_with,
                before.errors.as_deref().unwrap_or_default().iter().cloned(),
                Edition::CURRENT,
            )
            .unwrap();
            assert_eq!(range.len(), reparsed_len.into(), "reparsed fragment has wrong length");
            Parse::new(green, new_errors)
        };

        assert_eq_text!(
            &format!("{:#?}", fully_reparsed.tree().syntax()),
            &format!("{:#?}", incrementally_reparsed.tree().syntax()),
        );
        assert_eq!(fully_reparsed.errors(), incrementally_reparsed.errors());
    }

    #[test] // FIXME: some test here actually test token reparsing
    fn reparse_block_tests() {
        do_check(
            r"
fn foo() {
    let x = foo + $0bar$0
}
",
            "baz",
            3,
        );
        do_check(
            r"
fn foo() {
    let x = foo$0 + bar$0
}
",
            "baz",
            26,
        );
        do_check(
            r"
struct Foo {
    f: foo$0$0
}
",
            ",\n    g: (),",
            15,
        );
        do_check(
            r"
fn foo {
    let;
    1 + 1;
    $092$0;
}
",
            "62",
            32, // FIXME: reparse only int literal here
        );
        do_check(
            r"
mod foo {
    fn $0$0
}
",
            "bar",
            12,
        );

        do_check(
            r"
trait Foo {
    type $0Foo$0;
}
",
            "Output",
            3,
        );
        do_check(
            r"
impl IntoIterator<Item=i32> for Foo {
    f$0$0
}
",
            "n next(",
            10,
        );
        do_check(r"use a::b::{foo,$0,bar$0};", "baz", 10);
        do_check(
            r"
pub enum A {
    Foo$0$0
}
",
            "\nBar;\n",
            12,
        );
        do_check(
            r"
foo!{a, b$0$0 d}
",
            ", c[3]",
            9,
        );
        do_check(
            r"
fn foo() {
    vec![$0$0]
}
",
            "123",
            15,
        );
        do_check(
            r"
extern {
    fn$0;$0
}
",
            " exit(code: c_int)",
            12,
        );
    }

    #[test]
    fn reparse_token_tests() {
        do_check(
            r"$0$0
fn foo() -> i32 { 1 }
",
            "\n\n\n   \n",
            1,
        );
        do_check_fallback(
            r"
fn foo() -> $0$0 {}
",
            "  \n",
        );
        do_check(
            r"
fn $0foo$0() -> i32 { 1 }
",
            "bar",
            3,
        );
        do_check(
            r"
fn foo$0$0foo() {  }
",
            "bar",
            6,
        );
        do_check(
            r"
fn foo /* $0$0 */ () {}
",
            "some comment",
            6,
        );
        do_check_fallback(
            r"
fn baz $0$0 () {}
",
            "    \t\t\n\n",
        );
        do_check(
            r"
/// foo $0$0omment
mod { }
",
            "c",
            14,
        );
        do_check_fallback(
            r#"
fn -> &str { "Hello$0$0" }
"#,
            ", world",
        );
        do_check(
            r#"
fn -> &str { // "Hello$0$0"
"#,
            ", world",
            10,
        );
        do_check_fallback(
            r##"
fn -> &str { r#"Hello$0$0"#
"##,
            ", world",
        );
        do_check(
            r"
#[derive($0Copy$0)]
enum Foo {

}
",
            "Clone",
            4,
        );
    }

    #[test]
    fn reparse_str_token_with_error_unchanged() {
        do_check_fallback(r#""$0Unclosed$0 string literal"#, "Still unclosed");
    }

    #[test]
    fn reparse_str_token_with_error_fixed() {
        do_check_fallback(r#""unterminated$0$0"#, "\"");
    }

    #[test]
    fn reparse_block_with_error_in_middle_unchanged() {
        do_check(
            r#"fn main() {
                if {}
                32 + 4$0$0
                return
                if {}
            }"#,
            "23",
            105,
        )
    }

    #[test]
    fn reparse_block_with_error_in_middle_fixed() {
        do_check(
            r#"fn main() {
                if {}
                32 + 4$0$0
                return
                if {}
            }"#,
            ";",
            105,
        )
    }
}
