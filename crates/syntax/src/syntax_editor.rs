//! Syntax Tree editor
//!
//! Inspired by Roslyn's [`SyntaxEditor`].
//!
//! [`SyntaxEditor`]: https://github.com/dotnet/roslyn/blob/43b0b05cc4f492fd5de00f6f6717409091df8daa/src/Workspaces/Core/Portable/Editing/SyntaxEditor.cs

use std::{
    cell::RefCell,
    fmt, iter,
    num::NonZeroU32,
    ops::{RangeBounds, RangeInclusive},
    sync::atomic::{AtomicU32, Ordering},
};

use rowan::{GreenToken, SyntaxKind as RSyntaxKind, TextRange};
use rustc_hash::FxHashMap;

use crate::{
    AstNode, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken, T,
    ast::{make::tokens, syntax_factory::SyntaxFactory},
    syntax_node::token_payload,
};

mod edit_algo;
mod edits;
mod mapping;
mod normalize;

pub use edits::{GetOrCreateWhereClause, Removable};
pub use mapping::{SyntaxMapping, SyntaxMappingBuilder};

#[derive(Debug)]
pub struct SyntaxEditor {
    root: SyntaxNode,
    changes: RefCell<Vec<Change>>,
    annotations: RefCell<Vec<(SyntaxElement, SyntaxAnnotation)>>,
    make: SyntaxFactory,
}

impl SyntaxEditor {
    /// Creates a syntax editor from `root`.
    ///
    /// The returned `root` is guaranteed to be a detached, immutable node.
    /// If the provided node is not a root (i.e., has a parent), it is cloned
    /// into a fresh subtree to satisfy syntax editor invariants.
    pub fn new(root: SyntaxNode) -> (Self, SyntaxNode) {
        let mut root = root;

        if root.parent().is_some() {
            root = root.clone_subtree()
        };

        let editor = Self {
            root: root.clone(),
            changes: RefCell::new(Vec::new()),
            annotations: RefCell::new(Vec::new()),
            make: SyntaxFactory::with_mappings(),
        };

        (editor, root)
    }

    /// Typed-node variant of [`SyntaxEditor::new`].
    pub fn with_ast_node<T>(root: &T) -> (Self, T)
    where
        T: AstNode,
    {
        let (editor, root) = Self::new(root.syntax().clone());

        (editor, T::cast(root).unwrap())
    }

    pub fn make(&self) -> &SyntaxFactory {
        &self.make
    }

    pub fn add_annotation(&self, element: impl Element, annotation: SyntaxAnnotation) {
        let element = element.syntax_element();
        debug_assert!(!element.is_trivia(), "trivia cannot carry an annotation");
        self.annotations.borrow_mut().push((element, annotation))
    }

    pub fn add_annotation_all(&self, elements: Vec<impl Element>, annotation: SyntaxAnnotation) {
        for element in elements {
            self.add_annotation(element, annotation);
        }
    }

    pub fn merge(&self, other: SyntaxEditor) {
        debug_assert!(
            self.root == other.root || other.root.ancestors().any(|node| node == self.root),
            "{:?} is not in the same tree as {:?}",
            other.root,
            self.root
        );

        self.changes.borrow_mut().append(&mut other.changes.into_inner());
        if let Some(mut m) = self.make.mappings() {
            m.merge(other.make.take());
        }
        self.annotations.borrow_mut().append(&mut other.annotations.into_inner());
    }

    pub fn insert(&self, position: Position, element: impl Element) {
        debug_assert!(is_ancestor_or_self(&position.parent(), &self.root));
        self.changes.borrow_mut().push(Change::Insert(position, element.syntax_element()))
    }

    pub fn insert_all(&self, position: Position, elements: Vec<SyntaxElement>) {
        debug_assert!(is_ancestor_or_self(&position.parent(), &self.root));
        self.changes.borrow_mut().push(Change::InsertAll(position, elements))
    }

