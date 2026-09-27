//! Fills in the trivia of gaps next to elastic edges, in the spirit of Roslyn's elastic trivia.
use std::iter;

use itertools::Itertools;

use crate::{
    AstNode, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken, T,
    ast::{self, edit::IndentLevel, make::tokens},
    syntax_editor::SyntaxEditor,
};

pub(super) fn normalize(
    root: SyntaxNode,
    changed: impl Iterator<Item = SyntaxElement>,
) -> SyntaxNode {
    let elastic = |it: SyntaxToken| it.text().is_empty();
    let mut gaps = Vec::new();
    for token in changed.flat_map(|it| match it {
        SyntaxElement::Node(node) => {
            node.descendants_with_tokens().filter_map(|it| it.into_token()).collect()
        }
        SyntaxElement::Token(token) => vec![token],
    }) {
        if token.leading_trivia().any(elastic) {
            gaps.push((token.prev_non_trivia_token(), token.clone()));
        }
        if token.trailing_trivia().any(elastic)
            && let Some(next) = token.next_non_trivia_token()
        {
            gaps.push((Some(token), next));
        }
    }
    if gaps.is_empty() {
        return root;
    }
    let (editor, _) = SyntaxEditor::new(root);
    let text = |trivia: &mut dyn Iterator<Item = SyntaxToken>| {
        trivia.map(|it| it.text().to_owned()).collect::<String>()
    };
    for (prev, next) in gaps.into_iter().unique() {
        let Some(prev) = prev else {
            if next.parent_ancestors().last().is_some_and(|it| it.kind() == SyntaxKind::SOURCE_FILE)
            {
                let leading = text(&mut next.leading_trivia());
                let leading = tokens::trivia(leading.trim_start_matches('\n'));
                editor.splice_leading_trivia(&next, .., leading);
            }
            continue;
        };
        let (mut trailing, mut leading) =
            (text(&mut prev.trailing_trivia()), text(&mut next.leading_trivia()));
        let leading_breaks = (next.leading_trivia())
            .take_while(|it| it.kind() != SyntaxKind::COMMENT)
            .filter(|it| it.kind() == SyntaxKind::NEWLINE)
            .count();
        let existing =
            trailing.matches('\n').count().max(usize::from(!leading.is_empty())) + leading_breaks;
        let cap = match (prev.kind(), next.kind()) {
            (_, T![,] | T![;]) => 0,
            (T!['{'] | T!['('] | T!['['], _) | (_, T!['}'] | T![')'] | T![']']) => 1,
            _ => 2,
        };
        let breaks = existing.max(usize::from(line_break(&prev, &next))).min(cap);
        let comment = prev
            .trailing_trivia()
            .chain(next.leading_trivia())
            .any(|it| it.kind() == SyntaxKind::COMMENT);
        (trailing, leading) = match (comment, breaks) {
            (true, 0) => continue,
            (true, _) => {
                if !trailing.ends_with('\n') {
                    trailing = format!("{}\n", trailing.trim_end_matches([' ', '\t']));
                }
                let indent = indent(&next);
                let inner = indent + u8::from(matches!(next.kind(), T!['}'] | T![')'] | T![']']));
                let last = leading.matches('\n').count();
                let mut lines = leading.split('\n').enumerate().map(|(i, line)| {
                    match (i == last, line.trim()) {
                        (true, _) => format!("{indent}{}", line.trim_start()),
                        (false, "") => String::new(),
                        (false, _) => format!("{inner}{}", line.trim_start()),
                    }
                });
                let missing = breaks.saturating_sub(1 + leading_breaks);
                (trailing, format!("{}{}", "\n".repeat(missing), lines.join("\n")))
            }
            (false, 0) => (spacing(&prev, &next).to_owned(), String::new()),
            (false, _) => {
                let indent = own_indent(&next).unwrap_or_else(|| indent(&next).to_string());
                ("\n".to_owned(), format!("{}{indent}", "\n".repeat(breaks - 1)))
            }
        };
        editor.splice_trailing_trivia(&prev, .., tokens::trivia(&trailing));
        editor.splice_leading_trivia(&next, .., tokens::trivia(&leading));
    }
    editor.finish().new_root().clone()
}

