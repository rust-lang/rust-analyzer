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

pub trait WhitespaceAbstract {
    /// Walks siblings to search for pat.
    fn siblings_contain(&self, pat: &str) -> bool;
    /// Checks if previous token is whitespace kind.
    fn prev_is_whitespace(&self) -> bool;
    /// Text range of current token.
    fn text_range(&self) -> TextRange;
    /// Previous token's length.
    fn prev_tkn_len(&self) -> usize;
    /// Current token's length.
    fn text_len(&self) -> usize;
    /// Current token's start position.
    fn text_start(&self) -> usize;
    /// Current token's end position.
    fn text_end(&self) -> usize;
}

#[derive(Clone, Debug)]
/// Whitespace holds all whitespace information for each Block.
/// Accessed from any Block's get_whitespace fn.
pub(crate) struct Whitespace {
    surounding_pair: (Option<SyntaxToken>, Option<SyntaxToken>),
    pub(crate) original: SyntaxToken,
    pub(crate) text_range: TextRange,
    indent_spaces: u32,
    // additional_spaces: u32,
    new_line: (bool, bool),
    // Start and end location of token.
    locations: (u32, u32),
}

impl WhitespaceAbstract for Whitespace {
    fn siblings_contain(&self, pat: &str) -> bool {
        self.siblings_contain(pat)
    }
    fn prev_is_whitespace(&self) -> bool {
        if let Some(prev) = &self.surounding_pair.0 {
            prev.kind() == WHITESPACE
        } else {
            false
        }
    }
    fn text_range(&self) -> TextRange {
        self.original.text_range()
    }
    fn text_len(&self) -> usize {
        self.text_range().len().to_usize()
    }
    fn prev_tkn_len(&self) -> usize {
        if let Some(prev) = &self.surounding_pair.0 {
            prev.text_range().len().to_usize()
        } else {
            0
        }
    }
    fn text_start(&self) -> usize {
        self.text_range().start().to_usize()
    }
    fn text_end(&self) -> usize {
        self.text_range().end().to_usize()
    }
}

impl Whitespace {
    pub(crate) fn new(token: &SyntaxToken) -> Whitespace {
         match (token.prev_token(), token.next_token()) {
            (Some(prev), Some(next)) => {
                let prev_space = if prev.kind() == WHITESPACE {
                    calc_num_space(&prev)
                } else {0};
                let next_space = if next.kind() == WHITESPACE {
                    calc_num_space(&next)
                } else {0};

                let new_line =
                    (prev.text().as_str().contains('\n'), next.text().as_str().contains('\n'));

                Self {
                    surounding_pair: (Some(prev), Some(next)),
                    original: token.clone(),
                    text_range: token.text_range(),
                    indent_spaces: 0,
                    new_line,
                    // additional_spaces,
                    locations: (prev_space, next_space),
                }
            }
            (Some(prev), None) => {
                let prev_space = if prev.kind() == WHITESPACE {
                    calc_num_space(&prev)
                } else {0};

                let new_line = (prev.text().as_str().contains('\n'), false);

                Self {
                    surounding_pair: (Some(prev), None),
                    original: token.clone(),
                    text_range: token.text_range(),
                    indent_spaces: 0,
                    new_line,
                    // additional_spaces,
                    locations: (prev_space, 0),
                }
            }
            (None, Some(next)) => {
                let next_space = if next.kind() == WHITESPACE {
                    calc_num_space(&next)
                } else {0};

                let new_line = (false, next.text().as_str().contains('\n'));

                Self {
                    surounding_pair: (None, Some(next)),
                    original: token.clone(),
                    text_range: token.text_range(),
                    indent_spaces: 0,
                    new_line,
                    // additional_spaces,
                    locations: (0, next_space),
                }
            }
            _ => unreachable!("Whitespace::new"),
        }
        // TODO logic for indent_spaces
    } 

