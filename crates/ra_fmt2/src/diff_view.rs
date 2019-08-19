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
enum Formatted {
    Changed(SmolStr),
    Unchanged,
}

#[derive(Debug, Clone)]
pub(crate) struct DiffView {
    // TODO more diffing info not just a string
    /// Vec of location of insert or change and text to insert.
    edits: Vec<(usize, SmolStr)>,
    original: SmolStr,
    // formatted: Formatted,
}

impl DiffView {

    pub(crate) fn new(original: SmolStr) -> Self {
        Self { edits: vec![], original, }
    }

    pub(crate) fn collect_edits(&mut self, block: &Block, rule: &SpacingRule) {
        match rule.space.loc {
            SpaceLoc::After => self.collect_after(block, rule.space.value),
            SpaceLoc::Before => self.collect_before(block, rule.space.value),
            SpaceLoc::Around => self.collect_around(block, rule.space.value),
        }
    }

    fn collect_after(&mut self, block: &Block, value: SpaceValue) {
        match value {
            SpaceValue::Single => {
                let after = block.text_range().end().to_usize();
                self.edits.push((after, SmolStr::from(" ")))
            },
            SpaceValue::Newline => {
                let after = block.text_range().end().to_usize();
                self.edits.push((after, SmolStr::from("\n")))
            },
            SpaceValue::SingleOptionalNewline => {
                let before = block.text_range().end().to_usize();

                if !block.siblings_contain("\n") {
                    println!("SIBLINGS CONTAIN FALSE");
                    self.edits.push((before, SmolStr::from(" ")))
                } else {
                    println!("SIBLINGS CONTAIN TRUE");
                    self.edits.push((before, SmolStr::from("\n")))
                }
            },
            _ => {},
        }
    }

    fn collect_before(&mut self, block: &Block, value: SpaceValue) {
        match value {
            SpaceValue::Single => {
                let before = block.text_range().start().to_usize();
                self.edits.push((before, SmolStr::from(" ")))
            },
            SpaceValue::Newline => {
                let before = block.text_range().start().to_usize();
                self.edits.push((before, SmolStr::from("\n")))
            },
            SpaceValue::SingleOptionalNewline => {
                let before = block.text_range().start().to_usize();

                if !block.siblings_contain("\n") {
                    println!("SIBLINGS CONTAIN FALSE");
                    self.edits.push((before, SmolStr::from(" ")))
                } else {
                    println!("SIBLINGS CONTAIN TRUE");
                    self.edits.push((before, SmolStr::from("\n")))
                }
            },
            _ => {},
        }
    }

    fn collect_around(&mut self, block: &Block, value: SpaceValue) {
        match value {
            SpaceValue::Single => {
                let before = block.text_range().start().to_usize();
                let after = block.text_range().end().to_usize();

                let mut pair = vec![(before, SmolStr::from(" ")),(after, SmolStr::from(" "))];
                self.edits.append(&mut pair)
            },
            SpaceValue::Newline => {
                let before = block.text_range().start().to_usize();
                let after = block.text_range().end().to_usize();
                
                let mut pair = vec![(before, SmolStr::from("\n")),(after, SmolStr::from("\n"))];
                self.edits.append(&mut pair)
            },
            _ => {},
        }
    }

    /// Apply the collected changes to text and return `Formatted`
    /// 
    /// ?? Does it need to be result or are all errors handled in ra-syntax??
    pub(crate) fn apply(&mut self) -> Result<String, ()> {
        let mut fmt = self.original.as_bytes().to_vec();
        let mut space_added = 0;
        for (mut pos, text) in self.edits.iter() {
            pos += space_added;
            fmt.splice(pos..pos, text.as_bytes().iter().cloned());
            space_added += text.len();
        }
        Ok(String::from_utf8_lossy(&fmt).to_string())
    }
}


