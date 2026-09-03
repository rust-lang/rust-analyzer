use itertools::Itertools;
use syntax::{
    AstToken, SyntaxKind, TextRange,
    ast::{self, Comment, CommentKind, CommentShape, edit::IndentLevel},
};

use crate::{AssistContext, AssistId, Assists};

// Assist: line_to_block
//
// Converts comments between block and single-line form.
//
// ```
//    // Multi-line$0
//    // comment
// ```
// ->
// ```
//   /*
//   Multi-line
//   comment
//   */
// ```
pub(crate) fn convert_comment_block(acc: &mut Assists, ctx: &AssistContext<'_, '_>) -> Option<()> {
    let comment = ctx.find_token_at_offset::<ast::Comment>()?;
    // Only allow comments which are alone on their line
    let mut prev = comment.syntax().prev_token();
    while prev.as_ref().is_some_and(|it| it.kind() == syntax::SyntaxKind::WHITESPACE) {
        prev = prev.and_then(|it| it.prev_token());
    }
    if let Some(prev) = prev {
        (prev.kind() == syntax::SyntaxKind::NEWLINE).then_some(())?;
    }

    match comment.kind().shape {
        ast::CommentShape::Block => block_to_line(acc, comment),
        ast::CommentShape::Line => line_to_block(acc, comment),
    }
}

fn block_to_line(acc: &mut Assists, comment: ast::Comment) -> Option<()> {
    let target = comment.syntax().text_range();

    acc.add(
        AssistId::refactor_rewrite("block_to_line"),
        "Replace block comment with line comments",
        target,
        |edit| {
            let indentation = IndentLevel::from_token(comment.syntax());
            let line_prefix = CommentKind { shape: CommentShape::Line, ..comment.kind() }.prefix();

            let text = comment.text();
            let text = &text[comment.prefix().len()..(text.len() - "*/".len())].trim();

            let lines = text.lines().peekable();

            let indent_spaces = indentation.to_string();
            let output = lines
                .map(|line| {
                    let line = line.trim_start_matches(&indent_spaces);

                    // Don't introduce trailing whitespace
                    if line.is_empty() {
                        line_prefix.to_owned()
                    } else {
                        format!("{line_prefix} {line}")
                    }
                })
                .join(&format!("\n{indent_spaces}"));

            edit.replace(target, output)
        },
    )
}

fn line_to_block(acc: &mut Assists, comment: ast::Comment) -> Option<()> {
    // Find all the comments we'll be collapsing into a block
    let comments = relevant_line_comments(&comment);

    // Establish the target of our edit based on the comments we found
    let target = TextRange::new(
        comments[0].syntax().text_range().start(),
        comments.last()?.syntax().text_range().end(),
    );

    acc.add(
        AssistId::refactor_rewrite("line_to_block"),
        "Replace line comments with a single block comment",
        target,
        |edit| {
            // We pick a single indentation level for the whole block comment based on the
            // comment where the assist was invoked. This will be prepended to the
            // contents of each line comment when they're put into the block comment.
            let indentation = IndentLevel::from_token(comment.syntax());

            let block_comment_body = comments
                .into_iter()
                .map(|c| line_comment_text(indentation, c))
                .collect::<Vec<String>>()
                .into_iter()
                .join("\n");

            let block_prefix =
                CommentKind { shape: CommentShape::Block, ..comment.kind() }.prefix();

            let output = format!("{block_prefix}\n{block_comment_body}\n{indentation}*/");

            edit.replace(target, output)
        },
    )
}

/// The line -> block assist can  be invoked from anywhere within a sequence of line comments.
/// relevant_line_comments crawls backwards and forwards finding the complete sequence of comments that will
/// be joined.
pub(crate) fn relevant_line_comments(comment: &ast::Comment) -> Vec<Comment> {
    // The prefix identifies the kind of comment we're dealing with
    let prefix = comment.prefix();

    let run = |forward: bool| {
        let mut res = Vec::new();
        let mut token = comment.syntax().clone();
        let mut newlines = 0;
        loop {
            token = match if forward { token.next_token() } else { token.prev_token() } {
                Some(it) => it,
                None => break,
            };
            if matches!(token.kind(), SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE) {
                newlines += token.text().matches('\n').count();
                if newlines > 1 {
                    break;
                }
                continue;
            }
            match Comment::cast(token.clone()).filter(|c| c.prefix() == prefix) {
                Some(c) => {
                    res.push(c);
                    newlines = 0;
                }
                None => break,
            }
        }
        res
    };

    let mut comments = run(false);
    comments.reverse();
    comments.push(comment.clone());
    comments.extend(run(true));
    comments
}

