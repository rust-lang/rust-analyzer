use crate::dsl::{Space, SpaceLoc, SpaceValue, SpacingDsl, SpacingRule};
use crate::indent::{Indentation};
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};

use ra_syntax::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *}, Direction,
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};

use std::collections::{HashMap, HashSet};

/// A trait to abstract `Block`s and `Whitespace`.
/// 
/// When no whitespace is found the insert index of the final `SmolStr`
/// must be computed from a block. This also makes it posible to treat the 
/// root node as any other node or token (EOF case).
pub trait WhitespaceAbstract: std::fmt::Debug {
    /// Walks siblings to search for pat.
    fn siblings_contain(&self, pat: &str) -> bool;
    /// Match pattern with previous token.
    fn match_prev(&self, pat: &str) -> bool;
    /// Match pattern with next token.
    fn match_next(&self, pat: &str) -> bool;
    /// Checks if previous token is whitespace kind.
    fn prev_is_whitespace(&self) -> bool;
    /// Checks if next token is whitespace kind.
    fn next_is_whitespace(&self) -> bool;
    /// Text range of current token.
    fn text_range(&self) -> TextRange;
    /// Previous token's length.
    fn prev_tkn_len(&self) -> usize;
    /// Next token's length.
    fn next_tkn_len(&self) -> usize;

    /// Current token's length.
    fn text_len(&self) -> usize {
        self.text_range().end().to_usize()
    }
    /// Current token's end position.
    fn text_end(&self) -> usize {
        self.text_range().end().to_usize()
    }
    /// Current token's start position.
    fn text_start(&self) -> usize {
        self.text_range().start().to_usize()
    }
    
}

#[derive(Clone, Debug)]
/// Whitespace holds all whitespace information for each Block.
/// Accessed from any Block's get_whitespace fn.
pub(crate) struct Whitespace {
    original: SyntaxElement,
    text_range: TextRange,
    //indent: Indentation,
    // additional_spaces: u32,
    pub(crate) new_line: (bool, bool),
    // Start and end location of token.
    pub(crate) locations: (u32, u32),
}

impl WhitespaceAbstract for Whitespace {
    fn siblings_contain(&self, pat: &str) -> bool {
        self.siblings_contain(pat)
    }
    fn match_prev(&self, pat: &str) -> bool {
        if let Some(tkn) = &self.surounding_pair.0 {
            tkn.text() == pat
        } else {
            false
        }
    }
    fn match_next(&self, pat: &str) -> bool {
        if let Some(tkn) = &self.surounding_pair.1 {
            tkn.text() == pat
        } else {
            false
        }
    }
    fn prev_is_whitespace(&self) -> bool {
        if let Some(prev) = &self.surounding_pair.0 {
            prev.kind() == WHITESPACE
        } else {
            false
        }
    }
    fn next_is_whitespace(&self) -> bool {
        if let Some(prev) = &self.surounding_pair.1 {
            prev.kind() == WHITESPACE
        } else {
            false
        }
    }
    fn text_range(&self) -> TextRange {
        self.original.text_range()
    }
    fn prev_tkn_len(&self) -> usize {
        if let Some(prev) = &self.surounding_pair.0 {
            prev.text_range().len().to_usize()
        } else {
            0
        }
    }
    fn next_tkn_len(&self) -> usize {
        if let Some(prev) = &self.surounding_pair.1 {
            prev.text_range().len().to_usize()
        } else {
            0
        }
    }
}

impl Whitespace {
    pub(crate) fn new(element: &SyntaxElement) -> Whitespace {
        match &element {
            NodeOrToken::Node(node) => {
                Whitespace::from_node(&node)
            },
            NodeOrToken::Token(token) => {
                Whitespace::from_token(&token)
            },
        }
    }

    fn from_node(node: &SyntaxNode) -> Whitespace {
        match (node.siblings_with_tokens(Direction::Prev).next(), node.siblings_with_tokens(Direction::Next).next()) {
            (Some(prev), Some(next)) => {
                let prev_space = if prev.kind() == WHITESPACE {
                    calc_num_space_tkn(prev.as_token().unwrap())
                } else {
                    0
                };
                let next_space = if next.kind() == WHITESPACE {
                    calc_num_space_tkn(next.as_token().unwrap())
                } else {
                    0
                };
                let prev_line = match prev {
                    NodeOrToken::Node(_) => {
                        false
                    },
                    NodeOrToken::Token(tkn) => {
                        tkn.text().as_str().contains('\n')
                    },
                };
                let next_line = match next {
                    NodeOrToken::Node(_) => {
                        false
                    },
                    NodeOrToken::Token(tkn) => {
                        tkn.text().as_str().contains('\n')
                    },
                };

                Self {
                    original: NodeOrToken::Node(node.clone()),
                    text_range: node.text_range(),
                    new_line: (prev_line, next_line),
                    // additional_spaces,
                    locations: (prev_space, next_space),
                }
            },
            (Some(prev), None) => {
                let prev_space = if prev.kind() == WHITESPACE {
                    calc_num_space_tkn(prev.as_token().unwrap())
                } else {
                    0
                };
                let prev_line = match prev {
                    NodeOrToken::Node(_) => {
                        false
                    },
                    NodeOrToken::Token(tkn) => {
                        tkn.text().as_str().contains('\n')
                    },
                };

                Self {
                    original: NodeOrToken::Node(node.clone()),
                    text_range: node.text_range(),
                    new_line: (prev_line, false),
                    // additional_spaces,
                    locations: (prev_space, 0),
                }
            },
            (None, Some(next)) => {
                let next_space = if next.kind() == WHITESPACE {
                    calc_num_space_tkn(next.as_token().unwrap())
                } else {
                    0
                };
                let next_line = match next {
                    NodeOrToken::Node(_) => {
                        false
                    },
                    NodeOrToken::Token(tkn) => {
                        tkn.text().as_str().contains('\n')
                    },
                };
                Self {
                    original: NodeOrToken::Node(node.clone()),
                    text_range: node.text_range(),
                    new_line: (false, next_line),
                    // additional_spaces,
                    locations: (0, next_space),
                }
            },
            _ => unreachable!("Whitespace::new"),
        }
    }

