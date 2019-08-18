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
    diffs: Vec<(usize, SmolStr)>,
    original: SmolStr,
}

impl DiffView {
    fn apply(&mut self, block: &Block, rule: &SpacingRule) {
        match rule.space.loc {
            SpaceLoc::After => self.apply_after(block, rule.space.value),
            SpaceLoc::Before => self.apply_before(block, rule.space.value),
            SpaceLoc::Around => self.apply_around(block, rule.space.value),
        }
    }

    fn apply_after(&mut self, block: &Block, value: SpaceValue) {
        match value {
            SpaceValue::Single => {
                let after = block.text_range().end().to_usize();
                self.diffs.push((after, SmolStr::from(" ")))
            },
            SpaceValue::Newline => {
                let after = block.text_range().end().to_usize();
                self.diffs.push((after, SmolStr::from("\n")))
            },
            SpaceValue::SingleOptionalNewline => {
                let before = block.text_range().end().to_usize();

                if !block.siblings_contain("\n") {
                    println!("SIBLINGS CONTAIN FALSE");
                    self.diffs.push((before, SmolStr::from(" ")))
                } else {
                    println!("SIBLINGS CONTAIN TRUE");
                    self.diffs.push((before, SmolStr::from("\n")))
                }
            },
            _ => {},
        }
    }

    fn apply_before(&mut self, block: &Block, value: SpaceValue) {
        match value {
            SpaceValue::Single => {
                let before = block.text_range().start().to_usize();
                self.diffs.push((before, SmolStr::from(" ")))
            },
            SpaceValue::Newline => {
                let before = block.text_range().start().to_usize();
                self.diffs.push((before, SmolStr::from("\n")))
            },
            SpaceValue::SingleOptionalNewline => {
                let before = block.text_range().start().to_usize();

                if !block.siblings_contain("\n") {
                    println!("SIBLINGS CONTAIN FALSE");
                    self.diffs.push((before, SmolStr::from(" ")))
                } else {
                    println!("SIBLINGS CONTAIN TRUE");
                    self.diffs.push((before, SmolStr::from("\n")))
                }
            },
            _ => {},
        }
    }

    fn apply_around(&mut self, block: &Block, value: SpaceValue) {
        match value {
            SpaceValue::Single => {
                let before = block.text_range().start().to_usize();
                let after = block.text_range().end().to_usize();

                let mut pair = vec![(before, SmolStr::from(" ")),(after, SmolStr::from(" "))];
                self.diffs.append(&mut pair)
            },
            SpaceValue::Newline => {
                let before = block.text_range().start().to_usize();
                let after = block.text_range().end().to_usize();
                
                let mut pair = vec![(before, SmolStr::from("\n")),(after, SmolStr::from("\n"))];
                self.diffs.append(&mut pair)
            },
            _ => {},
        }
    }
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

    /// Checks if `Whitespace` and `SpacingRule` match then mutates `DiffView`.
    /// 
    /// # Arguments
    ///
    /// * `block` - A &Block that is always a token.
    /// * `rule` - A &SpaceRule.
    fn check_spacing(&self, rule: &SpacingRule, block: &Block) {
        // refcell for here for mutating DiffView?
        if let Some(whitespace) = block.get_spacing() {
            // is this a terible idea impl-ing eq??
            if whitespace != rule {
                self.diff.borrow_mut().apply(block, rule);
            }
        } else {
            self.diff.borrow_mut().apply(block, rule);
        }
    }

    pub(crate) fn spacing_diff(self) -> DiffView {
        let space_rules = spacing();
        let spacing = PatternSet::new(space_rules.rules.iter());

        for block in self.edit_tree.walk() {
            for rule in spacing.matching(block.to_element()) {
                // creates DiffView
                self.check_spacing(rule, block)
            }
        }
        self.diff.into_inner()
    }
}
