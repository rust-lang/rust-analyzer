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
use std::cell::{Cell, RefCell};
use std::rc::Rc;

// TODO make more like intellij's fmt model
// Model holds immutable tree and mutable intermediate model to produce diff
// the model will probably have to create its own tree to add the extra
// info to each token/node:
//
// [1,2,3];
// can be Brace token, ident, comma all of which knows their own rules and apply
// them accordingly to produce [1, 2, 3]; ???

#[derive(Clone, Debug, Copy)]
/// Whitespace holds all whitespace information for each Block
pub(crate) struct Whitespace<'w> {
    original: &'w SyntaxToken,
    indent_spaces: u32,
    additional_spaces: u32,
}

impl<'w> Whitespace<'w> {
    fn new(token: &'w SyntaxToken) -> Whitespace<'w> {
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
    element: &'b SyntaxElement,
    text: SmolStr,
    parent: Cell<Option<&'b Block<'b>>>,
    next_sib: Cell<Option<&'b Block<'b>>>,
    first_child: Cell<Option<&'b Block<'b>>>,
    range: TextRange,
    prev_whitespace: Option<Whitespace<'b>>,
}

// each block will have knowledge of spacing and indent, 
impl<'b> Block<'b> {
    pub(crate) fn build_block<'a>(element: &'b SyntaxElement) -> Block<'b> {
        // recursivly add to children
        let first_child = match &element {
            NodeOrToken::Node(node) => {
                if let Some(kid) = node.first_child_or_token() {
                    let child = Block::build_block(&kid);
                    Some(child)
                } else {
                    None
                }
            },
            NodeOrToken::Token(_) => {
                None
            }
        };
        let next_sib = if let Some(s) = element.next_sibling_or_token() {
            let sib_block = Block::build_block(&s);
            Some(sib_block)
        } else {
            None
        };
        let parent = if let Some(node) = element.parent() {
            let p = Self::build_block(&NodeOrToken::Node(node));
            Cell::new(Some(&p))
        } else {
            Cell::new(None)
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
                    Some(Whitespace::new(&tkn))   
                } else {
                    None
                }
            })
        } else {
            None
        };

        Self {
            element: &element,
            text,
            parent,
            first_child: Cell::new(first_child.as_ref()),
            next_sib: Cell::new(next_sib.as_ref()),
            range,
            prev_whitespace,
        }
    }

    /// Compare pointers to check if two Blocks are equal.
    fn compare(&self, other: &Block<'b>) -> bool {
        self as *const _ == other as *const _
    }

    /// Returns an iterator of parents from current element.
    fn ancestors(&self) -> Parents<'b> {
        Parents( self.parent.get() )
    }

    /// Returns an iterator of any sibling nodes and tokens from current element.
    fn siblings_with_tokens(&self) -> NextSibling<'b> {
        NextSibling( self.next_sib.get() )
    }

    /// Walk all the blocks 
    fn traverse(&'b self) -> WalkBlocks<'b> {
        WalkBlocks { root: self, next: Some(Branch::Continue(self)) }
    }

    /// Returns `Whitespace` which has knowledge of whitespace around current token.
    pub(crate) fn get_spacing(&self, tkn: &'b SyntaxToken) -> Whitespace<'b> {
        Whitespace::new(tkn)
    }

    /// Remove after dev
    fn to_string(&self) -> String {
        self.text.to_string()
    }
}

pub(crate) struct Parents<'p>(Option<&'p Block<'p>>);
impl<'p> Iterator for Parents<'p> {
    type Item = &'p Block<'p>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(blk) = self.0.take() {
            self.0 = blk.parent.get();
            Some(blk)
        } else {
            None
        }
    }
}

pub(crate) struct NextSibling<'s>(Option<&'s Block<'s>>);
impl<'s> Iterator for NextSibling<'s> {
    type Item = &'s Block<'s>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(blk) = self.0.take() {
            self.0 = blk.next_sib.get();
            Some(blk)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
/// Branch keeps track of where in the tree we are.
pub(crate) enum Branch<T> {
    /// At the begining or during child traversal.
    Continue(T),
    /// Terminal holds the next sibling in traversal after
    /// reaching the terminal child.
    Terminal(T),
}

impl<T> std::ops::Deref for Branch<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            Branch::Continue(t) => t,
            Branch::Terminal(t) => t,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct WalkBlocks<'b> {
    root: &'b Block<'b>,
    next: Option<Branch<&'b Block<'b>>>,
}

impl<'b> Iterator for WalkBlocks<'b> {
    type Item = Branch<&'b Block<'b>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(branch) = self.next.take() {
            self.next = match branch {
                Branch::Continue(block) => {
                    if let Some(child) = block.first_child.get() {
                        Some(Branch::Continue(child))
                    } else {
                        Some(Branch::Terminal(block))
                    }
                },
                Branch::Terminal(block) => {
                    // we have come back to root done
                    if block.compare(self.root) {
                        None
                    // reached end of children move on to next sibling
                    } else if let Some(sibling) = block.next_sib.get() {
                        Some(Branch::Continue(sibling))
                    // no child or sibling move up level to current Block's parent
                    } else if let Some(parent) = block.parent.get() {
                        // we reached the end of branch so current's parent's sibling is next
                        Some(Branch::Terminal(parent))
                    } else {
                        unreachable!("In Branch::Terminal( {:?} )", block)
                    }
                }
            };
            Some(branch)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EditTree<'e> {
    root: Block<'e>,
}

impl<'e> EditTree<'e> {
    pub(crate) fn new(root: SyntaxNode) -> Self {
        EditTree::build_tree(root)
    }

    fn build_tree<'a: 'e>(root: SyntaxNode) -> EditTree<'e> {
        let ele = NodeOrToken::Node(root.clone());
        let root = Block::build_block(&ele);
        EditTree { root }
    }

    /// only for dev, we dont need to convert or diff in editTree
    pub(crate) fn to_string(&'e self) -> String {
        self.root.traverse().map(|blk| blk.to_string()).collect::<String>()
        
    }
}
