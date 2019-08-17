use crate::dsl::{SpacingDsl, SpacingRule, SpaceLoc, SpaceValue};
use crate::edit_tree::{EditTree, Block};
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};

use ra_syntax::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};

use std::cell::RefCell;

#[derive(Debug)]
pub(crate) struct DiffView {
    // some diffing info not just a string
    diffs: Vec<String>
}

#[derive(Debug)]
/// 
pub(crate) struct FmtDiff<'d> {
    edit_tree: EditTree,
    spacing: PatternSet<&'d SpacingRule>,
    diff: RefCell<DiffView>,
}

impl<'d> FmtDiff<'d> {
    pub(crate) fn new(edit_tree: EditTree) -> Self {
        let space_rules = spacing();
        let spacing = PatternSet::new(space_rules.rules.iter());

        let diff = RefCell::new(DiffView { diffs: vec![], });

        Self { edit_tree, spacing, diff, }
    }

    fn check_spacing(&self, rule: &SpacingRule, block: &Block) {
        
    }

    pub(crate) fn compare(&self) -> DiffView {
        for block in self.edit_tree.walk() {
            for rule in self.spacing.matching(block.to_element()) {
                self.check_spacing(rule, block)
            }
        };
        self.diff.into_inner()
    }
}

impl SpacingRule {
    fn check_spacing(&self) 
}
