use crate::dsl::{SpacingDsl, SpacingRule, SpaceLoc, SpaceValue};
use crate::pattern::{Pattern, PatternSet};
use crate::rules::spacing;
use crate::trav_util::{walk, walk_nodes, walk_tokens};

use ra_syntax::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, TextRange, TextUnit, WalkEvent, T,
};
use rowan::{GreenNode, cursor};

use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub(crate) struct SynBlock {
    //indent: some enum?
    element: SyntaxElement,
    text: SmolStr,
    parent: Option<SyntaxNode>,
    children: Vec<SynBlock>,
    range: TextRange,
    prev_whitespace: Option<Whitespace>,
}
