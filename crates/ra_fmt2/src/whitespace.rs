use crate::dsl::{Space, SpaceLoc, SpaceValue, SpacingDsl, SpacingRule};
// use crate::indent::{Indentation};
use crate::edit_tree::Block;
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;

use ra_syntax::{
    Direction, NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};

use std::collections::{HashMap, HashSet};
use std::fmt;

pub(crate) const USER_INDENT_SIZE: u32 = 4;

impl Space {
    fn empty_before() -> Space {
        Self { loc: SpaceLoc::Before, value: SpaceValue::None }
    }
    fn empty_after() -> Space {
        Self { loc: SpaceLoc::After, value: SpaceValue::None }
    }
    fn before(token: SyntaxToken) -> Space {
        if !is_ws(&token) {
            return Self::empty_before();
        }
        let value = calc_space_value(&token);
        Self { loc: SpaceLoc::Before, value }
    }
    fn after(token: SyntaxToken) -> Space {
        if !is_ws(&token) {
            return Self::empty_after();
        }
        let value = calc_space_value(&token);
        Self { loc: SpaceLoc::After, value }
    }
}

impl fmt::Display for Space {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            SpaceValue::Single => write!(f, " "),
            SpaceValue::Newline => writeln!(f),
            SpaceValue::Indent{ level, alignment, } => write!(f, "\n{}", " ".repeat((level * USER_INDENT_SIZE + alignment) as usize)),
            SpaceValue::None => write!(f, ""),
            _ => {
                // unreachable!("no other writable variants")
                write!(f, " {:?} ", self.value)
            }
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
            NodeOrToken::Node(node) => Whitespace::from_node(&node),
            NodeOrToken::Token(token) => Whitespace::from_token(&token),
        }
    }

    pub(crate) fn from_rule(rule: &SpacingRule, l_blk: &Block, r_blk: &Block) -> Whitespace {
        match rule.space.loc {
            SpaceLoc::Before => {
                let space_before =
                    Space { loc: rule.space.loc, value: process_space_value(r_blk, rule) };

                Self { space_before, space_after: Space::empty_after() }
            }
            SpaceLoc::After => {
                let space_after =
                    Space { loc: SpaceLoc::Before, value: process_space_value(l_blk, rule) };
                Self { space_before: space_after, space_after: Space::empty_after() }
            }
            SpaceLoc::Around => {
                let space_before =
                    Space { loc: SpaceLoc::Before, value: process_space_value(r_blk, rule) };
                Self { space_before, space_after: r_blk.get_whitespace().space_after }
            }
        }
    }

    pub(crate) fn from_node(node: &SyntaxNode) -> Whitespace {
        // must skip first siblings_with_tokens returns 'me' token as first
        let mut previous = node.siblings_with_tokens(Direction::Prev).skip(1);
        let mut next = node.siblings_with_tokens(Direction::Next).skip(1);

        let (space_before, space_after) = filter_nodes(previous.next(), next.next());
        Self { space_before, space_after }
    }

    pub(crate) fn from_token(token: &SyntaxToken) -> Whitespace {
        let (space_before, space_after) = match (token.prev_token(), token.next_token()) {
            (Some(pre), Some(post)) => (Space::before(pre), Space::after(post)),
            (Some(pre), _) => (Space::before(pre), Space::empty_after()),
            (_, Some(post)) => (Space::empty_before(), Space::after(post)),
            (_, _) => unreachable!("next or previous token returned a node"),
        };

        Self { space_before, space_after }
    }

    pub(crate) fn space_kind_prev(&self) -> SpaceValue {
        self.space_before.value
    }

    pub(crate) fn space_loc_prev(&self) -> SpaceLoc {
        self.space_before.loc
    }
    /// Sets indent level calculated from indent count to produce
    ///  - level = count / USER_INDNET_SIZE
    ///  - alignment = count % USER_INDNET_SIZE
    /// Only set space_before as these are the only tokens that are
    /// read when building the final string.
    pub(crate) fn set_indent(&mut self, indent: u32) {
        let (level, alignment) = calc_indent(indent);
        self.space_before.value = SpaceValue::Indent{ level, alignment, }
    }

    /// Returns true if `space` a `SpaceRule` matches `Whitespace` value.
    pub(crate) fn match_space_before(&self, space: Space) -> bool {
        use SpaceValue::*;
        if self.space_before.value == space.value {
            return true;
        }
        match space.value {
            Single => match self.space_before.value {
                Single => true,
                _ => false,
            },
            SingleOrNewline => match self.space_before.value {
                Single | Newline | MultiLF(_) | Indent{ .. } => true,
                _ => false,
            },
            SingleOptionalNewline => match self.space_before.value {
                Single | Newline | MultiLF(_) | Indent{ .. } => true,
                _ => false,
            },
            // TODO make sure valid
            Newline => match self.space_before.value {
                Newline | MultiLF(_) | Indent{ .. } => true,
                _ => false,
            },
            NoneOrNewline => match self.space_before.value {
                Newline | MultiLF(_) | Indent{ .. } => true,
                _ => false,
            },
            NoneOptionalNewline => match self.space_before.value {
                Newline | MultiLF(_) | Indent{ .. } => true,
                _ => false,
            },
            // TODO from here on the rules never set these they will
            // never be checked.
            MultiSpace(_len) => match self.space_before.value {
                Single | MultiSpace(_) => true,
                _ => false,
            },
            MultiLF(_len) => match self.space_before.value {
                Newline | MultiLF(_) | Indent{ .. } => true,
                _ => false,
            },
            Indent{ .. } => match self.space_before.value {
                Newline | MultiLF(_) | Indent{ .. } => true,
                _ => false,
            },
            None => None == self.space_before.value,
        }
    }
}

