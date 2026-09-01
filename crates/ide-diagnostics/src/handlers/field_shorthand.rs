//! Suggests shortening `Foo { field: field }` to `Foo { field }` in both
//! expressions and patterns.

use ide_db::RootDatabase;
use ide_db::text_edit::TextEdit;
use ide_db::{EditionedFileId, FileRange, source_change::SourceChange};
use syntax::token_span;
use syntax::{AstNode, SyntaxNode, ast, match_ast};

use crate::{Diagnostic, DiagnosticCode, fix};

pub(crate) fn field_shorthand(
    db: &RootDatabase,
    acc: &mut Vec<Diagnostic>,
    file_id: EditionedFileId,
    node: &SyntaxNode,
) {
    match_ast! {
        match node {
            ast::RecordExpr(it) => check_expr_field_shorthand(db, acc, file_id, it),
            ast::RecordPat(it) => check_pat_field_shorthand(db, acc, file_id, it),
            _ => ()
        }
    };
}

fn check_expr_field_shorthand(
    db: &RootDatabase,
    acc: &mut Vec<Diagnostic>,
    file_id: EditionedFileId,
    record_expr: ast::RecordExpr,
) {
    let record_field_list = match record_expr.record_expr_field_list() {
        Some(it) => it,
        None => return,
    };
    for record_field in record_field_list.fields() {
        let (name_ref, expr) = match record_field.name_ref().zip(record_field.expr()) {
            Some(it) => it,
            None => continue,
        };

        let field_name = syntax::token_text(name_ref.syntax());
        let field_expr = syntax::token_text(expr.syntax());
        let field_name_is_tup_index = name_ref.as_tuple_field().is_some();
        if field_name != field_expr || field_name_is_tup_index {
            continue;
        }

        let mut edit_builder = TextEdit::builder();
        edit_builder.replace(token_span(record_field.syntax()), field_name);
        let edit = edit_builder.finish();

        let field_range = token_span(record_field.syntax());
        let vfs_file_id = file_id.file_id(db);
        acc.push(
            Diagnostic::new(
                DiagnosticCode::Clippy("redundant_field_names"),
                "Shorthand struct initialization",
                FileRange { file_id: vfs_file_id, range: field_range },
            )
            .with_fixes(Some(vec![fix(
                "use_expr_field_shorthand",
                "Use struct shorthand initialization",
                SourceChange::from_text_edit(vfs_file_id, edit),
                field_range,
            )])),
        );
    }
}

fn check_pat_field_shorthand(
    db: &RootDatabase,
    acc: &mut Vec<Diagnostic>,
    file_id: EditionedFileId,
    record_pat: ast::RecordPat,
) {
    let record_pat_field_list = match record_pat.record_pat_field_list() {
        Some(it) => it,
        None => return,
    };
    for record_pat_field in record_pat_field_list.fields() {
        let (name_ref, pat) = match record_pat_field.name_ref().zip(record_pat_field.pat()) {
            Some(it) => it,
            None => continue,
        };

        let field_name = syntax::token_text(name_ref.syntax());
        let field_pat = syntax::token_text(pat.syntax());
        let field_name_is_tup_index = name_ref.as_tuple_field().is_some();
        if field_name != field_pat || field_name_is_tup_index {
            continue;
        }

        let mut edit_builder = TextEdit::builder();
        edit_builder.replace(token_span(record_pat_field.syntax()), field_name);
        let edit = edit_builder.finish();

        let field_range = token_span(record_pat_field.syntax());
        let vfs_file_id = file_id.file_id(db);
        acc.push(
            Diagnostic::new(
                DiagnosticCode::Clippy("redundant_field_names"),
                "Shorthand struct pattern",
                FileRange { file_id: vfs_file_id, range: field_range },
            )
            .with_fixes(Some(vec![fix(
                "use_pat_field_shorthand",
                "Use struct field shorthand",
                SourceChange::from_text_edit(vfs_file_id, edit),
                field_range,
            )])),
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_diagnostics, check_fix};

    #[test]
    fn test_check_expr_field_shorthand() {
        check_diagnostics(
            r#"
struct A { a: &'static str }
fn main() { A { a: "hello" }; }
"#,
        );
        check_diagnostics(
            r#"
struct A(usize);
fn main() { A { 0: 0 }; }
"#,
        );

        check_fix(
            r#"
struct A { a: &'static str }
fn main() {
    let a = "haha";
    A { a$0: a };
}
"#,
            r#"
struct A { a: &'static str }
fn main() {
    let a = "haha";
    A { a };
}
"#,
        );

        check_fix(
            r#"
struct A { a: &'static str, b: &'static str }
fn main() {
    let a = "haha";
    let b = "bb";
    A { a$0: a, b };
}
"#,
            r#"
struct A { a: &'static str, b: &'static str }
fn main() {
    let a = "haha";
    let b = "bb";
    A { a, b };
}
"#,
        );
    }

    #[test]
    fn test_check_pat_field_shorthand() {
        check_diagnostics(
            r#"
struct A { a: &'static str }
fn f(a: A) { let A { a: _hello } = a; }
"#,
        );
        check_diagnostics(
            r#"
struct A(usize);
fn f(a: A) { let A { 0: 0 } = a; }
"#,
        );

        check_fix(
            r#"
struct A { a: &'static str }
fn f(a: A) {
    let A { a$0: a } = a;
    _ = a;
}
"#,
            r#"
struct A { a: &'static str }
fn f(a: A) {
    let A { a } = a;
    _ = a;
}
"#,
        );

        check_fix(
            r#"
struct A { a: &'static str, b: &'static str }
fn f(a: A) {
    let A { a$0: a, b } = a;
    _ = (a, b);
}
"#,
            r#"
struct A { a: &'static str, b: &'static str }
fn f(a: A) {
    let A { a, b } = a;
    _ = (a, b);
}
"#,
        );
    }

    #[test]
    fn diagnostic_range_respect_allows() {
        check_diagnostics(
            r#"
#![allow(clippy::redundant_field_names, unused)]

struct Foo {
    bar: u32,
}

fn main() {
    let bar = 23;
    let foo = Foo {
	    bar: bar,
    };
}
        "#,
        );
    }
}
