use crate::dsl::{Space, SpaceLoc, SpaceValue, SpacingDsl, SpacingRule};
// use crate::indent::{Indentation};
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};
use crate::et2::Block;

use ra_syntax::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *}, Direction,
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};

use std::collections::{HashMap, HashSet};
use std::fmt;

pub(crate) const INDENT: u32 = 4;
pub(crate) const ID_STR: &str = "    ";

impl Space {
    fn empty_before() -> Space {
        Self {
            loc: SpaceLoc::Before,
            value: SpaceValue::None,
        }
    }
    fn empty_after() -> Space {
        Self {
            loc: SpaceLoc::After,
            value: SpaceValue::None,
        }
    }
    fn before(token: SyntaxToken) -> Space {
        if !is_ws(&token) { return Self::empty_before() }
        let value = calc_space_value(&token);
        Self {
            loc: SpaceLoc::Before,
            value,
        }
    }
    fn after(token: SyntaxToken) -> Space {
        if !is_ws(&token) { return Self::empty_after() }
        let value = calc_space_value(&token);
        Self {
            loc: SpaceLoc::After,
            value,
        }
    }
}

impl fmt::Display for Space {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            SpaceValue::Single => write!(f, " "),
            SpaceValue::Newline => writeln!(f),
            SpaceValue::Indent(count) => write!(f, "\n{}", " ".repeat(count as usize)),
            SpaceValue::None => write!(f, ""),
            _ => {
                // unreachable!("no other writable variants")
                write!(f, " {:?} ", self.value)
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Whitespace {
    pub(crate) space_before: Space,
    pub(crate) space_after: Space,
}

impl fmt::Display for Whitespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.space_before)
    }
}

