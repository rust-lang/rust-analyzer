use std::iter::once;

use ide_db::syntax_helpers::format_string::is_format_string;
use syntax::{
    AstNode, AstToken, NodeOrToken, T,
    ast::{self, make::tokens, syntax_factory::SyntaxFactory},
};

use crate::{
    AssistId,
    assist_context::{AssistContext, Assists},
};

// Assist: convert_to_format_string
//
// Convert string literal to `format!()`.
//
// ```
// fn foo() {
//     let n = 2;
//     let s = "n: {n$0}";
// }
// ```
// ->
// ```
// fn foo() {
//     let n = 2;
//     let s = format!("n: {n}");
// }
// ```
pub(crate) fn convert_to_format_string(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let str_token = ctx.find_token_at_offset::<ast::String>()?;
    let parent = str_token.syntax().parent()?;
    let text = str_token.syntax().text();

    if ctx
        .sema
        .descend_into_macros(str_token.syntax().clone())
        .into_iter()
        .filter_map(ast::String::cast)
        .any(|string| is_format_string(&string))
    {
        return None;
    }

    let offset_in_text = ctx.offset().checked_sub(str_token.syntax().text_range().start())?;
    let (left, variable, right) = split_curly(text, offset_in_text.into())?;
    let scope = ctx.sema.scope(&parent)?;

    if !variable.is_empty() && !exist_variable(variable, scope) {
        return None;
    }

    acc.add(
        AssistId::refactor_rewrite("convert_to_format_string"),
        "Convert to `format!()`",
        str_token.syntax().text_range(),
        |builder| {
            let left = escape_format_string(left);
            let right = escape_format_string(right);

            let mut edit = builder.make_editor(&parent);
            let make = SyntaxFactory::with_mappings();

            let new_str = format!("{left}{{{variable}}}{right}");
            let new_str = tokens::literal(&new_str);
            let args = once(NodeOrToken::Token(new_str)).chain(
                variable
                    .is_empty()
                    .then(|| {
                        [
                            NodeOrToken::Token(make.token(T![,])),
                            NodeOrToken::Token(make.whitespace(" ")),
                        ]
                    })
                    .into_iter()
                    .flatten(),
            );
            let tt = make.token_tree(T!['('], args);
            let expr_macro = make.expr_macro(make.ident_path("format"), tt);
            edit.replace(str_token.syntax(), expr_macro.syntax());

            if variable.is_empty()
                && let Some(cap) = ctx.config.snippet_cap
                && let Some(macro_call) = expr_macro.macro_call()
                && let Some(token_tree) = macro_call.token_tree()
                && let Some(NodeOrToken::Token(last)) = token_tree.token_trees_and_tokens().last()
            {
                let annotation = builder.make_tabstop_before(cap);
                edit.add_annotation(last, annotation);
            }

            edit.add_mappings(make.finish_with_mappings());
            builder.add_file_edits(ctx.vfs_file_id(), edit);
        },
    )
}

fn escape_format_string(s: &str) -> String {
    let mut replaced = s.replace('{', "{{");
    stdx::replace(&mut replaced, '}', "}}");
    replaced
}

fn exist_variable(variable: &str, scope: hir::SemanticsScope<'_>) -> bool {
    let mut exist = false;
    scope.process_all_names(&mut |name, def| {
        if !matches!(def, hir::ScopeDef::Local(_)) || exist {
            return;
        }
        exist = name.as_str() == variable;
    });
    exist
}

fn split_curly(text: &str, offset: usize) -> Option<(&str, &str, &str)> {
    let offset = improve_side_offset(text, offset).unwrap_or(offset);
    let (left, right) = text.split_at_checked(offset)?;
    let l_curly = left.rfind('{')?;
    let r_curly = right.find('}')? + left.len();

    Some((&text[..l_curly], &text[l_curly + 1..r_curly], &text[r_curly + 1..]))
}

