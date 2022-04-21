use std::ops::ControlFlow;

use ide_db::{
    assists::{AssistId, AssistKind},
    syntax_helpers::format_string::{lex_format_specifiers, FormatSpecifier},
};
use itertools::Itertools;
use syntax::{
    ast::{self, IsString},
    AstNode, AstToken, SyntaxToken, TextRange, TextSize, T,
};

use crate::assist_context::{AssistContext, Assists};

// Assist: add_interpolation
//
// Adds interpolations in a string literal.
//
// ```
// fn get(map: HashMap<i32, i32>, key: i32) -> i32 {
//     map.get(&key).expect("Invalid key: $0")
// }
// ```
// ->
// ```
// fn get(map: HashMap<i32, i32>, key: i32) -> i32 {
//     map.get(&key).expect(&format!("Invalid key: {}", $0))
// }
// ```
pub(crate) fn add_interpolation(acc: &mut Assists, ctx: &AssistContext) -> Option<()> {
    let string_ast = ctx.find_token_at_offset::<ast::String>()?;

    if !string_ast.text_range_between_quotes()?.contains_inclusive(ctx.offset()) {
        return None;
    }

    if !ctx.has_empty_selection() {
        return None;
    }

    if is_inside_escape_char(&string_ast, ctx.offset()) {
        return None;
    }

    let string_token = string_ast.syntax();
    let string_range = string_token.text_range();

    let text_offset = usize::from(ctx.offset());
    let token_start = usize::from(string_range.start());
    let (left, right) = string_token.text().split_at(text_offset - token_start);

    let id = AssistId("add_interpolation", AssistKind::RefactorRewrite);
    let label = "Add interpolation";

    let insert_expr = if ctx.config.snippet_cap.is_some() { "$0" } else { "_" };

    if let Some(arg_delimiters) = match_format_macro_arg_delims(&string_ast) {
        let arg_idx = get_insert_arg_idx(&string_ast, ctx.offset())?;
        let delimiter_token = arg_delimiters.get(arg_idx)?;

        acc.add(id, label, string_range, |builder| {
            builder.replace(string_range, format!("{left}{{}}{right}"));

            let insert_index = delimiter_token.text_range().start();
            let insert_text = format!(", {insert_expr}");

            match ctx.config.snippet_cap {
                Some(cap) => builder.insert_snippet(cap, insert_index, insert_text),
                None => builder.insert(insert_index, insert_text),
            }
        })
    } else {
        acc.add(id, label, string_range, |builder| {
            let left = left.replace('{', "{{").replace('}', "}}");
            let right = right.replace('{', "{{").replace('}', "}}");

            let (range, reference) = if let Some(node) = match_to_string(&string_ast) {
                (node.text_range(), "")
            } else {
                (string_range, "&")
            };

            let replace_text = format!("{reference}format!({left}{{}}{right}, {insert_expr})");

            match ctx.config.snippet_cap {
                Some(cap) => builder.replace_snippet(cap, range, replace_text),
                None => builder.replace(range, replace_text),
            }
        })
    }
}

fn is_inside_escape_char(string_ast: &ast::String, offset: syntax::TextSize) -> bool {
    let string_token = string_ast.syntax();
    let string_range = string_token.text_range();
    let mut inside_escape = false;
    string_ast.escaped_char_ranges(&mut |range, _| {
        let range = range + string_range.start();
        if range.start() < offset && offset < range.end() {
            inside_escape = true;
        }
    });
    inside_escape
}

fn match_format_macro_arg_delims(string_ast: &ast::String) -> Option<Vec<SyntaxToken>> {
    let string_token = string_ast.syntax();

    let macro_call = ast::MacroCall::cast(string_token.ancestors().nth(1)?)?;
    let macro_name = macro_call.path()?.segment()?.name_ref()?;

    let format_arg_idx = match macro_name.text().as_str() {
        "eprint" | "eprintln" | "format" | "panic" | "print" | "println" | "todo"
        | "unimplemented" | "unreachable" => 0,
        "assert" | "debug_assert" | "write" | "writeln" => 1,
        "assert_eq" | "assert_ne" | "debug_assert_eq" | "debug_assert_ne" => 2,
        _ => return None,
    };

    let token_tree = macro_call.token_tree()?;
    let left_delim = token_tree.left_delimiter_token()?;
    let right_delim = token_tree.right_delimiter_token()?;
    let mut delimiters: Vec<SyntaxToken> = token_tree
        .token_trees_and_tokens()
        .flat_map(|not| not.as_token().cloned())
        .filter(|token| token.kind() == T![,] || token == &left_delim || token == &right_delim)
        .collect();

    let format_arg_start = delimiters.get(format_arg_idx)?.text_range().end();
    let format_arg_end = delimiters.get(format_arg_idx + 1)?.text_range().start();
    let format_arg_range = TextRange::new(format_arg_start, format_arg_end);

    if !format_arg_range.contains_range(string_token.text_range()) {
        return None;
    }

    delimiters.drain(..format_arg_idx + 1);

    if let Some(first_named) = first_named_argument(token_tree) {
        delimiters.retain(|delim| delim.text_range().end() <= first_named.text_range().start());
    }

    Some(delimiters)
}

