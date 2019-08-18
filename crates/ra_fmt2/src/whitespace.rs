use crate::dsl::{Space, SpaceLoc, SpaceValue, SpacingDsl, SpacingRule};
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};

use ra_syntax::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};

use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub(crate) enum Spaces {
    /// Space followed by noting.
    SpaceNone,
    /// Nothing followed by pace.
    NoneSpace,
    /// Space followed by space.
    SpaceSpace,
    /// Space line followed by new line.
    SpaceEol,
    /// new line followed by space.
    EolSpace,
    /// Noting followed by new line.
    NoneEol,
    /// New line followed by nothing.
    EolNone,
    /// New line followed by new line.
    EolEol,
    /// Nothing followed by nothing.
    NoneNone,
}

#[derive(Clone, Debug)]
/// Whitespace holds all whitespace information for each Block.
/// Accessed from any Block's get_whitespace fn.
pub(crate) struct Whitespace {
    original_pair: (Option<SyntaxToken>, Option<SyntaxToken>),
    indent_spaces: u32,
    // additional_spaces: u32,
    new_line: (bool, bool),
    surrounding: (u32, u32),
}

impl Whitespace {
    pub(crate) fn new(pair: (Option<SyntaxToken>, Option<SyntaxToken>)) -> Whitespace {
        let (surrounding, new_line) = match &pair {
            (Some(prev), Some(next)) => {
                let prev_space = if prev.kind() == WHITESPACE {
                    calc_num_space(&prev)
                } else {0};
                let next_space = if next.kind() == WHITESPACE {
                    calc_num_space(&next)
                } else {0};

                // maybe deal with in build block like whitespace check??
                let new_line =
                    (prev.text().as_str().contains('\n'), next.text().as_str().contains('\n'));

                ((prev_space, next_space), new_line)
            }
            (Some(prev), None) => {
                let prev_space = if prev.kind() == WHITESPACE {
                    calc_num_space(&prev)
                } else {0};

                // maybe deal with in build block like whitespace check??
                let new_line = (prev.text().as_str().contains('\n'), false);

                ((prev_space, 0), new_line)
            }
            (None, Some(next)) => {
                let next_space = if next.kind() == WHITESPACE {
                    calc_num_space(&next)
                } else {0};

                // maybe deal with in build block like whitespace check??
                let new_line = (false, next.text().as_str().contains('\n'));

                ((0, next_space), new_line)
            }
            _ => unreachable!("Whitespace::new"),
        };
        // TODO logic for indent_spaces
        Self {
            original_pair: pair,
            indent_spaces: 0,
            new_line,
            // additional_spaces,
            surrounding,
        }
    }

    pub(crate) fn match_space_after(&self, value: &SpaceValue) -> bool {
        match value {
            SpaceValue::Single => self.surrounding.1 > 0,
            SpaceValue::SingleOrNewline => self.surrounding.1 > 0 || self.new_line.1,
            SpaceValue::SingleOptionalNewline => self.surrounding.1 > 0 || self.new_line.1,
            SpaceValue::Newline => self.new_line.1,
            SpaceValue::NoneOrNewline => self.surrounding.1 == 0 || self.new_line.1,
            SpaceValue::NoneOptionalNewline => self.surrounding.1 > 0 && self.new_line.1,
            SpaceValue::None => self.surrounding.1 == 0 || !self.new_line.1,
        }
    }

    pub(crate) fn match_space_before(&self, value: &SpaceValue) -> bool {
        match value {
            SpaceValue::Single => self.surrounding.0 > 0,
            SpaceValue::SingleOrNewline => self.surrounding.0 > 0 || self.new_line.0,
            SpaceValue::SingleOptionalNewline => self.surrounding.0 > 0 || self.new_line.0,
            SpaceValue::Newline => self.new_line.0,
            SpaceValue::NoneOrNewline => self.surrounding.0 == 0 || self.new_line.0,
            SpaceValue::NoneOptionalNewline => self.surrounding.0 > 0 && self.new_line.0,
            SpaceValue::None => self.surrounding.0 == 0 || !self.new_line.0,
        }
    }

    pub(crate) fn match_space_around(&self, value: &SpaceValue) -> bool {
        match value {
            SpaceValue::Single => self.surrounding.0 > 0 && self.surrounding.1 > 0,
            SpaceValue::SingleOrNewline => {
                (self.surrounding.0 > 0 && self.surrounding.1 > 0)
                || (self.new_line.0 && self.new_line.1)
            },
            SpaceValue::SingleOptionalNewline => {
                (self.surrounding.0 > 0 && self.surrounding.1 > 0)
                || (self.new_line.0 && self.new_line.1)
            },
            SpaceValue::Newline => self.new_line.0 && self.new_line.1,
            SpaceValue::NoneOrNewline => {
                (self.surrounding.0 == 0 && self.surrounding.1 == 0)
                || (self.new_line.0 && self.new_line.1)
            },
            SpaceValue::NoneOptionalNewline => {
                (self.surrounding.0 > 0 && self.surrounding.1 > 0)
                && (self.new_line.0 && self.new_line.1)
            },
            SpaceValue::None => {
                (self.surrounding.0 == 0 && self.surrounding.1 == 0)
                && (!self.new_line.0 && !self.new_line.1)
            },
        }
    }
}

impl PartialEq<SpacingRule> for Whitespace {
    fn eq(&self, rhs: &SpacingRule) -> bool {
        match rhs.space.loc {
            SpaceLoc::After => self.match_space_after(&rhs.space.value),
            SpaceLoc::Before => self.match_space_before(&rhs.space.value),
            SpaceLoc::Around => self.match_space_around(&rhs.space.value),
        }
    }
}

fn calc_num_space(tkn: &SyntaxToken) -> u32 {
    let len = tkn.text_range();
    (len.end() - len.start()).into()
}
