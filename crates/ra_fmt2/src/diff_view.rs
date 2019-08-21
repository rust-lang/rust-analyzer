use crate::dsl::{Space, SpaceLoc, SpaceValue, SpacingDsl, SpacingRule};
use crate::edit_tree::{Block, EditTree};
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};
use crate::whitespace::{Whitespace};

use ra_syntax::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};

use std::collections::BTreeSet;

/// SpaceEdit enum keeps track of edit kind and holds edit location and text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SpaceEdit {
    /// Replace holds position of token, text to insert and lenghth of replaced text.
    Replace((isize, SmolStr), (isize, isize)),
    /// Insert holds position of token, text to insert.
    Insert((isize, SmolStr)),
}

impl Ord for SpaceEdit {
    fn cmp(&self, rhs: &SpaceEdit) -> std::cmp::Ordering {
        let loc = match self {
            SpaceEdit::Replace((loc, _), _) => loc,
            SpaceEdit::Insert((loc, _)) => loc,
        };
        let rhs_loc = match rhs {
            SpaceEdit::Replace((loc, _), _) => loc,
            SpaceEdit::Insert((loc, _)) => loc,
        };
        loc.cmp(rhs_loc)
    }
}

impl PartialOrd for SpaceEdit {
    fn partial_cmp(&self, rhs: &SpaceEdit) -> Option<std::cmp::Ordering> {
        let loc = match self {
            SpaceEdit::Replace((loc, _), _) => loc,
            SpaceEdit::Insert((loc, _)) => loc,
        };
        let rhs_loc = match rhs {
            SpaceEdit::Replace((loc, _), _) => loc,
            SpaceEdit::Insert((loc, _)) => loc,
        };
        loc.partial_cmp(rhs_loc)
    }
}

/// Enum of edit kinds `Space` and `Indent` each hold respective structs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Edit {
    Space(SpaceEdit),
    Indent,
}

impl Edit {
    // fn from_block(block: &Block, text: &str, space: SpaceLoc) -> Self {
    //     match space {
    //         SpaceLoc::Before => {
    //             let w_space = block.get_spacing_mut();
    //             // len of offending token
    //             let len = block.prev_tkn_len() as isize;
    //             let range = (len, 0);

    //             let edit = (block.text_start() as isize, SmolStr::from(text));
                
    //             println!("len {:?} {:?}", range, block.text_range());
    //             if (len > 1 && block.prev_is_whitespace()) || (!block.match_prev(text) && block.prev_is_whitespace()) {
    //                 Edit::Space(SpaceEdit::Replace(edit, range))
    //             } else {
    //                 Edit::Space(SpaceEdit::Insert(edit))
    //             }
    //         },
    //         SpaceLoc::After => {
    //             // len of offending token
    //             let len = block.next_tkn_len() as isize;
    //             let range = (0, len);

    //             let edit = (block.text_end() as isize, SmolStr::from(text));
                
    //             println!("len {:?} {:?}", range, block.text_range());
    //             if (len > 1 && block.next_is_whitespace()) || (!block.match_next(text) && block.next_is_whitespace()) {
    //                 Edit::Space(SpaceEdit::Replace(edit, range))
    //             } else {
    //                 Edit::Space(SpaceEdit::Insert(edit))
    //             }
    //         },
    //         _ => unreachable!("Edit::from_block should never be called with SpaceLoc::Around")
    //     }
    // }
}

#[derive(Debug, Clone)]
pub(crate) struct DiffView {
    // TODO more diffing info not just a string, better way than BTreeSet
    /// BTreeSet of `Edit`'s.
    ws_edits: BTreeSet<Edit>,
    original: SmolStr,
    // formatted: Formatted,
}

impl DiffView {