fn first_named_argument(token_tree: ast::TokenTree) -> Option<SyntaxToken> {
    #[derive(Clone, Copy)]
    enum NameState {
        Identifier,
        Equal,
        Expression,
    }

    let mut name_state = NameState::Identifier;
    let mut name_candiate = None;

    for not in token_tree.token_trees_and_tokens() {
        if let Some(token) = not.into_token() {
            match (token.kind(), name_state) {
                (syntax::SyntaxKind::IDENT, NameState::Identifier) => {
                    name_candiate = Some(token);
                    name_state = NameState::Equal;
                }
                (T![=], NameState::Equal) => {
                    debug_assert!(name_candiate.is_some());
                    return name_candiate;
                }
                (T![,], _) => {
                    name_state = NameState::Identifier;
                    name_candiate = None;
                }
                (kind, _) => {
                    if !kind.is_trivia() {
                        name_state = NameState::Expression;
                    }
                }
            };
        } else {
            name_state = NameState::Expression;
        }
    }

    None
}

fn get_insert_arg_idx(string_ast: &ast::String, offset: syntax::TextSize) -> Option<usize> {
    let string_token = string_ast.syntax();
    let string_start = string_token.text_range().start();

    let mut format_arg_end = TextSize::from(0);
    let mut arg_idx = 0usize;

    let mut visit_format_spec = |text_range: TextRange, format_spec| {
        match format_spec {
            FormatSpecifier::Open => {
                let format_arg_start = string_start + text_range.start();
                if format_arg_end <= offset && offset <= format_arg_start {
                    return ControlFlow::Break(Some(arg_idx));
                }
                arg_idx += 1;
            }
            FormatSpecifier::Identifier => arg_idx = arg_idx.saturating_sub(1), // skip named arguments and captured identifiers
            FormatSpecifier::Close => format_arg_end = string_start + text_range.end(),
            _ => {}
        }

        ControlFlow::Continue(())
    };

    let mut result = None;
    lex_format_specifiers(&string_ast, &mut |text_range, format_spec| {
        if result.is_some() {
            return;
        }

        if let ControlFlow::Break(arg_idx) = visit_format_spec(text_range, format_spec) {
            result = Some(arg_idx);
        }
    });

    if result.is_none() && format_arg_end <= offset {
        Some(arg_idx)
    } else {
        result.flatten()
    }
}