    fn from_token(token: &SyntaxToken) -> Whitespace {
        match (token.prev_token(), token.next_token()) {
            (Some(prev), Some(next)) => {
                let prev_space = if prev.kind() == WHITESPACE {
                    calc_num_space_tkn(&prev)
                } else {
                    0
                };
                let next_space = if next.kind() == WHITESPACE {
                    calc_num_space_tkn(&next)
                } else {
                    0
                };

                let new_line =
                    (prev.text().as_str().contains('\n'), next.text().as_str().contains('\n'));

                Self {
                    original: NodeOrToken::Token(token.clone()),
                    text_range: token.text_range(),
                    new_line,
                    // additional_spaces,
                    locations: (prev_space, next_space),
                }
            }
            (Some(prev), None) => {
                let prev_space = if prev.kind() == WHITESPACE {
                    calc_num_space_tkn(&prev)
                } else {
                    0
                };
                let new_line = (prev.text().as_str().contains('\n'), false);

                Self {
                    original: NodeOrToken::Token(token.clone()),
                    text_range: token.text_range(),
                    new_line,
                    // additional_spaces,
                    locations: (prev_space, 0),
                }
            }
            (None, Some(next)) => {
                let next_space = if next.kind() == WHITESPACE {
                    calc_num_space_tkn(&next)
                } else {
                    0
                };

                let new_line = (false, next.text().as_str().contains('\n'));

                Self {
                    original: NodeOrToken::Token(token.clone()),
                    text_range: token.text_range(),
                    new_line,
                    // additional_spaces,
                    locations: (0, next_space),
                }
            }
            _ => unreachable!("Whitespace::new"),
        }
    }

    /// Walks siblings to search for pat.
    pub(crate) fn siblings_contain(&self, pat: &str) -> bool {
        if let Some(tkn) = self.original.clone().into_token() {
            println!("SIB CON {:?}", tkn.parent());
            walk_tokens(&tkn.parent())
                // TODO there is probably a better/more accurate way to do this
                .any(|tkn| {
                    tkn.text().as_str() == pat
                })
        } else {
            false
        }
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

    pub(crate) fn apply_fix(&mut self, rule: &SpacingRule) {
        // println!("PRE {:#?}", self);
        match rule.space.loc {
            SpaceLoc::After => self.fix_spacing_after(rule.space),
            SpaceLoc::Before => self.fix_spacing_before(rule.space),
            SpaceLoc::Around => self.fix_spacing_around(rule.space),
        };
        // println!("POST {:#?}", self)
    }

    pub(crate) fn explicit_fix(&mut self, rule: &SpacingRule) {
        //println!("PRE {:#?}", self);
        match rule.space.loc {
            SpaceLoc::After => self.fix_spacing_after(rule.space),
            SpaceLoc::Before => self.fix_spacing_before(rule.space),
            SpaceLoc::Around => self.fix_spacing_around(rule.space),
        };
        //println!("POST {:#?}", self)
    }

    fn fix_spacing_after(&mut self, space: Space) {
        match space.value {
            SpaceValue::Single => {
                // add space or set to single
                self.locations.1 = 1;
                // remove new line if any
                self.new_line.1 = false;
            },
            SpaceValue::Newline => {
                // add new line
                self.new_line.1 = true;
                // remove space if any
                self.locations.1 = 0;
;            },
            SpaceValue::SingleOptionalNewline => {
                if self.siblings_contain("\n") {
                    println!("TRUE");
                    self.new_line.1 = true;
                    self.locations.1 = 0;
                } else {
                    println!("FALSE");
                    self.locations.1 = 1;
                    self.new_line.1 = false;
                }
            },
            _ => {},
        };
    }

    fn fix_spacing_before(&mut self, space: Space) {
        match space.value {
            SpaceValue::Single => {
                self.locations.0 = 1;
                self.new_line.0 = false;
            },
            SpaceValue::Newline => {
                self.new_line.0 = true;
                self.locations.0 = 0;
;            },
            SpaceValue::SingleOptionalNewline => {
                if self.siblings_contain("\n") {
                    println!("TRUE");
                    self.new_line.0 = true;
                    self.locations.0 = 0;
                } else {
                    println!("FALSE");
                    self.locations.0 = 1;
                    self.new_line.0 = false;
                }
            },
            _ => {},
        }
    }

    fn fix_spacing_around(&mut self, space: Space) {
        match space.value {
            SpaceValue::Single => {
                self.locations = (1, 1);
                self.new_line = (false, false);
            },
            SpaceValue::Newline => {
                self.new_line = (true, true);
                self.locations = (0, 0);
            },
            _ => {},
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

fn calc_num_space_tkn(tkn: &SyntaxToken) -> u32 {
    let orig = tkn.text().as_str();
    let len = orig.chars().count();
    if orig.contains('\n') {
        (len - orig.matches('\n').count()) as u32
    } else {
        len as u32
    }
}
