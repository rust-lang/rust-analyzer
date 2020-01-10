// use crate::diff_view::DiffView;
use crate::dsl::{self, SpacingRule, SpacingDsl, IndentDsl, SpaceLoc, IndentRule, IndentValue};
use crate::edit_tree::{EditTree, Block};
use crate::pattern::{Pattern, PatternSet};
use crate::rules::{indentation, spacing};
use crate::trav_util::{has_newline};
use crate::whitespace::{Whitespace, INDENT};

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
}

impl Into<EditTree> for FmtDiff {
    fn into(self) -> EditTree {
        self.edit_tree
    }
}

pub(crate) struct SpaceBlock {
    spaces: u32,
    newline: bool,
    indent: bool,
}

impl FmtDiff {
    pub(crate) fn new(edit_tree: EditTree) -> Self {
        Self { edit_tree }
    }

    /// Checks if `Whitespace` and `SpacingRule` match then applies edit to `Block`.
    /// 
    /// # Arguments
    ///
    /// * `left_blk` - A `Block` that is always a token, previous token used to determine
    ///     space after token.
    /// * `right_blk` - A `Block` that is always a token, check right preceding whitespace.
    /// * `rule` - A `SpaceRule`.
    fn apply_edit(
        &self,
        rule: &SpacingRule,
        left_blk: &Block,
        right_blk: &Block
    ) {
        let left_ws = left_blk.get_whitespace();
        let right_ws = right_blk.get_whitespace();
        // only edit right preceding whitespace if it doesn't match and the rule applies before.
        if !right_ws.borrow().match_space_before(rule.space.value)
            && rule.pattern.matches(right_blk.as_element())
            && rule.space.loc == SpaceLoc::Before
        {
            // println!("RIGHT {:#?}", right_blk);
            // println!("RULES {:#?}", rule);
            right_blk.set_spacing_before(rule);
            // return Some(SpaceBlock::from(rule.clone()));
        };
        // if previous token has space after but only if token is one we want to edit whitespace of.
        // This cleans up cases like `struct Foo { x: u8 }`
        //                                       ^^^
        // it is much easier to declare the rule as "1 space after '{' inside of a struct definition"
        if !left_ws.borrow().match_space_after(rule.space.value)
            && rule.pattern.matches(left_blk.as_element())
            && rule.space.loc == SpaceLoc::After
        {
            // println!("LEFT {:#?}", right_blk);
            // println!("RULES {:#?}", rule);
            // this fixes after spacing "{" in
            // struct Test{x:usize}
            right_blk.set_spacing_before(rule);
        }
    }

    pub(crate) fn spacing_diff(self, space_rules: &SpacingDsl) -> FmtDiff {
        let spacing = PatternSet::new(space_rules.rules.iter());

        let blocks = self.edit_tree.walk_tokens().zip(self.edit_tree.walk_tokens().skip(1));

        for (left, right) in blocks {
            // chain left and right matching rules
            let rules = spacing.matching(left.to_element())
                .chain(spacing.matching(right.to_element()))
                .collect::<Vec<_>>();
                
            for rule in rules {
                // mutates EditTree
                self.apply_edit(rule, left, right);
            }
        }
        self
    }

    /// Checks if `Indent` and `IndentRule` match then mutates `Block`.
    /// 
    /// # Arguments
    ///
    /// * `block` - A `Block` that is always a token because rules match tokens.
    /// * `rule` - A `IndentRule`.
    fn check_indent(
        &self,
        anchor_set: &PatternSet<&Pattern>,
        block: &Block,
    ) {
        //println!("\n{:?}\n", rule);
        let mut anchors = INDENT;
        // TODO ancestor_nodes is NOT refs to blocks from the edit tree they are built on demand
        for node in block.ancestor_nodes() {
            if anchor_set.matching(node.to_element()).next().is_some() {
                //println!("FOUND ANCHOR {:?}\n {}\n", node, node.get_indent());
                // walk all the way up the tree adding indent as we go
                anchors += node.get_indent();

            }
        }
        // don't format if block already is indented properly
        if block.get_indent() != anchors {
            //println!("{:?}", block);
            // after calculating anchoring blocks indent apply fix to first token
            // found after node because in order to make our string we walk tokens
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

            // TODO do we ever have a rule that applies to a token??
            if let Some(tkn) = next_closest_tkn {
                tkn.set_indent(anchors);
            } else {
                // so we indent the token
                block.set_indent(anchors)
            };
            // println!("INDENT {} CURR {:?}", anchors, next_closest_tkn);
        }
    }

    pub(crate) fn indent_diff(self, indent_rules: &IndentDsl) -> FmtDiff {
        // println!("{:#?}", indent_rules);
        let anchors = PatternSet::new(indent_rules.anchors.iter());
        // TODO only walk nodes???
        let blocks = self.edit_tree.walk_exc_root().collect::<Vec<_>>();

        for block in blocks.iter() {
            let mut matching = indent_rules.rules.iter().filter(|it| it.matches(block.as_element()));
            // println!("in matching indent rule {:?}", matching);
            if let Some(_rule) = matching.next() {
                // TODO this works now but at some point we must walk sibling nodes/tokens
                // to check for "\n" because of 
                // let x = foo()
                //     .bar()
                // .baz();
                // only check_indent if prev token starts with "\n" 
                if block.starts_with_lf() {
                    self.check_indent(&anchors, block);
                    assert!(matching.next().is_none(), "more than one indent rule matched");
                }
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

pub(crate) fn format_str(file: &str) -> Result<String, std::fmt::Error> {
    let p = SourceFile::parse(file);
    let root = p.syntax_node();
    let space = spacing();
    let indent = indentation();

    format_pass(&space, &indent, &root).tokens_to_string()
}