    fn next_surviving_token(&self, token: &SyntaxToken) -> Option<SyntaxToken> {
        let changes = self.changes.borrow();
        let covering = |it: &SyntaxToken| {
            changes.iter().find(|change| {
                !matches!(
                    change,
                    Change::Insert(..)
                        | Change::InsertAll(..)
                        | Change::Replace(SyntaxElement::Token(_), Some(_))
                ) && change.target_range().contains_range(it.text_range())
            })
        };
        for it in iter::successors(token.next_non_trivia_token(), |it| it.next_non_trivia_token()) {
            match covering(&it) {
                None => return Some(it),
                Some(Change::Replace(_, None)) => (),
                Some(Change::ReplaceWithMany(_, new) | Change::ReplaceAll(_, new))
                    if new.is_empty() => {}
                Some(_) => return None,
            }
        }
        None
    }

    pub fn delete(&self, element: impl Element) {
        let element = element.syntax_element();
        debug_assert!(!element.is_trivia(), "trivia is not a structural delete target");
        debug_assert!(is_ancestor_or_self_of_element(&element, &self.root));
        debug_assert!(
            !matches!(&element, SyntaxElement::Node(node) if node == &self.root),
            "should not delete root node"
        );
        if let SyntaxElement::Node(node) = &element
            && !self.deleted(node)
            && let Some(separator) = self.separator_of(node)
        {
            self.delete(separator);
        }
        self.queue_replace(element, None);
    }

    fn separator_of(&self, node: &SyntaxNode) -> Option<SyntaxToken> {
        let separator = |it: Option<SyntaxElement>| {
            it.and_then(|it| it.into_token()).filter(|it| it.kind() == T![,] && !self.deleted(it))
        };
        let ends_line =
            |it: &SyntaxToken| it.trailing_trivia().any(|it| it.kind() == SyntaxKind::NEWLINE);
        let prev = separator(node.prev_sibling_or_token());
        let next = separator(node.next_sibling_or_token());
        match next {
            Some(next)
                if next.leading_trivia().len() == 0
                    && !node.last_non_trivia_token().is_some_and(|it| ends_line(&it))
                    && ends_line(&next) =>
            {
                Some(next)
            }
            next => prev.or(next),
        }
    }

    pub fn delete_all(&self, range: RangeInclusive<SyntaxElement>) {
        debug_assert!(
            !range.start().is_trivia() && !range.end().is_trivia(),
            "trivia is not a structural delete target"
        );
        if range.start() == range.end() {
            self.delete(range.start());
            return;
        }

        debug_assert!(is_ancestor_or_self_of_element(range.start(), &self.root));
        self.changes.borrow_mut().push(Change::ReplaceAll(range, Vec::new()))
    }

    pub fn replace(&self, old: impl Element, new: impl Element) {
        self.queue_replace(old.syntax_element(), Some(new.syntax_element()));
    }

    fn queue_replace(&self, old: SyntaxElement, new: Option<SyntaxElement>) {
        debug_assert!(!old.is_trivia(), "trivia is not a structural replace target");
        debug_assert!(is_ancestor_or_self_of_element(&old, &self.root));
        let mut changes = self.changes.borrow_mut();
        let queued = changes.iter_mut().find_map(|change| match change {
            Change::Replace(existing, replacement) if *existing == old => Some(replacement),
            _ => None,
        });
        match queued {
            Some(queued) if queued.is_some() => *queued = new,
            Some(_) => (),
            None => changes.push(Change::Replace(old, new)),
        }
    }

    pub fn splice_leading_trivia<'a>(
        &self,
        token: &SyntaxToken,
        range: impl RangeBounds<usize>,
        replacement: impl IntoIterator<Item = (SyntaxKind, &'a str)>,
    ) {
        self.splice_trivia(token, true, range, replacement);
    }

