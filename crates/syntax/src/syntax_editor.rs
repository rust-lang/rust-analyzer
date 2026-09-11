//! Syntax Tree editor
//!
//! Inspired by Roslyn's [`SyntaxEditor`].
//!
//! [`SyntaxEditor`]: https://github.com/dotnet/roslyn/blob/43b0b05cc4f492fd5de00f6f6717409091df8daa/src/Workspaces/Core/Portable/Editing/SyntaxEditor.cs

use std::{
    cell::RefCell,
    fmt,
    num::NonZeroU32,
    ops::{RangeBounds, RangeInclusive},
    sync::atomic::{AtomicU32, Ordering},
};

use rowan::TextRange;
use rustc_hash::FxHashMap;

use crate::{
    AstNode, Direction, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken, T,
    ast::{self, edit::IndentLevel, syntax_factory::SyntaxFactory},
    syntax_node::token_payload,
};

mod edit_algo;
mod edits;
mod mapping;

pub use edits::{GetOrCreateWhereClause, Removable};
pub use mapping::{SyntaxMapping, SyntaxMappingBuilder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TriviaSide {
    Leading,
    Trailing,
}

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

    pub(super) fn with_trivia(
        &self,
        element: &SyntaxElement,
        side: TriviaSide,
        text: &str,
    ) -> SyntaxElement {
        let decorated = match side {
            TriviaSide::Leading => self.make.with_leading_trivia(element, text),
            TriviaSide::Trailing => self.make.with_trailing_trivia(element, text),
        };
        self.transfer_annotations(element, &decorated);
        decorated
    }

    fn transfer_annotations(&self, from: &SyntaxElement, to: &SyntaxElement) {
        if from.as_token().is_none() {
            return;
        }
        let mut annotations = self.annotations.borrow_mut();
        let transferred: Vec<_> = annotations
            .iter()
            .filter(|(annotated, _)| annotated == from)
            .map(|(_, annotation)| (to.clone(), *annotation))
            .collect();
        annotations.extend(transferred);
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
        let element = element.syntax_element();
        self.make_room_at(&position, &element);
        let element = self.take_line_break(&position, element);
        self.changes.borrow_mut().push(Change::Insert(position, element))
    }

    pub fn insert_all(&self, position: Position, mut elements: Vec<SyntaxElement>) {
        debug_assert!(is_ancestor_or_self(&position.parent(), &self.root));
        if let Some(first) = elements.first().cloned() {
            self.make_room_at(&position, &first);
            elements[0] = self.take_line_break(&position, first);
        }
        self.changes.borrow_mut().push(Change::InsertAll(position, elements))
    }

    fn take_line_break(&self, position: &Position, element: SyntaxElement) -> SyntaxElement {
        let PositionRepr::Before(anchor) = &position.repr else { return element };
        let Some(token) = anchor.first_non_trivia_token() else { return element };
        if !is_ancestor_or_self_of_element(&SyntaxElement::Token(token.clone()), &self.root) {
            return element;
        }
        let starts_own_line = element
            .first_non_trivia_token()
            .and_then(|it| it.leading_trivia().next())
            .is_some_and(|it| it.kind() == SyntaxKind::NEWLINE);
        match starts_own_line {
            true => self.open_line_before(&token, element),
            false => self.take_anchor_line_prefix(&token, element),
        }
    }

    fn open_line_before(&self, token: &SyntaxToken, element: SyntaxElement) -> SyntaxElement {
        if let Some(decorated) = self.take_comment_lines(token, &element) {
            return decorated;
        }
        if let Some(prev) = self.surviving_token(token, Direction::Prev)
            && is_ancestor_or_self_of_element(&SyntaxElement::Token(prev.clone()), &self.root)
            && !self.covered_by(prev.text_range(), |_| true)
            && let Some(pending) = self.pending_token(&prev)
        {
            let blank = crate::algo::outer_blank_trivia(&pending, TriviaSide::Trailing);
            let mut tail = pending.trailing_trivia().skip(blank.start);
            if tail.any(|it| it.kind() == SyntaxKind::NEWLINE) {
                self.splice_trailing_trivia(&prev, blank, []);
            }
        }
        let Some(pending) = self.pending_token(token) else { return element };
        let end = pending
            .leading_trivia()
            .position(|it| it.kind() != SyntaxKind::WHITESPACE)
            .unwrap_or(0);
        let opens_line =
            pending.leading_trivia().nth(end).is_some_and(|it| it.kind() == SyntaxKind::NEWLINE);
        if end > 0 && opens_line {
            self.splice_leading_trivia(token, ..end, []);
        }
        let anchor_keeps_its_lines =
            pending.leading_trivia().any(|it| it.kind() == SyntaxKind::COMMENT);
        let Some(last) = element.last_non_trivia_token() else { return element };
        if !anchor_keeps_its_lines
            || last.trailing_trivia().next_back().is_some_and(|it| it.kind() == SyntaxKind::NEWLINE)
        {
            return element;
        }
        let trailing: String = last.trailing_trivia().map(|it| it.text().to_owned()).collect();
        self.with_trivia(&element, TriviaSide::Trailing, &format!("{trailing}\n"))
    }

    fn take_anchor_line_prefix(
        &self,
        token: &SyntaxToken,
        element: SyntaxElement,
    ) -> SyntaxElement {
        if token.prev_non_trivia_token().is_none() {
            return element;
        }
        let Some(pending) = self.pending_token(token) else { return element };
        let Some(end) = pending
            .leading_trivia()
            .rposition(|it| it.kind() == SyntaxKind::NEWLINE)
            .map(|index| index + 1)
        else {
            return element;
        };
        let comment_attached_to_anchor = pending
            .leading_trivia()
            .take(end)
            .rposition(|it| it.kind() == SyntaxKind::COMMENT)
            .is_some_and(|comment| {
                pending
                    .leading_trivia()
                    .take(end)
                    .skip(comment + 1)
                    .filter(|it| it.kind() == SyntaxKind::NEWLINE)
                    .count()
                    < 2
            });
        if comment_attached_to_anchor {
            return element;
        }
        self.move_leading_trivia(token, end, element)
    }

    fn take_comment_lines(
        &self,
        token: &SyntaxToken,
        element: &SyntaxElement,
    ) -> Option<SyntaxElement> {
        let pending = self.pending_token(token)?;
        let first = pending.leading_trivia().position(|it| it.kind() == SyntaxKind::COMMENT)?;
        let newline = |it: &SyntaxToken| it.kind() == SyntaxKind::NEWLINE;
        let blank_lines_before = self
            .surviving_token(token, Direction::Prev)
            .and_then(|prev| self.pending_token(&prev))
            .map_or(0, |prev| prev.trailing_trivia().filter(newline).count())
            + pending.leading_trivia().take(first).filter(newline).count();
        if blank_lines_before > 1 {
            return None;
        }
        let end = pending.leading_trivia().rposition(|it| it.kind() == SyntaxKind::COMMENT)? + 1;
        if !matches!(token.kind(), T!['}'] | T![')'] | T![']'])
            && pending.leading_trivia().skip(end).filter(newline).count() < 2
        {
            return None;
        }
        Some(self.move_leading_trivia(token, end, element.clone()))
    }

    fn move_leading_trivia(
        &self,
        token: &SyntaxToken,
        end: usize,
        element: SyntaxElement,
    ) -> SyntaxElement {
        let Some(pending) = self.pending_token(token).filter(|_| end > 0) else { return element };
        let mut moved: String =
            pending.leading_trivia().take(end).map(|it| it.text().to_owned()).collect();
        if let Some(first) = element.first_non_trivia_token() {
            moved.extend(first.leading_trivia().map(|it| it.text().to_owned()));
        }
        self.splice_leading_trivia(token, ..end, []);
        self.with_trivia(&element, TriviaSide::Leading, &moved)
    }

    fn covered_by(&self, range: TextRange, mut which: impl FnMut(&Change) -> bool) -> bool {
        self.changes.borrow().iter().any(|change| {
            let target = match change {
                Change::Insert(..) | Change::InsertAll(..) => return false,
                Change::Replace(target, _) | Change::ReplaceWithMany(target, _) => {
                    target.text_range()
                }
                Change::ReplaceAll(targets, _) => TextRange::new(
                    targets.start().text_range().start(),
                    targets.end().text_range().end(),
                ),
            };
            which(change) && target.contains_range(range)
        })
    }

    fn surviving_token(&self, token: &SyntaxToken, direction: Direction) -> Option<SyntaxToken> {
        let step = |it: &SyntaxToken| match direction {
            Direction::Next => it.next_non_trivia_token(),
            Direction::Prev => it.prev_non_trivia_token(),
        };

        let mut current = step(token);
        while let Some(it) = &current {
            let deleted = self.covered_by(it.text_range(), |change| match change {
                Change::Replace(_, replacement) => replacement.is_none(),
                Change::ReplaceWithMany(_, new) | Change::ReplaceAll(_, new) => new.is_empty(),
                Change::Insert(..) | Change::InsertAll(..) => false,
            });
            if !deleted {
                break;
            }
            current = step(it);
        }
        current
    }

    fn make_room_at(&self, position: &Position, element: &SyntaxElement) {
        let PositionRepr::After(anchor) = &position.repr else { return };
        let Some(token) = anchor.last_non_trivia_token() else { return };
        let Some(next) = token.next_non_trivia_token() else { return };
        if !is_ancestor_or_self_of_element(&SyntaxElement::Token(next.clone()), &self.root) {
            return;
        }
        let inside_replacement = |range: TextRange| {
            self.changes.borrow().iter().any(|change| match change {
                Change::Replace(SyntaxElement::Node(target), _)
                | Change::ReplaceWithMany(SyntaxElement::Node(target), _) => {
                    let target = target.text_range();
                    target.contains_range(range) && target != range
                }
                _ => false,
            })
        };
        if inside_replacement(token.text_range()) || inside_replacement(next.text_range()) {
            return;
        }
        let Some(pending) = self.pending_token(&token) else { return };
        let len = pending.trailing_trivia().len();
        let line_comment = pending
            .trailing_trivia()
            .any(|it| it.kind() == SyntaxKind::COMMENT && it.text().starts_with("//"));
        let starts_own_line = element
            .first_non_trivia_token()
            .and_then(|it| it.leading_trivia().next())
            .is_some_and(|it| it.kind() == SyntaxKind::NEWLINE);
        let start = match line_comment && !starts_own_line {
            true => 0,
            false => crate::algo::outer_blank_trivia(&pending, TriviaSide::Trailing).start,
        };
        if start == len || self.pending_token(&next).is_none() {
            return;
        }
        let moved: Vec<_> = pending.trailing_trivia().skip(start).collect();
        self.splice_leading_trivia(&next, ..0, moved.iter().map(|it| (it.kind(), it.text())));
        self.splice_trailing_trivia(&token, start..len, []);
    }

    pub fn insert_with_whitespace(&self, position: Position, element: impl Element) {
        let mut element = element.syntax_element();
        if let Some(ws) = ws_before(&position, &element) {
            element = self.with_trivia(&element, TriviaSide::Leading, &ws);
        }
        if let Some(ws) = ws_after(&position, &element) {
            element = self.with_trivia(&element, TriviaSide::Trailing, &ws);
        }
        self.insert(position, element)
    }

    pub fn delete(&self, element: impl Element) {
        let element = element.syntax_element();
        debug_assert!(!element.is_trivia(), "trivia is not a structural delete target");
        debug_assert!(is_ancestor_or_self_of_element(&element, &self.root));
        debug_assert!(
            !matches!(&element, SyntaxElement::Node(node) if node == &self.root),
            "should not delete root node"
        );
        let mut changes = self.changes.borrow_mut();
        let queued = changes.iter_mut().find_map(|change| match change {
            Change::Replace(existing, replacement) if *existing == element => Some(replacement),
            _ => None,
        });
        match queued {
            Some(replacement) => *replacement = None,
            None => changes.push(Change::Replace(element, None)),
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
        let old = old.syntax_element();
        let new = self.keep_edges(&old, &old, vec![new.syntax_element()]);
        self.replace_verbatim(old, new.into_iter().next().expect("one replacement in, one out"));
    }

    pub fn replace_verbatim(&self, old: impl Element, new: impl Element) {
        let old = old.syntax_element();
        debug_assert!(!old.is_trivia(), "trivia is not a structural replace target");
        debug_assert!(is_ancestor_or_self_of_element(&old, &self.root));
        let new = new.syntax_element();
        let mut changes = self.changes.borrow_mut();
        let queued = changes.iter_mut().find_map(|change| match change {
            Change::Replace(existing, replacement) if *existing == old => Some(replacement),
            _ => None,
        });
        match queued {
            Some(Some(existing)) => *existing = new,
            Some(None) => (),
            None => changes.push(Change::Replace(old, Some(new))),
        }
    }

    pub fn replace_token(&self, old: &SyntaxToken, new: &SyntaxToken) {
        let Some(current) = self.pending_token(old) else { return };
        let green = current.green();
        let payload = token_payload(
            new.kind(),
            new.text(),
            green.leading_trivia().to_vec(),
            green.trailing_trivia().to_vec(),
        );

        self.transfer_annotations(
            &SyntaxElement::Token(new.clone()),
            &SyntaxElement::Token(payload.clone()),
        );
        self.replace_verbatim(old, payload);
    }

    pub fn splice_leading_trivia<'a>(
        &self,
        token: &SyntaxToken,
        range: impl RangeBounds<usize>,
        replacement: impl IntoIterator<Item = (SyntaxKind, &'a str)>,
    ) {
        self.splice_trivia(token, TriviaSide::Leading, range, replacement);
    }

    fn keep_edges(
        &self,
        start: &SyntaxElement,
        end: &SyntaxElement,
        mut new: Vec<SyntaxElement>,
    ) -> Vec<SyntaxElement> {
        if new.is_empty() || matches!(start, SyntaxElement::Node(node) if node == &self.root) {
            return new;
        }

        let leading: String = start
            .first_non_trivia_token()
            .and_then(|token| self.pending_token(&token))
            .map(|token| token.leading_trivia().map(|it| it.text().to_owned()).collect())
            .unwrap_or_default();
        if !leading.is_empty()
            && let Some(first) = new[0].first_non_trivia_token()
            && first.leading_trivia().next().is_none_or(|it| it.kind() == SyntaxKind::COMMENT)
        {
            let own: String = first.leading_trivia().map(|it| it.text().to_owned()).collect();
            new[0] = self.with_trivia(&new[0], TriviaSide::Leading, &format!("{leading}{own}"));
        }

        let trailing: String = end
            .last_non_trivia_token()
            .and_then(|token| self.pending_token(&token))
            .map(|token| token.trailing_trivia().map(|it| it.text().to_owned()).collect())
            .unwrap_or_default();
        let index = new.len() - 1;
        if !trailing.is_empty()
            && new[index].last_non_trivia_token().is_some_and(|it| it.trailing_trivia().len() == 0)
        {
            new[index] = self.with_trivia(&new[index], TriviaSide::Trailing, &trailing);
        }

        new
    }

    fn delete_moving_trivia(
        &self,
        element: impl Element,
        take: fn(&SyntaxToken, &SyntaxToken) -> String,
    ) {
        let element = element.syntax_element();
        if let (Some(first), Some(last)) =
            (element.first_non_trivia_token(), element.last_non_trivia_token())
            && let Some(next) = self.surviving_token(&last, Direction::Next)
            && is_ancestor_or_self_of_element(&SyntaxElement::Token(next.clone()), &self.root)
        {
            let moved = take(&first, &last);
            if !moved.is_empty() {
                self.prepend_leading_trivia(&next, &moved);
            }
        }
        self.delete(element);
    }

    pub fn delete_keeping_edges(&self, element: impl Element) {
        self.delete_moving_trivia(element, |first, last| {
            first
                .leading_trivia()
                .chain(last.trailing_trivia())
                .map(|it| it.text().to_owned())
                .collect()
        });
    }

    pub fn insert_taking_leading(&self, anchor: impl Element, element: impl Element) {
        let anchor = anchor.syntax_element();
        let mut element = element.syntax_element();
        if let Some(first) = anchor.first_non_trivia_token()
            && let Some(token) = self.pending_token(&first)
            && token.leading_trivia().len() > 0
        {
            let leading: String = token.leading_trivia().map(|it| it.text().to_owned()).collect();
            self.splice_leading_trivia(&first, .., []);
            element = self.with_trivia(&element, TriviaSide::Leading, &leading);
        }
        self.insert(Position::before(anchor), element);
    }

    pub fn delete_keeping_leading(&self, element: impl Element) {
        self.delete_moving_trivia(element, |first, _| {
            first.leading_trivia().map(|it| it.text().to_owned()).collect()
        });
    }

    pub fn delete_keeping_lines(&self, element: impl Element) {
        self.delete_moving_trivia(element, |first, _| {
            let trivia: Vec<_> = first.leading_trivia().collect();
            match trivia.iter().rposition(|it| it.kind() == SyntaxKind::NEWLINE) {
                Some(end) => trivia[..=end].iter().map(|it| it.text()).collect(),
                None => String::new(),
            }
        });
    }

    pub fn strip_trailing_blank_trivia(&self, token: &SyntaxToken) {
        let blank = crate::algo::outer_blank_trivia(token, TriviaSide::Trailing);
        if !blank.is_empty() {
            self.splice_trailing_trivia(token, blank, []);
        }
    }

    pub fn prepend_leading_trivia(&self, element: impl Element, text: &str) {
        let Some(token) = element.syntax_element().first_non_trivia_token() else { return };
        self.splice_leading_trivia(&token, ..0, ast::make::tokens::trivia(text));
    }

    pub fn splice_trailing_trivia<'a>(
        &self,
        token: &SyntaxToken,
        range: impl RangeBounds<usize>,
        replacement: impl IntoIterator<Item = (SyntaxKind, &'a str)>,
    ) {
        self.splice_trivia(token, TriviaSide::Trailing, range, replacement);
    }

    fn splice_trivia<'a>(
        &self,
        token: &SyntaxToken,
        side: TriviaSide,
        range: impl RangeBounds<usize>,
        replacement: impl IntoIterator<Item = (SyntaxKind, &'a str)>,
    ) {
        let Some(current) = self.pending_token(token) else { return };
        let replacement = replacement.into_iter().collect::<Vec<_>>();
        debug_assert!(
            replacement.iter().all(|(kind, _)| kind.is_trivia()),
            "a trivia token must have a trivia kind"
        );

        let rewritten = self.covered_by(token.text_range(), |change| {
            matches!(change, Change::ReplaceWithMany(..) | Change::ReplaceAll(..))
        });
        if rewritten {
            return;
        }

        let green = current.green();
        let mut leading = green.leading_trivia().to_vec();
        let mut trailing = green.trailing_trivia().to_vec();
        let target = match side {
            TriviaSide::Leading => &mut leading,
            TriviaSide::Trailing => &mut trailing,
        };

        let replacement = replacement
            .into_iter()
            .map(|(kind, text)| rowan::GreenToken::new(rowan::SyntaxKind(kind.into()), text));
        target.splice(range, replacement).for_each(drop);

        debug_assert!(
            trailing.iter().enumerate().all(|(index, it)| {
                SyntaxKind::from(it.kind().0) != SyntaxKind::NEWLINE || index + 1 == trailing.len()
            }),
            "trailing trivia must end at the newline delimiter"
        );

        let payload = token_payload(current.kind(), current.text(), leading, trailing);
        self.replace_verbatim(token, payload);
    }

    pub fn pending_token(&self, token: &SyntaxToken) -> Option<SyntaxToken> {
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
        let new = self.keep_edges(&old, &old, new);
        debug_assert!(!old.is_trivia(), "trivia is not a structural replace target");
        debug_assert!(is_ancestor_or_self_of_element(&old, &self.root));
        debug_assert!(
            !(matches!(&old, SyntaxElement::Node(node) if node == &self.root) && new.len() > 1),
            "cannot replace root node with many elements"
        );
        self.changes.borrow_mut().push(Change::ReplaceWithMany(old.syntax_element(), new));
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
        let new = self.keep_edges(range.start(), range.end(), new);

        debug_assert!(is_ancestor_or_self_of_element(range.start(), &self.root));
        self.changes.borrow_mut().push(Change::ReplaceAll(range, new))
    }

    pub fn finish(self) -> SyntaxEdit {
        edit_algo::apply_edits(self)
    }

    pub fn deleted(&self, element: impl Element) -> bool {
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
            PositionRepr::After(child) | PositionRepr::LastChild(child) => {
                (child.parent().unwrap(), child.index().expect("checked at insertion") + 1)
            }
            PositionRepr::Before(child) => {
                (child.parent().unwrap(), child.index().expect("checked at insertion"))
            }
        }
    }
}