// Line comments usually begin with a single space character following the prefix as seen here:
//^
// But comments can also include indented text:
//    > Hello there
//
// We handle this by stripping *AT MOST* one space character from the start of the line
// This has its own problems because it can cause alignment issues:
//
//              /*
// a      ----> a
//b       ----> b
//              */
//
// But since such comments aren't idiomatic we're okay with this.
pub(crate) fn line_comment_text(indentation: IndentLevel, comm: ast::Comment) -> String {
    let text = comm.text();
    let contents_without_prefix = text.strip_prefix(comm.prefix()).unwrap_or(text);
    let contents = contents_without_prefix.strip_prefix(' ').unwrap_or(contents_without_prefix);

    // Don't add the indentation if the line is empty
    if contents.is_empty() { contents.to_owned() } else { indentation.to_string() + contents }
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn single_line_to_block() {
        check_assist(
            convert_comment_block,
            r#"
// line$0 comment
fn main() {
    foo();
}
"#,
            r#"
/*
line comment
*/
fn main() {
    foo();
}
"#,
        );
    }

    #[test]
    fn single_line_to_block_indented() {
        check_assist(
            convert_comment_block,
            r#"
fn main() {
    // line$0 comment
    foo();
}
"#,
            r#"
fn main() {
    /*
    line comment
    */
    foo();
}
"#,
        );
    }

    #[test]
    fn multiline_to_block() {
        check_assist(
            convert_comment_block,
            r#"
fn main() {
    // above
    // line$0 comment
    //
    // below
    foo();
}
"#,
            r#"
fn main() {
    /*
    above
    line comment

    below
    */
    foo();
}
"#,
        );
    }

    #[test]
    fn end_of_line_to_block() {
        check_assist_not_applicable(
            convert_comment_block,
            r#"
fn main() {
    foo(); // end-of-line$0 comment
}
"#,
        );
    }

    #[test]
    fn single_line_different_kinds() {
        check_assist(
            convert_comment_block,
            r#"
fn main() {
    /// different prefix
    // line$0 comment
    // below
    foo();
}
"#,
            r#"
fn main() {
    /// different prefix
    /*
    line comment
    below
    */
    foo();
}
"#,
        );
    }

    #[test]
    fn single_line_separate_chunks() {
        check_assist(
            convert_comment_block,
            r#"
fn main() {
    // different chunk

    // line$0 comment
    // below
    foo();
}
"#,
            r#"
fn main() {
    // different chunk

    /*
    line comment
    below
    */
    foo();
}
"#,
        );
    }

    #[test]
    fn doc_block_comment_to_lines() {
        check_assist(
            convert_comment_block,
            r#"
/**
 hi$0 there
*/
"#,
            r#"
/// hi there
"#,
        );
    }

    #[test]
    fn block_comment_to_lines() {
        check_assist(
            convert_comment_block,
            r#"
/*
 hi$0 there
*/
"#,
            r#"
// hi there
"#,
        );
    }

    #[test]
    fn inner_doc_block_to_lines() {
        check_assist(
            convert_comment_block,
            r#"
/*!
 hi$0 there
*/
"#,
            r#"
//! hi there
"#,
        );
    }

    #[test]
    fn block_to_lines_indent() {
        check_assist(
            convert_comment_block,
            r#"
fn main() {
    /*!
    hi$0 there

    ```
      code_sample
    ```
    */
}
"#,
            r#"
fn main() {
    //! hi there
    //!
    //! ```
    //!   code_sample
    //! ```
}
"#,
        );
    }

    #[test]
    fn empty_block_to_line() {
        check_assist(
            convert_comment_block,
            r#"
/**/$0
fn main() {}
"#,
            r#"

fn main() {}
"#,
        );
    }

    #[test]
    fn end_of_line_block_to_line() {
        check_assist_not_applicable(
            convert_comment_block,
            r#"
fn main() {
    foo(); /* end-of-line$0 comment */
}
"#,
        );
    }
}
