use crate::dsl::{self, SpaceLoc, SpaceValue, SpacingRule, SpacingDsl};
use crate::fmt_model::{FmtModel, SpaceBlock, BlockPosition};
use crate::pattern::PatternSet;
use crate::trav_util::{has_newline};

use ra_syntax::{
    ast::{self, AstNode, AstToken},
    Parse, SmolStr, SourceFile, SyntaxElement, SyntaxKind,
    SyntaxKind::*,
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};
use std::collections::HashMap;

impl SpaceLoc {
    fn is_before(self) -> bool {
        match self {
            SpaceLoc::Before | SpaceLoc::Around => true,
            SpaceLoc::After => false,
        }
    }
    fn is_after(self) -> bool {
        match self {
            SpaceLoc::After | SpaceLoc::Around => true,
            SpaceLoc::Before => false,
        }
    }
}

fn ensure_space(element: &SyntaxElement, block: &mut SpaceBlock, value: SpaceValue) {
    match value {
        SpaceValue::Single => block.set_text(" "),
        SpaceValue::SingleOptionalNewline => {
            if !block.has_newline() {
                block.set_text(" ")
            }
        }
        SpaceValue::Newline => block.set_text("\n"),
        SpaceValue::None => block.set_text(""),
        SpaceValue::NoneOptionalNewline => {
            if !block.has_newline() {
                block.set_text("")
            }
        }
        SpaceValue::SingleOrNewline => {
            let parent_is_multiline = element.parent().map_or(false, |it| has_newline(&it));
            if parent_is_multiline {
                block.set_line_break_preserving_existing_newlines()
            } else {
                block.set_text(" ")
            }
        }
        SpaceValue::NoneOrNewline => {
            let parent_is_multiline = element.parent().map_or(false, |it| has_newline(&it));
            if parent_is_multiline {
                block.set_line_break_preserving_existing_newlines()
            } else {
                block.set_text("")
            }
        }
    }
}

impl SpacingRule {
    pub(super) fn apply(&self, ele: &SyntaxElement, model: &mut FmtModel) {
        if !self.pattern.matches(ele) {
            return;
        }
        if self.space.loc.is_before() {
            let block = model.block_for(ele, BlockPosition::Before);
            ensure_space(ele, block, self.space.value);
        }
        if self.space.loc.is_after() {
            let block = model.block_for(ele, BlockPosition::After);
            ensure_space(ele, block, self.space.value);
        }
    }
}

pub fn format_pass(space_rules: &SpacingDsl, root: &SyntaxNode) {
    let mut fmt_root = FmtModel::new(root.clone());

    let rules_set = PatternSet::new(space_rules.rules.iter());

    for node in walk(root) {
        for rule in rules_set.matching(node.clone()) {
            rule.apply(&node, &mut fmt_root)
        }
    }
}