#[derive(Debug)]
enum PositionRepr {
    FirstChild(SyntaxNode),
    After(SyntaxElement),
    LastChild(SyntaxElement),
    Before(SyntaxElement),
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
        debug_assert!(elem.parent().is_some(), "a root is not an insertion anchor");
        Position { repr: PositionRepr::Before(elem) }
    }

    pub fn first_child_of(node: &(impl Into<SyntaxNode> + Clone)) -> Position {
        Position { repr: PositionRepr::FirstChild(node.clone().into()) }
    }

    pub fn last_child_of(node: &(impl Into<SyntaxNode> + Clone)) -> Position {
        let node = node.clone().into();
        let repr = match node.last_child_or_token() {
            Some(it) => PositionRepr::LastChild(it),
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
                PositionRepr::After(child) | PositionRepr::LastChild(child) => {
                    TextRange::at(child.text_range_including_trivia().end(), 0.into())
                }
                PositionRepr::Before(child) => {
                    TextRange::at(child.text_range_including_trivia().start(), 0.into())
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

fn ws_before(position: &Position, new: &SyntaxElement) -> Option<String> {
    let prev = match &position.repr {
        PositionRepr::FirstChild(_) => return None,
        PositionRepr::After(it) | PositionRepr::LastChild(it) => it.clone(),
        PositionRepr::Before(it) => it.prev_sibling_or_token()?,
    };
    let prev = &prev;

    if prev.kind() == T!['{']
        && new.kind() == SyntaxKind::USE
        && let Some(item_list) = prev.parent().and_then(ast::ItemList::cast)
    {
        let mut indent = IndentLevel::from_element(&item_list.syntax().clone().into());
        indent.0 += 1;
        return Some(format!("\n{indent}"));
    }

    if prev.kind() == T!['{']
        && ast::Stmt::can_cast(new.kind())
        && let Some(stmt_list) = prev.parent().and_then(ast::StmtList::cast)
    {
        let mut indent = IndentLevel::from_element(&stmt_list.syntax().clone().into());
        indent.0 += 1;
        return Some(format!("\n{indent}"));
    }

    ws_between(prev, new)
}

fn ws_after(position: &Position, new: &SyntaxElement) -> Option<String> {
    let next = match &position.repr {
        PositionRepr::FirstChild(parent) => parent.first_child_or_token()?,
        PositionRepr::After(sibling) | PositionRepr::LastChild(sibling) => {
            sibling.next_sibling_or_token()?
        }
        PositionRepr::Before(sibling) => sibling.clone(),
    };
    ws_between(new, &next)
}

fn ws_between(left: &SyntaxElement, right: &SyntaxElement) -> Option<String> {
    if left.last_non_trivia_token().is_some_and(|token| token.trailing_trivia().len() > 0)
        || right.first_non_trivia_token().is_some_and(|token| token.leading_trivia().len() > 0)
    {
        return None;
    }
    if matches!(right.kind(), T![;] | T![,] | SyntaxKind::EOF) {
        return None;
    }
    if left.kind() == T![<] || right.kind() == T![>] {
        return None;
    }
    if left.kind() == T![&] && right.kind() == SyntaxKind::LIFETIME {
        return None;
    }
    if right.kind() == SyntaxKind::GENERIC_ARG_LIST {
        return None;
    }
    if right.kind() == SyntaxKind::USE {
        let mut indent = IndentLevel::from_element(left);
        if left.kind() == SyntaxKind::USE {
            indent.0 = IndentLevel::from_element(right).0.max(indent.0);
        }
        return Some(format!("\n{indent}"));
    }
    if left.kind() == SyntaxKind::ATTR {
        let mut indent = IndentLevel::from_element(right);
        if right.kind() == SyntaxKind::ATTR {
            indent.0 = IndentLevel::from_element(left).0.max(indent.0);
        }
        return Some(format!("\n{indent}"));
    }
    Some(" ".to_owned())
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
            let first = 1;{
                let second = 2;let third = 3;
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
            let first = 1;    {
                let second = 2;let third = 3;
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
            let first = 1;    {
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
            let first = 1;    {
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

        let expect = expect![["(()1, ()2)"]];
        expect.assert_eq(&edit.new_root.to_string());
    }
}