fn line_break(prev: &SyntaxToken, next: &SyntaxToken) -> bool {
    let chain = |token: &SyntaxToken, edge: fn(&SyntaxNode) -> Option<SyntaxToken>| {
        let ancestors = token.parent_ancestors().take_while(|it| edge(it).as_ref() == Some(token));
        iter::once(SyntaxElement::Token(token.clone()))
            .chain(ancestors.map(SyntaxElement::Node))
            .collect::<Vec<_>>()
    };
    let rights = chain(next, SyntaxNode::first_non_trivia_token);
    chain(prev, SyntaxNode::last_non_trivia_token).iter().any(|left| {
        rights.iter().any(|right| {
            let Some(parent) = left.parent().filter(|it| Some(it) == right.parent().as_ref())
            else {
                return false;
            };
            let attr = matches!(left.kind(), SyntaxKind::ATTR | SyntaxKind::DOC_COMMENT)
                && (ast::Item::can_cast(parent.kind())
                    || ast::Stmt::can_cast(parent.kind())
                    || parent.parent().is_some_and(|list| {
                        matches!(
                            list.kind(),
                            SyntaxKind::RECORD_FIELD_LIST | SyntaxKind::VARIANT_LIST
                        ) && lists_lines(&list)
                    }));
            attr || lists_lines(&parent)
        })
    })
}

fn lists_lines(node: &SyntaxNode) -> bool {
    let (mut lines, mut inline) = (false, false);
    let pairs = node.children_with_tokens().tuple_windows();
    for (left, right) in pairs.filter(|(_, right)| right.kind() != SyntaxKind::EOF) {
        let trailing = left.last_non_trivia_token().into_iter().flat_map(|it| it.trailing_trivia());
        let leading = right.first_non_trivia_token().into_iter().flat_map(|it| it.leading_trivia());
        let gap: Vec<_> = trailing.chain(leading).collect();
        if gap.iter().any(|it| it.kind() == SyntaxKind::NEWLINE) {
            lines = true;
        } else if gap.iter().all(|it| !it.text().is_empty()) {
            inline = true;
        }
    }
    match node.kind() {
        SyntaxKind::SOURCE_FILE
        | SyntaxKind::ITEM_LIST
        | SyntaxKind::ASSOC_ITEM_LIST
        | SyntaxKind::EXTERN_ITEM_LIST
        | SyntaxKind::MATCH_ARM_LIST
        | SyntaxKind::VARIANT_LIST
        | SyntaxKind::RECORD_FIELD_LIST => lines || !inline,
        SyntaxKind::STMT_LIST => {
            let statements = node.children().filter(|it| ast::Stmt::can_cast(it.kind()));
            let mut statements =
                statements.map(|it| it.text_without_outer_trivia().contains_char('\n'));
            lines || (!inline && statements.clone().next().is_some()) || statements.any(|it| it)
        }
        SyntaxKind::RECORD_EXPR_FIELD_LIST | SyntaxKind::RECORD_PAT_FIELD_LIST => lines,
        _ => false,
    }
}

fn indent(next: &SyntaxToken) -> IndentLevel {
    let opener = |node: &SyntaxNode| {
        node.children_with_tokens()
            .filter_map(|it| it.into_token())
            .find(|it| matches!(it.kind(), T!['{'] | T!['('] | T!['[']))
    };
    if matches!(next.kind(), T!['}'] | T![')'] | T![']'])
        && let Some(open) = next.parent().and_then(|it| opener(&it))
    {
        return line_indent(&open);
    }
    next.parent_ancestors()
        .find_map(|it| {
            opener(&it).filter(|open| open.text_range().end() <= next.text_range().start())
        })
        .map_or(IndentLevel(0), |open| line_indent(&open) + 1)
}

