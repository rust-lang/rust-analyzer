//! Conversions between proc_macro bridge types and tt types.

use intern::sym;
use proc_macro_api::token_stream::SpanLike;

pub(super) mod ours {
    pub(crate) type Literal<Span> = tt::Literal<Span>;
    pub(crate) type Punct<Span> = tt::Punct<Span>;
    pub(crate) type Ident<Span> = tt::Ident<Span>;
    pub(crate) type DocComment<Span> = tt::DocComment<Span>;
    pub(crate) type Leaf<Span> = tt::Leaf<Span>;
    pub(crate) type Group<Span> = proc_macro_api::token_stream::Group<Span>;
    pub(crate) type TokenTree<Span> = proc_macro_api::token_stream::TokenTree<Span>;
    pub(crate) type TokenStream<Span> = proc_macro_api::token_stream::TokenStream<Span>;
}

#[expect(clippy::module_inception, reason = "this is not a mistake")]
pub(super) mod bridge {
    use rustc_proc_macro::bridge as pm_bridge;

    use super::ours;

    pub(crate) use pm_bridge::*;

    pub(crate) type TokenTree<Span> =
        pm_bridge::TokenTree<ours::TokenStream<Span>, Span, intern::Symbol>;
    pub(crate) type Literal<Span> = pm_bridge::Literal<Span, intern::Symbol>;
    pub(crate) type Group<Span> = pm_bridge::Group<ours::TokenStream<Span>, Span>;
    pub(crate) type Punct<Span> = pm_bridge::Punct<Span>;
    pub(crate) type Ident<Span> = pm_bridge::Ident<Span, intern::Symbol>;
}

pub(super) fn literal_into_bridge<Span>(literal: ours::Literal<Span>) -> bridge::Literal<Span> {
    let kind = match literal.kind {
        tt::LitKind::Byte => bridge::LitKind::Byte,
        tt::LitKind::Char => bridge::LitKind::Char,
        tt::LitKind::Integer => bridge::LitKind::Integer,
        tt::LitKind::Float => bridge::LitKind::Float,
        tt::LitKind::Str => bridge::LitKind::Str,
        tt::LitKind::StrRaw(count) => bridge::LitKind::StrRaw(count),
        tt::LitKind::ByteStr => bridge::LitKind::ByteStr,
        tt::LitKind::ByteStrRaw(count) => bridge::LitKind::ByteStrRaw(count),
        tt::LitKind::CStr => bridge::LitKind::CStr,
        tt::LitKind::CStrRaw(count) => bridge::LitKind::CStrRaw(count),
        tt::LitKind::Err(()) => bridge::LitKind::ErrWithGuar,
    };
    let (symbol, suffix) = literal.text_and_suffix_symbols();
    bridge::Literal { kind, symbol, suffix, span: literal.span }
}

fn punct_into_bridge<Span>(punct: ours::Punct<Span>) -> bridge::Punct<Span> {
    bridge::Punct {
        // FIXME: Is `as u8` correct here?
        ch: punct.char as u8,
        joint: punct.spacing != tt::Spacing::Alone,
        span: punct.span,
    }
}

fn ident_into_bridge<Span>(ident: ours::Ident<Span>) -> bridge::Ident<Span> {
    bridge::Ident { sym: ident.sym, is_raw: ident.is_raw.yes(), span: ident.span }
}

fn group_into_bridge<Span: SpanLike>(group: ours::Group<Span>) -> bridge::Group<Span> {
    let delimiter = match group.delimiter.kind {
        tt::DelimiterKind::Parenthesis => rustc_proc_macro::Delimiter::Parenthesis,
        tt::DelimiterKind::Brace => rustc_proc_macro::Delimiter::Brace,
        tt::DelimiterKind::Bracket => rustc_proc_macro::Delimiter::Bracket,
        tt::DelimiterKind::Invisible => rustc_proc_macro::Delimiter::None,
    };
    let span = bridge::DelimSpan {
        open: group.delimiter.open,
        close: group.delimiter.close,
        entire: group.delimiter.open.cover(group.delimiter.close),
    };
    bridge::Group { delimiter, stream: group.stream, span }
}

fn doc_comment_into_bridge<Span: Copy>(
    doc_comment: ours::DocComment<Span>,
    output: &mut Vec<bridge::TokenTree<Span>>,
) {
    let is_inner = doc_comment.doc_style == tt::DocCommentStyle::Inner;
    let to_reserve = output.spare_capacity_mut().len() + 2 + usize::from(is_inner);
    output.reserve(to_reserve);
    output.push(bridge::TokenTree::Punct(bridge::Punct {
        ch: b'#',
        joint: false,
        span: doc_comment.span,
    }));
    if is_inner {
        output.push(bridge::TokenTree::Punct(bridge::Punct {
            ch: b'!',
            joint: false,
            span: doc_comment.span,
        }));
    }
    output.push(bridge::TokenTree::Group(bridge::Group {
        delimiter: rustc_proc_macro::Delimiter::Bracket,
        span: bridge::DelimSpan::from_single(doc_comment.span),
        stream: Some(ours::TokenStream::new(vec![
            ours::TokenTree::Leaf(ours::Leaf::Ident(ours::Ident {
                sym: sym::doc,
                is_raw: tt::IdentIsRaw::No,
                span: doc_comment.span,
            })),
            ours::TokenTree::Leaf(ours::Leaf::Punct(ours::Punct {
                char: '=',
                spacing: tt::Spacing::Alone,
                span: doc_comment.span,
            })),
            ours::TokenTree::Leaf(ours::Leaf::Literal(doc_comment.literal_for_proc_macros())),
        ])),
    }));
}