fn is_ws(token: &SyntaxToken) -> bool {
    token.kind() == WHITESPACE
}

fn calc_indent(len: u32) -> (u32, u32) {
    let level = len / USER_INDENT_SIZE;
    let alignment = len % USER_INDENT_SIZE;
    (level, alignment)
}

fn calc_space_value(tkn: &SyntaxToken) -> SpaceValue {
    let orig = tkn.text().as_str();
    let tkn_len = orig.chars().count();
    // indent is `\n\s\s\s\s` or some variation
    if orig.contains('\n') && orig.contains(' ') {
        let (level, alignment) = calc_indent(orig.matches(' ').count() as u32);
        SpaceValue::Indent { level, alignment }
    // just new line
    } else if orig.contains('\n') {
        if tkn_len == 1 {
            SpaceValue::Newline
        } else {
            SpaceValue::MultiLF((orig.matches('\n').count()) as u32)
        }
    // just spaces
    } else if orig.contains(' ') {
        if tkn_len == 1 {
            SpaceValue::Single
        } else {
            SpaceValue::MultiSpace((orig.matches(' ').count()) as u32)
        }
    } else {
        SpaceValue::None
    }
}

fn filter_nodes(pre: Option<SyntaxElement>, post: Option<SyntaxElement>) -> (Space, Space) {
    match (pre, post) {
        (Some(SyntaxElement::Token(pre)), Some(SyntaxElement::Token(post))) => {
            (Space::before(pre), Space::after(post))
        }
        (Some(SyntaxElement::Token(pre)), _) => (Space::before(pre), Space::empty_after()),
        (_, Some(SyntaxElement::Token(post))) => (Space::empty_before(), Space::after(post)),
        (None, None) => (Space::empty_before(), Space::empty_after()),
        _non_token_tuple => {
            // println!("this is anything that is not a token {:?}", a);
            (Space::empty_before(), Space::empty_after())
        }
    }
}

/// TODO some left block parents may have diff contains, check when SpaceLoc::After
fn process_space_value(blk: &Block, space: &SpacingRule) -> SpaceValue {
    use SpaceValue::*;
    match space.space.value {
        Newline | MultiLF(_) => Newline,
        Single | MultiSpace(_) => Single,
        NoneOptionalNewline | NoneOrNewline => {
            if blk.siblings_contain("\n") {
                Newline
            } else {
                SpaceValue::None
            }
        }
        SingleOptionalNewline | SingleOrNewline => {
            if blk.siblings_contain("\n") {
                Newline
            } else {
                Single
            }
        }
        Indent {level, alignment, } => Indent { level, alignment, },
        None => space.space.value,
    }
}
