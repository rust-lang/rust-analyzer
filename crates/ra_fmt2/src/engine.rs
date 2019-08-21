use crate::diff_view::DiffView;
use crate::dsl::{self, SpacingRule, SpacingDsl, IndentDsl, IndentRule};
use crate::edit_tree::{EditTree, Block};
use crate::pattern::PatternSet;
use crate::rules::spacing;
use crate::trav_util::{has_newline};

use ra_syntax::{
    ast::{self, AstNode, AstToken},
    Parse, SmolStr, SourceFile, SyntaxElement, SyntaxKind,
    SyntaxKind::*,
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};
use std::collections::HashMap;
use std::cell::RefCell;

#[derive(Debug, Clone)]
///
pub(crate) struct FmtDiff {
    edit_tree: EditTree,
    diff: RefCell<DiffView>,
}

impl FmtDiff {
    pub(crate) fn new(edit_tree: EditTree) -> Self {
        let original = edit_tree.text();
        let diff = RefCell::new(DiffView::new(original));

        Self { edit_tree, diff }
    }

    /// Checks if `Whitespace` and `SpacingRule` match then mutates `DiffView`.
    /// 
    /// # Arguments
    ///
    /// * `block` - A `Block` that is always a token because rules match tokens.
    /// * `rule` - A `SpaceRule`.
    fn check_spacing(&self, rule: &SpacingRule, block: &Block) {
        let whitespace = block.get_spacing();
        // is this a terible idea impl-ing eq??
        if *whitespace.borrow() != *rule {
            block.get_spacing().borrow_mut().apply_fix(rule)
        }
    }

    pub(crate) fn spacing_diff(self, space_rules: &SpacingDsl) -> EditTree {
        let spacing = PatternSet::new(space_rules.rules.iter());
        let blcks = self.edit_tree.walk_exc_root().collect::<Vec<_>>();
        // TODO better way to keep track of if next space is needed
        for block in blcks.iter() {
            for rule in spacing.matching(block.to_element()) {
                // creates DiffView
                self.check_spacing(rule, block)
            }
        } else {
            self.diff.borrow_mut().collect_edits(block, rule);
        }
        // take care of EOF new line this is HACKY
        let rule = SpacingRule {
            pattern: SOURCE_FILE.into(),
            space: dsl::Space { loc: dsl::SpaceLoc::After, value: dsl::SpaceValue::Newline }
        };
        self.edit_tree.last_token()
            .expect("cannot format empty file")
            .get_spacing()
            .borrow_mut().explicit_fix(&rule);
        self.edit_tree
    }

    /// Checks if `Indent` and `IndentRule` match then mutates `DiffView`.
    /// 
    /// # Arguments
    ///
    /// * `block` - A `Block` that is always a token because rules match tokens.
    /// * `rule` - A `IndentRule`.
    fn check_indent(&self, rule: &IndentRule, block: &Block) {
        let mut indent = block.get_indent();
        
    }

    pub(crate) fn indent_diff(self, indent_rules: &IndentDsl) -> EditTree {
        let indent = PatternSet::new(indent_rules.anchors.iter());
        // TODO only walk nodes???
        let blcks = self.edit_tree.walk_nodes().collect::<Vec<_>>();
        // TODO better way to keep track of if next space is needed
        for block in blcks.iter() {
            for rule in indent.matching(block.to_element()) {
                // creates DiffView
                self.check_indent(rule, block)
            }
        }
        self.edit_tree
    }
}

pub(crate) fn format_pass(space_dsl: &SpacingDsl, root: &SyntaxNode) -> EditTree {
    let fmt = EditTree::new(root.clone());
    FmtDiff::new(fmt).spacing_diff(space_dsl)
}

pub(crate) fn format_str(file: &str) -> Result<String, ()> {
    let p = SourceFile::parse(file);
    let root = p.syntax_node();
    let space = spacing();
    
    Ok(format_pass(&space, &root).apply_edits())
}