fn improve_side_offset(text: &str, offset_in_text: usize) -> Option<usize> {
    text.get(offset_in_text..)
        .and_then(|s| (s.chars().next()? == '{').then_some(offset_in_text + 1))
        .or_else(|| {
            (text.get(..offset_in_text)?.chars().next_back()? == '}').then_some(offset_in_text - 1)
        })
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn empty_format() {
        check_assist(
            convert_to_format_string,
            r#"
            fn foo() {
                let s = "{$0}";
            }
            "#,
            r#"
            fn foo() {
                let s = format!("{}", $0);
            }
            "#,
        );

        check_assist(
            convert_to_format_string,
            r#"
            fn foo() {
                let s = "left{$0}right";
            }
            "#,
            r#"
            fn foo() {
                let s = format!("left{}right", $0);
            }
            "#,
        );
    }

    #[test]
    fn curly_offsets() {
        check_assist(
            convert_to_format_string,
            r#"
            fn foo() {
                let s = "left{}$0right";
            }
            "#,
            r#"
            fn foo() {
                let s = format!("left{}right", $0);
            }
            "#,
        );

        check_assist(
            convert_to_format_string,
            r#"
            fn foo() {
                let s = "left$0{}right";
            }
            "#,
            r#"
            fn foo() {
                let s = format!("left{}right", $0);
            }
            "#,
        );
    }

    #[test]
    fn biased_curlys() {
        check_assist(
            convert_to_format_string,
            r#"
            fn foo() {
                let s = "left{}$0{}right";
            }
            "#,
            r#"
            fn foo() {
                let s = format!("left{{}}{}right", $0);
            }
            "#,
        );
    }

    #[test]
    fn not_format_other_curlys() {
        check_assist(
            convert_to_format_string,
            r#"
            fn foo() {
                let s = "{left{$0}right}";
            }
            "#,
            r#"
            fn foo() {
                let s = format!("{{left{}right}}", $0);
            }
            "#,
        );

        check_assist(
            convert_to_format_string,
            r#"
            fn foo() {
                let s = "{{}left{{$0}}right{}}";
            }
            "#,
            r#"
            fn foo() {
                let s = format!("{{{{}}left{{{}}}right{{}}}}", $0);
            }
            "#,
        );
    }

    #[test]
    fn format_variable() {
        check_assist(
            convert_to_format_string,
            r#"
            fn foo() {
                let var = 2;
                let s = "{left{var$0}right}";
            }
            "#,
            r#"
            fn foo() {
                let var = 2;
                let s = format!("{{left{var}right}}");
            }
            "#,
        );

        check_assist(
            convert_to_format_string,
            r#"
            fn foo() {
                let s = "{{}left{{$0}}right{}}";
            }
            "#,
            r#"
            fn foo() {
                let s = format!("{{{{}}left{{{}}}right{{}}}}", $0);
            }
            "#,
        );
    }

    #[test]
    fn applicable_in_macro() {
        check_assist(
            convert_to_format_string,
            r#"
            fn foo() {
                let var = 2;
                let s = some_macro!("{left{var$0}right}");
            }
            "#,
            r#"
            fn foo() {
                let var = 2;
                let s = some_macro!(format!("{{left{var}right}}"));
            }
            "#,
        );

        check_assist(
            convert_to_format_string,
            r#"
            //- minicore: fmt
            fn foo() {
                let var = 2;
                let s = print!("{}", "{left{var$0}right}");
            }
            "#,
            r#"
            fn foo() {
                let var = 2;
                let s = print!("{}", format!("{{left{var}right}}"));
            }
            "#,
        );
    }

    #[test]
    fn not_applicable_outside_curly() {
        check_assist_not_applicable(
            convert_to_format_string,
            r#"
            fn foo() {
                let s = "left{}r$0ight";
            }
            "#,
        );

        check_assist_not_applicable(
            convert_to_format_string,
            r#"
            fn foo() {
                let s = "l$0eft{}right";
            }
            "#,
        );
    }

    #[test]
    fn not_applicable_unknown_variable() {
        check_assist_not_applicable(
            convert_to_format_string,
            r#"
            fn foo() {
                let s = "left{var$0}right";
            }
            "#,
        );
    }

    #[test]
    fn not_applicable_is_format_string() {
        check_assist_not_applicable(
            convert_to_format_string,
            r#"
            //- minicore: fmt
            fn foo() {
                let s = print!("left{$0}right");
            }
            "#,
        );
    }
}
