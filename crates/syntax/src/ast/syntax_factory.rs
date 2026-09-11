//! Builds upon [`crate::ast::make`] constructors to create ast fragments with
//! optional syntax mappings.
//!
//! Instead of forcing make constructors to perform syntax mapping, we instead
//! let [`SyntaxFactory`] handle constructing the mappings. Care must be taken
//! to remember to feed the syntax mappings into a [`SyntaxEditor`],
//! if applicable.

mod constructors;

use std::cell::{RefCell, RefMut};

use crate::{
    SyntaxElement, SyntaxKind, SyntaxNode,
    ast::make,
    syntax_editor::{Element, SyntaxEditor, SyntaxMapping, SyntaxMappingBuilder},
};

#[derive(Debug)]
pub struct SyntaxFactory {
    // Stored in a refcell so that the factory methods can be &self
    mappings: Option<RefCell<SyntaxMapping>>,
}

impl SyntaxFactory {
    pub fn with_leading_trivia(&self, element: impl Element, text: &str) -> SyntaxElement {
        self.with_trivia(element.syntax_element(), text, crate::syntax_editor::TriviaSide::Leading)
    }

    pub fn with_trailing_trivia(&self, element: impl Element, text: &str) -> SyntaxElement {
        self.with_trivia(element.syntax_element(), text, crate::syntax_editor::TriviaSide::Trailing)
    }

    pub fn prepend_leading_trivia(&self, element: impl Element, text: &str) -> SyntaxElement {
        let element = element.syntax_element();
        let existing: String = element
            .first_non_trivia_token()
            .map(|token| token.leading_trivia().map(|it| it.text().to_owned()).collect())
            .unwrap_or_default();
        self.with_trivia(
            element,
            &format!("{text}{existing}"),
            crate::syntax_editor::TriviaSide::Leading,
        )
    }

    fn with_trivia(
        &self,
        element: SyntaxElement,
        text: &str,
        side: crate::syntax_editor::TriviaSide,
    ) -> SyntaxElement {
        debug_assert!(
            self.mappings.is_some(),
            "decorate through the editor's factory, not one without mappings"
        );
        let green = match &element {
            SyntaxElement::Node(node) => rowan::NodeOrToken::Node(node.green().to_owned()),
            SyntaxElement::Token(token) => rowan::NodeOrToken::Token(token.green().to_owned()),
        };
        let root = SyntaxNode::new_root(rowan::GreenNode::new(
            rowan::SyntaxKind(SyntaxKind::ERROR as u16),
            [green],
        ));
        let (editor, root) = SyntaxEditor::new(root);
        match side {
            crate::syntax_editor::TriviaSide::Leading => {
                if let Some(token) = root.first_non_trivia_token() {
                    editor.splice_leading_trivia(&token, .., make::tokens::trivia(text));
                }
            }
            crate::syntax_editor::TriviaSide::Trailing => {
                if let Some(token) = root.last_non_trivia_token() {
                    editor.splice_trailing_trivia(&token, .., make::tokens::trivia(text));
                }
            }
        }
        let edit = editor.finish();
        let output = edit.new_root().first_child_or_token().unwrap();
        if let SyntaxElement::Node(input) = element
            && let Some(mut mappings) = self.mappings()
        {
            let mut builder = SyntaxMappingBuilder::new(edit.new_root().clone());
            builder.map_node(input, output.as_node().unwrap().clone());
            builder.finish(&mut mappings);
        }
        output
    }

    /// Creates a new [`SyntaxFactory`], generating mappings between input nodes and generated nodes.
    pub(crate) fn with_mappings() -> Self {
        Self { mappings: Some(RefCell::new(SyntaxMapping::default())) }
    }

    /// Creates a [`SyntaxFactory`] without generating mappings.
    pub fn without_mappings() -> Self {
        Self { mappings: None }
    }

    /// Take all of the tracked syntax mappings, leaving `SyntaxMapping::default()` in its place, if any.
    pub(crate) fn take(&self) -> SyntaxMapping {
        self.mappings.as_ref().map(|mappings| mappings.take()).unwrap_or_default()
    }

    pub(crate) fn mappings(&self) -> Option<RefMut<'_, SyntaxMapping>> {
        self.mappings.as_ref().map(|it| it.borrow_mut())
    }
}

impl Default for SyntaxFactory {
    fn default() -> Self {
        Self::without_mappings()
    }
}