fn token_tree_into_bridge<Span: SpanLike>(
    token_tree: ours::TokenTree<Span>,
    output: &mut Vec<bridge::TokenTree<Span>>,
) {
    let tree = match token_tree {
        ours::TokenTree::Leaf(ours::Leaf::Literal(literal)) => {
            bridge::TokenTree::Literal(literal_into_bridge(literal))
        }
        ours::TokenTree::Leaf(ours::Leaf::Ident(ident)) => {
            bridge::TokenTree::Ident(ident_into_bridge(ident))
        }
        ours::TokenTree::Leaf(ours::Leaf::Punct(punct)) => {
            bridge::TokenTree::Punct(punct_into_bridge(punct))
        }
        ours::TokenTree::Leaf(ours::Leaf::DocComment(doc_comment)) => {
            return doc_comment_into_bridge(doc_comment, output);
        }
        ours::TokenTree::Group(group) => bridge::TokenTree::Group(group_into_bridge(group)),
    };
    output.push(tree);
}

pub(super) fn token_stream_into_bridge<Span: SpanLike>(
    token_stream: ours::TokenStream<Span>,
) -> Vec<bridge::TokenTree<Span>> {
    let mut result = Vec::with_capacity(token_stream.len());
    for tree in token_stream.iter() {
        token_tree_into_bridge(tree.clone(), &mut result);
    }
    result
}

fn literal_from_bridge<Span>(literal: bridge::Literal<Span>) -> ours::Literal<Span> {
    let kind = match literal.kind {
        bridge::LitKind::Byte => tt::LitKind::Byte,
        bridge::LitKind::Char => tt::LitKind::Char,
        bridge::LitKind::Integer => tt::LitKind::Integer,
        bridge::LitKind::Float => tt::LitKind::Float,
        bridge::LitKind::Str => tt::LitKind::Str,
        bridge::LitKind::StrRaw(count) => tt::LitKind::StrRaw(count),
        bridge::LitKind::ByteStr => tt::LitKind::ByteStr,
        bridge::LitKind::ByteStrRaw(count) => tt::LitKind::ByteStrRaw(count),
        bridge::LitKind::CStr => tt::LitKind::CStr,
        bridge::LitKind::CStrRaw(count) => tt::LitKind::CStrRaw(count),
        bridge::LitKind::ErrWithGuar => tt::LitKind::Err(()),
    };
    match literal.suffix {
        Some(suffix) => {
            tt::Literal::new(literal.symbol.as_str(), literal.span, kind, suffix.as_str())
        }
        None => {
            tt::Literal { text_and_suffix: literal.symbol, span: literal.span, kind, suffix_len: 0 }
        }
    }
}

fn punct_from_bridge<Span>(punct: bridge::Punct<Span>) -> ours::Punct<Span> {
    ours::Punct {
        char: char::from(punct.ch),
        spacing: if punct.joint { tt::Spacing::Joint } else { tt::Spacing::Alone },
        span: punct.span,
    }
}

fn ident_from_bridge<Span>(ident: bridge::Ident<Span>) -> ours::Ident<Span> {
    ours::Ident {
        sym: ident.sym,
        is_raw: if ident.is_raw { tt::IdentIsRaw::Yes } else { tt::IdentIsRaw::No },
        span: ident.span,
    }
}

fn group_from_bridge<Span>(group: bridge::Group<Span>) -> ours::Group<Span> {
    let kind = match group.delimiter {
        rustc_proc_macro::Delimiter::Parenthesis => tt::DelimiterKind::Parenthesis,
        rustc_proc_macro::Delimiter::Brace => tt::DelimiterKind::Brace,
        rustc_proc_macro::Delimiter::Bracket => tt::DelimiterKind::Bracket,
        rustc_proc_macro::Delimiter::None => tt::DelimiterKind::Invisible,
    };
    let delimiter = tt::Delimiter { open: group.span.open, close: group.span.close, kind };
    ours::Group { delimiter, stream: group.stream }
}

pub(super) fn token_tree_from_bridge<Span>(
    token_tree: bridge::TokenTree<Span>,
) -> ours::TokenTree<Span> {
    match token_tree {
        bridge::TokenTree::Literal(literal) => {
            ours::TokenTree::Leaf(ours::Leaf::Literal(literal_from_bridge(literal)))
        }
        bridge::TokenTree::Ident(ident) => {
            ours::TokenTree::Leaf(ours::Leaf::Ident(ident_from_bridge(ident)))
        }
        bridge::TokenTree::Punct(punct) => {
            ours::TokenTree::Leaf(ours::Leaf::Punct(punct_from_bridge(punct)))
        }
        bridge::TokenTree::Group(group) => ours::TokenTree::Group(group_from_bridge(group)),
    }
}
