use crate::dsl::{SpacingDsl, SpacingRule, SpaceLoc, SpaceValue};
use crate::edit_tree::EditTree;
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};

use ra_syntax::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};

pub(crate) struct DiffView {
    // some diffing info not just a string
    diffs: Vec<String>
}

#[derive(Debug)]
/// 
pub(crate) struct FmtDiff {
    edit_tree: EditTree,
    patterns: PatternSet,
    diff: DiffView,
}

impl FmtDiff {
    pub(crate) fn new(edit_tree: EditTree) -> Self {
        let space_rules = spacing();
        let patterns = PatternSet::new(space.rules.iter());
        Self { edit_tree, patterns, }
    }

    pub(crate) fn compare(&self) -> DiffView {
        
    }
}