fn match_to_string(string_ast: &ast::String) -> Option<syntax::SyntaxNode> {
    let parent = string_ast
        .syntax()
        .ancestors()
        .map(ast::Expr::cast)
        .while_some()
        .filter(|expr| !matches!(expr, ast::Expr::ParenExpr(_) | ast::Expr::Literal(_)))
        .next()?;

    let method_call = match parent {
        ast::Expr::MethodCallExpr(method_call) => method_call,
        _ => return None,
    };

    match method_call.name_ref()?.text().as_str() {
        "to_string" | "to_owned" | "into" => Some(method_call.syntax().clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable, check_assist_target};

    use super::*;

    #[test]
    fn string_literal_target() {
        let check_ok = |input: &str| {
            check_assist_target(add_interpolation, input, r#""Hello""#);
        };
        let check_not_ok = |input: &str| {
            check_assist_not_applicable(add_interpolation, input);
        };

        check_not_ok(r#"fn m() { let _ = $0"Hello"; }"#);
        check_ok(r#"fn m() { let _ = "$0Hello"; }"#);
        check_ok(r#"fn m() { let _ = "Hel$0lo"; }"#);
        check_ok(r#"fn m() { let _ = "Hello$0"; }"#);
        check_not_ok(r#"fn m() { let _ = "Hello"$0; }"#);
    }

    #[test]
    fn raw_string_literal_target() {
        let check_ok = |input: &str| {
            check_assist_target(add_interpolation, input, r##"r#"Hello"#"##);
        };
        let check_not_ok = |input: &str| {
            check_assist_not_applicable(add_interpolation, input);
        };

        check_not_ok(r##"fn m() { let _ = $0r#"Hello"#; }"##);
        check_not_ok(r##"fn m() { let _ = r$0#"Hello"#; }"##);
        check_not_ok(r##"fn m() { let _ = r#$0"Hello"#; }"##);
        check_ok(r##"fn m() { let _ = r#"$0Hello"#; }"##);
        check_ok(r##"fn m() { let _ = r#"Hel$0lo"#; }"##);
        check_ok(r##"fn m() { let _ = r#"Hello$0"#; }"##);
        check_not_ok(r##"fn m() { let _ = r#"Hello"$0#; }"##);
        check_not_ok(r##"fn m() { let _ = r#"Hello"#$0; }"##);
    }

    #[test]
    fn format_string_target() {
        let check_ok = |input: &str| {
            check_assist_target(add_interpolation, input, r#""H{}ello""#);
        };
        let check_not_ok = |input: &str| {
            check_assist_not_applicable(add_interpolation, input);
        };

        check_ok(r#"fn m() { let _ = format!("H$0{}ello", 1)"#);
        check_not_ok(r#"fn m() { let _ = format!("H{$0}ello", 1)"#);
        check_ok(r#"fn m() { let _ = format!("H{}$0ello", 1)"#);
    }

    #[test]
    fn braces_in_string_literal_target() {
        check_assist_target(
            add_interpolation,
            r#"fn m() { let _ = "H{$0}ello"; }"#,
            r#""H{}ello""#,
        );
    }

    #[test]
    fn string_literal() {
        check_assist(
            add_interpolation,
            r#"fn m() { let _ = "H$0ello"; }"#,
            r#"fn m() { let _ = &format!("H{}ello", $0); }"#,
        );
    }

    #[test]
    fn string_literal_with_braces() {
        check_assist(
            add_interpolation,
            r#"fn m() { let _ = "H$0e}{llo"; }"#,
            r#"fn m() { let _ = &format!("H{}e}}{{llo", $0); }"#,
        );
    }

    #[test]
    fn string_literal_with_tostring() {
        check_assist(
            add_interpolation,
            r#"fn m() { let _ = "H$0ello".to_string(); }"#,
            r#"fn m() { let _ = format!("H{}ello", $0); }"#,
        );
    }

    #[test]
    fn string_literal_in_format_macro_no_args() {
        check_assist(
            add_interpolation,
            r#"fn m() { let _ = format!("H$0ello"); }"#,
            r#"fn m() { let _ = format!("H{}ello", $0); }"#,
        );
    }

    #[test]
    fn string_literal_in_format_macro_insert_arg0() {
        check_assist(
            add_interpolation,
            r#"fn m() { let _ = format!("H$0e{}llo", 123); }"#,
            r#"fn m() { let _ = format!("H{}e{}llo", $0, 123); }"#,
        );
    }

    #[test]
    fn string_literal_in_format_macro_insert_arg1() {
        check_assist(
            add_interpolation,
            r#"fn m() { let _ = format!("He{}l$0lo", 123); }"#,
            r#"fn m() { let _ = format!("He{}l{}lo", 123, $0); }"#,
        );
    }

    #[test]
    fn string_literal_in_assert_macro() {
        check_assist(
            add_interpolation,
            r#"assert!(1 = 1, "H$0ello"); }"#,
            r#"assert!(1 = 1, "H{}ello", $0); }"#,
        );
    }

    #[test]
    fn string_literal_nested() {
        check_assist(
            add_interpolation,
            r#"fn m() { let _ = format!("H{}ello", "World$0"); }"#,
            r#"fn m() { let _ = format!("H{}ello", &format!("World{}", $0)); }"#,
        );
    }

    #[test]
    fn string_literal_in_vec_macro() {
        check_assist(
            add_interpolation,
            r#"fn m() { let _ = vec!["H$0ello"]; }"#,
            r#"fn m() { let _ = vec![&format!("H{}ello", $0)]; }"#,
        );
    }

    #[test]
    fn string_literal_named_args() {
        check_assist(
            add_interpolation,
            r#"fn m() { let _ = format!("H{e}ll$0o", e = 1)"#,
            r#"fn m() { let _ = format!("H{e}ll{}o", $0, e = 1)"#,
        );
    }

    #[test]
    fn string_literal_captured_identifiers() {
        check_assist(
            add_interpolation,
            r#"fn m() { let e = 1; let _ = format!("H{e}l$0lo{}", 123)"#,
            r#"fn m() { let e = 1; let _ = format!("H{e}l{}lo{}", $0, 123)"#,
        );
    }
}