    pub fn delete_with(&self, element: impl Element, options: RemoveOptions) {
        let element = element.syntax_element();
        if let (Some(first), Some(last)) =
            (element.first_non_trivia_token(), element.last_non_trivia_token())
            && let Some(next) = self.next_surviving_token(&last)
            && next.parent_ancestors().any(|it| it == self.root)
        {
            let residual: Vec<SyntaxToken> = (first.leading_trivia())
                .filter(|_| options.keep_leading)
                .chain(last.trailing_trivia().filter(|_| options.keep_trailing))
                .collect();
            let marker = options.add_elastic_marker.then_some(tokens::ELASTIC_MARKER);
            let pieces: Vec<_> =
                residual.iter().map(|it| (it.kind(), it.text())).chain(marker).collect();
            if !pieces.is_empty() {
                self.splice_leading_trivia(&next, ..0, pieces);
            }
        }
        self.delete(element);
    }

    pub fn prepend_leading_trivia(&self, element: impl Element, text: &str) {
        let Some(token) = element.syntax_element().first_non_trivia_token() else { return };
        self.splice_leading_trivia(&token, ..0, tokens::trivia(text));
    }

    pub fn splice_trailing_trivia<'a>(
        &self,
        token: &SyntaxToken,
        range: impl RangeBounds<usize>,
        replacement: impl IntoIterator<Item = (SyntaxKind, &'a str)>,
    ) {
        self.splice_trivia(token, false, range, replacement);
    }

    fn splice_trivia<'a>(
        &self,
        token: &SyntaxToken,
        leading_side: bool,
        range: impl RangeBounds<usize>,
        replacement: impl IntoIterator<Item = (SyntaxKind, &'a str)>,
    ) {
        let Some(current) = self.pending_token(token) else { return };
        let green = current.green();
        let mut leading = green.leading_trivia().to_vec();
        let mut trailing = green.trailing_trivia().to_vec();
        let target = if leading_side { &mut leading } else { &mut trailing };

        let replacement = replacement.into_iter().map(|(kind, text)| {
            debug_assert!(kind.is_trivia(), "a trivia token must have a trivia kind");
            GreenToken::new(RSyntaxKind(kind.into()), text)
        });
        target.splice(range, replacement).for_each(drop);

        debug_assert!(
            !trailing
                .iter()
                .rev()
                .skip(1)
                .any(|it| SyntaxKind::from(it.kind().0) == SyntaxKind::NEWLINE),
            "trailing trivia must end at the newline delimiter"
        );

        let payload = token_payload(current.kind(), current.text(), leading, trailing);
        self.queue_replace(token.clone().into(), Some(payload.into()));
    }

    fn pending_token(&self, token: &SyntaxToken) -> Option<SyntaxToken> {
        let element = SyntaxElement::Token(token.clone());
        let changes = self.changes.borrow();
        let queued = changes.iter().find_map(|change| match change {
            Change::Replace(existing, replacement) if *existing == element => Some(replacement),
            _ => None,
        });
        match queued {
            Some(replacement) => replacement.as_ref()?.as_token().cloned(),
            None => Some(token.clone()),
        }
    }

    pub fn replace_with_many(&self, old: impl Element, new: Vec<SyntaxElement>) {
        let old = old.syntax_element();
        debug_assert!(!old.is_trivia(), "trivia is not a structural replace target");
        debug_assert!(is_ancestor_or_self_of_element(&old, &self.root));
        debug_assert!(
            !(matches!(&old, SyntaxElement::Node(node) if node == &self.root) && new.len() > 1),
            "cannot replace root node with many elements"
        );
        self.changes.borrow_mut().push(Change::ReplaceWithMany(old, new));
    }

    pub fn replace_all(&self, range: RangeInclusive<SyntaxElement>, new: Vec<SyntaxElement>) {
        debug_assert!(
            !range.start().is_trivia() && !range.end().is_trivia(),
            "trivia is not a structural replace target"
        );
        if range.start() == range.end() {
            self.replace_with_many(range.start(), new);
            return;
        }
        debug_assert!(is_ancestor_or_self_of_element(range.start(), &self.root));
        self.changes.borrow_mut().push(Change::ReplaceAll(range, new))
    }

    pub fn finish(self) -> SyntaxEdit {
        edit_algo::apply_edits(self)
    }

    pub(crate) fn deleted(&self, element: impl Element) -> bool {
        let element = element.syntax_element();
        self.changes
            .borrow()
            .iter()
            .any(|change| matches!(change, Change::Replace(existing, None) if *existing == element))
    }
}

