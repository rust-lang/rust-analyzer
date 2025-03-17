use crate::assist_context::{AssistContext, Assists};
// use hir::sym::consts::doc;
use ide_db::assists::{AssistId, AssistKind};
use syntax::{
    ast::{self, edit::IndentLevel, HasDocComments},
    AstNode, AstToken, TextRange, TextSize,
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
    let indent_level = IndentLevel::from_node(fn_node.syntax());

    // Get the parameter list from the function.
    let param_list = fn_node.param_list()?;
    let params: Vec<String> =
        param_list.params().filter_map(|param| param.pat().map(|p| p.to_string())).collect();

    if params.is_empty() {
        return None;
    }

    // Retrieve existing doc comment lines (if any)
    let existing_docs: Vec<String> =
        fn_node.doc_comments().map(|doc| doc.text().to_string()).collect();

    let existing_docs_len = {
        let size = TextSize::from(0);
        let size = existing_docs.iter().fold(size, |acc, doc| acc + TextSize::of(&*doc));
        size
    };

    // Build documentation lines for parameters not yet documented.
    let mut new_doc_lines = Vec::new();
    let has_param_docs =
        existing_docs.iter().enumerate().find(|(_, line)| line.contains("## Parameters"));

    if existing_docs.is_empty() {
        println!("1");

        // We need to add everything from scratch.
        new_doc_lines.push(format_doc_line(" ## Parameters", indent_level));
        new_doc_lines.push(format_doc_line("", indent_level));
        for param in params {
            let new_doc = format!(" * `{}` - TODO: Description", param);
            let formatted = format_doc_line(&new_doc, indent_level);
            new_doc_lines.push(formatted);
        }
        let text_range = fn_node.syntax().text_range();
        let offset = text_range.start();

        let mut doc_heading = new_doc_lines.join("\n");
        doc_heading.push('\n');

        acc.add(
            AssistId("generate_missing_param_docs", AssistKind::Generate),
            "Generate missing parameter documentation",
            text_range,
            |builder| {
                builder.insert(offset, doc_heading);
            },
        );
        return Some(());
    }

    if has_param_docs.is_none() {
        println!("2");

        // We need to add a new parameter documentation section at the end of the existing docs.
        new_doc_lines.push("\n".to_string());
        new_doc_lines.push(format_doc_line(" ## Parameters", indent_level));
        new_doc_lines.push(format_doc_line("", indent_level));
        for param in params {
            let new_doc = format!(" * `{}` - TODO: Description", param);
            let formatted = format_doc_line(&new_doc, indent_level);
            new_doc_lines.push(formatted);
        }

        let text_range = fn_node.syntax().text_range();
        let offset = text_range.start() + existing_docs_len;

        let mut doc_heading = new_doc_lines.join("\n");
        doc_heading.push('\n');

        acc.add(
            AssistId("generate_missing_param_docs", AssistKind::Generate),
            "Generate missing parameter documentation",
            text_range,
            |builder| {
                builder.insert(offset, doc_heading);
            },
        );
        return Some(());
    }

    println!("3");

    // Check if the function already has a parameter documentation section.

    let Some((i, _)) = has_param_docs else {
        return None;
    };

    let param_docs_end =
        existing_docs.iter().enumerate().find(|(j, _)| *j > i && existing_docs[*j].contains(" ##"));
    let param_docs_end = match param_docs_end {
        Some((j, _)) => {
            println!("Found end : {j}");
            j
        }
        None => {
            println!("Defaulting to len : {}", existing_docs.len());
            existing_docs.len()
        }
    };

    let mut start = fn_node.syntax().text_range().start() + offset_at_index(&existing_docs, i);
    let end = fn_node.syntax().text_range().start()
        + offset_at_index(&existing_docs, param_docs_end)
        + TextSize::from((param_docs_end - i) as u32);

    if start > TextSize::from(0) {
        start = start - TextSize::from(1);
    }

    let param_docs_range = TextRange::new(start, end - TextSize::from(1));

    let mut doc_heading = existing_docs[i..param_docs_end].to_vec();

    let mut found_existing_params = Vec::new();

    for param in params.iter() {
        if let Some((index, _)) = doc_heading
            .iter()
            .enumerate()
            .find(|(_, line)| line.contains(&format!("* `{}` -", param)))
        {
            found_existing_params.push((index, param));
        }
    }

    if found_existing_params.len() == params.len() {
        return None;
    }

    if found_existing_params.len() > 2 {
        // check if all params are documented sequentially
        let mut param_iterator = found_existing_params.iter();
        let mut index = param_iterator.next().unwrap().0;
        for (i, _) in param_iterator {
            if *i != index + 1 {
                // formatting is hard if not all params are documented sequentially and on one line
                return None;
            }
            index = *i;
        }
    }

    println!("doc heading : {:?}", doc_heading);
    ordered_insertion(&mut doc_heading, &params, indent_level);
    if param_docs_end != existing_docs.len() {
        doc_heading.pop();
        doc_heading.push("".to_string());
    }
    let mut doc_heading = doc_heading.join("\n");

    if start > TextSize::from(0) {
        doc_heading = format!("\n{}", format_doc_line("\n", indent_level)) + &doc_heading;
    }

    acc.add(
        AssistId("generate_missing_param_docs", AssistKind::Generate),
        "Generate missing parameter documentation",
        param_docs_range,
        |builder| {
            builder.replace(param_docs_range, doc_heading);
        },
    );

    return Some(());

    // Helper function to build documentation lines.
    fn format_doc_line(line: &str, indent_level: IndentLevel) -> String {
        format!("{}///{}", indent_level, line)
    }

    fn ordered_insertion(
        doc_heading: &mut Vec<String>,
        params: &[String],
        indent_level: IndentLevel,
    ) {
        let mut params_iter = params.iter();
        let mut last_index = None;

        while let Some(param) = params_iter.next() {
            if let Some((index, _)) = doc_heading
                .iter()
                .enumerate()
                .find(|(_, line)| line.contains(&format!("* `{}` -", param)))
            {
                last_index = Some(index);
                continue;
            }
            println!("inserting param: {}", param);
            let new_doc = format!(" * `{}` - TODO: Description", param);
            let formatted = format_doc_line(&new_doc, indent_level);

            if let Some(index) = last_index {
                doc_heading.insert(index + 1, formatted.clone());
                last_index = Some(index + 1);
            } else {
                doc_heading.insert(2, formatted);
                last_index = Some(2);
            }
        }
    }

    fn offset_at_index(doc_heading: &[String], index: usize) -> TextSize {
        let mut offset = TextSize::from(0);
        for i in 0..index {
            offset += TextSize::of(&doc_heading[i]);
        }
        offset
    }
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
/// ## Parameters
///
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
/// ## Parameters
///
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
/// ## Parameters
///
/// * `y` - Already documented
fn foo($0x: i32, y: i32) {}
"#,
            r#"
/// ## Parameters
///
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
/// ## Parameters
///
/// * `y` - Already documented
fn foo($0x: i32, y: i32, z: i32) {}
"#,
            r#"
/// ## Parameters
///
/// * `x` - TODO: Description
/// * `y` - Already documented
/// * `z` - TODO: Description
fn foo(x: i32, y: i32, z: i32) {}
"#,
        );
    }

    #[test]
    fn super_mixed_documentation_case() {
        check_assist(
            generate_missing_param_docs,
            r#"
/// ## My special section
/// 
/// Super awesome documentation
///
/// ## Parameters
///
/// * `y` - Already documented
/// 
/// ## Errors
/// 
/// * Errors are documented here
fn foo($0x: i32, y: i32, z: i32) {}
"#,
            r#"
/// ## My special section
/// 
/// Super awesome documentation
///
/// ## Parameters
///
/// * `x` - TODO: Description
/// * `y` - Already documented
/// * `z` - TODO: Description
/// 
/// ## Errors
/// 
/// * Errors are documented here
fn foo(x: i32, y: i32, z: i32) {}
"#,
        );
    }
}