impl PartialEq<SpacingRule> for Whitespace {
    /// TODO not sure exactly how to compare these
    fn eq(&self, rule: &SpacingRule) -> bool {
        match rule.space.loc {
            SpaceLoc::Before => self.space_before == rule.space,
            SpaceLoc::After => self.space_after == rule.space,
            // TODO is this correct
            SpaceLoc::Around => self.space_before == rule.space && self.space_after == rule.space,
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

    pub(crate) fn from_rule(rule: &SpacingRule, l_blk: &Block, r_blk: &Block) -> Whitespace {
        match rule.space.loc {
            SpaceLoc::Before => {
                let space_before = Space { 
                    loc: rule.space.loc,
                    value: process_space_value(r_blk, rule.space.value),
                };
                
                Self {
                    space_before,
                    space_after: Space::empty_after(),
                }
            },
            SpaceLoc::After => {
                let space_after = Space { 
                    loc: SpaceLoc::Before,
                    value: process_space_value(l_blk, rule.space.value),
                };
                Self {
                    space_before: space_after,
                    space_after: Space::empty_after(),
                }
            },
            SpaceLoc::Around => {
                println!("SPACE AROUND");
                let space_before = Space { loc: SpaceLoc::Before, value: process_space_value(r_blk, rule.space.value), };
                let space_after = Space { loc: SpaceLoc::After, value: process_space_value(r_blk, rule.space.value), };
                Self {
                    space_before,
                    space_after,
                }
            },
        }
    }

    pub(crate) fn from_node(node: &SyntaxNode) -> Whitespace {
        // must skip first siblings_with_tokens returns 'me' token as first
        let mut previous = node.siblings_with_tokens(Direction::Prev);
        let mut next = node.siblings_with_tokens(Direction::Next);

        previous.next();
        next.next();

        let (space_before, space_after) = filter_non_ws_node(previous.next(), next.next());
        
        // println!("{:#?} -- {:#?}", node, space_before);
        
        Self { space_before, space_after, }
    }

    pub(crate) fn from_token(token: &SyntaxToken) -> Whitespace {
        println!("TOKENS {:?} {:?}", token.prev_token(), token.next_token());
        let (space_before, space_after) = match (token.prev_token(), token.next_token()) {
            (Some(pre), Some(post)) => {
                (Space::before(pre), Space::after(post))
            },
            (Some(pre), _) => {
                (Space::before(pre), Space::empty_after())
            },
            (_, Some(post)) => {
                (Space::empty_before(), Space::after(post))
            },
            (_, _) => unimplemented!("this should be unreachable test out")
        };
        
        // println!("{:#?} -- {:#?}", token, space_before);
        
        Self { space_before, space_after, }
    }

    pub(crate) fn space_kind(&self) -> SpaceValue {
        self.space_before.value
    }

    pub(crate) fn space_loc(&self) -> SpaceLoc {
        self.space_before.loc
    }

    pub(crate) fn set_indent(&mut self, indent: u32) {
        self.space_before.value = SpaceValue::Indent(indent)
    }
    /// TODO is this how to do it??
    pub(crate) fn set_space(&mut self, space: Space) {
        assert!(space.loc == SpaceLoc::Before);
        self.space_before.value = space.value
    }

    pub(crate) fn set_from_whitespace(&mut self, space: Whitespace) {
        println!("ORIG {:#?}", self);
        let Whitespace {
            space_before, space_after,
        } = space;
        *self = Whitespace { space_before, space_after };
        println!("MUT {:#?}", self);
    }

    pub(crate) fn match_space_before(&self, space: Space) -> bool {
        self.space_before.value == space.value
    }
    pub(crate) fn match_space_after(&self, space: Space) -> bool {
        self.space_after.value == space.value
    }
}

fn is_ws(token: &SyntaxToken) -> bool {
    token.kind() == WHITESPACE
}

fn calc_space_value(tkn: &SyntaxToken) -> SpaceValue {
    let orig = tkn.text().as_str();
    let tkn_len = orig.chars().count();

    // indent is `\n\s\s\s\s` or some variation
    if orig.contains('\n') && orig.contains(' ') {
        //                 subtract everything that is not a space
        SpaceValue::Indent((tkn_len - orig.matches('\n').count()) as u32)
    // just new line
    } else if orig.contains('\n') {
        if tkn_len == 1 {
            SpaceValue::Newline
        } else {
            SpaceValue::MultiLF((tkn_len - orig.matches('\n').count()) as u32)
        }
    // just spaces
    } else if orig.contains(' ') {
        if tkn_len == 1 {
            SpaceValue::Single
        } else {
            SpaceValue::MultiSpace((tkn_len - orig.matches('\n').count()) as u32)
        }
    } else {
        SpaceValue::None
    }
}

fn filter_non_ws_node(pre: Option<SyntaxElement>, post: Option<SyntaxElement>) -> (Space, Space) {
    match (pre, post) {
        (Some(SyntaxElement::Token(pre)), Some(SyntaxElement::Token(post))) => {
            (Space::before(pre), Space::after(post))
        },
        (Some(SyntaxElement::Token(pre)), _) => {
            (Space::before(pre), Space::empty_after())
        },
        (_, Some(SyntaxElement::Token(post))) => {
            (Space::empty_before(), Space::after(post))
        },
        (None, None) => (Space::empty_before(), Space::empty_after()),
        a => {
            println!("this should be unreachable test out {:?}", a);
            (Space::empty_before(), Space::empty_after())
        },
    }
}

/// TODO some left block parents may have diff contains, check when SpaceLoc::After
fn process_space_value(blk: &Block, space: SpaceValue) -> SpaceValue {
    use SpaceValue::*;
    match space {
        Newline | MultiLF(_) => Newline,
        Single | MultiSpace(_) => Single,
        NoneOptionalNewline | NoneOrNewline => {
            if blk.siblings_contain("\n") {
                Newline
            } else {
                None
            }
        },
        SingleOptionalNewline | SingleOrNewline => {
            if blk.siblings_contain("\n") {
                Newline
            } else {
                Single
            }
        },
        Indent(count) => Indent(count),
        None => space,
    }
}