    pub(crate) fn from_eof(eof: SyntaxToken) -> Option<Whitespace> {
        if eof.kind() == WHITESPACE && eof.text().to_string() == "\n" {
            Some(Whitespace {
                surounding_pair: (None, Some(eof.clone())),
                original: eof.clone(),
                text_range: eof.text_range(),
                indent_spaces: 0,
                new_line: (false, true),
                // additional_spaces,
                locations: (0, 1),
            })
        } else {
            None
        }
        // Whitespace {
        //     surounding_pair: (None, Some(eof)),
        //     original: eof.clone(),
        //     indent_spaces: 0,
        //     new_line: (false, false),
        //     // additional_spaces,
        //     locations: (0, 0),
        // }
    }

    pub(crate) fn siblings_contain(&self, pat: &str) -> bool {
        walk_tokens(&self.original.parent())
            // TODO there is probably a better/more accurate way to do this
            .any(|tkn| {
                tkn.text().as_str() == pat
            })
    }

    // TODO check if NewLine needs to check for space
    pub(crate) fn match_space_after(&self, value: SpaceValue) -> bool {
        match value {
            SpaceValue::Single => self.locations.1 == 1,
            SpaceValue::SingleOrNewline => self.locations.1 == 1 || self.new_line.1,
            SpaceValue::SingleOptionalNewline => self.locations.1 == 1 || self.new_line.1,
            SpaceValue::Newline => self.new_line.1,
            SpaceValue::NoneOrNewline => self.locations.1 == 0 || self.new_line.1,
            SpaceValue::NoneOptionalNewline => self.locations.1 == 0 && self.new_line.1,
            SpaceValue::None => self.locations.1 == 0 || !self.new_line.1,
        }
    }

    pub(crate) fn match_space_before(&self, value: SpaceValue) -> bool {
        match value {
            SpaceValue::Single => self.locations.0 == 1,
            SpaceValue::SingleOrNewline => self.locations.0 == 1 || self.new_line.0,
            SpaceValue::SingleOptionalNewline => self.locations.0 == 1 || self.new_line.0,
            SpaceValue::Newline => self.new_line.0,
            SpaceValue::NoneOrNewline => self.locations.0 == 0 || self.new_line.0,
            SpaceValue::NoneOptionalNewline => self.locations.0 == 0 && self.new_line.0,
            SpaceValue::None => self.locations.0 == 0 || !self.new_line.0,
        }
    }

    pub(crate) fn match_space_around(&self, value: SpaceValue) -> bool {
        match value {
            SpaceValue::Single => self.locations.0 == 1 && self.locations.1 == 1,
            SpaceValue::SingleOrNewline => {
                (self.locations.0 == 1 && self.locations.1 == 1)
                || (self.new_line.0 && self.new_line.1)
            },
            SpaceValue::SingleOptionalNewline => {
                (self.locations.0 == 1 && self.locations.1 == 1)
                || (self.new_line.0 && self.new_line.1)
            },
            SpaceValue::Newline => self.new_line.0 && self.new_line.1,
            SpaceValue::NoneOrNewline => {
                (self.locations.0 == 0 && self.locations.1 == 0)
                || (self.new_line.0 && self.new_line.1)
            },
            SpaceValue::NoneOptionalNewline => {
                (self.locations.0 == 0 && self.locations.1 == 0)
                && (self.new_line.0 && self.new_line.1)
            },
            SpaceValue::None => {
                (self.locations.0 == 0 && self.locations.1 == 0)
                && (!self.new_line.0 && !self.new_line.1)
            },
        }
    }
}

impl PartialEq<SpacingRule> for Whitespace {
    fn eq(&self, rhs: &SpacingRule) -> bool {
        match rhs.space.loc {
            SpaceLoc::After => self.match_space_after(rhs.space.value),
            SpaceLoc::Before => self.match_space_before(rhs.space.value),
            SpaceLoc::Around => self.match_space_around(rhs.space.value),
        }
    }
}

fn calc_num_space(tkn: &SyntaxToken) -> u32 {
    let len = tkn.text_range();
    (len.end() - len.start()).into()
}