/// Represents a completed [`SyntaxEditor`] operation.
pub struct SyntaxEdit {
    old_root: SyntaxNode,
    new_root: SyntaxNode,
    changed_elements: Vec<SyntaxElement>,
    annotations: FxHashMap<SyntaxAnnotation, Vec<SyntaxElement>>,
}

impl SyntaxEdit {
    /// Root of the initial unmodified syntax tree.
    pub fn old_root(&self) -> &SyntaxNode {
        &self.old_root
    }

    /// Root of the modified syntax tree.
    pub fn new_root(&self) -> &SyntaxNode {
        &self.new_root
    }

    /// Which syntax elements in the modified syntax tree were inserted or
    /// modified as part of the edit.
    ///
    /// Note that for syntax nodes, only the upper-most parent of a set of
    /// changes is included, not any child elements that may have been modified.
    pub fn changed_elements(&self) -> &[SyntaxElement] {
        self.changed_elements.as_slice()
    }

    /// Finds which syntax elements have been annotated with the given
    /// annotation.
    ///
    /// Note that an annotation might not appear in the modified syntax tree if
    /// the syntax elements that were annotated did not make it into the final
    /// syntax tree.
    pub fn find_annotation(&self, annotation: SyntaxAnnotation) -> &[SyntaxElement] {
        self.annotations.get(&annotation).as_ref().map_or(&[], |it| it.as_slice())
    }

    pub fn find_element(&self, old_node: &SyntaxNode) -> Option<SyntaxNode> {
        let old_root_start = self.old_root.text_range().start();
        let old_start = old_node.text_range().start() - old_root_start;
        let new_root_start = self.new_root.text_range().start();
        let kind = old_node.kind();

        self.new_root
            .descendants()
            .find(|it| it.kind() == kind && it.text_range().start() - new_root_start == old_start)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoveOptions {
    pub keep_leading: bool,
    pub keep_trailing: bool,
    pub add_elastic_marker: bool,
}

impl RemoveOptions {
    pub const KEEP_NO_TRIVIA: Self =
        Self { keep_leading: false, keep_trailing: false, add_elastic_marker: false };
    pub const KEEP_LEADING: Self = Self { keep_leading: true, ..Self::KEEP_NO_TRIVIA };
    pub const KEEP_EXTERIOR: Self = Self { keep_trailing: true, ..Self::KEEP_LEADING };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SyntaxAnnotation(NonZeroU32);

impl Default for SyntaxAnnotation {
    fn default() -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(1);

        // Only consistency within a thread matters, as SyntaxElements are !Send
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);

        Self(NonZeroU32::new(id).expect("syntax annotation id overflow"))
    }
}

/// Position describing where to insert elements
#[derive(Debug)]
pub struct Position {
    repr: PositionRepr,
}

impl Position {
    pub(crate) fn parent(&self) -> SyntaxNode {
        self.place().0
    }

    pub(crate) fn place(&self) -> (SyntaxNode, usize) {
        match &self.repr {
            PositionRepr::FirstChild(parent) => (parent.clone(), 0),
            PositionRepr::After(child) => {
                (child.parent().unwrap(), child.index().expect("checked at insertion") + 1)
            }
        }
    }
}

#[derive(Debug)]
enum PositionRepr {
    FirstChild(SyntaxNode),
    After(SyntaxElement),
}

impl Position {
    pub fn after(elem: impl Element) -> Position {
        let elem = elem.syntax_element();
        debug_assert!(!elem.is_trivia(), "trivia is not a structural insertion anchor");
        Position { repr: PositionRepr::After(elem) }
    }

    pub fn before(elem: impl Element) -> Position {
        let elem = elem.syntax_element();
        debug_assert!(!elem.is_trivia(), "trivia is not a structural insertion anchor");
        let repr = match elem.prev_sibling_or_token() {
            Some(it) => PositionRepr::After(it),
            None => PositionRepr::FirstChild(elem.parent().unwrap()),
        };
        Position { repr }
    }

