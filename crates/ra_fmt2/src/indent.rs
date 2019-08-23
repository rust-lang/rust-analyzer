use crate::dsl::{IndentDsl, IndentRule, IndentValue, SpaceLoc, SpaceValue};
use crate::edit_tree::Block;
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};

use ra_syntax::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};

use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone, Debug)]
/// Whitespace holds all whitespace information for each Block.
/// Accessed from any Block's get_whitespace fn.
pub(crate) struct Indentation {
    original: Option<SyntaxElement>,
    indent_level: usize,
    indent_spaces: usize,
    // additional_spaces: u32,
    location: usize,
}

impl Default for Indentation {
    fn default() -> Self {
        Self {
            original: None,
            indent_level: 0,
            indent_spaces: 0,
            // make sure this is ok or none needs to take element to get text_range
            location: 0,
        }
    }
}

impl Indentation {

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
                indent_level: 0,
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
                indent_level: 0,
                indent_spaces,
                location: curr.text_range().start().to_usize(),
            }
        } else {
            Indentation::empty_token(curr)
        }
    }
}

pub(crate) struct IndentBuilder<'b> {
    collected: Vec<&'b Block>,
}

impl<'b> Default for IndentBuilder<'b> {
    fn default() -> Self {
        Self { collected: vec![], }
    }
}

impl<'b> IndentBuilder<'b> {

    pub(crate) fn new(blk: &'b Block, ) -> IndentBuilder {
        Self { collected: blk.traverse_exc().collect() }.calc_indent_level()
    }

    fn push(&mut self, blk: &'b Block) {
        self.collected.push(blk)
    }

    fn calc_indent_level(self) -> IndentBuilder<'b> {
        self.collected.iter()
            .scan(0, |depth, &blk| {
                let indent_lev = blk.get_indent().borrow().indent_spaces;
                if indent_lev > 0 {
                    if indent_lev > *depth {
                        *depth = blk.get_indent().borrow().indent_spaces;
                    }
                    blk.get_indent().borrow_mut().indent_level = *depth;
                }
                Some(())
            });
        self
    }

    pub(crate) fn indent(&self, block: &'b Block) -> Option<Rc<RefCell<Indentation>>> {
        self.collected.iter()
            .find_map(|blk| {
                if *blk == block {
                    Some(blk.get_indent())
                } else {
                    None
                }
            })
    }
}
