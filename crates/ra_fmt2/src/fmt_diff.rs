use crate::dsl::{SpaceLoc, SpaceValue, SpacingDsl, SpacingRule};
use crate::edit_tree::{Block, EditTree};
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};

use ra_syntax::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub(crate) struct DiffView {
    // TODO more diffing info not just a string
    diffs: Vec<SmolStr>,
    original: SmolStr,
}

#[derive(Debug, Clone)]
///
pub(crate) struct FmtDiff {
    edit_tree: EditTree,
    diff: RefCell<DiffView>,
}

impl FmtDiff {
    pub(crate) fn new(edit_tree: EditTree) -> Self {
        let original = edit_tree.text();
        let diff = RefCell::new(DiffView { diffs: vec![], original });

        Self { edit_tree, diff }
    }

    fn check_spacing(&self, rule: &SpacingRule, block: &Block) {
        // refcell for here for mutating DiffView?
        if let Some(whitespace) = block.get_spacing() {
            println!("WHITESPACE {:#?}", whitespace);
            // is this a terible idea
            if whitespace != rule {
                self.diff.borrow_mut().update(rule)
            }
        } else {
            // total diff
        }
    }

    pub(crate) fn spacing_diff(&self) -> DiffView {
        let space_rules = spacing();
        let spacing = PatternSet::new(space_rules.rules.iter());

        for block in self.edit_tree.walk() {
            for rule in spacing.matching(block.to_element()) {
                println!("BLOCK {:#?}\nRULE {:?}", block, rule);

                // creates DiffView
                self.check_spacing(rule, block)
            }
        }
        self.diff.clone().into_inner()
    }
}
