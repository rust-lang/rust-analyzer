use crate::dsl::{SpacingDsl, SpacingRule, SpaceLoc, SpaceValue};
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};

use ra_syntax::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};

use std::collections::{HashMap, HashSet};

// TODO make more like intellij's fmt model
// Model holds immutable tree and mutable intermediate model to produce diff
// the model will probably have to create its own tree to add the extra
// info to each token/node:
//
// [1,2,3];
// can be Brace token, ident, comma all of which knows their own rules and apply
// them accordingly to produce [1, 2, 3]; ???

#[derive(Clone, Debug)]
/// Whitespace holds all whitespace information for each Block.
/// Accessed from any Block's get_whitespace fn.
pub(crate) struct Whitespace {
    original: SyntaxToken,
    indent_spaces: u32,
    additional_spaces: u32,
}

impl Whitespace {
    fn new(token: SyntaxToken) -> Whitespace {
        let additional_spaces = if token.kind() == WHITESPACE {
            let len = token.text_range();
            (len.end() - len.start()).into()
        } else {
            0
        };
        Self {
            original: token,
            indent_spaces: 0,
            additional_spaces,
        }
    }
}

#[derive(Clone, Debug)]
/// Holds nodes and tokens as a tree with whitespace information
/// 
pub(crate) struct Block {
    //indent: some enum?
    element: SyntaxElement,
    children: Vec<Block>,
    text: SmolStr,
    range: TextRange,
    whitespace: Option<Whitespace>,
}

// each block will have knowledge of spacing and indent, 
impl Block {
    pub(crate) fn build_block(element: SyntaxElement) -> Block {
        // recursivly add to children
        let children = match &element {
            NodeOrToken::Node(node) => {
                node.children_with_tokens()
                    .map(Self::build_block)
                    .collect::<Vec<_>>()
            },
            NodeOrToken::Token(_) => {
                vec![]
            }
        };
        let range = match &element {
            NodeOrToken::Node(node) => node.text_range(),
            NodeOrToken::Token(token) => token.text_range()
        };
        let text = match &element {
            NodeOrToken::Node(node) => SmolStr::from(node.text().to_string()),
            NodeOrToken::Token(token) => token.text().clone()
        };
        let whitespace = if let NodeOrToken::Token(token) = &element {
            token.prev_token().and_then(|prev_tkn| {
                // does it make sense to create whitespace if token is not ws??
                if prev_tkn.kind() == WHITESPACE{
                    Some(Whitespace::new(prev_tkn))
                } else {
                    None
                }
            })
        } else {
            None
        };

        Self {
            element,
            text,
            children,
            range,
            whitespace,
        }
    }

    /// Compare pointers to check if two Blocks are equal.
    fn compare(&self, other: &Block) -> bool {
        self as *const _ == other as *const _
    }

    /// Returns an iterator of children from current element.
    fn children(&self) -> impl Iterator<Item=&Block> {
        self.children.iter()
    }

    /// Traverse all blocks in order, convenience for order_flatten_blocks.
    pub(crate) fn traverse(&self) -> impl Iterator<Item=&Block> {
        Traversal { blocks: self.order_flatten_blocks(), idx: 0, }
    }

    /// Vec of all Blocks in order, parent then children.
    fn order_flatten_blocks(&self) -> Vec<&Block> {
        let mut blocks = vec![];
        for blk in self.children() {
            blocks.push(blk);
            if !blk.children.is_empty() {
                let mut kids = Block::order_flatten_blocks(blk);
                blocks.append(&mut kids);
            }
        };
        blocks
    }

    /// Returns `Whitespace` which has knowledge of whitespace around current token.
    pub(crate) fn get_spacing(&self, tkn: SyntaxToken) -> Option<&Whitespace> {
        // TODO walk tree find `tkn` then return matches whitespace
        self.whitespace.as_ref()
    }

    /// Remove after dev
    fn to_string(&self) -> String {
        self.traverse().map(|blk| blk.text.to_string()).collect::<String>()
    }
}

#[derive(Debug, Clone)]
/// Traversal struct is the Iterator for flattened
/// ordered Block's, needed to fixes lifetime issue when
/// returning impl Iterator<_> for Block and EditTree.
pub(super) struct Traversal<'t> {
    blocks: Vec<&'t Block>,
    idx: usize,
}
impl<'t> Iterator for Traversal<'t> {
    type Item = &'t Block;

    fn next(&mut self) -> Option<Self::Item> {
        self.idx += 1;
        // copied otherwise we have a &&Block
        self.blocks.get(self.idx - 1).copied()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EditTree {
    root: Block,
}

impl EditTree {
    pub(crate) fn new(root: SyntaxNode) -> Self {
        EditTree::build_tree(root)
    }

    fn build_tree(root: SyntaxNode) -> EditTree {
        let ele = NodeOrToken::Node(root.clone());
        let root = Block::build_block(ele);
        EditTree { root }
    }

    pub(crate) fn walk(&self) -> Traversal {
        Traversal { blocks: self.root.order_flatten_blocks(), idx: 0, }
    }

    /// only for dev, we dont need to convert or diff in editTree
    pub(crate) fn to_string(&self) -> String {
        self.root.traverse().map(|blk| blk.text.to_string()).collect::<String>()
    }
}
