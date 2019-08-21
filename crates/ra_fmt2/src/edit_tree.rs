use crate::dsl::{Space, SpaceLoc, SpaceValue, SpacingDsl, SpacingRule};
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};
use crate::whitespace::{Whitespace};

use ra_syntax::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};

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
/// Holds nodes and tokens as a tree with whitespace information
///
pub(crate) struct Block {
    //indent: some enum?
    element: SyntaxElement,
    children: Vec<Block>,
    text: SmolStr,
    range: TextRange,
    whitespace: Rc<RefCell<Whitespace>>,
}

impl Eq for Block {}
impl PartialEq for Block {
    fn eq(&self, rhs: &Block) -> bool {
        self.range == rhs.range && self.text == rhs.text
        && self.element == rhs.element
    }
}

impl Ord for Block {
    fn cmp(&self, rhs: &Block) -> std::cmp::Ordering {
        self.range.start().cmp(&rhs.range.start())
    }
}

impl PartialOrd for Block {
    fn partial_cmp(&self, rhs: &Block) -> Option<std::cmp::Ordering> {
        self.range.start().partial_cmp(&rhs.range.start())
    }
}

// each block will have knowledge of spacing and indent,
impl Block {
    pub(crate) fn build_block(element: SyntaxElement) -> Block {
        // recursivly add to children
        let children = match &element {
            NodeOrToken::Node(node) => {
                node.children_with_tokens()
                .filter(|ele| match ele{
                    NodeOrToken::Node(_) => true,
                    NodeOrToken::Token(t) => t.kind() != WHITESPACE,
                })
                .map(Block::build_block)
                .collect::<Vec<_>>()
            }
            NodeOrToken::Token(_) => vec![],
        };
        let range = match &element {
            NodeOrToken::Node(node) => node.text_range(),
            NodeOrToken::Token(token) => token.text_range(),
        };
        let text = match &element {
            NodeOrToken::Node(node) => SmolStr::from(node.text().to_string()),
            NodeOrToken::Token(token) => token.text().clone(),
        };

        let whitespace = Rc::new(RefCell::new(Whitespace::new(&element)));

        Self { element, text, children, range, whitespace }
    }

    /// Compare pointers to check if two Blocks are equal.
    /// Remove??
    fn compare(&self, other: &Block) -> bool {
        self as *const _ == other as *const _
    }

    /// Text range of current token.
    pub(crate) fn text_range(&self) -> TextRange {
        self.range
    }

    /// Returns an iterator of children from current element.
    fn children(&self) -> impl Iterator<Item = &Block> {
        self.children.iter()
    }

    /// Returns SyntaxKind.
    pub(crate) fn kind(&self) -> SyntaxKind {
        self.element.kind()
    }

    /// Returns an owned `SyntaxElement`.
    pub(crate) fn to_element(&self) -> SyntaxElement {
        self.element.clone()
    }

    /// Returns a reference to a `SyntaxElement`.
    pub(crate) fn as_element(&self) -> &SyntaxElement {
        &self.element
    }

    /// Traverse all blocks in order including current, convenience for order_flatten_blocks.
    pub(crate) fn traverse_inc(&self) -> impl Iterator<Item = &Block> {
        Traversal { blocks: self.order_flatten_blocks_inc(), idx: 0 }
    }

    /// Traverse all blocks in order excluding current, convenience for order_flatten_blocks.
    pub(crate) fn traverse_exc(&self) -> impl Iterator<Item = &Block> {
        Traversal { blocks: self.order_flatten_blocks_exc(), idx: 0 }
    }

    /// Vec of all Blocks in order including current, parent then children.
    fn order_flatten_blocks_inc(&self) -> Vec<&Block> {
        let mut blocks = vec![self];
        for blk in self.children() {
            blocks.push(blk);
            if !blk.children.is_empty() {
                let mut kids = Block::order_flatten_blocks_inc(blk);
                blocks.append(&mut kids);
            }
        }
        blocks
    }

    /// Vec of all Blocks in order excluding current, parent then children.
    fn order_flatten_blocks_exc(&self) -> Vec<&Block> {
        let mut blocks = vec![self];
        for blk in self.children() {
            blocks.push(blk);
            if !blk.children.is_empty() {
                let mut kids = Block::order_flatten_blocks_inc(blk);
                blocks.append(&mut kids);
            }
        }
        blocks
    }