    pub(crate) fn new(original: SmolStr) -> Self {
        Self { ws_edits: BTreeSet::default(), original, }
    }

//     pub(crate) fn collect_space_edits(&mut self, block: &Block, rule: &SpacingRule) {
//         match rule.space.loc {
//             SpaceLoc::After => self.fix_spacing_after(block, rule.space),
//             SpaceLoc::Before => self.fix_spacing_before(block, rule.space),
//             SpaceLoc::Around => self.fix_spacing_around(block, rule.space),
//         }
//     }

//     fn fix_spacing_after(&mut self, block: &Block, space: Space) {
//         match space.value {
//             SpaceValue::Single => {
//                 let edit = Edit::from_block(block, " ", space.loc);
//                 self.ws_edits.insert(edit);
//             },
//             SpaceValue::Newline => {
//                 let edit = Edit::from_block(block, "\n", space.loc);
//                 self.ws_edits.insert(edit);
// ;            },
//             SpaceValue::SingleOptionalNewline => {
//                 if !block.siblings_contain("\n") {
//                     let edit = Edit::from_block(block, " ", space.loc);
//                     self.ws_edits.insert(edit);
//                 } else {
//                     let edit = Edit::from_block(block, "\n", space.loc);
//                     self.ws_edits.insert(edit);
//                 }
//             },
//             _ => {},
//         }
//     }

//     fn fix_spacing_before(&mut self, block: &Block, space: Space) {
//         match space.value {
//             SpaceValue::Single => {
//                 let edit = Edit::from_block(block, " ", space.loc);
//                 self.ws_edits.insert(edit);
// ;            },
//             SpaceValue::Newline => {
//                 let edit = Edit::from_block(block, "\n", space.loc);
//                 self.ws_edits.insert(edit);
//             },
//             SpaceValue::SingleOptionalNewline => {
//                 if !block.siblings_contain("\n") {
//                     let edit = Edit::from_block(block, " ", space.loc);
//                     self.ws_edits.insert(edit);
//                 } else {
//                     let edit = Edit::from_block(block, "\n", space.loc);
//                     self.ws_edits.insert(edit);
//                 }
//             },
//             _ => {},
//         }
//     }

//     fn fix_spacing_around(&mut self, block: &Block, space: Space) {
//         match space.value {
//             SpaceValue::Single => {
//                 let pair = vec![
//                     Edit::from_block(block, " ", SpaceLoc::Before),
//                     Edit::from_block(block, " ", SpaceLoc::After),
//                 ];
//                 self.ws_edits.extend(pair)
//             },
//             SpaceValue::Newline => {
//                 let pair = vec![
//                     Edit::from_block(block, "\n", SpaceLoc::Before),
//                     Edit::from_block(block, "\n", SpaceLoc::After),
//                 ];
//                 self.ws_edits.extend(pair)
//             },
//             _ => {},
//         }
//     }

//     /// Apply the collected changes to text and return `Formatted`.
//     /// 
//     /// ?? Does it need to be result or are all errors handled in ra-syntax??
//     pub(crate) fn apply_diff(&mut self) -> Result<String, ()> {
//         let mut fmt = self.original.as_bytes().to_vec();
//         let mut space_added = 0_isize;
//         for edit in self.ws_edits.iter() {
//             match edit {
//                 Edit::Space(space) => {
//                     match space {
//                         SpaceEdit::Insert((mut pos, text)) => {
//                             pos += space_added;

//                             // pos should never be negative it is the index of token in the SmolStr
//                             let idx = pos as usize;
//                             fmt.splice(idx..idx, text.as_bytes().iter().cloned());

//                             space_added += text.len() as isize;
//                         },
//                         SpaceEdit::Replace((mut pos, text), (remove, add)) => {
//                             pos += space_added;
//                             let start = (pos - remove) as usize;
//                             let end = (pos + add) as usize;
//                             fmt.splice(start..end, text.as_bytes().iter().cloned());
//                             //println!("POST SPLICE {} -= {} + {} - {}", space_added, remove, add, text.len());
//                             space_added -= remove + add - text.len() as isize;
//                         },
//                     }
                    
//                 },
//                 Edit::Indent => unimplemented!("Get to work on Indents"),
//             }
//         }
//         Ok(String::from_utf8_lossy(&fmt).to_string())
//     }
}


