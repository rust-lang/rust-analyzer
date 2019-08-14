use crate::dsl::{SpacingDsl, SpacingRule, SpaceLoc, SpaceValue};
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};

use ra_syntax::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *}, Direction,
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
    pub(crate) fn build_block<P: AsRef<Pattern>>(
        element: SyntaxElement,
        p_set: &PatternSet<&P>,
    ) -> Self {
        let prev_whitespace = if let NodeOrToken::Token(token) = &element {
            println!("found Node {:?}", element);
            let patterns = p_set.matching(element.clone()).collect::<Vec<_>>();
            token.prev_token().map(|tkn| {
                if let Some(p) = patterns.iter().find(|pat| pat.as_ref().matches(&element)) {
                    println!("\n{:?}\n{:?}\n", p.as_ref(), tkn);
                    Some(Whitespace::new(tkn))
                } else {
                    None
                }
            }).unwrap_or(None)
        } else {
            None
        };
        // recursivly add to children
        let children = match &element {
            NodeOrToken::Node(node) => {
                node.children_with_tokens()
                    .map(|n| Self::build_block(n, &p_set))
                    .collect::<Vec<_>>()
            },
            NodeOrToken::Token(_) => {
                println!("IN TOKEN ET");
                vec![]
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

    // pub(crate) fn get_spacing(&self) -> Whitespace {

    // }

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
        let space = spacing();
        let ws_rules = PatternSet::new(space.rules.iter());

        self.blocks = SynBlock::build_block(NodeOrToken::Node(root.clone()), &ws_rules);
        self
    }

    /// only for dev, we dont need to convert or diff in editTree
    pub(crate) fn to_string(&self) -> String {
        self.blocks.walk().map(|blk| blk.to_string()).collect::<String>()
        
    }
}
