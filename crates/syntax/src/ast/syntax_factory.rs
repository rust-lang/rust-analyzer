//! Builds upon [`crate::ast::make`] constructors to create ast fragments with
//! optional syntax mappings.
//!
//! Instead of forcing make constructors to perform syntax mapping, we instead
//! let [`SyntaxFactory`] handle constructing the mappings. Care must be taken
//! to remember to feed the syntax mappings into a [`SyntaxEditor`],
//! if applicable.

mod constructors;

use std::cell::{RefCell, RefMut};

use rowan::{GreenNode, SyntaxKind as RSyntaxKind};

use crate::{
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken,
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
        self.splice_edges(element.syntax_element(), Some(make::tokens::trivia(text)), None)
    }

    pub fn with_trailing_trivia(&self, element: impl Element, text: &str) -> SyntaxElement {
        self.splice_edges(element.syntax_element(), None, Some(make::tokens::trivia(text)))
    }

    pub fn with_trivia_from(&self, element: impl Element, from: impl Element) -> SyntaxElement {
        let from = from.syntax_element();
        let leading: Vec<SyntaxToken> = from
            .first_non_trivia_token()
            .map(|it| it.leading_trivia().collect())
            .unwrap_or_default();
        let trailing: Vec<SyntaxToken> = from
            .last_non_trivia_token()
            .map(|it| it.trailing_trivia().collect())
            .unwrap_or_default();
        fn pieces(trivia: &[SyntaxToken]) -> Vec<(SyntaxKind, &str)> {
            trivia.iter().map(|it| (it.kind(), it.text())).collect()
        }
        self.splice_edges(element.syntax_element(), Some(pieces(&leading)), Some(pieces(&trailing)))
    }

    pub fn prepend_leading_trivia(&self, element: impl Element, text: &str) -> SyntaxElement {
        let element = element.syntax_element();
        let existing: Vec<SyntaxToken> = element
            .first_non_trivia_token()
            .map(|it| it.leading_trivia().collect())
            .unwrap_or_default();
        let mut pieces = make::tokens::trivia(text);
        pieces.extend(existing.iter().map(|it| (it.kind(), it.text())));
        self.splice_edges(element, Some(pieces), None)
    }

    pub fn with_elastic_line_break(&self, element: impl Element) -> SyntaxElement {
        let line_break = make::tokens::ELASTIC_LINE_BREAK.to_vec();
        self.splice_edges(element.syntax_element(), None, Some(line_break))
    }

    pub fn clear_trivia(&self, element: impl Element) -> SyntaxElement {
        let elastic = vec![make::tokens::ELASTIC_MARKER];
        self.splice_edges(element.syntax_element(), Some(elastic.clone()), Some(elastic))
    }

    pub fn clear_leading_trivia(&self, element: impl Element) -> SyntaxElement {
        self.splice_edges(element.syntax_element(), Some(vec![make::tokens::ELASTIC_MARKER]), None)
    }

    pub fn clear_trailing_trivia(&self, element: impl Element) -> SyntaxElement {
        self.splice_edges(element.syntax_element(), None, Some(vec![make::tokens::ELASTIC_MARKER]))
    }

    fn splice_edges(
        &self,
        element: SyntaxElement,
        leading: Option<Vec<(SyntaxKind, &str)>>,
        trailing: Option<Vec<(SyntaxKind, &str)>>,
    ) -> SyntaxElement {
        debug_assert!(
            self.mappings.is_some(),
            "decorate through the editor's factory, not one without mappings"
        );
        let green = match &element {
            SyntaxElement::Node(node) => NodeOrToken::Node(node.green().to_owned()),
            SyntaxElement::Token(token) => NodeOrToken::Token(token.green().to_owned()),
        };
        let root =
            SyntaxNode::new_root(GreenNode::new(RSyntaxKind(SyntaxKind::ERROR as u16), [green]));
        let (editor, root) = SyntaxEditor::new(root);
        if let Some(trivia) = leading
            && let Some(token) = root.first_non_trivia_token()
        {
            editor.splice_leading_trivia(&token, .., trivia);
        }
        if let Some(trivia) = trailing
            && let Some(token) = root.last_non_trivia_token()
        {
            editor.splice_trailing_trivia(&token, .., trivia);
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