    pub fn first_child_of(node: &(impl Into<SyntaxNode> + Clone)) -> Position {
        Position { repr: PositionRepr::FirstChild(node.clone().into()) }
    }

    pub fn last_child_of(node: &(impl Into<SyntaxNode> + Clone)) -> Position {
        let node = node.clone().into();
        let repr = match node.last_child_or_token() {
            Some(it) => PositionRepr::After(it),
            None => PositionRepr::FirstChild(node),
        };
        Position { repr }
    }
}

#[derive(Debug)]
enum Change {
    /// Inserts a single element at the specified position.
    Insert(Position, SyntaxElement),
    /// Inserts many elements in-order at the specified position.
    InsertAll(Position, Vec<SyntaxElement>),
    /// Represents both a replace single element and a delete element operation.
    Replace(SyntaxElement, Option<SyntaxElement>),
    /// Replaces a single element with many elements.
    ReplaceWithMany(SyntaxElement, Vec<SyntaxElement>),
    /// Replaces a range of elements with another list of elements.
    /// Range will always have start != end.
    ReplaceAll(RangeInclusive<SyntaxElement>, Vec<SyntaxElement>),
}

impl Change {
    fn target_range(&self) -> TextRange {
        match self {
            Change::Insert(target, _) | Change::InsertAll(target, _) => match &target.repr {
                PositionRepr::FirstChild(parent) => {
                    TextRange::at(parent.text_range().start(), 0.into())
                }
                PositionRepr::After(child) => {
                    TextRange::at(child.text_range_including_trivia().end(), 0.into())
                }
            },
            Change::Replace(target, _) | Change::ReplaceWithMany(target, _) => {
                target.text_range_including_trivia()
            }
            Change::ReplaceAll(range, _) => range
                .start()
                .text_range_including_trivia()
                .cover(range.end().text_range_including_trivia()),
        }
    }

    fn target_parent(&self) -> SyntaxNode {
        match self {
            Change::Insert(target, _) | Change::InsertAll(target, _) => target.parent(),
            Change::Replace(target, _) | Change::ReplaceWithMany(target, _) => match target {
                SyntaxElement::Node(target) => target.parent().unwrap_or_else(|| target.clone()),
                SyntaxElement::Token(target) => target.parent().unwrap(),
            },
            Change::ReplaceAll(target, _) => target.start().parent().unwrap(),
        }
    }

