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

#[derive(Debug)]
// this will probably be an enum that holds a struct for space/indent/ect
struct AtomEdit {
    space_loc: SpaceLoc,
    val: SpaceValue,
    start: TextUnit,
}

impl AtomEdit {}

#[derive(Debug)]
pub(crate) struct SynBlock {
    //indent: some enum?
    kind: SyntaxKind,
    node: SyntaxNode,
    tokens: Vec<SyntaxToken>,
    range: TextRange,
    edits: Vec<AtomEdit>,
    ws_rules: Vec<SpacingRule>,
}

#[derive(Debug)]
pub(crate) struct EditTree {
    blocks: Vec<SynBlock>,
    flat_edit: Vec<SyntaxElement>,
    edits: Vec<AtomEdit>,
}

// each block will have knowledge of spacing and indent, 
// 
impl SynBlock {
    pub(crate) fn build_block(
        node: SyntaxNode,
        // this must change when indent comes back to generic?
        patt: &PatternSet<&'_ SpacingRule>,
    ) -> Self {
        // are there node rules?
        let mut ws_rules = patt.matching(NodeOrToken::Node(node.clone()))
            .cloned()
            .collect::<Vec<_>>();
        
        let mut token_rules = walk_tokens(&node)
            .flat_map::<Vec<_>, _>(|t| patt.matching(NodeOrToken::Token(t)).collect())
            // TODO get rid of clone?
            .cloned()
            .collect::<Vec<SpacingRule>>();
        
        ws_rules.append(&mut token_rules);
        //println!("{:?}", ws_rules);
        Self {
            kind: node.kind(),
            node: node.clone(),
            tokens: walk_tokens(&node).collect(),
            range: node.text_range(),
            edits: vec![],
            ws_rules,
        }
    }
}

impl Default for EditTree {
    fn default() -> Self {
        Self {
            blocks: vec![],
            flat_edit: vec![],
            edits: vec![],
        }
    }
}

impl EditTree {
    pub(crate) fn from_root(root: &SyntaxNode, patt: PatternSet<&'_ SpacingRule>) -> Self {
        Self::default().build_tree(root, patt)
    }

    pub(crate) fn build_tree(
        mut self,
        root: &SyntaxNode,
        patt: PatternSet<&'_ SpacingRule>,
    ) -> Self {
        self.blocks = walk_nodes(root).map(|n| SynBlock::build_block(n, &patt)).collect();
        self
    }

    pub(crate) fn to_string(&self) -> String {
        let ordered = walk_tokens(&self.blocks[0].node)
            .map(|t| t.to_string())
            .collect::<String>();
        //let set: HashSet<SyntaxElement> = HashSet::from_iter(ordered);
        println!("{:?}", ordered);
        ordered
    }
}
