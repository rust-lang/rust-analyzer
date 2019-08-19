use crate::diff_view::DiffView;
use crate::dsl::{self, SpaceLoc, SpaceValue, SpacingRule, SpacingDsl};
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
    /// * `block` - A `Block` that is always a token.
    /// * `rule` - A `SpaceRule`.
    fn check_spacing(&self, rule: &SpacingRule, block: &Block) {
        if let Some(whitespace) = block.get_spacing() {
            // is this a terible idea impl-ing eq??
            if whitespace != rule {
                self.diff.borrow_mut().collect_space_edits(whitespace, rule);
            }
        } else {
            self.diff.borrow_mut().collect_space_edits(block, rule);
        }
    }

    pub(crate) fn spacing_diff(self, space_rules: &SpacingDsl) -> DiffView {
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

pub(crate) fn format_pass(space_dsl: &SpacingDsl, root: &SyntaxNode) -> DiffView {
    let fmt = EditTree::new(root.clone());

    let orig = fmt.text().to_string();

    let mut diff = FmtDiff::new(fmt).spacing_diff(space_dsl);

    println!("original: {:?}\nformatted: {:?}", orig, diff.apply().unwrap());

    diff
}

pub(crate) fn format_str(code: &str) -> Result<String, ()> {
    let p = SourceFile::parse(code);
    let root = p.syntax_node();
    let space = spacing();

    format_pass(&space, &root).apply()
}
