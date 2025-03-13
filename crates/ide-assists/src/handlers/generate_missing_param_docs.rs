use crate::assist_context::{AssistContext, Assists};
use ide_db::assists::{AssistId, AssistKind};
use syntax::{
    ast::{self, edit::IndentLevel, HasDocComments},
    AstNode, AstToken,
};

// Currently missing features:
// - keep order of params : Issue - ?
// - make todo string configurable : Issue - ?
// - move cursor to todo declarations like with snippets : Issue - ?

// Assist: generate_missing_param_docs
//
// Generates doc comment for a function parameter.
//
// ```
// fn some_function(x: i32$0) {}
// ```
// ->
// ```
// /// * `x` - TODO: Description
// fn some_function(x: i32) {
// }
// ```
pub(crate) fn generate_missing_param_docs(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
) -> Option<()> {
    // Find the function node at the cursor offset.
    let fn_node = ctx.find_node_at_offset::<ast::Fn>()?;

    // Get the parameter list from the function.
    let param_list = fn_node.param_list()?;
    let params: Vec<String> =
        param_list.params().filter_map(|param| param.pat().map(|p| p.to_string())).collect();

    // Retrieve existing doc comment lines (if any)
    let existing_docs: Vec<String> =
        fn_node.doc_comments().map(|doc| doc.text().to_string()).collect();

    // Build documentation lines for parameters not yet documented.
    let mut new_doc_lines = Vec::new();
    for param in params {
        // Check if any existing doc comment already mentions this parameter.
        if !existing_docs.iter().any(|line| line.contains(&format!("* `{}` -", param))) {
            new_doc_lines.push(format!("* `{}` - TODO: Description", param));
        }
    }

    // If all parameters are already documented, there's nothing to do.
    if new_doc_lines.is_empty() {
        return None;
    }

    // Determine where to insert the new documentation.
    let text_range = fn_node.syntax().text_range();
    let indent_level = IndentLevel::from_node(fn_node.syntax());

    // Add the assist: insert new doc lines above the function.
    acc.add(
        AssistId("generate_missing_param_docs", AssistKind::Generate),
        "Generate missing parameter documentation",
        text_range,
        |builder| {
            // You can use a helper similar to documentation_from_lines from the template assist.
            builder
                .insert(text_range.start(), documentation_from_lines(new_doc_lines, indent_level));
        },
    )
}

/// Helper function to transform lines of documentation into a Rust code documentation
/// (stolen from generate_documentation_template)
fn documentation_from_lines(doc_lines: Vec<String>, indent_level: IndentLevel) -> String {
    let mut result = String::new();
    for doc_line in doc_lines {
        result.push_str("///");
        if !doc_line.is_empty() {
            result.push(' ');
            result.push_str(&doc_line);
        }
        result.push('\n');
        result.push_str(&indent_level.to_string());
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn generate_missing_param_documentation() {
        check_assist(
            generate_missing_param_docs,
            r#"
fn foo($0y: i32) {}
"#,
            r#"
/// * `y` - TODO: Description
fn foo(y: i32) {}
"#,
        );
    }

    #[test]
    fn not_applicable_generating_already_documented_documentation() {
        check_assist_not_applicable(
            generate_missing_param_docs,
            r#"
/// * `y` - Already documented
fn foo(y: i32$0) {}
"#,
        );
    }

    #[test]
    fn skip_generating_already_documented_documentation() {
        check_assist(
            generate_missing_param_docs,
            r#"
/// * `y` - Already documented
fn foo($0x: i32, y: i32) {}
"#,
            r#"
/// * `x` - TODO: Description
/// * `y` - Already documented
fn foo(x: i32, y: i32) {}
"#,
        );

    }

    #[test]
    fn mixed_documentation_case() {
    check_assist(
        generate_missing_param_docs,
        r#"
/// * `y` - Already documented
fn foo($0x: i32, y: i32, z: i32) {}
"#,
        r#"
/// * `x` - TODO: Description
/// * `z` - TODO: Description
/// * `y` - Already documented
fn foo(x: i32, y: i32, z: i32) {}
"#,
    );
}

}
