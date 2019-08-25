// use crate::diff_view::DiffView;
use crate::dsl::{self, SpacingRule, SpacingDsl, IndentDsl, IndentRule};
use crate::edit_tree::{EditTree, Block};
use crate::pattern::{Pattern, PatternSet};
use crate::rules::{indentation, spacing};
use crate::trav_util::{has_newline};
use crate::whitespace::INDENT;

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
    // diff: RefCell<DiffView>,
}

impl Into<EditTree> for FmtDiff {
    fn into(self) -> EditTree {
        self.edit_tree
    }
}

impl FmtDiff {
    pub(crate) fn new(edit_tree: EditTree) -> Self {
        Self { edit_tree }
    }

    /// Checks if `Whitespace` and `SpacingRule` match then mutates `DiffView`.
    /// 
    /// # Arguments
    ///
    /// * `block` - A `Block` that is always a token because rules match tokens.
    /// * `rule` - A `SpaceRule`.
    fn check_spacing(&self, rule: &SpacingRule, block: &Block) {
        let whitespace = block.get_whitespace();
        if *whitespace.borrow() != *rule {
            block.set_spacing(rule)
        }
    }

    pub(crate) fn spacing_diff(self, space_rules: &SpacingDsl) -> FmtDiff {
        let spacing = PatternSet::new(space_rules.rules.iter());
        let blcks = self.edit_tree.walk_exc_root().collect::<Vec<_>>();
        for block in blcks.iter() {
            for rule in spacing.matching(block.to_element()) {
                // creates DiffView
                self.check_spacing(rule, block)
            }
        }
        // take care of EOF new line
        let rule = SpacingRule {
            pattern: SOURCE_FILE.into(),
            space: dsl::Space { loc: dsl::SpaceLoc::After, value: dsl::SpaceValue::Newline }
        };
        self.edit_tree.last_token()
            .expect("cannot format empty file")
            .set_spacing(&rule);

        self
    }

    /// Checks if `Indent` and `IndentRule` match then mutates `DiffView`.
    /// 
    /// # Arguments
    ///
    /// * `block` - A `Block` that is always a token because rules match tokens.
    /// * `rule` - A `IndentRule`.
    fn check_indent(&self, anchor_set: &PatternSet<&Pattern>, block: &Block) {
        // println!("\n{:?}\n{:?}\n", block);
        let mut anchors = INDENT;
        // TODO ancestors is NOT refs to blocks from the edit tree they are built on demand
        for node in block.ancestors() {
            if anchor_set.matching(node.to_element()).next().is_some() {
                // println!("FOUND ANCHOR {:?}\n {}\n", node, node.get_indent());
                // walk all the way up the tree adding indent as we go
                anchors += node.get_indent()
            }
        }
        // don't format if we don't have to
        if block.get_indent() != anchors && block.get_whitespace().borrow().starts_with_lf {
            // after calculating anchoring blocks indent apply fix
            // to first token found after node, to make string we walk tokens
            // TODO probably not a great solution a bit hacky 
            let next_closest_tkn = std::iter::successors(block.children().next(), |kid| {
                if kid.as_element().as_token().is_some() {
                    Some(kid)
                } else {
                    kid.children().next()
                }
            }).find(|blk| {
                blk.as_element().as_token().is_some()
            });

            next_closest_tkn.unwrap().set_indent(anchors);
            // println!("INDENT {} CURR {:?}", anchors, next_closest_tkn);
        }
    }

    pub(crate) fn indent_diff(self, indent_rules: &IndentDsl) -> FmtDiff {
        // println!("{:#?}", indent_rules);
        let anchors = PatternSet::new(indent_rules.anchors.iter());
        // TODO only walk nodes???
        let blocks = self.edit_tree.walk_exc_root().collect::<Vec<_>>();

        let mut iter = blocks.iter().peekable();

        for block in blocks.iter() {

            iter.next();
            let mut matching = indent_rules.rules.iter().filter(|it| it.matches(block.as_element()));
            // println!("in matching indent rule {:?}", matching);
            if let Some(_rule) = matching.next() {
                // This block is the anchor in check_indent we walk children to find 
                // node to indent ??
                // TODO better name check_indent??
                self.check_indent(&anchors, block);
                assert!(matching.next().is_none(), "more than one indent rule matched");
            }
        }
        self
    }
}

pub(crate) fn format_pass(space_dsl: &SpacingDsl, indent_dsl: &IndentDsl, root: &SyntaxNode) -> EditTree {
    let fmt = EditTree::new(root.clone());
    FmtDiff::new(fmt)
        .spacing_diff(space_dsl)
        .indent_diff(indent_dsl)
        .into()
}

pub(crate) fn format_str(file: &str) -> Result<String, ()> {
    let p = SourceFile::parse(file);
    let root = p.syntax_node();
    let space = spacing();
    let indent = indentation();

    Ok(format_pass(&space, &indent, &root).apply_edits())
}
