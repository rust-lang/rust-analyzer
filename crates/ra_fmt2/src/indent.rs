use crate::dsl::{IndentDsl, IndentRule, IndentValue, SpaceLoc, SpaceValue};
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};

use ra_syntax::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};

use std::collections::{HashMap, HashSet};

const INDENT: usize = 4;
const ID_STR: &str = "    ";

#[derive(Clone, Debug)]
/// Whitespace holds all whitespace information for each Block.
/// Accessed from any Block's get_whitespace fn.
pub(crate) struct Indentation<'i> {
    original: Option<SyntaxElement>,
    parent: Option<&'i Indentation<'i>>,
    indent_spaces: usize,
    // additional_spaces: u32,
    location: usize,
}

impl<'i> Default for Indentation<'i> {
    fn default() -> Self {
        Self {
            original: None,
            parent: None,
            indent_spaces: 0,
            // make sure this is ok or none needs to take element to get text_range
            location: 0,
        }
    }
}

impl<'i> Indentation<'i> {

    pub(crate) fn new(element: &SyntaxElement) -> Self {
        match element {
            NodeOrToken::Node(node) => {
                if let Some(NodeOrToken::Token(prev)) = node.prev_sibling_or_token() {
                    Indentation::from_node(node, &prev)
                } else {
                    Indentation::empty_node(node)
                }
            },
            NodeOrToken::Token(token) => {
                if let Some(NodeOrToken::Token(prev)) = token.prev_sibling_or_token() {
                    Indentation::from_token(token, &prev)
                } else {
                    Indentation::empty_token(token)
                }
            },
        }
    }

    fn empty_node(curr: &SyntaxNode) -> Self {
        Indentation::default().location(curr.text_range().start().to_usize())
    }

    fn empty_token(curr: &SyntaxToken) -> Self {
        Indentation::default().location(curr.text_range().start().to_usize())
    }

    fn location(mut self, location: usize) -> Self {
        self.location = location;
        self
    }

    fn from_node(curr: &SyntaxNode, prev: &SyntaxToken) -> Self {
        if prev.text().starts_with('\n') {
            let original = Some(NodeOrToken::Node(curr.clone()));
            let indent_spaces = prev.text().trim_start_matches('\n').len();

            Self {
                original,
                parent: None,
                indent_spaces,
                location: curr.text_range().start().to_usize(),
            }
        } else {
            Indentation::empty_node(curr)
        }
    }

    fn from_token(curr: &SyntaxToken, prev: &SyntaxToken) -> Self {
        if prev.text().starts_with('\n') {
            let original = Some(NodeOrToken::Token(curr.clone()));
            let indent_spaces = prev.text().trim_start_matches('\n').len();

            Self {
                original,
                parent: None,
                indent_spaces,
                location: curr.text_range().start().to_usize(),
            }
        } else {
            Indentation::empty_token(curr)
        }
    }
}
