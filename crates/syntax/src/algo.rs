//! Collection of assorted algorithms for syntax trees.

use std::{hash::BuildHasherDefault, mem::Discriminant};

use indexmap::IndexMap;
use itertools::Itertools;
use rustc_hash::FxHashMap;
use text_edit::TextEditBuilder;

use crate::{
    AstNode, Direction, NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken, TextRange,
    TextSize,
};

use tree_edit_distance::{Edit, Node, Tree};

/// Returns ancestors of the node at the offset, sorted by length. This should
/// do the right thing at an edge, e.g. when searching for expressions at `{
/// $0foo }` we will get the name reference instead of the whole block, which
/// we would get if we just did `find_token_at_offset(...).flat_map(|t|
/// t.parent().ancestors())`.
pub fn ancestors_at_offset(
    node: &SyntaxNode,
    offset: TextSize,
) -> impl Iterator<Item = SyntaxNode> {
    node.token_at_offset(offset)
        .map(|token| token.ancestors())
        .kmerge_by(|node1, node2| node1.text_range().len() < node2.text_range().len())
}

/// Finds a node of specific Ast type at offset. Note that this is slightly
/// imprecise: if the cursor is strictly between two nodes of the desired type,
/// as in
///
/// ```no_run
/// struct Foo {}|struct Bar;
/// ```
///
/// then the shorter node will be silently preferred.
pub fn find_node_at_offset<N: AstNode>(syntax: &SyntaxNode, offset: TextSize) -> Option<N> {
    ancestors_at_offset(syntax, offset).find_map(N::cast)
}

pub fn find_node_at_range<N: AstNode>(syntax: &SyntaxNode, range: TextRange) -> Option<N> {
    syntax.covering_element(range).ancestors().find_map(N::cast)
}

/// Skip to next non `trivia` token
pub fn skip_trivia_token(mut token: SyntaxToken, direction: Direction) -> Option<SyntaxToken> {
    while token.kind().is_trivia() {
        token = match direction {
            Direction::Next => token.next_token()?,
            Direction::Prev => token.prev_token()?,
        }
    }
    Some(token)
}
/// Skip to next non `whitespace` token
pub fn skip_whitespace_token(mut token: SyntaxToken, direction: Direction) -> Option<SyntaxToken> {
    while token.kind() == SyntaxKind::WHITESPACE {
        token = match direction {
            Direction::Next => token.next_token()?,
            Direction::Prev => token.prev_token()?,
        }
    }
    Some(token)
}

/// Finds the first sibling in the given direction which is not `trivia`
pub fn non_trivia_sibling(element: SyntaxElement, direction: Direction) -> Option<SyntaxElement> {
    return match element {
        NodeOrToken::Node(node) => node.siblings_with_tokens(direction).skip(1).find(not_trivia),
        NodeOrToken::Token(token) => token.siblings_with_tokens(direction).skip(1).find(not_trivia),
    };

    fn not_trivia(element: &SyntaxElement) -> bool {
        match element {
            NodeOrToken::Node(_) => true,
            NodeOrToken::Token(token) => !token.kind().is_trivia(),
        }
    }
}

pub fn least_common_ancestor(u: &SyntaxNode, v: &SyntaxNode) -> Option<SyntaxNode> {
    if u == v {
        return Some(u.clone());
    }

    let u_depth = u.ancestors().count();
    let v_depth = v.ancestors().count();
    let keep = u_depth.min(v_depth);

    let u_candidates = u.ancestors().skip(u_depth - keep);
    let v_candidates = v.ancestors().skip(v_depth - keep);
    let (res, _) = u_candidates.zip(v_candidates).find(|(x, y)| x == y)?;
    Some(res)
}

pub fn neighbor<T: AstNode>(me: &T, direction: Direction) -> Option<T> {
    me.syntax().siblings(direction).skip(1).find_map(T::cast)
}

pub fn has_errors(node: &SyntaxNode) -> bool {
    node.children().any(|it| it.kind() == SyntaxKind::ERROR)
}

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<rustc_hash::FxHasher>>;

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
        let _p = profile::span("into_text_edit");

        for (anchor, to) in &self.insertions {
            let offset = match anchor {
                TreeDiffInsertPos::After(it) => it.text_range().end(),
                TreeDiffInsertPos::AsFirstChild(it) => it.text_range().start(),
            };
            to.iter().for_each(|to| builder.insert(offset, to.to_string()));
        }
        for (from, to) in &self.replacements {
            builder.replace(from.text_range(), to.to_string());
        }
        for text_range in self.deletions.iter().map(SyntaxElement::text_range) {
            builder.delete(text_range);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.replacements.is_empty() && self.deletions.is_empty() && self.insertions.is_empty()
    }
}

