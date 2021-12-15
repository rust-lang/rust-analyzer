use hir::db::AstDatabase;
use hir::diagnostics::RemoveTrailingReturn;
use hir::InFile;
use ide_db::assists::Assist;
use ide_db::source_change::SourceChange;
use syntax::{ast, AstNode, NodeOrToken, TextRange, T};
use text_edit::TextEdit;

use crate::{fix, Diagnostic, DiagnosticsContext, Severity};

pub(crate) fn remove_trailing_return(
    ctx: &DiagnosticsContext<'_>,
    d: &RemoveTrailingReturn,
) -> Diagnostic {
    Diagnostic::new(
        "remove-trailing-return",
        "replace return <expr>; with <expr>",
        ctx.sema.diagnostics_display_range(InFile::new(d.file, d.return_expr.clone().into())).range,
    )
    .severity(Severity::WeakWarning)
    .with_fixes(fixes(ctx, d))
}

fn fixes(ctx: &DiagnosticsContext<'_>, d: &RemoveTrailingReturn) -> Option<Vec<Assist>> {
    let root = ctx.sema.db.parse_or_expand(d.file)?;

    let return_expr = d.return_expr.to_node(&root);

    let return_expr = ast::ReturnExpr::cast(return_expr.syntax().clone())?;

    let semi = match return_expr.syntax().next_sibling_or_token() {
        Some(NodeOrToken::Token(token)) if token.kind() == T![;] => Some(token),
        _ => None,
    };

    let range_to_replace = match semi {
        Some(semi) => {
            TextRange::new(return_expr.syntax().text_range().start(), semi.text_range().end())
        }
        None => return_expr.syntax().text_range(),
    };

    let replacement =
        return_expr.expr().map_or_else(String::new, |expr| format!("{}", expr.syntax().text()));

    // this *seems* like a reasonable range to trigger in?
    let trigger_range = range_to_replace;

    let edit = TextEdit::replace(range_to_replace, replacement);

    let source_change = SourceChange::from_text_edit(d.file.original_file(ctx.sema.db), edit);

    Some(vec![fix(
        "replace_with_inner",
        "Replace return <expr>; with <expr>",
        source_change,
        trigger_range,
    )])
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_diagnostics, check_fix};

    #[test]
    fn remove_trailing_return() {
        // fixme: this should include the semi.
        check_diagnostics(
            r#"
fn foo() -> i8 {
    return 2;
} //^^^^^^^^ 💡 weak: replace return <expr>; with <expr>
"#,
        );
    }

    // fixme: implement this for lambdas and inner functions.
    #[test]
    fn remove_trailing_return_no_lambda() {
        // fixme: this should include the semi.
        check_diagnostics(
            r#"
fn foo() -> i8 {
    let bar = || return 2;
    bar()
}
"#,
        );
    }

    #[test]
    fn remove_trailing_return_unit() {
        check_diagnostics(
            r#"
fn foo() -> i8 {
    return
} //^^^^^^ 💡 weak: replace return <expr>; with <expr>
"#,
        );
    }

    #[test]
    fn remove_trailing_return_no_diagnostic_if_no_return_keyword() {
        check_diagnostics(
            r#"
fn foo() -> i8 {
    3
}
"#,
        );
    }

    #[test]
    fn remove_trailing_return_no_diagnostic_if_not_at_and() {
        check_diagnostics(
            r#"
fn foo() -> i8 {
    if true { return 2; }
    3
}
"#,
        );
    }

    #[test]
    fn replace_with_expr() {
        check_fix(
            r#"
fn foo() -> i8 {
    return$0 2;
}
"#,
            r#"
fn foo() -> i8 {
    2
}
"#,
        );
    }
    #[test]
    fn replace_with_unit() {
        check_fix(
            r#"
fn foo() {
    return$0
}
"#,
            r#"
fn foo() {
    
}
"#,
        );
    }
}
