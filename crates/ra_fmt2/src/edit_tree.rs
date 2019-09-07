use crate::dsl::{Space, SpaceLoc, SpaceValue, SpacingDsl, SpacingRule};
// use crate::indent::Indentation;
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};
use crate::whitespace::Whitespace;

use ra_syntax::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *}, Direction,
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};

use std::collections::BTreeSet;
use std::cell::{Cell, RefCell};
use std::fmt::Write;

// TODO make more like intellij's fmt model
// Model holds immutable tree and mutable intermediate model to produce diff.
#[derive(Clone, Debug)]
/// Holds node or token with whitespace information.
///
pub(crate) struct Block {
    //indent: some enum?
    element: SyntaxElement,
    children: Vec<Block>,
    text: SmolStr,
    range: TextRange,
    whitespace: RefCell<Whitespace>,
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

/// Block abstracts every node and token in a `SourceFile` of SyntaxElement`s, keeping indent
/// and `Whitespace` information for later formatting.
impl Block {
    /// Returns `Block` from either `SyntaxNode` or `SyntaxToken`.
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

        let whitespace = RefCell::new(Whitespace::new(&element));

        let whitespace = Rc::new(RefCell::new(Whitespace::new(&element)));
        let indentation = Rc::new(RefCell::new(Indentation::new(&element)));

    /// Creates a non connected node for checking anchoring node's indentation.
    fn build_single(node: SyntaxElement) -> Block {
        let range =  node.text_range();
        let text = match &node {
            NodeOrToken::Node(node) => SmolStr::from(node.text().to_string()),
            NodeOrToken::Token(token) => token.text().clone(),
        };
        let whitespace = RefCell::new(Whitespace::new(&node));

        Self {
            element: node,
            text,
            children: vec![],
            range,
            whitespace,
        }
    }

    /// Compare pointers to check if two Blocks are equal.
    /// Remove??
    fn compare(&self, other: &Block) -> bool {
        self as *const _ == other as *const _
    }

    /// FIX probably not the best way to do this, building all new Blocks.
    pub(crate) fn siblings(&self) -> Vec<Block> {
        match &self.element {
            NodeOrToken::Node(node) => {
                node.siblings_with_tokens(Direction::Prev)
                    .map(Block::build_single)
                    .collect::<Vec<_>>()
            },
            NodeOrToken::Token(token) => {
                token.siblings_with_tokens(Direction::Prev)
                    .map(Block::build_single)
                    .collect::<Vec<_>>()
            },
        }
    }
    /// Returns an vec of ancestors from current element.
    /// TODO is the box better than a vec????????
    /// FIX probably not the best way to do this, building all new Blocks.
    pub(crate) fn ancestor_nodes(&self) -> Box<dyn Iterator<Item=Block>> {
        match &self.element {
            NodeOrToken::Node(node) => {
                let ret = std::iter::successors(node.parent(), |this| {
                    this.parent()
                })
                .map(|n| Block::build_single(NodeOrToken::Node(n)));
                Box::new(ret)
                //.collect::<Vec<_>>()
            },
            NodeOrToken::Token(token) => {
                let ret = std::iter::successors(Some(token.parent()), |this| {
                    this.parent()
                })
                .map(|n| Block::build_single(NodeOrToken::Node(n)));
                Box::new(ret)
                //.collect::<Vec<_>>()
            },
        }
    }

    /// Traverse all blocks in order including current.
    pub(crate) fn traverse_inc(&self) -> impl Iterator<Item = &Block> {
        Traversal { blocks: self.order_flatten_blocks_inc(), idx: 0 }
    }