    fn change_kind(&self) -> ChangeKind {
        match self {
            Change::Insert(_, _) | Change::InsertAll(_, _) => ChangeKind::Insert,
            Change::Replace(_, _) | Change::ReplaceWithMany(_, _) => ChangeKind::Replace,
            Change::ReplaceAll(_, _) => ChangeKind::ReplaceRange,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ChangeKind {
    Insert,
    ReplaceRange,
    Replace,
}

impl fmt::Display for Change {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Change::Insert(position, node_or_token) => {
                let parent = position.parent();
                let mut parent_str = parent.to_string();
                let target_range = self.target_range().start() - parent.text_range().start();

                parent_str.insert_str(
                    target_range.into(),
                    &format!("\x1b[42m{node_or_token}\x1b[0m\x1b[K"),
                );
                f.write_str(&parent_str)
            }
            Change::InsertAll(position, vec) => {
                let parent = position.parent();
                let mut parent_str = parent.to_string();
                let target_range = self.target_range().start() - parent.text_range().start();
                let insertion: String = vec.iter().map(|it| it.to_string()).collect();

                parent_str
                    .insert_str(target_range.into(), &format!("\x1b[42m{insertion}\x1b[0m\x1b[K"));
                f.write_str(&parent_str)
            }
            Change::Replace(old, new) => {
                if let Some(new) = new {
                    write!(f, "\x1b[41m{old}\x1b[42m{new}\x1b[0m\x1b[K")
                } else {
                    write!(f, "\x1b[41m{old}\x1b[0m\x1b[K")
                }
            }
            Change::ReplaceWithMany(old, vec) => {
                let new: String = vec.iter().map(|it| it.to_string()).collect();
                write!(f, "\x1b[41m{old}\x1b[42m{new}\x1b[0m\x1b[K")
            }
            Change::ReplaceAll(range, vec) => {
                let parent = range.start().parent().unwrap();
                let parent_str = parent.to_string();
                let pre_range = TextRange::new(
                    parent.text_range().start(),
                    range.start().text_range_including_trivia().start(),
                );
                let old_range = TextRange::new(
                    range.start().text_range_including_trivia().start(),
                    range.end().text_range_including_trivia().end(),
                );
                let post_range = TextRange::new(
                    range.end().text_range_including_trivia().end(),
                    parent.text_range().end(),
                );

                let pre_str = &parent_str[pre_range - parent.text_range().start()];
                let old_str = &parent_str[old_range - parent.text_range().start()];
                let post_str = &parent_str[post_range - parent.text_range().start()];
                let new: String = vec.iter().map(|it| it.to_string()).collect();

                write!(f, "{pre_str}\x1b[41m{old_str}\x1b[42m{new}\x1b[0m\x1b[K{post_str}")
            }
        }
    }
}

/// Utility trait to allow calling syntax editor functions with references or owned
/// nodes. Do not use outside of this module.
pub trait Element {
    fn syntax_element(self) -> SyntaxElement;
}

impl<E: Element + Clone> Element for &'_ E {
    fn syntax_element(self) -> SyntaxElement {
        self.clone().syntax_element()
    }
}

impl Element for SyntaxElement {
    fn syntax_element(self) -> SyntaxElement {
        self
    }
}

impl Element for SyntaxNode {
    fn syntax_element(self) -> SyntaxElement {
        self.into()
    }
}

impl Element for SyntaxToken {
    fn syntax_element(self) -> SyntaxElement {
        self.into()
    }
}

fn is_ancestor_or_self(node: &SyntaxNode, ancestor: &SyntaxNode) -> bool {
    node == ancestor || node.ancestors().any(|it| &it == ancestor)
}

fn is_ancestor_or_self_of_element(node: &SyntaxElement, ancestor: &SyntaxNode) -> bool {
    matches!(node, SyntaxElement::Node(node) if node == ancestor)
        || node.ancestors().any(|it| &it == ancestor)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::{
        AstNode,
        ast::{self, make},
    };

    use super::*;

    #[test]
    fn basic_usage() {
        let root = make::match_arm(
            make::wildcard_pat().into(),
            None,
            make::expr_tuple([
                make::expr_bin_op(
                    make::expr_literal("2").into(),
                    ast::BinaryOp::ArithOp(ast::ArithOp::Add),
                    make::expr_literal("2").into(),
                ),
                make::expr_literal("true").into(),
            ])
            .into(),
        );

        let (editor, root) = SyntaxEditor::with_ast_node(&root);
        let make = editor.make();

        let to_wrap = root.syntax().descendants().find_map(ast::TupleExpr::cast).unwrap();
        let to_replace = root.syntax().descendants().find_map(ast::BinExpr::cast).unwrap();

        let name = make::name("var_name");
        let name_ref = make::name_ref("var_name");

        let placeholder_snippet = SyntaxAnnotation::default();
        editor.add_annotation(name.syntax(), placeholder_snippet);
        editor.add_annotation(name_ref.syntax(), placeholder_snippet);

        let new_block = make.block_expr(
            [editor
                .make()
                .let_stmt(
                    make.ident_pat(false, false, name.clone()).into(),
                    None,
                    Some(to_replace.clone().into()),
                )
                .into()],
            Some(to_wrap.clone().into()),
        );

        editor.replace(to_replace.syntax(), name_ref.syntax());
        editor.replace(to_wrap.syntax(), new_block.syntax());

        let edit = editor.finish();

        let expect = expect![[r#"
            _ => {
                let var_name = 2 + 2;
                (var_name, true)
            },"#]];
        expect.assert_eq(&edit.new_root.to_string());

        assert_eq!(edit.find_annotation(placeholder_snippet).len(), 2);
        assert!(
            edit.annotations
                .values()
                .flatten()
                .all(|element| element.ancestors().any(|it| &it == edit.new_root()))
        )
    }

    #[test]
    fn test_insert_independent() {
        let root = make::block_expr(
            [make::let_stmt(
                make::ext::simple_ident_pat(make::name("second")).into(),
                None,
                Some(make::expr_literal("2").into()),
            )
            .into()],
            None,
        );

        let (editor, root) = SyntaxEditor::with_ast_node(&root);
        let make = editor.make();
        let second_let = root.syntax().descendants().find_map(ast::LetStmt::cast).unwrap();

        editor.insert(
            Position::first_child_of(root.stmt_list().unwrap().syntax()),
            make.let_stmt(
                make::ext::simple_ident_pat(make::name("first")).into(),
                None,
                Some(make::expr_literal("1").into()),
            )
            .syntax(),
        );

        editor.insert(
            Position::after(second_let.syntax()),
            make.let_stmt(
                make::ext::simple_ident_pat(make::name("third")).into(),
                None,
                Some(make::expr_literal("3").into()),
            )
            .syntax(),
        );

        let edit = editor.finish();

        let expect = expect![[r#"
            let first = 1;
            {
                let second = 2;
                let third = 3;
            }"#]];
        expect.assert_eq(&edit.new_root.to_string());
    }

    #[test]
    fn test_insert_dependent() {
        let root = make::block_expr(
            [],
            Some(
                make::block_expr(
                    [make::let_stmt(
                        make::ext::simple_ident_pat(make::name("second")).into(),
                        None,
                        Some(make::expr_literal("2").into()),
                    )
                    .into()],
                    None,
                )
                .into(),
            ),
        );

        let (editor, root) = SyntaxEditor::with_ast_node(&root);
        let make = editor.make();

        let inner_block =
            root.syntax().descendants().flat_map(ast::BlockExpr::cast).nth(1).unwrap();
        let second_let = root.syntax().descendants().find_map(ast::LetStmt::cast).unwrap();

        let new_block_expr = make.block_expr([], Some(ast::Expr::BlockExpr(inner_block.clone())));

        let first_let = make.let_stmt(
            make::ext::simple_ident_pat(make::name("first")).into(),
            None,
            Some(make::expr_literal("1").into()),
        );

        let third_let = make.let_stmt(
            make::ext::simple_ident_pat(make::name("third")).into(),
            None,
            Some(make::expr_literal("3").into()),
        );

        editor.insert(
            Position::first_child_of(inner_block.stmt_list().unwrap().syntax()),
            first_let.syntax(),
        );
        editor.insert(Position::after(second_let.syntax()), third_let.syntax());
        editor.replace(inner_block.syntax(), new_block_expr.syntax());

        let edit = editor.finish();

        let expect = expect![[r#"
            {
                {
                    let first = 1;
                {
                let second = 2;
                    let third = 3;
                }
            }
            }"#]];
        expect.assert_eq(&edit.new_root.to_string());
    }

    #[test]
    fn test_dependent_change_prefers_nearest_changed_ancestor() {
        let root = make::block_expr(
            [],
            Some(
                make::block_expr(
                    [make::let_stmt(
                        make::ext::simple_ident_pat(make::name("second")).into(),
                        None,
                        Some(make::expr_literal("2").into()),
                    )
                    .into()],
                    None,
                )
                .into(),
            ),
        );

        let (editor, root) = SyntaxEditor::with_ast_node(&root);
        let make = editor.make();

        let inner_block =
            root.syntax().descendants().flat_map(ast::BlockExpr::cast).nth(1).unwrap();

        let outer_replacement = make.block_expr([], Some(ast::Expr::BlockExpr(root.clone())));
        let inner_replacement =
            make.block_expr([], Some(ast::Expr::BlockExpr(inner_block.clone())));

        let first_let = make.let_stmt(
            make::ext::simple_ident_pat(make::name("first")).into(),
            None,
            Some(make::expr_literal("1").into()),
        );

        editor.insert(
            Position::first_child_of(inner_block.stmt_list().unwrap().syntax()),
            first_let.syntax(),
        );
        editor.replace(inner_block.syntax(), inner_replacement.syntax());
        editor.replace(root.syntax(), outer_replacement.syntax());

        let edit = editor.finish();

        let expect = expect![[r#"
            {
                {
                    {
                        let first = 1;
                {
                let second = 2;
            }
            }
                }
            }"#]];
        expect.assert_eq(&edit.new_root.to_string());
    }

    #[test]
    fn test_replace_root_with_dependent() {
        let root = make::block_expr(
            [make::let_stmt(
                make::ext::simple_ident_pat(make::name("second")).into(),
                None,
                Some(make::expr_literal("2").into()),
            )
            .into()],
            None,
        );

        let (editor, root) = SyntaxEditor::with_ast_node(&root);
        let make = editor.make();

        let inner_block = root;

        let new_block_expr = make.block_expr([], Some(ast::Expr::BlockExpr(inner_block.clone())));

        let first_let = make.let_stmt(
            make::ext::simple_ident_pat(make::name("first")).into(),
            None,
            Some(make::expr_literal("1").into()),
        );

        editor.insert(
            Position::first_child_of(inner_block.stmt_list().unwrap().syntax()),
            first_let.syntax(),
        );
        editor.replace(inner_block.syntax(), new_block_expr.syntax());

        let edit = editor.finish();

        let expect = expect![[r#"
            {
                let first = 1;
                {
                let second = 2;
            }
            }"#]];
        expect.assert_eq(&edit.new_root.to_string());
    }

    #[test]
    fn test_replace_token_in_parent() {
        let parent_fn = make::fn_(
            None,
            None,
            make::name("it"),
            None,
            None,
            make::param_list(None, []),
            make::block_expr([], Some(make::ext::expr_unit())),
            Some(make::ret_type(make::ty_unit())),
            false,
            false,
            false,
            false,
        );

        let (editor, parent_fn) = SyntaxEditor::with_ast_node(&parent_fn);

        if let Some(ret_ty) = parent_fn.ret_type() {
            editor.delete(ret_ty.syntax().clone());
        }

        if let Some(tail) = parent_fn.body().unwrap().tail_expr() {
            editor.delete(tail.syntax().clone());
        }

        let edit = editor.finish();

        let expect = expect![[r#"
            fn it() {
            }"#]];
        expect.assert_eq(&edit.new_root.to_string());
    }

    #[test]
    fn test_more_times_replace_node_to_same_token() {
        let arg_list =
            make::arg_list([make::expr_literal("1").into(), make::expr_literal("2").into()]);

        let (editor, arg_list) = SyntaxEditor::with_ast_node(&arg_list);

        let target_expr = make::token(parser::SyntaxKind::UNDERSCORE);

        for arg in arg_list.args() {
            editor.replace(arg.syntax(), &target_expr);
        }

        let edit = editor.finish();

        let expect = expect![["(_, _)"]];
        expect.assert_eq(&edit.new_root.to_string());
    }

    #[test]
    fn test_more_times_replace_node_to_same_node() {
        let arg_list =
            make::arg_list([make::expr_literal("1").into(), make::expr_literal("2").into()]);

        let (editor, arg_list) = SyntaxEditor::with_ast_node(&arg_list);

        let target_expr = make::expr_literal("3");

        for arg in arg_list.args() {
            editor.replace(arg.syntax(), target_expr.syntax());
        }

        let edit = editor.finish();

        let expect = expect![["(3, 3)"]];
        expect.assert_eq(&edit.new_root.to_string());
    }

    #[test]
    fn test_more_times_insert_node_to_same_node() {
        let arg_list =
            make::arg_list([make::expr_literal("1").into(), make::expr_literal("2").into()]);

        let (editor, arg_list) = SyntaxEditor::with_ast_node(&arg_list);

        let target_expr = make::ext::expr_unit();

        for arg in arg_list.args() {
            editor.insert(Position::before(arg.syntax()), target_expr.syntax());
        }

        let edit = editor.finish();

        let expect = expect!["(() 1, () 2)"];
        expect.assert_eq(&edit.new_root.to_string());
    }
}
