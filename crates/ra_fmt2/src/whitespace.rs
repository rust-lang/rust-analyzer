use crate::dsl::{Space, SpaceLoc, SpaceValue, SpacingDsl, SpacingRule};
// use crate::indent::{Indentation};
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};
use crate::edit_tree::Block;

use ra_syntax::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *}, Direction,
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};

use std::collections::{HashMap, HashSet};

pub(crate) const INDENT: u32 = 4;
pub(crate) const ID_STR: &str = "    ";

#[derive(Clone, Debug)]
/// Whitespace holds all whitespace information for each Block.
/// Accessed from any Block's get_whitespace fn.
pub(crate) struct Whitespace {
    original: SyntaxElement,
    text_range: TextRange,
    // additional_spaces: u32,
    /// Previous and next contain "\n".
    pub(crate) new_line: (bool, bool),
    /// Start and end location of token.
    pub(crate) text_len: (u32, u32),
    pub(crate) starts_with_lf: bool,
}

impl std::fmt::Display for Whitespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.new_line.0 {
            if self.starts_with_lf {
                write!(f, "\n{}", " ".repeat(self.text_len.0 as usize))
            } else {
                writeln!(f)
            }
        } else {
            write!(f, "{}", " ".repeat(self.text_len.0 as usize))
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
        let mut previous = node.siblings_with_tokens(Direction::Prev);
        let mut next = node.siblings_with_tokens(Direction::Next);
        // must call next twice siblings_with_tokens returns 'me' token as first
        previous.next();
        next.next();

        match (previous.next(), next.next()) {
            (Some(prev), Some(next)) => {
                let (starts_with_lf, prev_space) = if prev.kind() == WHITESPACE {
                    let prev = prev.as_token().unwrap();
                    (prev.text().starts_with('\n'), calc_num_space_tkn(prev))
                } else {
                    (false, 0)
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
                    text_len: (prev_space, next_space),
                    starts_with_lf,
                }
            },
            (Some(prev), None) => {
                let (starts_with_lf, prev_space) = if prev.kind() == WHITESPACE {
                    let prev = prev.as_token().unwrap();
                    (prev.text().starts_with('\n'), calc_num_space_tkn(prev))
                } else {
                    (false, 0)
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
                    text_len: (prev_space, 0),
                    starts_with_lf,
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
                    text_len: (0, next_space),
                    starts_with_lf: false,
                }
            },
            // handles root node
            (None, None) => {
                Self {
                    original: NodeOrToken::Node(node.clone()),
                    text_range: node.text_range(),
                    new_line: (false, false),
                    // additional_spaces,
                    text_len: (0, 0),
                    starts_with_lf: false,
                }
            },
        }
    }

    fn from_token(token: &SyntaxToken) -> Whitespace {
        match (token.prev_token(), token.next_token()) {
            (Some(prev), Some(next)) => {
                let (starts_with_lf, prev_space) = if prev.kind() == WHITESPACE {
                    (prev.text().starts_with('\n'), calc_num_space_tkn(&prev))
                } else {
                    (false, 0)
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
                    text_len: (prev_space, next_space),
                    starts_with_lf,
                }
            }
            (Some(prev), None) => {
                let (starts_with_lf, prev_space) = if prev.kind() == WHITESPACE {
                    (prev.text().starts_with('\n'), calc_num_space_tkn(&prev))
                } else {
                    (false, 0)
                };
                let new_line = (prev.text().as_str().contains('\n'), false);

                Self {
                    original: NodeOrToken::Token(token.clone()),
                    text_range: token.text_range(),
                    new_line,
                    // additional_spaces,
                    text_len: (prev_space, 0),
                    starts_with_lf,
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
                    text_len: (0, next_space),
                    starts_with_lf: false,
                }
            }
            _ => unreachable!("Whitespace::new"),
        }
    }

    /// Walks siblings to search for pat.
    pub(crate) fn siblings_contain(&self, pat: &str) -> bool {
        if let Some(tkn) = self.original.clone().into_token() {
            walk_tokens(&tkn.parent())
                .any(|tkn| {
                    tkn.text().as_str() == pat
                })
        } else {
            false
        }
    }

    pub(crate) fn match_space_after(&self, value: SpaceValue) -> bool {
        match value {
            SpaceValue::Single => self.text_len.1 == 1,
            SpaceValue::SingleOrNewline => self.text_len.1 == 1 || self.new_line.1,
            SpaceValue::SingleOptionalNewline => self.text_len.1 == 1 || self.new_line.1,
            SpaceValue::Newline => self.new_line.1,
            SpaceValue::NoneOrNewline => self.text_len.1 == 0 || self.new_line.1,
            SpaceValue::NoneOptionalNewline => self.text_len.1 == 0 && self.new_line.1,
            SpaceValue::None => self.text_len.1 == 0 || !self.new_line.1,
            _ => { panic!("found multi") },
        }
    }

    pub(crate) fn match_space_before(&self, value: SpaceValue) -> bool {
        match value {
            SpaceValue::Single => self.text_len.0 == 1,
            SpaceValue::SingleOrNewline => self.text_len.0 == 1 || self.new_line.0,
            SpaceValue::SingleOptionalNewline => self.text_len.0 == 1 || self.new_line.0,
            SpaceValue::Newline => self.new_line.0,
            SpaceValue::NoneOrNewline => self.text_len.0 == 0,
            SpaceValue::NoneOptionalNewline => self.text_len.0 == 0,
            SpaceValue::None => self.text_len.0 == 0 || !self.new_line.0,
            _ => { panic!("found multi") },
        }
    }

    pub(crate) fn match_space_around(&self, value: SpaceValue) -> bool {
        match value {
            SpaceValue::Single => self.text_len.0 == 1 && self.text_len.1 == 1,
            SpaceValue::SingleOrNewline => {
                (self.text_len.0 == 1 && self.text_len.1 == 1)
                || (self.new_line.0 && self.new_line.1)
            },
            SpaceValue::SingleOptionalNewline => {
                (self.text_len.0 == 1 && self.text_len.1 == 1)
                || (self.new_line.0 && self.new_line.1)
            },
            SpaceValue::Newline => self.new_line.0 && self.new_line.1,
            SpaceValue::NoneOrNewline => {
                (self.text_len.0 == 0 && self.text_len.1 == 0)
                || (self.new_line.0 && self.new_line.1)
            },
            SpaceValue::NoneOptionalNewline => {
                (self.text_len.0 == 0 && self.text_len.1 == 0)
                && (self.new_line.0 && self.new_line.1)
            },
            SpaceValue::None => {
                (self.text_len.0 == 0 && self.text_len.1 == 0)
                && (!self.new_line.0 && !self.new_line.1)
            },
            _ => { panic!("found multi") },
        }
    }

    pub(crate) fn fix_spacing_after(&mut self, space: Space) {
        match space.value {
            SpaceValue::Single => {
                // add space or set to single
                self.text_len.1 = 1;
                // remove new line if any
                self.new_line.1 = false;
            },
            SpaceValue::Newline => {
                // add new line
                self.new_line.1 = true;
                // remove space if any
                self.text_len.1 = 0;
            },
            SpaceValue::SingleOptionalNewline => {
                if self.siblings_contain("\n") {
                    self.new_line.1 = true;
                    self.text_len.1 = 0;
                } else {
                    self.text_len.1 = 1;
                    self.new_line.1 = false;
                }
            },
            _ => {},
        };
    }

    pub(crate) fn fix_spacing_before(&mut self, space: Space) {
        match space.value {
            SpaceValue::Single => {
                self.text_len.0 = 1;
                self.new_line.0 = false;
            },
            SpaceValue::Newline => {
                self.new_line.0 = true;
                self.text_len.0 = 0;
            },
            SpaceValue::SingleOptionalNewline => {
                if self.siblings_contain("\n") {
                    self.new_line.0 = true;
                    self.text_len.0 = 0;
                } else {
                    self.text_len.0 = 1;
                    self.new_line.0 = false;
                }
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

fn calc_node_len(tkn: &SyntaxNode) -> u32 {
    let orig = tkn.text().to_string();
    orig.chars().count() as u32
}