    /// Traverse all blocks in order excluding current.
    pub(crate) fn traverse_exc(&self) -> impl Iterator<Item = &Block> {
        Traversal { blocks: self.order_flatten_blocks_exc_curr(), idx: 0 }
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
    fn order_flatten_blocks_exc_curr(&self) -> Vec<&Block> {
        let mut blocks = vec![];
        for blk in self.children() {
            blocks.push(blk);
            if !blk.children.is_empty() {
                // we only want to exlcude the root
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

    /// Vec of `Blocks` containing nodes, in order.
    fn order_flatten_blocks_nodes(&self) -> Vec<&Block> {
        let mut blocks = vec![self];
        for blk in self.children() {
            if blk.as_element().as_node().is_some() {
                blocks.push(blk);
            }
            if !blk.children.is_empty() {
                let mut kids = Block::order_flatten_blocks_nodes(blk);
                blocks.append(&mut kids);
            }
        }
        blocks
    }

    /// Text range of current token.
    pub(crate) fn text_range(&self) -> TextRange {
        self.range
    }

    /// Returns an iterator of children from current element.
    pub(crate) fn children(&self) -> impl Iterator<Item = &Block> {
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

    pub(crate) fn is_leaf(&self) -> bool {
        self.element.as_token().is_some()
    }

    /// Returns `Whitespace` which has knowledge of whitespace around current token.
    pub(crate) fn get_whitespace(&self) -> RefCell<Whitespace> {
        self.whitespace.clone()
    }

    /// Returns amount indenting whitespace.
    pub(crate) fn get_indent(&self) -> u32 {
        if self.whitespace.borrow().starts_with_lf {
            self.whitespace.borrow().text_len.0 
        } else {
            0
        }
    }

    /// Sets amount indenting whitespace.
    pub(crate) fn set_indent(&self, indent: u32) {
        self.whitespace.borrow_mut().text_len.0 = indent
    }

    /// Returns previous and next space amounts as tuple.
    pub(crate) fn space_value(&self) -> (u32, u32) {
        self.whitespace.borrow().text_len
    }

    /// Sets preceding spacing based on rule.
    pub(crate) fn set_spacing_before(&self, rule: &SpacingRule) {
        let mut whitespace = self.whitespace.borrow_mut();
        match rule.space.value {
            SpaceValue::Single => {
                whitespace.text_len.0 = 1;
                whitespace.new_line.0 = false;
            },
            SpaceValue::Newline => {
                whitespace.new_line.0 = true;
                whitespace.text_len.0 = 0;
;            },
            SpaceValue::SingleOptionalNewline => {
                if whitespace.siblings_contain("\n") {
                    whitespace.new_line.0 = true;
                    whitespace.text_len.0 = 0;
                } else {
                    whitespace.text_len.0 = 1;
                    whitespace.new_line.0 = false;
                }
            },
            _ => {},
        };
        //self.whitespace.borrow_mut().fix_spacing_before(rule.space)
    }

    /// Sets spacing after token based on rule.
    /// Fixes the "{" in "struct Test{x:usize}"
    pub(crate) fn set_spacing_after(&self, rule: &SpacingRule) {
        self.whitespace.borrow_mut().fix_spacing_after(rule.space)
    }

    /// Returns previous and next new line flags as tuple.
    pub(crate) fn eol_value(&self) -> (bool, bool) {
        self.whitespace.borrow().new_line
    }
    /// Returns true if `Block` starts with new line char.
    pub(crate) fn starts_with_lf(&self) -> bool {
        self.whitespace.borrow().starts_with_lf
    }

    /// Walks siblings to search for pat.
    pub(crate) fn siblings_contain(&self, pat: &str) -> bool {
        if let Some(tkn) = self.element.as_token() {
            walk_tokens(&tkn.parent())
                .any(|tkn| {
                    tkn.text().as_str() == pat
                })
        } else {
            false
        }
    }

    /// Remove after dev ??
    fn to_string(&self) -> String {
        self.text.to_string()
    }

    /// Returns `Block`s text as str.
    pub(crate) fn as_str(&self) -> &str {
        self.text.as_str()
    }
}

#[derive(Debug, Clone)]
/// Traversal is the Iterator for flattened
/// ordered Block's, needed to fix lifetime issue when
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
    pub(crate) fn new(root: SyntaxNode) -> EditTree {
        EditTree::build_tree(root)
    }
    fn build_tree(root: SyntaxNode) -> EditTree {
        let ele = NodeOrToken::Node(root);
        let root = Block::build_block(ele);
        EditTree { root }
    }
    /// Returns the last token when ordered and flattened.
    pub(crate) fn last_token(&self) -> Option<&Block> {
        self.walk_tokens().last()
    }
    /// Walk all blocks including root.
    pub(crate) fn walk(&self) -> Traversal {
        Traversal { blocks: self.root.order_flatten_blocks_inc(), idx: 0 }
    }
    /// Walk blocks that represent tokens.
    pub(crate) fn walk_tokens(&self) -> Traversal {
        Traversal { blocks: self.root.order_flatten_blocks_tokens(), idx: 0 }
    }
    /// Walk blocks that represent nodes.
    pub(crate) fn walk_nodes(&self) -> Traversal {
        Traversal { blocks: self.root.order_flatten_blocks_nodes(), idx: 0 }
    }
    /// Walk all blocks excluding root.
    pub(crate) fn walk_exc_root(&self) -> Traversal {
        Traversal { blocks: self.root.order_flatten_blocks_exc_curr(), idx: 0 }
    }

    /// Returns the SmolStr of the root node, the whole text
    pub(crate) fn text(&self) -> SmolStr {
        self.root.text.clone()
    }

    /// Walks tokens and compares `Whitespace` to build the final String from `Blocks`.
    pub(crate) fn tokens_to_string(&self) -> Result<String, std::fmt::Error> {
        let mut ret = str_from_root(&self.root);
        if self.root.kind() == SOURCE_FILE && !ret.ends_with('\n') {
            ret.push('\n');
        }
        Ok(ret)
    }
}

fn str_from_root(block: &Block) -> String {
    let mut buff = String::new();
    eat_tkns(block, &mut buff);
    // check file ends with "\n"
    
    return buff;

    fn eat_tkns(block: &Block, mut buff: &mut String) {
        if block.is_leaf() {
            write!(buff, "{}", block.whitespace.borrow()).expect("write to buffer failed");
            write!(buff, "{}", block.element).expect("write to buffer failed");
        } else {
            block.children().for_each(|kid| eat_tkns(kid, &mut buff));
        }
    }
} 
