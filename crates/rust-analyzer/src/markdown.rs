//! Transforms markdown from rustdoc format to a something more suitable for
//! displaying in the editor.
//!
//! This is done by:
//! - Adding `rust` to code blocks that are missing a language
//! - Removing lines that are ignored by rustdoc. (Lines starting with `#`)
//! - Rewriting `##` at the start of lines as `#`

use ide_db::rust_doc::is_rust_fence;
use markedit::{parse, pulldown_cmark::{Event, CodeBlockKind, Tag}, Matcher, rewrite, Writer};
use pulldown_cmark_to_cmark::{cmark_resume_with_options, Options as CMarkOptions};

/// Matches all events _after_ start and _before_ end. (Excluding start and end.)
struct Between<A, B>
where
    A: Matcher,
    B: Matcher
{
    start: A,
    end: B,
    between: bool,
}

impl<A, B> Between<A, B>
where
    A: Matcher,
    B: Matcher
{
    fn new(start: A, end: B) -> Self {
        Self { start, end, between: false }
    }
}

impl<A, B> Matcher for Between<A, B>
where
    A: Matcher,
    B: Matcher
{
    fn matches_event(&mut self, event: &Event<'_>) -> bool {
        if self.start.matches_event(event) {
            self.between = true;
            return false;
        } else if self.end.matches_event(event) {
            self.between = false;
        }
        self.between
    }
}

pub(crate) fn format_docs(src: &str) -> String {
    let events = parse(src);

    let rust_code_start = |ev: Event<'_>| matches!(ev, Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(header))) if is_rust_fence(&header));
    let rust_code_end = |ev: Event<'_>| matches!(ev, Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(header))) if is_rust_fence(&header));

    let rewritten = markedit::rewrite(events, |ev: Event<'_>, w: &mut Writer<'_>| {
        if rust_code_start(ev) {
            w.push(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced("rust".into()))))
        } else if rust_code_end(ev) {
            w.push(Event::End(Tag::CodeBlock(CodeBlockKind::Fenced("rust".into()))))
        } else {
            w.push(ev)
        }
    });

    let mut btw = Between::new(rust_code_start, rust_code_end);
    let rewritten = rewrite(rewritten, |ev: Event<'_>, w: &mut Writer<'_>| {
        if btw.matches(&ev) {
            let Event::Text(text) = ev else {
                w.push(ev);
                return;
            };

            let mut output = String::with_capacity(text.len());
            for line in text.lines() {
                if code_line_ignored_by_rustdoc(line) {
                    continue;
                }
                output.push_str(line);
            }

            let trimmed = text.trim_start();

            if trimmed.starts_with("##") {
                output.push_str(&trimmed[1..]);
            } else {
                output.push_str(&text);
            }
            w.push(Event::Text(output.into()))
        } else {
            w.push(ev)
        }
    });

    let mut out = String::new();
    cmark_resume_with_options(
        rewritten,
        &mut out,
        None,
        CMarkOptions { code_block_token_count: 3, ..Default::default() },
    )
    .ok();

    out
}

fn code_line_ignored_by_rustdoc(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == "#" || trimmed.starts_with("# ") || trimmed.starts_with("#\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_docs_adds_rust() {
        let comment = "```\nfn some_rust() {}\n```";
        assert_eq!(format_docs(comment), "```rust\nfn some_rust() {}\n```");
    }

    #[test]
    fn test_format_docs_handles_plain_text() {
        let comment = "```text\nthis is plain text\n```";
        assert_eq!(format_docs(comment), "```text\nthis is plain text\n```");
    }

    #[test]
    fn test_format_docs_handles_non_rust() {
        let comment = "```sh\nsupposedly shell code\n```";
        assert_eq!(format_docs(comment), "```sh\nsupposedly shell code\n```");
    }

    #[test]
    fn test_format_docs_handles_rust_alias() {
        let comment = "```ignore\nlet z = 55;\n```";
        assert_eq!(format_docs(comment), "```rust\nlet z = 55;\n```");
    }

    #[test]
    fn test_format_docs_handles_complex_code_block_attrs() {
        let comment = "```rust,no_run\nlet z = 55;\n```";
        assert_eq!(format_docs(comment), "```rust\nlet z = 55;\n```");
    }

    #[test]
    fn test_format_docs_handles_error_codes() {
        let comment = "```compile_fail,E0641\nlet b = 0 as *const _;\n```";
        assert_eq!(format_docs(comment), "```rust\nlet b = 0 as *const _;\n```");
    }

    #[test]
    fn test_format_docs_skips_comments_in_rust_block() {
        let comment =
            "```rust\n # skip1\n# skip2\n#stay1\nstay2\n#\n #\n   #    \n #\tskip3\n\t#\t\n```";
        assert_eq!(format_docs(comment), "```rust\n#stay1\nstay2\n```");
    }

    #[test]
    fn test_format_docs_does_not_skip_lines_if_plain_text() {
        let comment =
            "```text\n # stay1\n# stay2\n#stay3\nstay4\n#\n #\n   #    \n #\tstay5\n\t#\t\n```";
        assert_eq!(
            format_docs(comment),
            "```text\n # stay1\n# stay2\n#stay3\nstay4\n#\n #\n   #    \n #\tstay5\n\t#\t\n```",
        );
    }

    #[test]
    fn test_format_docs_keeps_comments_outside_of_rust_block() {
        let comment = " # stay1\n# stay2\n#stay3\nstay4\n#\n #\n   #    \n #\tstay5\n\t#\t";
        assert_eq!(format_docs(comment), comment);
    }

    #[test]
    fn test_format_docs_preserves_newlines() {
        let comment = "this\nis\nmultiline";
        assert_eq!(format_docs(comment), comment);
    }

    #[test]
    fn test_code_blocks_in_comments_marked_as_rust() {
        let comment = r#"```rust
fn main(){}
```
Some comment.
```
let a = 1;
```"#;

        assert_eq!(
            format_docs(comment),
            "```rust\nfn main(){}\n```\nSome comment.\n```rust\nlet a = 1;\n```"
        );
    }

    #[test]
    fn test_code_blocks_in_comments_marked_as_text() {
        let comment = r#"```text
filler
text
```
Some comment.
```
let a = 1;
```"#;

        assert_eq!(
            format_docs(comment),
            "```text\nfiller\ntext\n```\nSome comment.\n```rust\nlet a = 1;\n```"
        );
    }

    #[test]
    fn test_format_docs_handles_escape_double_hashes() {
        let comment = r#"```rust
let s = "foo
## bar # baz";
```"#;

        assert_eq!(format_docs(comment), "```rust\nlet s = \"foo\n# bar # baz\";\n```");
    }

/*     #[test]
    fn test_format_docs_handles_nested_code_blocks() {
        let comment = r#"  /// # Examples
        ///
        /// `````markdown
        /// ```
        /// code block
        /// ```
        ///
        /// ```rust
        /// code block
        /// ```
        /// `````"#;

        assert_eq!(format_docs(comment), "");
    } */
}
