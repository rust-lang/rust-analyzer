use crate::dsl::{Space, SpaceLoc, SpaceValue, SpacingDsl, SpacingRule};
// use crate::indent::{Indentation};
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};
use crate::edit_tree::Block;
use crate::whitespace::Whitespace;

use ra_syntax::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *}, Direction,
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};

use std::collections::{HashMap, HashSet};

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
        if check_ws(&token) { return Self::empty_before() }
        let value = calc_space_value(&token);
        Self {
            loc: SpaceLoc::Before,
            value,
        }
    }
    fn after(token: SyntaxToken) -> Space {
        if check_ws(&token) { return Self::empty_after() }
        let value = calc_space_value(&token);
        Self {
            loc: SpaceLoc::After,
            value,
        }
    }
}

pub struct Wspace {
    element: SyntaxElement,
    space_before: Space,
    space_after: Space,
}

impl PartialEq<SpacingRule> for Wspace {
    /// TODO not sure exactly how to compare these
    fn eq(&self, rule: &SpacingRule) -> bool {
        self.space_after == rule.space
        || self.space_after == rule.space
    }
}

impl Wspace {
    pub(crate) fn new(element: &SyntaxElement) -> Wspace {
        match &element {
            NodeOrToken::Node(node) => {
                Wspace::from_node(&node)
            },
            NodeOrToken::Token(token) => {
                Wspace::from_token(&token)
            },
        }
    }

    fn from_node(node: &SyntaxNode) -> Wspace {
        // must skip first siblings_with_tokens returns 'me' token as first
        let mut previous = node.siblings_with_tokens(Direction::Prev).skip(1);
        let mut next = node.siblings_with_tokens(Direction::Next).skip(1);

        let element = NodeOrToken::Node(node.clone());

        let (space_before, space_after) = filter_non_ws_node(previous.next(), next.next());

        Self { element, space_before, space_after, }
    }

    fn from_token(token: &SyntaxToken) -> Wspace {
        let element = NodeOrToken::Token(token.clone());

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

        Self { element, space_before, space_after, }
    }
}

fn check_ws(token: &SyntaxToken) -> bool {
    token.kind() == WHITESPACE
}

fn calc_space_value(tkn: &SyntaxToken) -> SpaceValue {
    let orig = tkn.text().as_str();
    let len = orig.chars().count();
    if orig.contains('\n') {
        SpaceValue::MultiLF((len - orig.matches('\n').count()) as u32)
    } else {
        SpaceValue::MultiSpace(len as u32)
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
        (_, _) => unimplemented!("this should be unreachable test out")
    }
}
