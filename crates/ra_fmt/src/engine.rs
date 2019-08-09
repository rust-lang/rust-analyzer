use crate::dsl;
use ra_syntax::{
    ast::{self, AstNode, AstToken},
    Parse, SmolStr, SourceFile,
    SyntaxElement, TextUnit,
    SyntaxKind, SyntaxKind::*, 
    SyntaxNode, SyntaxToken, T,
    WalkEvent, TextRange,
};
use std::collections::HashMap;

#[derive(Debug)]

struct FmtTree {
    /// owned original root node
    original_node: SyntaxNode,
    /// We store `SpaceBlock`s in array. With this setup, we can refer to a
    /// specific block by index, dodging many lifetime issues.
    blocks: Vec<SpaceBlock>,
    /// Maps offset to an index of the block, for which the offset is the start
    /// offset.
    by_start_offset: HashMap<TextUnit, usize>,
    /// Maps offset to an index of the block, for which the offset is the end
    /// offset.
    by_end_offset: HashMap<TextUnit, usize>,
    // Arbitrary non-whitespace edits created by the last formatter phase.
    //fixes: Vec<AtomEdit>,
}

#[derive(Debug)]
pub(super) struct SpaceBlock {
    original: OriginalSpace,
    /// Block's textual content, which is seen and modified by formatting rules.
    new_text: Option<SmolStr>,
    /// If this block requires a newline to preserve semantics.
    ///
    /// True for blocks after comments. The engine takes care to never remove
    /// newline, even if some interaction of rules asks us to do so.
    semantic_newline: bool,
}



#[derive(Debug, Clone, Copy)]
pub(super) enum BlockPosition {
    Before,
    After,
}

/// Original whitespace token, if any, that backs a `SpaceBlock.
#[derive(Debug)]
pub(super) enum OriginalSpace {
    Some(SyntaxToken),
    None { offset: TextUnit },
}

impl OriginalSpace {
    fn text_range(&self) -> TextRange {
        match self {
            OriginalSpace::Some(token) => token.text_range(),
            OriginalSpace::None { offset } => TextRange::from_to(*offset, *offset),
        }
    }
}

impl SpaceBlock {
    fn new(original: OriginalSpace) -> SpaceBlock {
        let semantic_newline = match &original {
            OriginalSpace::Some(token) => {
                token.text().contains('\n')
            }
            OriginalSpace::None { .. } => false,
        };
        SpaceBlock { original, new_text: None, semantic_newline }
    }
    pub(super) fn set_line_break_preserving_existing_newlines(&mut self) {
        if self.has_newline() {
            return;
        }
        self.set_text("\n");
    }
    pub(super) fn set_text(&mut self, text: &str) {
        if self.semantic_newline && !text.contains('\n') {
            return;
        }
        match &self.original {
            OriginalSpace::Some(token) if token.text() == text && self.new_text.is_none() => return,
            _ => self.new_text = Some(text.into()),
        }
    }
    pub(super) fn text(&self) -> &str {
        if let Some(text) = self.new_text.as_ref() {
            return text.as_str();
        }
        self.original_text()
    }
    pub(crate) fn original_text(&self) -> &str {
        match &self.original {
            OriginalSpace::Some(token) => token.text().as_str(),
            OriginalSpace::None { .. } => "",
        }
    }
    pub(super) fn has_newline(&self) -> bool {
        self.text().contains('\n')
    }
}

impl FmtTree {
    pub(super) fn new(original_node: SyntaxNode) -> Self {
        Self {
            original_node,
            blocks: vec![],
            by_start_offset: HashMap::default(),
            by_end_offset: HashMap::default(),
        }
    }
}

fn walk(node: &SyntaxNode) -> impl Iterator<Item = SyntaxElement> {
    node.preorder_with_tokens().filter_map(|event| {
        if let WalkEvent::Enter(ele) = event {
            Some(ele)
        } else {
            None
        }
    })
}

fn format_pass(space_rules: &dsl::SpacingDsl, root: &SyntaxNode) {
    let mut fmt_root = FmtTree::new(root.clone());
    for node in walk(root) {
       for rule in space_rules.matching(node.clone()) {
           rule.apply(&node, &mut fmt_root)
       } 
    }
}
