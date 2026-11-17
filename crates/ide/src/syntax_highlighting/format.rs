//! Syntax highlighting for format macro strings.
use ide_db::{
    defs::Definition,
    syntax_helpers::format_string::{FormatSpecifier, is_format_string, lex_format_specifiers},
};
use span::Edition;
use syntax::{AstNode, AstToken, ast, ast::HasName};

use crate::{
    HlRange, HlTag,
    syntax_highlighting::{highlight::highlight_def, highlights::Highlights},
};

pub(super) fn highlight_format_string(
    stack: &mut Highlights,
    sema: &hir::Semantics<'_, ide_db::RootDatabase>,
    krate: Option<hir::Crate>,
    string: &ast::String,
    expanded_string: &ast::String,
    edition: Edition,
) {
    let start = string.syntax().text_range().start();
    if is_format_string(expanded_string) {
        // FIXME: Replace this with the HIR info we have now.
        lex_format_specifiers(string, &mut |piece_range, kind| {
            if let Some(highlight) = highlight_format_specifier(kind) {
                stack.add(HlRange {
                    range: piece_range + start,
                    highlight: highlight.into(),
                    binding_hash: None,
                });
            }
        });
    }
    let named_args: Vec<String> = expanded_string
        .syntax()
        .parent()
        .and_then(|lit| lit.parent())
        .and_then(ast::FormatArgsExpr::cast)
        .map(|fa| {
            fa.args().filter_map(|arg| arg.name().map(|n| n.syntax().text().to_string())).collect()
        })
        .unwrap_or_default();

    if let Some(parts) = sema.as_format_args_parts(string) {
        parts.into_iter().for_each(|(range, res)| {
            if let Some(res) = res {
                let rel = range - start;
                let text = &string.syntax().text()[rel];
                if named_args.iter().any(|n| n.as_str() == text) {
                    return;
                }
                stack.add(HlRange {
                    range,
                    highlight: highlight_def(sema, krate, Definition::from(res), edition, true),
                    binding_hash: None,
                })
            }
        })
    }
}

fn highlight_format_specifier(kind: FormatSpecifier) -> Option<HlTag> {
    match kind {
        FormatSpecifier::Open
        | FormatSpecifier::Close
        | FormatSpecifier::Colon
        | FormatSpecifier::Fill
        | FormatSpecifier::Align
        | FormatSpecifier::Sign
        | FormatSpecifier::NumberSign
        | FormatSpecifier::DollarSign
        | FormatSpecifier::Dot
        | FormatSpecifier::Asterisk
        | FormatSpecifier::QuestionMark => Some(HlTag::FormatSpecifier),

        FormatSpecifier::Integer | FormatSpecifier::Zero => Some(HlTag::NumericLiteral),

        FormatSpecifier::Escape => Some(HlTag::EscapeSequence),
        FormatSpecifier::Identifier => None,
    }
}
