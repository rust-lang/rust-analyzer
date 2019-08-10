use crate::trav_util::walk_tokens;

use ra_syntax::{SyntaxNode, TextRange, SmolStr};

/// The result of formatting.
///
/// From this Diff, you can get either the resulting `String`, or the
/// reformatted syntax node.
#[derive(Debug)]
pub struct FmtDiff {
    original_node: SyntaxNode,
    edits: Vec<AtomEdit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AtomEdit {
    delete: TextRange,
    insert: SmolStr,
}

impl FmtDiff {

    pub fn reformat_node(node: &SyntaxNode) -> Self {
        let spacing = rules::spacing();
        engine::format_pass(space_rules: &SpacingDsl, root: &SyntaxNode)
    }

    fn replace(&mut self, range: TextRange, text: SmolStr) {
        self.edits.push(AtomEdit { delete: range, insert: text })
    }
}
