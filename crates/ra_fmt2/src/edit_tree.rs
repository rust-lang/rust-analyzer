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
use std::cell::RefCell;
use std::rc::Rc;

// TODO make more like intellij's fmt model
// Model holds immutable tree and mutable intermediate model to produce diff
// the model will probably have to create its own tree to add the extra
// info to each token/node:
//
// [1,2,3];
// can be Brace token, ident, comma all of which knows their own rules and apply
// them accordingly to produce [1, 2, 3]; ???

#[derive(Clone, Debug)]
/// Whitespace holds all whitespace information for each Block
pub(crate) struct Whitespace {
    original: SyntaxToken,
    indent_spaces: u32,
    additional_spaces: u32,
}

impl Whitespace {
    fn new(token: SyntaxToken) -> Self {
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
pub(crate) struct Block<'b> {
    //indent: some enum?
    element: SyntaxElement,
    text: SmolStr,
    parent: Option<Rc<Block<'b>>>,
    next_sib: Option<&'b Block<'b>>,
    first_child: Option<&'b Block<'b>>,
    range: TextRange,
    prev_whitespace: Option<Whitespace>,
}

// each block will have knowledge of spacing and indent, 
impl<'b> Block<'b> {
    pub(crate) fn build_block(element: SyntaxElement) -> Self {
        // recursivly add to children
        let first_child = match &element {
            NodeOrToken::Node(node) => {
                if let Some(kid) = node.first_child_or_token() {
                    Rc::new(Some(Block::build_block(kid)))
                } else {
                    Rc::new(None)
                }
            },
            NodeOrToken::Token(_) => {
                None
            }
        };

        let parent = if let Some(node) = element.parent() {
            let p = Self::build_block(NodeOrToken::Node(node));
            Rc::new(Some(p))
        } else {
            Rc::new(None)
        };
        let range = match &element {
            NodeOrToken::Node(node) => node.text_range(),
            NodeOrToken::Token(token) => token.text_range()
        };
        let text = match &element {
            NodeOrToken::Node(node) => SmolStr::from(node.text().to_string()),
            NodeOrToken::Token(token) => token.text().clone()
        };
        let prev_whitespace = if let NodeOrToken::Token(token) = &element {
            token.prev_token().and_then(|tkn| {
                // does it make sense to create whitespace if token is not ws
                if tkn.kind() == WHITESPACE{
                    Some(Whitespace::new(tkn))   
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
            parent,
            first_child,
            range,
            prev_whitespace,
        }
    }

    fn children(&self) -> impl Iterator<Item=&Block> {
        self.children.iter()
    }

    fn parent(&self) -> Option<&Block> {
        self.parent
    }

    pub(crate) fn walk_blocks(&self) -> impl Iterator<Item=&Block> {
        IterBlock {
            root: self,
            current: None,
            children: &self.children,
            next: IterKid::new(&self),
            idx: 0,
            root_flag: true,
        }
    }

    pub(crate) fn get_spacing(&self, tkn: &SyntaxToken) -> Whitespace {
        Whitespace::new(tkn.clone())
    }

    /// Remove after dev?
    fn to_string(&self) -> String {
        self.text.to_string()
    }
}

pub(crate) struct IterKid<'k> {
    flat_ord: Vec<&'k Block<'k>>,
    idx: usize,
}

impl<'k> IterKid<'k> {
    fn new(current: &'k Block) -> Self {
        let flat_ord = current.children()
            .map(IterKid::new)
            .flatten()
            .collect();

        Self { flat_ord, idx: 0, }
    }
}

impl<'k> Iterator for IterKid<'k> {
    type Item = &'k Block<'k>;
    fn next(&mut self) -> Option<Self::Item> {
        self.idx += 1;
        self.flat_ord.get(self.idx - 1).map(|blk| *blk)
    }
}

pub(crate) struct IterBlock<'b> {
    root: &'b Block<'b>,
    current: Option<&'b Block<'b>>,
    next: IterKid<'b>,
    children: &'b [Block<'b>],
    idx: usize,
    root_flag: bool,
}

impl<'b> Iterator for IterBlock<'b> {
    type Item = &'b Block<'b>;

    fn next(&mut self) -> Option<Self::Item> {
        // return root first
        if self.root_flag {
            self.root_flag = false;
            self.children = &self.root.children;
            self.current = self.children.get(self.idx);
            self.current
        } else {
            self.next.next()
        }
    }
}

#[derive(Debug)]
pub(crate) struct EditTree<'e> {
    root: Block<'e>,
}

impl<'e> EditTree<'e> {
    pub(crate) fn new(root: &SyntaxNode) -> Self {
        EditTree::build_tree(root)
    }

    fn build_tree(root: &SyntaxNode) -> Self {
        let space = spacing();
        let ws_rules = PatternSet::new(space.rules.iter());

        let root = Block::build_block(NodeOrToken::Node(root.clone()));
        EditTree { root }
    }

    /// only for dev, we dont need to convert or diff in editTree
    pub(crate) fn to_string(&self) -> String {
        self.root.children.iter().map(|blk| blk.to_string()).collect::<String>()
        
    }
}
