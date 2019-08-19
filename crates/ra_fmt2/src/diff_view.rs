use crate::dsl::{Space, SpaceLoc, SpaceValue, SpacingDsl, SpacingRule};
use crate::edit_tree::{Block, EditTree};
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};
use crate::whitespace::{Whitespace, WhitespaceAbstract};

use ra_syntax::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};

use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SpaceEdit {
    /// Replace holds position of token, text to insert and lenghth of replaced text.
    Replace((usize, SmolStr), usize),
    /// Insert holds position of token, text to insert.
    Insert((usize, SmolStr)),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Edit {
    Space(SpaceEdit),
    Indent,
}

impl Edit {
    fn from_block(ws_abs: &dyn WhitespaceAbstract, text: &str, space: SpaceLoc) -> Self {
        match space {
            SpaceLoc::After => {
                // len of offending token
                let len = ws_abs.text_len();

                let edit = (ws_abs.text_end(), SmolStr::from(text));
                
                println!("len {} {:?}", len, ws_abs.text_range());
                if len > 1 && ws_abs.prev_is_whitespace() {
                    Edit::Space(SpaceEdit::Replace(edit, len))
                } else {
                    Edit::Space(SpaceEdit::Insert(edit))
                }
            },
            SpaceLoc::Before => {
                // len of offending token
                let len = ws_abs.prev_tkn_len();

                let edit = (ws_abs.text_start(), SmolStr::from(text));
                
                println!("len {} {:?}", len, ws_abs.text_range());
                if len > 1 && ws_abs.prev_is_whitespace() {
                    Edit::Space(SpaceEdit::Replace(edit, len))
                } else {
                    Edit::Space(SpaceEdit::Insert(edit))
                }
            },
            _ => unreachable!("Edit::from_block should never be called with SpaceLoc::Around")
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DiffView {
    // TODO more diffing info not just a string, better way than BTreeSet
    /// BTreeSet of location to insert or change and text to insert.
    /// A BTreeSet so we dont duplicate any inserts doubling the text inserted.
    edits: BTreeSet<Edit>,
    original: SmolStr,
    // formatted: Formatted,
}

impl DiffView {

    pub(crate) fn new(original: SmolStr) -> Self {
        Self { edits: BTreeSet::default(), original, }
    }

    pub(crate) fn collect_space_edits(&mut self, ws_abs: &dyn WhitespaceAbstract, rule: &SpacingRule) {
        match rule.space.loc {
            SpaceLoc::After => self.collect_space_after(ws_abs, rule.space),
            SpaceLoc::Before => self.collect_space_before(ws_abs, rule.space),
            SpaceLoc::Around => self.collect_space_around(ws_abs, rule.space),
        }
    }

    fn collect_space_after(&mut self, ws_abs: &dyn WhitespaceAbstract, space: Space) {
        match space.value {
            SpaceValue::Single => {
                let edit = Edit::from_block(ws_abs, " ", space.loc);
                self.edits.insert(edit);
            },
            SpaceValue::Newline => {
                let edit = Edit::from_block(ws_abs, "\n", space.loc);
                self.edits.insert(edit);
;            },
            SpaceValue::SingleOptionalNewline => {
                if !ws_abs.siblings_contain("\n") {
                    println!("SIBLINGS CONTAIN FALSE");
                    let edit = Edit::from_block(ws_abs, " ", space.loc);
                    self.edits.insert(edit);
                } else {
                    println!("SIBLINGS CONTAIN TRUE");
                    let edit = Edit::from_block(ws_abs, "\n", space.loc);
                    self.edits.insert(edit);
                }
            },
            _ => {},
        }
    }

    fn collect_space_before(&mut self, ws_abs: &dyn WhitespaceAbstract, space: Space) {
        match space.value {
            SpaceValue::Single => {
                let edit = Edit::from_block(ws_abs, " ", space.loc);
                self.edits.insert(edit);
;            },
            SpaceValue::Newline => {
                let edit = Edit::from_block(ws_abs, "\n", space.loc);
                self.edits.insert(edit);
            },
            SpaceValue::SingleOptionalNewline => {
                if !ws_abs.siblings_contain("\n") {
                    println!("SIBLINGS CONTAIN FALSE");
                    let edit = Edit::from_block(ws_abs, " ", space.loc);
                    self.edits.insert(edit);
                } else {
                    println!("SIBLINGS CONTAIN TRUE");
                    let edit = Edit::from_block(ws_abs, "\n", space.loc);
                    self.edits.insert(edit);
                }
            },
            _ => {},
        }
    }

    fn collect_space_around(&mut self, ws_abs: &dyn WhitespaceAbstract, space: Space) {
        match space.value {
            SpaceValue::Single => {
                let pair = vec![
                    Edit::from_block(ws_abs, " ", SpaceLoc::Before),
                    Edit::from_block(ws_abs, " ", SpaceLoc::After),
                ];
                self.edits.extend(pair)
            },
            SpaceValue::Newline => {
                let pair = vec![
                    Edit::from_block(ws_abs, "\n", SpaceLoc::Before),
                    Edit::from_block(ws_abs, "\n", SpaceLoc::After),
                ];
                self.edits.extend(pair)
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
        for edit in self.edits.iter() {
            match edit {
                Edit::Space(space) => {
                    match space {
                        SpaceEdit::Insert((mut pos, text)) => {
                            println!("INSERT pos: {} text: {:?}", pos, text);
                            pos += space_added;
                            fmt.splice(pos..pos, text.as_bytes().iter().cloned());
                            space_added += text.len();
                        },
                        SpaceEdit::Replace((mut pos, text), orig_tkn_len) => {
                            println!("REPLACE pos: {} text: {:?}", pos, text);
                            pos += space_added;
                            fmt.splice(pos..pos+orig_tkn_len, text.as_bytes().iter().cloned());
                            space_added -= orig_tkn_len - text.len();
                        },
                    }
                    
                },
                Edit::Indent => unimplemented!("Get to work on Indents"),
            }
        }
        Ok(String::from_utf8_lossy(&fmt).to_string())
    }
}


