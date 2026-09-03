//! Basic tree diffing functionality.
use rustc_hash::FxHashMap;
use syntax::{NodeOrToken, SyntaxElement, SyntaxNode};

use crate::{FxIndexMap, text_edit::TextEditBuilder};

#[derive(Debug, Hash, PartialEq, Eq)]
enum TreeDiffInsertPos {
    After(SyntaxElement),
    AsFirstChild(SyntaxElement),
}

#[derive(Debug)]
pub struct TreeDiff {
    replacements: FxHashMap<SyntaxElement, SyntaxElement>,
    deletions: Vec<SyntaxElement>,
    // the vec as well as the indexmap are both here to preserve order
    insertions: FxIndexMap<TreeDiffInsertPos, Vec<SyntaxElement>>,
}

impl TreeDiff {
    pub fn into_text_edit(&self, builder: &mut TextEditBuilder) {
        let _p = tracing::info_span!("into_text_edit").entered();

        for (anchor, to) in &self.insertions {
            let offset = match anchor {
                TreeDiffInsertPos::After(it) => it.text_range().end(),
                TreeDiffInsertPos::AsFirstChild(it) => it.text_range().start(),
            };
            to.iter().for_each(|to| builder.insert(offset, full_text(to)));
        }
        for (from, to) in &self.replacements {
            builder.replace(from.text_range(), full_text(to));
        }
        for text_range in self.deletions.iter().map(SyntaxElement::text_range) {
            builder.delete(text_range);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.replacements.is_empty() && self.deletions.is_empty() && self.insertions.is_empty()
    }
}

fn full_text(element: &SyntaxElement) -> String {
    match element {
        NodeOrToken::Node(it) => it.text().to_string(),
        NodeOrToken::Token(it) => it.text_including_trivia(),
    }
}

/// Finds a (potentially minimal) diff, which, applied to `from`, will result in `to`.
///
/// Specifically, returns a structure that consists of a replacements, insertions and deletions
/// such that applying this map on `from` will result in `to`.
///
/// This function tries to find a fine-grained diff.
pub fn diff(from: &SyntaxNode, to: &SyntaxNode) -> TreeDiff {
    let _p = tracing::info_span!("diff").entered();

    let mut diff = TreeDiff {
        replacements: FxHashMap::default(),
        insertions: FxIndexMap::default(),
        deletions: Vec::new(),
    };
    let (from, to) = (from.clone().into(), to.clone().into());

    if !syntax_element_eq(&from, &to) {
        go(&mut diff, from, to);
    }
    return diff;

    fn syntax_element_trimmed_eq(lhs: &SyntaxElement, rhs: &SyntaxElement) -> bool {
        let text = |element: &SyntaxElement| match element {
            NodeOrToken::Node(it) => syntax::token_text(it),
            NodeOrToken::Token(it) => it.text().to_owned(),
        };
        lhs.kind() == rhs.kind() && text(lhs) == text(rhs)
    }

    fn syntax_element_eq(lhs: &SyntaxElement, rhs: &SyntaxElement) -> bool {
        lhs.kind() == rhs.kind()
            && lhs.text_range().len() == rhs.text_range().len()
            && match (&lhs, &rhs) {
                (NodeOrToken::Node(lhs), NodeOrToken::Node(rhs)) => {
                    lhs == rhs || lhs.text() == rhs.text()
                }
                (NodeOrToken::Token(lhs), NodeOrToken::Token(rhs)) => {
                    lhs.text_including_trivia() == rhs.text_including_trivia()
                }
                _ => false,
            }
    }

    // FIXME: this is horribly inefficient. I bet there's a cool algorithm to diff trees properly.
    fn go(diff: &mut TreeDiff, lhs: SyntaxElement, rhs: SyntaxElement) {
        let (lhs, rhs) = match lhs.as_node().zip(rhs.as_node()) {
            Some((lhs, rhs)) => (lhs, rhs),
            _ => {
                cov_mark::hit!(diff_node_token_replace);
                diff.replacements.insert(lhs, rhs);
                return;
            }
        };

        let mut look_ahead_scratch = Vec::default();

        let mut rhs_children = rhs.children_with_tokens();
        let mut lhs_children = lhs.children_with_tokens();
        let mut last_lhs = None;
        let mut consumed_last: Option<SyntaxElement> = None;
        loop {
            let lhs_child = lhs_children.next();
            match (lhs_child.clone(), rhs_children.next()) {
                (None, None) => break,
                (None, Some(element)) => {
                    let insert_pos = match last_lhs.clone() {
                        Some(prev) => {
                            cov_mark::hit!(diff_insert);
                            TreeDiffInsertPos::After(prev)
                        }
                        // first iteration, insert into out parent as the first child
                        None => {
                            cov_mark::hit!(diff_insert_as_first_child);
                            TreeDiffInsertPos::AsFirstChild(lhs.clone().into())
                        }
                    };
                    diff.insertions.entry(insert_pos).or_default().push(element);
                }
                (Some(element), None) => {
                    cov_mark::hit!(diff_delete);
                    diff.deletions.push(element);
                }
                (Some(ref lhs_ele), Some(ref rhs_ele)) if syntax_element_eq(lhs_ele, rhs_ele) => {}
                (Some(lhs_ele), Some(rhs_ele)) => {
                    // nodes differ, look for lhs_ele in rhs, if it's found we can mark everything up
                    // until that element as insertions. This is important to keep the diff minimal
                    // in regards to insertions that have been actually done, this is important for
                    // use insertions as we do not want to replace the entire module node.
                    look_ahead_scratch.push(rhs_ele.clone());
                    let mut rhs_children_clone = rhs_children.clone();
                    let mut matched = None;
                    for rhs_child in &mut rhs_children_clone {
                        if syntax_element_trimmed_eq(&lhs_ele, &rhs_child) {
                            cov_mark::hit!(diff_insertions);
                            matched = Some(rhs_child);
                            break;
                        }
                        look_ahead_scratch.push(rhs_child);
                    }
                    let drain = std::mem::take(&mut look_ahead_scratch);
                    if let Some(matched) = matched {
                        let insert_pos = if let Some(prev) = last_lhs.clone() {
                            TreeDiffInsertPos::After(prev)
                        } else {
                            cov_mark::hit!(insert_first_child);
                            TreeDiffInsertPos::AsFirstChild(lhs.clone().into())
                        };

                        diff.insertions.entry(insert_pos).or_default().extend(drain);
                        rhs_children = rhs_children_clone;

                        if !syntax_element_eq(&lhs_ele, &matched) {
                            if lhs_ele.kind() != matched.kind() {
                                diff.replacements.insert(lhs_ele, matched);
                            } else {
                                go(diff, lhs_ele, matched);
                            }
                        }
                    } else {
                        let mut lhs_children_clone = lhs_children.clone();
                        let mut deleted = vec![lhs_ele.clone()];
                        let mut survivor = None;
                        for lhs_child in &mut lhs_children_clone {
                            if syntax_element_trimmed_eq(&lhs_child, &rhs_ele) {
                                survivor = Some(lhs_child);
                                break;
                            }
                            deleted.push(lhs_child);
                        }
                        match survivor {
                            Some(survivor) => {
                                cov_mark::hit!(diff_deletions);
                                diff.deletions.extend(deleted);
                                lhs_children = lhs_children_clone;
                                consumed_last = Some(survivor.clone());
                                if !syntax_element_eq(&survivor, &rhs_ele) {
                                    if survivor.kind() != rhs_ele.kind() {
                                        diff.replacements.insert(survivor, rhs_ele);
                                    } else {
                                        go(diff, survivor, rhs_ele);
                                    }
                                }
                            }
                            None if lhs_ele.kind() != rhs_ele.kind() => {
                                cov_mark::hit!(diff_node_token_replace);
                                diff.replacements.insert(lhs_ele, rhs_ele);
                            }
                            None => go(diff, lhs_ele, rhs_ele),
                        }
                    }
                }
            }
            last_lhs = consumed_last.take().or(lhs_child).or(last_lhs);
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::{Expect, expect};
    use itertools::Itertools;
    use parser::{Edition, SyntaxKind};
    use syntax::{AstNode, SourceFile, SyntaxElement};

    use crate::text_edit::TextEdit;

    #[test]
    fn replace_node_token() {
        cov_mark::check!(diff_node_token_replace);
        check_diff(
            r#"use node;"#,
            r#"ident"#,
            expect![[r#"
                insertions:



                replacements:

                Line 1: Token(EOF@9..9 "" [] []) -> ident

                deletions:

                Line 0: use node;
            "#]],
        );
    }

    #[test]
    fn replace_parent() {
        cov_mark::check!(insert_first_child);
        check_diff(
            r#""#,
            r#"use foo::bar;"#,
            expect![[r#"
                insertions:

                Line 0: AsFirstChild(Node(SOURCE_FILE@0..0))
                -> use foo::bar;

                replacements:



                deletions:


            "#]],
        );
    }

    #[test]
    fn insert_last() {
        cov_mark::check!(diff_insertions);
        check_diff(
            r#"
use foo;
use bar;"#,
            r#"
use foo;
use bar;
use baz;"#,
            expect![[r#"
                insertions:

                Line 2: After(Node(USE@10..18))
                -> use baz;

                replacements:

                Line 3: Token(SEMICOLON@17..18 ";" [] []) -> ;


                deletions:


            "#]],
        );
    }

    #[test]
    fn insert_middle() {
        check_diff(
            r#"
use foo;
use baz;"#,
            r#"
use foo;
use bar;
use baz;"#,
            expect![[r#"
                insertions:

                Line 0: After(Node(USE@0..10))
                -> use bar;

                replacements:



                deletions:


            "#]],
        )
    }

    #[test]
    fn insert_first() {
        check_diff(
            r#"
use bar;
use baz;"#,
            r#"
use foo;
use bar;
use baz;"#,
            expect![[r#"
                insertions:

                Line 0: AsFirstChild(Node(SOURCE_FILE@0..18))
                -> use foo;

                replacements:

                Line 0: Token(USE_KW@1..4 "use" [NEWLINE("\n")] [WHITESPACE(" ")]) -> use

                deletions:


            "#]],
        )
    }

    #[test]
    fn first_child_insertion() {
        cov_mark::check!(insert_first_child);
        check_diff(
            r#"fn main() {
        stdi
    }"#,
            r#"use foo::bar;

    fn main() {
        stdi
    }"#,
            expect![[r#"
                insertions:

                Line 0: AsFirstChild(Node(SOURCE_FILE@0..30))
                -> use foo::bar;

                replacements:

                Line 0: Token(FN_KW@0..2 "fn" [] [WHITESPACE(" ")]) ->
                    fn

                deletions:


            "#]],
        );
    }

    #[test]
    fn delete_last() {
        cov_mark::check!(diff_deletions);
        check_diff(
            r#"use foo;
            use bar;"#,
            r#"use foo;"#,
            expect![[r#"
                insertions:



                replacements:

                Line 1: Token(SEMICOLON@7..8 ";" [] [NEWLINE("\n")]) -> ;

                deletions:

                Line 1: use bar;
            "#]],
        );
    }

    #[test]
    fn delete_middle() {
        cov_mark::check!(diff_deletions);
        check_diff(
            r#"
use expect_test::{expect, Expect};
use text_edit::TextEdit;

use crate::AstNode;
"#,
            r#"
use expect_test::{expect, Expect};

use crate::AstNode;
"#,
            expect![[r#"
                insertions:



                replacements:



                deletions:

                Line 2: use text_edit::TextEdit;
            "#]],
        )
    }

    #[test]
    fn delete_first() {
        check_diff(
            r#"
use text_edit::TextEdit;

use crate::AstNode;
"#,
            r#"
use crate::AstNode;
"#,
            expect![[r#"
                insertions:



                replacements:



                deletions:

                Line 0: use text_edit::TextEdit;
            "#]],
        )
    }

    #[test]
    fn merge_use() {
        check_diff(
            r#"
use std::{
    fmt,
    hash::BuildHasherDefault,
    ops::{self, RangeInclusive},
};
"#,
            r#"
use std::fmt;
use std::hash::BuildHasherDefault;
use std::ops::{self, RangeInclusive};
"#,
            expect![[r#"
                insertions:

                Line 2: After(Node(PATH_SEGMENT@5..8))
                -> ::
                -> fmt
                Line 0: After(Node(USE@0..87))
                -> use std::hash::BuildHasherDefault;
                -> use std::ops::{self, RangeInclusive};

                replacements:

                Line 2: Node(PATH_SEGMENT@5..8) -> std

                deletions:

                Line 2: ::
                Line 2: {
                    fmt,
                    hash::BuildHasherDefault,
                    ops::{self, RangeInclusive},
                }
            "#]],
        )
    }

    #[test]
    fn early_return_assist() {
        check_diff(
            r#"
fn main() {
    if let Ok(x) = Err(92) {
        foo(x);
    }
}
            "#,
            r#"
fn main() {
    let x = match Err(92) {
        Ok(it) => it,
        _ => return,
    };
    foo(x);
}
            "#,
            expect![[r#"
                insertions:

                Line 2: After(Node(IF_EXPR@13..64))
                -> foo(x);

                replacements:

                Line 2: Node(IF_EXPR@13..64) -> let x = match Err(92) {
                        Ok(it) => it,
                        _ => return,
                    };

                deletions:


            "#]],
        )
    }

    fn check_diff(from: &str, to: &str, expected_diff: Expect) {
        let from_node = SourceFile::parse(from, Edition::CURRENT).tree().syntax().clone();
        let to_node = SourceFile::parse(to, Edition::CURRENT).tree().syntax().clone();
        let diff = super::diff(&from_node, &to_node);

        let line_number =
            |syn: &SyntaxElement| from[..syn.text_range().start().into()].lines().count();

        let fmt_syntax = |syn: &SyntaxElement| match syn.kind() {
            SyntaxKind::WHITESPACE => format!("{:?}", syn.to_string()),
            _ => format!("{syn}"),
        };

        let insertions =
            diff.insertions.iter().format_with("\n", |(k, v), f| -> Result<(), std::fmt::Error> {
                f(&format!(
                    "Line {}: {:?}\n-> {}",
                    line_number(match k {
                        super::TreeDiffInsertPos::After(syn) => syn,
                        super::TreeDiffInsertPos::AsFirstChild(syn) => syn,
                    }),
                    k,
                    v.iter().format_with("\n-> ", |v, f| f(&fmt_syntax(v)))
                ))
            });

        let replacements = diff
            .replacements
            .iter()
            .sorted_by_key(|(syntax, _)| syntax.text_range().start())
            .format_with("\n", |(k, v), f| {
                f(&format!("Line {}: {k:?} -> {}", line_number(k), fmt_syntax(v)))
            });

        let deletions = diff
            .deletions
            .iter()
            .format_with("\n", |v, f| f(&format!("Line {}: {}", line_number(v), fmt_syntax(v))));

        let actual = format!(
            "insertions:\n\n{insertions}\n\nreplacements:\n\n{replacements}\n\ndeletions:\n\n{deletions}\n"
        );
        expected_diff.assert_eq(&actual);

        let mut from = from.to_owned();
        let mut text_edit = TextEdit::builder();
        diff.into_text_edit(&mut text_edit);
        text_edit.finish().apply(&mut from);
        assert_eq!(&*from, to, "diff did not turn `from` to `to`");
    }
}