/// Finds a (potentially minimal) diff, which, applied to `from`, will result in `to`.
///
/// Specifically, returns a structure that consists of a replacements, insertions and deletions
/// such that applying this map on `from` will result in `to`.
///
/// This function tries to find a fine-grained diff.
pub fn diff(from: &SyntaxNode, to: &SyntaxNode) -> TreeDiff {
    let _p = profile::span("diff");

    let mut diff = TreeDiff {
        replacements: FxHashMap::default(),
        insertions: FxIndexMap::default(),
        deletions: Vec::new(),
    };
    let (from, to) = (from.clone().into(), to.clone().into());
    let f = tree_node(&from);
    let t = tree_node(&to);
    let (edits, _) = tree_edit_distance::diff(&f, &t);
    generate_diff(
        &mut diff,
        generate_edits(&edits),
        None,
        Some(from.clone()).into_iter(),
        Some(to.clone()).into_iter(),
    );
    return diff;

    #[derive(Debug)]
    struct TreeNode(TreeNodeKind, Vec<TreeNode>);

    #[derive(Debug)]
    enum TreeNodeKind {
        Node(Discriminant<SyntaxKind>),
        Token(String),
    }

    use std::mem::discriminant;

    impl<'n> PartialEq for TreeNodeKind {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::Node(l0), Self::Node(r0)) => l0 == r0,
                (Self::Token(l0), Self::Token(r0)) => l0 == r0,
                _ => false,
            }
        }
    }
    impl<'n> Node<'n> for TreeNode {
        type Kind = &'n TreeNodeKind;
        fn kind(&'n self) -> Self::Kind {
            &self.0
        }

        type Weight = u32;
        fn weight(&'n self) -> Self::Weight {
            1
        }
    }

    impl<'t> Tree<'t> for TreeNode {
        type Children = std::slice::Iter<'t, TreeNode>;
        fn children(&'t self) -> Self::Children {
            self.1.iter()
        }
    }

    fn tree_node(elt: &SyntaxElement) -> TreeNode {
        if let Some(node) = elt.as_node() {
            TreeNode(
                TreeNodeKind::Node(discriminant(&node.kind())),
                node.children_with_tokens().map(|c| tree_node(&c)).collect::<Vec<_>>(),
            )
        } else {
            TreeNode(TreeNodeKind::Token(elt.to_string()), vec![])
        }
    }

    #[derive(Debug, Clone)]
    enum TreeEdit {
        Same,
        InsertFirst(usize),
        Insert(usize),
        Remove,
        Replace(Vec<TreeEdit>),
        RemoveInsert,
    }

    fn generate_edits(edits: &[Edit]) -> Vec<TreeEdit> {
        let ret = edits
            .iter()
            .map(|n| match n {
                Edit::Insert => TreeEdit::Insert(1),
                Edit::Remove => TreeEdit::Remove,
                Edit::Replace(ledits) => {
                    let ledits = generate_edits(ledits);
                    if ledits.is_empty() {
                        TreeEdit::Same
                    } else {
                        if ledits.iter().all(|e| match e {
                            TreeEdit::Same => true,
                            _ => false,
                        }) {
                            TreeEdit::Same
                        } else {
                            TreeEdit::Replace(ledits)
                        }
                    }
                }
            })
            .coalesce(|a, b| match (&a, &b) {
                (TreeEdit::Remove, TreeEdit::Insert(_)) => Ok(TreeEdit::RemoveInsert),
                (TreeEdit::Insert(_), TreeEdit::Remove) => Ok(TreeEdit::RemoveInsert),
                _ => Err((a, b)),
            })
            .group_by(|e| match e {
                TreeEdit::Insert(_) => true,
                _ => false,
            })
            .into_iter()
            .flat_map(|(is_insert, group)| {
                if is_insert {
                    vec![TreeEdit::Insert(group.count())]
                } else {
                    group.into_iter().collect_vec()
                }
            })
            .enumerate()
            // insert first
            .map(|(i, d)| match (i, d) {
                (0, TreeEdit::Insert(i)) => TreeEdit::InsertFirst(i),
                (_, a) => a,
            })
            .collect_vec();
        return ret;
    }

    fn generate_diff(
        diff: &mut TreeDiff,
        edits: Vec<TreeEdit>,
        left_parent: Option<SyntaxNode>,
        mut left_childs: impl Iterator<Item = SyntaxElement>,
        mut right_childs: impl Iterator<Item = SyntaxElement>,
    ) {
        let mut current_left: Option<SyntaxElement> = None;

        for edit in edits.into_iter() {
            match edit {
                TreeEdit::RemoveInsert => {
                    cov_mark::hit!(diff_replace);
                    current_left = left_childs.next();
                    diff.replacements
                        .insert(current_left.clone().unwrap(), right_childs.next().unwrap());
                }
                TreeEdit::Insert(i) => {
                    cov_mark::hit!(diff_insert);
                    let pos = TreeDiffInsertPos::After(current_left.clone().unwrap());
                    diff.insertions.insert(
                        pos,
                        (0..i).into_iter().map(|_| right_childs.next()).flatten().collect_vec(),
                    );
                }
                TreeEdit::InsertFirst(i) => {
                    cov_mark::hit!(diff_insert_first);
                    let pos = TreeDiffInsertPos::AsFirstChild(NodeOrToken::Node(
                        left_parent.clone().unwrap(),
                    ));
                    diff.insertions.insert(
                        pos,
                        (0..i).into_iter().map(|_| right_childs.next()).flatten().collect_vec(),
                    );
                }
                TreeEdit::Remove => {
                    cov_mark::hit!(diff_delete);
                    current_left = left_childs.next();
                    diff.deletions.push(current_left.clone().unwrap());
                }
                TreeEdit::Replace(edits) => {
                    current_left = left_childs.next();
                    let left_parent = current_left.clone().map(|f| f.into_node()).flatten();
                    generate_diff(
                        diff,
                        edits,
                        left_parent.clone(),
                        left_parent.clone().map(|f| f.children_with_tokens()).unwrap(),
                        right_childs
                            .next()
                            .map(|f| f.into_node())
                            .flatten()
                            .map(|f| f.children_with_tokens())
                            .unwrap(),
                    );
                }
                TreeEdit::Same => {
                    current_left = left_childs.next();
                    right_childs.next();
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};
    use itertools::Itertools;
    use parser::SyntaxKind;
    use text_edit::TextEdit;

    use crate::{AstNode, SyntaxElement};

    #[test]
    fn replace_node_token() {
        cov_mark::check!(diff_replace);
        check_diff(
            r#"use node;"#,
            r#"ident"#,
            expect![[r#"
                insertions:



                replacements:

                Line 0: Node(USE@0..9) -> ident

                deletions:


            "#]],
        );
    }

    #[test]
    fn replace_parent() {
        cov_mark::check!(diff_insert_first);
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
        cov_mark::check!(diff_insert);
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
                -> "\n"
                -> use baz;

                replacements:



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

                Line 1: After(Node(USE@1..9))
                -> "\n"
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
                -> "\n"
                -> use foo;

                replacements:



                deletions:


            "#]],
        )
    }

    #[test]
    fn first_child_insertion() {
        cov_mark::check!(diff_insert_first);
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
                -> "\n\n    "

                replacements:



                deletions:


            "#]],
        );
    }

    #[test]
    fn delete_last() {
        cov_mark::check!(diff_delete);
        check_diff(
            r#"use foo;
            use bar;"#,
            r#"use foo;"#,
            expect![[r#"
                insertions:



                replacements:



                deletions:

                Line 1: "\n            "
                Line 2: use bar;
            "#]],
        );
    }

    #[test]
    fn delete_middle() {
        cov_mark::check!(diff_delete);
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

                Line 2: "\n"
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

                Line 1: use text_edit::TextEdit;
                Line 2: "\n\n"
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

                Line 0: AsFirstChild(Node(SOURCE_FILE@0..87))
                -> "\n"
                -> use std::fmt;
                -> "\n"
                -> use std::hash::BuildHasherDefault;
                Line 2: AsFirstChild(Node(PATH@5..8))
                -> std
                -> ::

                replacements:

                Line 2: Token(IDENT@5..8 "std") -> ops
                Line 3: Token(IDENT@16..19 "fmt") -> self
                Line 3: Token(WHITESPACE@20..25 "\n    ") -> " "
                Line 4: Token(IDENT@31..49 "BuildHasherDefault") -> RangeInclusive

                deletions:

                Line 2: "\n    "
                Line 4: hash
                Line 4: ::
                Line 4: ,
                Line 4: "\n    "
                Line 5: ops::{self, RangeInclusive}
                Line 5: ,
                Line 5: "\n"
            "#]],
        )
    }

    #[test]
    fn early_return_assist() {
        cov_mark::check!(diff_insert);
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

                Line 2: After(Token(L_CURLY@11..12 "{"))
                -> "\n    "
                -> let x = match Err(92) {
                        Ok(it) => it,
                        _ => return,
                    };

                replacements:

                Line 3: Node(IF_EXPR@17..63) -> foo(x);

                deletions:


            "#]],
        )
    }

    fn check_diff(from: &str, to: &str, expected_diff: Expect) {
        let from_node = crate::SourceFile::parse(from).tree().syntax().clone();
        let to_node = crate::SourceFile::parse(to).tree().syntax().clone();
        let diff = super::diff(&from_node, &to_node);

        let line_number =
            |syn: &SyntaxElement| from[..syn.text_range().start().into()].lines().count();

        let fmt_syntax = |syn: &SyntaxElement| match syn.kind() {
            SyntaxKind::WHITESPACE => format!("{:?}", syn.to_string()),
            _ => format!("{}", syn),
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
                f(&format!("Line {}: {:?} -> {}", line_number(k), k, fmt_syntax(v)))
            });

        let deletions = diff
            .deletions
            .iter()
            .format_with("\n", |v, f| f(&format!("Line {}: {}", line_number(v), &fmt_syntax(v))));

        let actual = format!(
            "insertions:\n\n{}\n\nreplacements:\n\n{}\n\ndeletions:\n\n{}\n",
            insertions, replacements, deletions
        );
        expected_diff.assert_eq(&actual);

        let mut from = from.to_owned();
        let mut text_edit = TextEdit::builder();
        diff.into_text_edit(&mut text_edit);
        text_edit.finish().apply(&mut from);
        assert_eq!(&*from, to, "diff did not turn `from` to `to`");
    }
}