fn line_indent(token: &SyntaxToken) -> IndentLevel {
    let mut start = token.clone();
    while let Some(prev) = start.prev_non_trivia_token() {
        let gap: Vec<_> = prev.trailing_trivia().chain(start.leading_trivia()).collect();
        if gap.iter().any(|it| it.kind() == SyntaxKind::NEWLINE)
            || start.leading_trivia().any(|it| !it.text().is_empty())
        {
            return match own_indent(&start) {
                Some(line) => {
                    IndentLevel((line.chars().take_while(|&it| it == ' ').count() / 4) as u8)
                }
                None if gap.iter().any(|it| it.text().is_empty()) => indent(&start),
                None => IndentLevel(0),
            };
        }
        start = prev;
    }
    IndentLevel::from_token(&start)
}

fn own_indent(token: &SyntaxToken) -> Option<String> {
    let leading: String = token.leading_trivia().map(|it| it.text().to_owned()).collect();
    let line = leading.rsplit('\n').next().unwrap_or_default();
    let blank = !line.is_empty() && line.chars().all(|it| it == ' ' || it == '\t');
    (blank && token.leading_trivia().all(|it| !it.text().is_empty())).then(|| line.to_owned())
}

fn spacing(prev: &SyntaxToken, next: &SyntaxToken) -> &'static str {
    let parent = |token: &SyntaxToken| token.parent().map(|it| it.kind());
    let generic = |token: &SyntaxToken| {
        matches!(
            parent(token),
            Some(
                SyntaxKind::GENERIC_ARG_LIST
                    | SyntaxKind::GENERIC_PARAM_LIST
                    | SyntaxKind::PATH_SEGMENT
                    | SyntaxKind::TYPE_ANCHOR
            )
        )
    };
    let closure_pipe = |token: &SyntaxToken| {
        token.kind() == T![|] && parent(token) == Some(SyntaxKind::PARAM_LIST)
    };
    let use_braces = |token: &SyntaxToken| parent(token) == Some(SyntaxKind::USE_TREE_LIST);
    let range = |token: &SyntaxToken| {
        matches!(parent(token), Some(SyntaxKind::RANGE_EXPR | SyntaxKind::RANGE_PAT))
    };
    let prefix = || {
        matches!(
            parent(prev),
            Some(
                SyntaxKind::PREFIX_EXPR
                    | SyntaxKind::REF_EXPR
                    | SyntaxKind::REF_TYPE
                    | SyntaxKind::REF_PAT
                    | SyntaxKind::SELF_PARAM
                    | SyntaxKind::MACRO_CALL
            )
        )
    };
    let callee = matches!(
        prev.kind(),
        SyntaxKind::IDENT
            | T![self]
            | T![Self]
            | T![super]
            | T![crate]
            | T![>]
            | T![')']
            | T![']']
            | T![!]
    );
    match (prev.kind(), next.kind()) {
        (_, T![;] | T![,] | T![')'] | T![']'] | T![.] | T![?] | T![:]) => "",
        (_, T![::]) if callee => "",
        (T!['('] | T!['['] | T![.] | T![::] | T![#], _) => "",
        (_, T!['('] | T!['[']) if callee => "",
        (_, T![!]) if parent(next) == Some(SyntaxKind::MACRO_CALL) => "",
        (T![<], _) if generic(prev) => "",
        (_, T![<] | T![>]) if generic(next) => "",
        (T![&] | T![!] | T![-] | T![*], _) if prefix() => "",
        (T![..] | T![..=], _) if range(prev) => "",
        (_, T![..] | T![..=]) if range(next) => "",
        (T![|], _) if closure_pipe(prev) && prev.next_sibling_or_token().is_some() => "",
        (_, T![|]) if closure_pipe(next) && next.prev_sibling_or_token().is_some() => "",
        (T!['{'], _) if use_braces(prev) => "",
        (_, T!['}']) if use_braces(next) => "",
        _ => " ",
    }
}
