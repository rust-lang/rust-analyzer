use crate::dsl::{SpacingDsl, SpacingRule, SpaceLoc, SpaceValue};
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};

use ra_syntax::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};
use rowan::{GreenNode, cursor};

use std::collections::{HashMap, HashSet};

// TODO make more like intellij's fmt model
// Model holds immutable tree and mutable intermediate model to produce diff
// the model will probably have to create its own tree to add the extra
// info to each token/node:
//
// [1,2,3];
// can be Brace token, ident, comma all of which knows their own rules and apply
// them accordingly to produce [1, 2, 3]; ???

#[derive(Debug)]
/// Whitespace holds all whitespace info for each SynBlock
struct Whitespace {
    original: SyntaxToken,
    indent_spaces: u32,
    additional_spaces: u32,
}

impl Whitespace {
    fn new(token: SyntaxToken) -> Self {
        Self {
            original: token,
            indent_spaces: 0,
            additional_spaces: 0,
        }
    }
}

#[derive(Debug)]
pub(crate) struct SynBlock {
    //indent: some enum?
    element: SyntaxElement,
    text: SmolStr,
    parent: Option<SyntaxNode>,
    children: Vec<SynBlock>,
    range: TextRange,
    prev_whitespace: Option<Whitespace>,
}

impl Default for SynBlock {
    fn default() -> Self {
        // TODO not sure if importing rowan just for this is worth it find another way?
        let empty_root = SyntaxNode::new_root(GreenNode::new(cursor::SyntaxKind(0), Box::new([])));
        let element = NodeOrToken::Node(empty_root);
        Self {
            element,
            text: SmolStr::default(),
            parent: None,
            children: vec![],
            range: TextRange::from_to(TextUnit::from(0), TextUnit::from(0)),
            prev_whitespace: None,
        }
    }
}

// each block will have knowledge of spacing and indent, 
impl SynBlock {
    pub(crate) fn build_block(
        element: SyntaxElement,
    ) -> Self {
        let prev_whitespace = if let NodeOrToken::Token(tkn) = &element {
            println!("found tkn");
            if tkn.prev_token().unwrap().kind() == WHITESPACE {
                println!("found ws");
                tkn.prev_token().map(Whitespace::new)
            } else {
                None
            }
        } else {
            None
        };
        // recursivly add to children
        let children = match &element {
            NodeOrToken::Node(node) => {
                node.children()
                    .map(|n| Self::build_block(NodeOrToken::Node(n)))
                    .collect::<Vec<_>>()
            },
            NodeOrToken::Token(token) => {
                let mut tkns = vec![];
                while let Some(tkn) = token.next_token() {
                    tkns.push(Self::build_block(NodeOrToken::Token(tkn)));
                }
                tkns
            }
        };
        let (parent, range) = match &element {
            NodeOrToken::Node(node) => (node.parent(), node.text_range()),
            NodeOrToken::Token(token) => (Some(token.parent()), token.text_range())
        };
        let text = match &element {
            NodeOrToken::Node(node) => SmolStr::from(node.text().to_string()),
            NodeOrToken::Token(token) => token.text().clone()
        };

        Self {
            element,
            text,
            parent,
            children,
            range,
            prev_whitespace,
        }
    }

    pub(crate) fn walk(&self) -> impl Iterator<Item=&SynBlock> {
        self.children.iter().flat_map(|kid| {
            &kid.children
        })
    }

    /// Remove after dev?
    fn to_string(&self) -> String {
        self.text.to_string()
    }
}

#[derive(Debug)]
pub(crate) struct EditTree {
    blocks: SynBlock,
}

impl Default for EditTree {
    fn default() -> Self {
        Self { blocks: SynBlock::default() }
    }
}

impl EditTree {
    pub(crate) fn new(root: &SyntaxNode) -> Self {
        Self::default().build_tree(root)
    }

    fn build_tree(
        mut self,
        root: &SyntaxNode,
    ) -> Self {
        self.blocks = SynBlock::build_block(NodeOrToken::Node(root.clone()));
        self
    }

    /// only for dev, we dont need to convert or diff in editTree
    pub(crate) fn to_string(&self) -> String {
        let ordered = self.blocks.walk().map(|blk| blk.to_string()).collect::<String>();
        ordered
    }
}