    /// Vec of `Blocks` containing tokens, in order.
    fn order_flatten_blocks_tokens(&self) -> Vec<&Block> {
        let mut blocks = vec![];
        for blk in self.children() {
            if blk.as_element().as_token().is_some() {
                blocks.push(blk);
            }
            if !blk.children.is_empty() {
                let mut kids = Block::order_flatten_blocks_tokens(blk);
                blocks.append(&mut kids);
            }
        }
        blocks
    }

    /// Returns `Whitespace` which has knowledge of whitespace around current token.
    pub(crate) fn get_spacing(&self) -> Rc<RefCell<Whitespace>> {
        Rc::clone(&self.whitespace)
    }

    /// Returns previous and next space amounts as tuple.
    pub(crate) fn space_value(&self) -> (u32, u32) {
        self.whitespace.borrow().locations
    }

    /// Returns previous and next new line flags as tuple.
    pub(crate) fn eol_value(&self) -> (bool, bool) {
        self.whitespace.borrow().new_line
    }

    /// Returns &mut `Whitespace` which has knowledge of whitespace around current token.
    // pub(crate) fn get_spacing_mut(&self) -> &mut Whitespace {
    //     &mut self.whitespace.borrow_mut()
    // }

    /// Remove after dev
    fn to_string(&self) -> String {
        self.text.to_string()
    }

    /// Remove after dev
    fn as_str(&self) -> &str {
        self.text.as_str()
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
    /// Walks all `SyntaxNode`s building an `EditTree`. 
    pub(crate) fn new(root: SyntaxNode) -> Self {
        EditTree::build_tree(root)
    }
    fn build_tree(root: SyntaxNode) -> EditTree {
        let ele = NodeOrToken::Node(root.clone());
        let root = Block::build_block(ele);
        EditTree { root }
    }
    pub(crate) fn root(&self) -> &Block {
        &self.root
    }
    /// Walk all blocks including root.
    pub(crate) fn walk(&self) -> Traversal {
        Traversal { blocks: self.root.order_flatten_blocks_inc(), idx: 0 }
    }
    /// Walk blocks that represent tokens.
    pub(crate) fn walk_tokens(&self) -> Traversal {
        Traversal { blocks: self.root.order_flatten_blocks_tokens(), idx: 0 }
    }
    /// Walk all blocks excluding root.
    pub(crate) fn walk_exc_root(&self) -> Traversal {
        Traversal { blocks: self.root.order_flatten_blocks_exc(), idx: 0 }
    }

    /// Returns the SmolStr of the root node, the whole text
    pub(crate) fn text(&self) -> SmolStr {
        self.root.text.clone()
    }

    /// only for dev, we dont need to convert or diff in editTree
    pub(crate) fn to_string(&self) -> String {
        let mut traverse = self.walk_exc_root().peekable();
        let de_dup = self.walk_tokens().cloned().collect::<std::collections::BTreeSet<_>>();
        let mut iter_clone = de_dup.iter();
        iter_clone.next();
        iter_clone.next();

        traverse.peek();
        let first = traverse.peek().cloned();

        traverse.scan(first, |next, blk| {
            let res = match blk.as_element() {
                NodeOrToken::Token(tkn) => {
                    if tkn.kind() != WHITESPACE {
                        let ret = string_from_block(&blk, next);
                        *next = iter_clone.next();
                        ret
                    } else {
                        "".into()
                    }
                },
                _ => {
                    "".into()
                },
            };
            Some(res)
        })
        .map(|b| b)
        .collect::<String>()
    }
}

fn string_from_block(blk: &Block, next: &mut Option<&Block>) -> String {
    //println!{"BLK {:#?}\nNEXT {:#?}", blk, next}
    let mut ret = String::default();
    let (prev_s, next_s) = blk.space_value();
    let (prev_n, next_n) = blk.eol_value();

    // if new line
    if prev_n {
        ret.push('\n');
    // else push space
    } else {
        ret.push_str(&" ".repeat(prev_s as usize));
    }
    // add text token
    ret.push_str(blk.as_str());

    if let Some(block) = next {
        let (ps, _) = block.space_value();
        let (pn, _) = block.eol_value();

        // if the next token has no previous space but the current token has next space marked
        if ps == 0 && next_s > 0 {
            ret.push_str(&" ".repeat(next_s as usize));
        // same for new line add only if current says to and next does not
        } else if pn && !next_n {
            ret.push('\n');
        }
    }
    println!("{:?}", ret);
    ret
}
