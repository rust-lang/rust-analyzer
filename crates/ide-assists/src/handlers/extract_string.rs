use ide_db::syntax_helpers::format_string::is_format_string;
use syntax::{
    AstToken, NodeOrToken, T, TextRange,
    ast::{self, AstNode, IsString, edit::IndentLevel, syntax_factory::SyntaxFactory},
    syntax_editor::Position,
};

use crate::{AssistContext, AssistId, Assists};

// Assist: extract_string
//
// Extract substring inside format string.
//
// ```
// //- minicore: fmt
// fn main() {
//     let out = format_args!("foo$0bar$0");
// }
// ```
// ->
// ```
// fn main() {
//     let new_str = "bar";
//     let out = format_args!("foo{new_str}");
// }
// ```
pub(crate) fn extract_string(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let tok = ctx.find_token_at_offset::<ast::String>()?;

    if ctx.has_empty_selection() {
        return None;
    }
    if !tok.text_range_between_quotes()?.contains_range(ctx.selection_trimmed()) {
        return None;
    }

    let (left, extract, right) = split_selection(&tok, ctx.selection_trimmed())?;
    if ![left, extract, right].into_iter().all(is_valid_string) {
        cov_mark::hit!(extract_string_invalid_format_string);
        return None;
    }
    let is_format_string = ctx
        .sema
        .descend_into_macros(tok.syntax().clone())
        .into_iter()
        .filter_map(ast::String::cast)
        .any(|it| is_format_string(&it));
    if !is_format_string {
        cov_mark::hit!(extract_string_not_a_format_string);
        return None;
    }
    let (open_quote, close_quote) = quotes(&tok)?;
    let insert_before = tok
        .syntax()
        .parent_ancestors()
        .find(|it| it.parent().is_some_and(|parent| ast::StmtList::can_cast(parent.kind())))?;

    acc.add(
        AssistId::refactor_extract("extract_string"),
        "Extract substring",
        tok.syntax().text_range(),
        |builder| {
            let mut edit = builder.make_editor(&insert_before);
            let make = SyntaxFactory::with_mappings();
            let indent = IndentLevel::from_node(&insert_before);

            let var = "new_str";
            let extracted_lit = format!("{open_quote}{extract}{close_quote}");
            let needs_format_args = extract.contains(['{', '}']);

            let initializer = if needs_format_args {
                let literal = make.expr_literal(&extracted_lit).token();
                let tt = make.token_tree(T!['('], [NodeOrToken::Token(literal)]);
                make.expr_macro(make.ident_path("format_args"), tt).into()
            } else {
                make.expr_literal(&extracted_lit).into()
            };
            let let_stmt = make.let_stmt(
                make.ident_pat(false, false, make.name(var)).into(),
                None,
                Some(initializer),
            );

            edit.insert_all(
                Position::before(insert_before),
                vec![
                    let_stmt.syntax().clone().into(),
                    make.whitespace(&format!("\n{indent}")).into(),
                ],
            );
            edit.replace(
                tok.syntax(),
                make.expr_literal(&format!("{left}{{{var}}}{right}")).token(),
            );

            edit.add_mappings(make.finish_with_mappings());
            builder.add_file_edits(ctx.vfs_file_id(), edit);
        },
    )
}

fn split_selection(tok: &ast::String, selection: TextRange) -> Option<(&str, &str, &str)> {
    let raw_range = tok.syntax().text_range();
    let range = raw_range.checked_sub(raw_range.start())?;
    let selection = selection.checked_sub(raw_range.start())?;

    let left = &tok.text()[TextRange::new(range.start(), selection.start())];
    let right = &tok.text()[TextRange::new(selection.end(), range.end())];
    let extract = &tok.text()[selection];

    Some((left, extract, right))
}

fn quotes(tok: &ast::String) -> Option<(&str, &str)> {
    let start = tok.syntax().text_range().start();
    let text = tok.text();

    let open = tok.open_quote_text_range()?.checked_sub(start)?;
    let close = tok.open_quote_text_range()?.checked_sub(start)?;

    Some((&text[open], &text[close]))
}

fn is_valid_string(s: &str) -> bool {
    let backslash_count = s.len() - s.trim_end_matches('\\').len();
    if backslash_count % 2 == 1 {
        return false;
    }
    is_valid_format_string(s)
}

fn is_valid_format_string(s: &str) -> bool {
    let mut chars = s.chars().peekable();
    let is_ident = |&ch: &char| !ch.is_ascii() || ch.is_ascii_alphanumeric() || ch == '_';

    while let Some(ch) = chars.next() {
        match (ch, chars.peek()) {
            ('{', Some('{')) | ('}', Some('}')) => {
                chars.next();
            }
            ('}', _) => return false,
            ('{', _) => {
                while chars.next_if(is_ident).is_some() {}

                if chars.next().is_none_or(|ch| ch != '}') {
                    return false;
                }
            }
            _ => (),
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn extract_to_format_args() {
        check_assist(
            extract_string,
            r#"
//- minicore: fmt
fn main() {
    let n = 2;
    let out = format_args!("foo$0bar{n}$0");
}"#,
            r#"
fn main() {
    let n = 2;
    let new_str = format_args!("bar{n}");
    let out = format_args!("foo{new_str}");
}"#,
        );
    }

    #[test]
    fn extract_escaped_braces() {
        check_assist(
            extract_string,
            r#"
//- minicore: fmt
fn main() {
    let out = format_args!("foo$0bar{{n$0");
}"#,
            r#"
fn main() {
    let new_str = format_args!("bar{{n");
    let out = format_args!("foo{new_str}");
}"#,
        );
    }

    #[test]
    fn extract_escaped_backslashes() {
        check_assist(
            extract_string,
            r#"
//- minicore: fmt
fn main() {
    let out = format_args!("foo$0bar\\n$0");
}"#,
            r#"
fn main() {
    let new_str = "bar\\n";
    let out = format_args!("foo{new_str}");
}"#,
        );
    }

    #[test]
    fn extract_with_tails() {
        check_assist(
            extract_string,
            r#"
//- minicore: fmt
fn main() {
    let n = 2;
    let out = format_args!("foo$0bar{n}$0tail");
}"#,
            r#"
fn main() {
    let n = 2;
    let new_str = format_args!("bar{n}");
    let out = format_args!("foo{new_str}tail");
}"#,
        );
    }

    #[test]
    fn invalid_format_string() {
        cov_mark::check_count!(extract_string_invalid_format_string, 3);
        check_assist_not_applicable(
            extract_string,
            r#"
//- minicore: fmt
fn main() {
    let n = 2;
    let out = format_args!("foo$0bar{$0n}tail");
}"#,
        );
        check_assist_not_applicable(
            extract_string,
            r#"
//- minicore: fmt
fn main() {
    let n = 2;
    let out = format_args!("foo$0bar{n$0}tail");
}"#,
        );
        check_assist_not_applicable(
            extract_string,
            r#"
//- minicore: fmt
fn main() {
    let n = 2;
    let out = format_args!("foobar{n$0}ta$0il");
}"#,
        );
    }

    #[test]
    fn not_a_format_string() {
        cov_mark::check!(extract_string_not_a_format_string);
        check_assist_not_applicable(
            extract_string,
            r#"
fn main() {
    let n = 2;
    let out = not_format!("foob$0ar{n}ta$0il");
}"#,
        );
    }
}
