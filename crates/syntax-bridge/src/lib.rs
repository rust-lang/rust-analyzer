//! Conversions between [`SyntaxNode`] and [`tt::TokenTree`].

use std::fmt;

use intern::{sym, Symbol};
use rustc_hash::{FxHashMap, FxHashSet};
use span::{Edition, SpanAnchor, SpanData, SpanMap};
use stdx::{format_to, never, non_empty_vec::NonEmptyVec};
use syntax::{
    ast::{self, make::tokens::doc_comment},
    format_smolstr, AstToken, Parse, PreorderWithTokens, SmolStr, SyntaxElement,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, SyntaxTreeBuilder, TextRange, TextSize, WalkEvent, T,
};
use tt::{
    token_to_literal, BinOpToken, Delimiter, IdentIsRaw, RefTokenCursor, Spacing, TokenKind,
    TokenStream,
};

pub mod prettify_macro_expansion;
mod to_parser_input;
pub use to_parser_input::to_parser_input;
// FIXME: we probably should re-think  `token_tree_to_syntax_node` interfaces
pub use ::parser::TopEntryPoint;

#[cfg(test)]
mod tests;

pub trait SpanMapper<S> {
    fn span_for(&self, range: TextRange) -> S;
}

impl<S> SpanMapper<SpanData<S>> for SpanMap<S>
where
    SpanData<S>: Copy,
{
    fn span_for(&self, range: TextRange) -> SpanData<S> {
        self.span_at(range.start())
    }
}

impl<S: Copy, SM: SpanMapper<S>> SpanMapper<S> for &SM {
    fn span_for(&self, range: TextRange) -> S {
        SM::span_for(self, range)
    }
}

/// Dummy things for testing where spans don't matter.
pub mod dummy_test_span_utils {

    use span::{Span, SyntaxContextId};

    use super::*;

    pub const DUMMY: Span = Span {
        range: TextRange::empty(TextSize::new(0)),
        anchor: span::SpanAnchor {
            file_id: span::EditionedFileId::new(
                span::FileId::from_raw(0xe4e4e),
                span::Edition::CURRENT,
            ),
            ast_id: span::ROOT_ERASED_FILE_AST_ID,
        },
        ctx: SyntaxContextId::ROOT,
    };

    pub struct DummyTestSpanMap;

    impl SpanMapper<Span> for DummyTestSpanMap {
        fn span_for(&self, range: syntax::TextRange) -> Span {
            Span {
                range,
                anchor: span::SpanAnchor {
                    file_id: span::EditionedFileId::new(
                        span::FileId::from_raw(0xe4e4e),
                        span::Edition::CURRENT,
                    ),
                    ast_id: span::ROOT_ERASED_FILE_AST_ID,
                },
                ctx: SyntaxContextId::ROOT,
            }
        }
    }
}

/// Doc comment desugaring differs between mbe and proc-macros.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DocCommentDesugarMode {
    /// Desugars doc comments as quoted raw strings
    Mbe,
    /// Desugars doc comments as quoted strings
    ProcMacro,
}

/// Converts a syntax tree to a [`tt::Subtree`] using the provided span map to populate the
/// subtree's spans.
pub fn syntax_node_to_token_tree<Ctx, SpanMap>(
    node: &SyntaxNode,
    map: SpanMap,
    span: SpanData<Ctx>,
    mode: DocCommentDesugarMode,
) -> tt::TokenStream<SpanData<Ctx>>
where
    SpanData<Ctx>: Copy + fmt::Debug,
    SpanMap: SpanMapper<SpanData<Ctx>>,
{
    let mut c = Converter::new(node, map, Default::default(), Default::default(), span, mode);
    convert_to_token_tree(&mut c)
}

/// Converts a syntax tree to a [`tt::Subtree`] using the provided span map to populate the
/// subtree's spans. Additionally using the append and remove parameters, the additional tokens can
/// be injected or hidden from the output.
pub fn syntax_node_to_token_tree_modified<Ctx, SpanMap>(
    node: &SyntaxNode,
    map: SpanMap,
    append: FxHashMap<SyntaxElement, Vec<tt::Token<SpanData<Ctx>>>>,
    remove: FxHashSet<SyntaxElement>,
    call_site: SpanData<Ctx>,
    mode: DocCommentDesugarMode,
) -> tt::TokenStream<SpanData<Ctx>>
where
    SpanMap: SpanMapper<SpanData<Ctx>>,
    SpanData<Ctx>: Copy + fmt::Debug,
{
    let mut c = Converter::new(node, map, append, remove, call_site, mode);
    convert_to_token_tree(&mut c)
}

// The following items are what `rustc` macro can be parsed into :
// link: https://github.com/rust-lang/rust/blob/9ebf47851a357faa4cd97f4b1dc7835f6376e639/src/libsyntax/ext/expand.rs#L141
// * Expr(P<ast::Expr>)                     -> token_tree_to_expr
// * Pat(P<ast::Pat>)                       -> token_tree_to_pat
// * Ty(P<ast::Ty>)                         -> token_tree_to_ty
// * Stmts(SmallVec<[ast::Stmt; 1]>)        -> token_tree_to_stmts
// * Items(SmallVec<[P<ast::Item>; 1]>)     -> token_tree_to_items
//
// * TraitItems(SmallVec<[ast::TraitItem; 1]>)
// * AssocItems(SmallVec<[ast::AssocItem; 1]>)
// * ForeignItems(SmallVec<[ast::ForeignItem; 1]>

/// Converts a [`tt::TokenStream`] back to a [`SyntaxNode`].
/// The produced `SpanMap` contains a mapping from the syntax nodes offsets to the subtree's spans.
pub fn token_tree_to_syntax_node<Ctx>(
    tt: &tt::TokenStream<SpanData<Ctx>>,
    entry_point: parser::TopEntryPoint,
    edition: parser::Edition,
) -> (Parse<SyntaxNode>, SpanMap<Ctx>)
where
    SpanData<Ctx>: Copy + fmt::Debug,
    Ctx: PartialEq,
{
    let parser_input = to_parser_input(edition, RefTokenCursor::new(tt));
    let parser_output = entry_point.parse(&parser_input, edition);
    let mut tree_sink = TtTreeSink::new(RefTokenCursor::new(tt));
    for event in parser_output.iter() {
        match event {
            parser::Step::Token { kind, n_input_tokens: n_raw_tokens } => {
                tree_sink.token(kind, n_raw_tokens)
            }
            parser::Step::FloatSplit { ends_in_dot: has_pseudo_dot } => {
                todo!()
                // tree_sink.float_split(has_pseudo_dot)
            }
            parser::Step::Enter { kind } => tree_sink.start_node(kind),
            parser::Step::Exit => tree_sink.finish_node(),
            parser::Step::Error { msg } => tree_sink.error(msg.to_owned()),
        }
    }
    tree_sink.finish()
}

/// Convert a string to a `TokenTree`. The spans of the subtree will be anchored to the provided
/// anchor with the given context.
pub fn parse_to_token_tree<Ctx>(
    edition: Edition,
    anchor: SpanAnchor,
    ctx: Ctx,
    text: &str,
) -> Option<tt::TokenStream<SpanData<Ctx>>>
where
    SpanData<Ctx>: Copy + fmt::Debug,
    Ctx: Copy,
{
    let lexed = parser::LexedStr::new(edition, text);
    if lexed.errors().next().is_some() {
        return None;
    }
    let mut conv =
        RawConverter { lexed, anchor, pos: 0, ctx, mode: DocCommentDesugarMode::ProcMacro };
    Some(convert_to_token_tree(&mut conv))
}

/// Convert a string to a `TokenTree`. The passed span will be used for all spans of the produced subtree.
pub fn parse_to_token_tree_static_span<S>(
    edition: Edition,
    span: S,
    text: &str,
) -> Option<tt::TokenStream<S>>
where
    S: Copy + fmt::Debug,
{
    let lexed = parser::LexedStr::new(edition, text);
    if lexed.errors().next().is_some() {
        return None;
    }
    let mut conv =
        StaticRawConverter { lexed, pos: 0, span, mode: DocCommentDesugarMode::ProcMacro };
    Some(convert_to_token_tree(&mut conv))
}

struct TokenStreamBuilder<S> {
    tt: Vec<tt::TokenTree<S>>,
}

fn convert_to_token_tree<S, C>(conv: &mut C) -> tt::TokenStream<S>
where
    C: TokenConverter<S>,
    S: Copy + fmt::Debug,
{
    let mut stack = NonEmptyVec::new((None, vec![]));

    while let Some((token, abs_range, text)) = conv.bump() {
        let (b, tt) = stack.last_mut();

        let kind = match token {
            L_PAREN => {
                stack.push((Some((Delimiter::Parenthesis, conv.span_for(abs_range))), vec![]));
                continue;
            }
            L_CURLY => {
                stack.push((Some((Delimiter::Brace, conv.span_for(abs_range))), vec![]));
                continue;
            }
            L_BRACK => {
                stack.push((Some((Delimiter::Bracket, conv.span_for(abs_range))), vec![]));
                continue;
            }

            R_CURLY if matches!(b, Some((Delimiter::Brace, _))) => {
                if let Some((Some((delim, open)), tt)) = stack.pop() {
                    stack.last_mut().1.push(tt::TokenTree::Delimited(
                        tt::DelimSpan { open, close: conv.span_for(abs_range) },
                        tt::DelimSpacing::new(Spacing::Alone, Spacing::Alone),
                        delim,
                        TokenStream(tt.into_boxed_slice()),
                    ));
                }
                continue;
            }
            R_PAREN if matches!(b, Some((Delimiter::Parenthesis, _))) => {
                if let Some((Some((delim, open)), tt)) = stack.pop() {
                    stack.last_mut().1.push(tt::TokenTree::Delimited(
                        tt::DelimSpan { open, close: conv.span_for(abs_range) },
                        tt::DelimSpacing::new(Spacing::Alone, Spacing::Alone),
                        delim,
                        TokenStream(tt.into_boxed_slice()),
                    ));
                }
                continue;
            }
            R_BRACK if matches!(b, Some((Delimiter::Bracket, _))) => {
                if let Some((Some((delim, open)), tt)) = stack.pop() {
                    stack.last_mut().1.push(tt::TokenTree::Delimited(
                        tt::DelimSpan { open, close: conv.span_for(abs_range) },
                        tt::DelimSpacing::new(Spacing::Alone, Spacing::Alone),
                        delim,
                        TokenStream(tt.into_boxed_slice()),
                    ));
                }
                continue;
            }

            L_ANGLE => TokenKind::Lt,
            R_ANGLE => TokenKind::Gt,

            DOLLAR => TokenKind::Dollar,
            SEMICOLON => TokenKind::Semi,
            COMMA => TokenKind::Comma,
            AT => TokenKind::At,
            POUND => TokenKind::Pound,
            TILDE => TokenKind::Tilde,
            QUESTION => TokenKind::Question,
            AMP => TokenKind::BinOp(BinOpToken::And),
            PIPE => TokenKind::BinOp(BinOpToken::Or),
            PLUS => TokenKind::BinOp(BinOpToken::Plus),
            STAR => TokenKind::BinOp(BinOpToken::Star),
            SLASH => TokenKind::BinOp(BinOpToken::Slash),
            CARET => TokenKind::BinOp(BinOpToken::Caret),
            PERCENT => TokenKind::BinOp(BinOpToken::Percent),
            MINUS => TokenKind::BinOp(BinOpToken::Minus),
            UNDERSCORE => TokenKind::Ident(sym::underscore.clone(), tt::IdentIsRaw::No),
            DOT => TokenKind::Dot,
            DOT2 => unreachable!(),
            DOT3 => unreachable!(),
            DOT2EQ => unreachable!(),
            COLON => TokenKind::Colon,
            COLON2 => unreachable!(),
            EQ => TokenKind::Eq,
            EQ2 => TokenKind::EqEq,
            FAT_ARROW => TokenKind::FatArrow,
            BANG => TokenKind::Not,
            NEQ => TokenKind::Ne,
            THIN_ARROW => TokenKind::RArrow,
            LTEQ => TokenKind::Le,
            GTEQ => TokenKind::Ge,
            PLUSEQ => unreachable!(),
            MINUSEQ => unreachable!(),
            PIPEEQ => unreachable!(),
            AMPEQ => unreachable!(),
            CARETEQ => unreachable!(),
            SLASHEQ => unreachable!(),
            STAREQ => unreachable!(),
            PERCENTEQ => unreachable!(),
            AMP2 => unreachable!(),
            PIPE2 => unreachable!(),
            SHL => TokenKind::Lt,
            SHR => TokenKind::Gt,
            SHLEQ => unreachable!(),
            SHREQ => unreachable!(),
            // FIXME: split up (raw) c string literals to an ident and a string literal when edition < 2021.
            k if k.is_literal() => TokenKind::Literal(Box::new(token_to_literal(&text))),
            // FIXME: Doc desugaring
            COMMENT => continue,
            LIFETIME_IDENT => TokenKind::Lifetime(Symbol::intern(text)),
            ident if ident.is_any_identifier() => {
                let (raw, sym) = IdentIsRaw::split_from_symbol(text);
                TokenKind::Ident(Symbol::intern(sym), raw)
            }
            WHITESPACE => continue,
            kind => unreachable!("{kind:?}"),
        };
        let spacing = match conv.peek() {
            Some(kind) if is_single_token_op(kind) => tt::Spacing::Joint,
            _ => tt::Spacing::Alone,
        };

        tt.push(tt::TokenTree::Token(tt::Token::new(kind, conv.span_for(abs_range)), spacing));
    }

    // If we get here, we've consumed all input tokens.
    // We might have more than one subtree in the stack, if the delimiters are improperly balanced.
    // Merge them so we're left with one.
    // while let Some(entry) = stack.pop() {
    //     let parent = stack.last_mut();

    //     let leaf: tt::Leaf<_> = tt::Punct {
    //         span: entry.delimiter.open,
    //         char: match entry.delimiter.kind {
    //             tt::DelimiterKind::Parenthesis => '(',
    //             tt::DelimiterKind::Brace => '{',
    //             tt::DelimiterKind::Bracket => '[',
    //             tt::DelimiterKind::Invisible => '$',
    //         },
    //         spacing: tt::Spacing::Alone,
    //     }
    //     .into();
    //     parent.token_trees.push(leaf.into());
    //     parent.token_trees.extend(entry.token_trees);
    // }

    let (_delim, token_trees) = stack.into_last();
    assert!(_delim.is_none());
    TokenStream(token_trees.into_boxed_slice())
}

fn is_single_token_op(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        EQ | L_ANGLE
            | R_ANGLE
            | BANG
            | AMP
            | PIPE
            | TILDE
            | AT
            | DOT
            | COMMA
            | SEMICOLON
            | COLON
            | POUND
            | DOLLAR
            | QUESTION
            | PLUS
            | MINUS
            | STAR
            | SLASH
            | PERCENT
            | CARET
            // LIFETIME_IDENT will be split into a sequence of `'` (a single quote) and an
            // identifier.
            | LIFETIME_IDENT
    )
}

/// Returns the textual content of a doc comment block as a quoted string
/// That is, strips leading `///` (or `/**`, etc)
/// and strips the ending `*/`
/// And then quote the string, which is needed to convert to `tt::Literal`
///
/// Note that proc-macros desugar with string literals where as macro_rules macros desugar with raw string literals.
pub fn desugar_doc_comment_text(text: &str, mode: DocCommentDesugarMode) -> (Symbol, tt::LitKind) {
    match mode {
        DocCommentDesugarMode::Mbe => {
            let mut num_of_hashes = 0;
            let mut count = 0;
            for ch in text.chars() {
                count = match ch {
                    '"' => 1,
                    '#' if count > 0 => count + 1,
                    _ => 0,
                };
                num_of_hashes = num_of_hashes.max(count);
            }

            // Quote raw string with delimiters
            (Symbol::intern(text), tt::LitKind::StrRaw(num_of_hashes))
        }
        // Quote string with delimiters
        DocCommentDesugarMode::ProcMacro => {
            (Symbol::intern(&format_smolstr!("{}", text.escape_debug())), tt::LitKind::Str)
        }
    }
}

// fn convert_doc_comment<S: Copy>(
//     token: &syntax::SyntaxToken,
//     span: S,
//     mode: DocCommentDesugarMode,
// ) -> Option<Vec<tt::TokenTree<S>>> {
//     let comment = ast::Comment::cast(token.clone())?;
//     let doc = comment.kind().doc?;

//     let mk_ident = |s: &str| {
//         tt::TokenTree::from(tt::Leaf::from(tt::Ident {
//             sym: Symbol::intern(s),
//             span,
//             is_raw: tt::IdentIsRaw::No,
//         }))
//     };

//     let mk_punct = |c: char| {
//         tt::TokenTree::from(tt::Leaf::from(tt::Punct {
//             char: c,
//             spacing: tt::Spacing::Alone,
//             span,
//         }))
//     };

//     let mk_doc_literal = |comment: &ast::Comment| {
//         let prefix_len = comment.prefix().len();
//         let mut text = &comment.text()[prefix_len..];

//         // Remove ending "*/"
//         if comment.kind().shape == ast::CommentShape::Block {
//             text = &text[0..text.len() - 2];
//         }
//         let (text, kind) = desugar_doc_comment_text(text, mode);
//         let lit = tt::Literal { symbol: text, span, kind, suffix: None };

//         tt::TokenTree::from(tt::Leaf::from(lit))
//     };

//     // Make `doc="\" Comments\""
//     let meta_tkns = Box::new([mk_ident("doc"), mk_punct('='), mk_doc_literal(&comment)]);

//     // Make `#![]`
//     let mut token_trees = Vec::with_capacity(3);
//     token_trees.push(mk_punct('#'));
//     if let ast::CommentPlacement::Inner = doc {
//         token_trees.push(mk_punct('!'));
//     }
//     token_trees.push(tt::TokenTree::from(tt::Subtree {
//         delimiter: tt::Delimiter { open: span, close: span, kind: tt::DelimiterKind::Bracket },
//         token_trees: meta_tkns,
//     }));

//     Some(token_trees)
// }

/// A raw token (straight from lexer) converter
struct RawConverter<'a, Ctx> {
    lexed: parser::LexedStr<'a>,
    pos: usize,
    anchor: SpanAnchor,
    ctx: Ctx,
    mode: DocCommentDesugarMode,
}
/// A raw token (straight from lexer) converter that gives every token the same span.
struct StaticRawConverter<'a, S> {
    lexed: parser::LexedStr<'a>,
    pos: usize,
    span: S,
    mode: DocCommentDesugarMode,
}

trait TokenConverter<S>: Sized {
    // fn convert_doc_comment(&self, token: &Self::Token, span: S) -> Option<Vec<tt::TokenTree<S>>>;

    fn bump(&mut self) -> Option<(SyntaxKind, TextRange, &str)>;

    fn peek(&self) -> Option<SyntaxKind>;

    fn span_for(&self, range: TextRange) -> S;

    fn call_site(&self) -> S;
}

impl<Ctx: Copy> TokenConverter<SpanData<Ctx>> for RawConverter<'_, Ctx>
where
    SpanData<Ctx>: Copy,
{
    // fn convert_doc_comment(
    //     &self,
    //     &token: &usize,
    //     span: SpanData<Ctx>,
    // ) -> Option<Vec<tt::TokenTree<SpanData<Ctx>>>> {
    //     let text = self.lexed.text(token);
    //     convert_doc_comment(&doc_comment(text), span, self.mode)
    // }

    fn bump(&mut self) -> Option<(SyntaxKind, TextRange, &str)> {
        if self.pos == self.lexed.len() {
            return None;
        }
        let token = self.pos;
        self.pos += 1;
        let range = self.lexed.text_range(token);
        let range = TextRange::new(range.start.try_into().ok()?, range.end.try_into().ok()?);

        Some((self.lexed.kind(token), range, self.lexed.text(token)))
    }

    fn peek(&self) -> Option<SyntaxKind> {
        if self.pos == self.lexed.len() {
            return None;
        }
        Some(self.lexed.kind(self.pos))
    }

    fn span_for(&self, range: TextRange) -> SpanData<Ctx> {
        SpanData { range, anchor: self.anchor, ctx: self.ctx }
    }

    fn call_site(&self) -> SpanData<Ctx> {
        SpanData { range: TextRange::empty(0.into()), anchor: self.anchor, ctx: self.ctx }
    }
}

impl<S> TokenConverter<S> for StaticRawConverter<'_, S>
where
    S: Copy,
{
    // fn convert_doc_comment(&self, &token: &usize, span: S) -> Option<Vec<tt::TokenTree<S>>> {
    //     let text = self.lexed.text(token);
    //     convert_doc_comment(&doc_comment(text), span, self.mode)
    // }

    fn bump(&mut self) -> Option<(SyntaxKind, TextRange, &str)> {
        if self.pos == self.lexed.len() {
            return None;
        }
        let token = self.pos;
        self.pos += 1;
        let range = self.lexed.text_range(token);
        let range = TextRange::new(range.start.try_into().ok()?, range.end.try_into().ok()?);

        Some((self.lexed.kind(self.pos), range, self.lexed.text(self.pos)))
    }

    fn peek(&self) -> Option<SyntaxKind> {
        if self.pos == self.lexed.len() {
            return None;
        }
        Some(self.lexed.kind(self.pos))
    }

    fn span_for(&self, _: TextRange) -> S {
        self.span
    }

    fn call_site(&self) -> S {
        self.span
    }
}

struct Converter<SpanMap, S> {
    current: Option<SyntaxToken>,
    current_leaves: Vec<tt::Token<S>>,
    preorder: PreorderWithTokens,
    range: TextRange,
    punct_offset: Option<(SyntaxToken, TextSize)>,
    /// Used to make the emitted text ranges in the spans relative to the span anchor.
    map: SpanMap,
    append: FxHashMap<SyntaxElement, Vec<tt::Token<S>>>,
    remove: FxHashSet<SyntaxElement>,
    call_site: S,
    mode: DocCommentDesugarMode,
}

impl<SpanMap, S> Converter<SpanMap, S> {
    fn new(
        node: &SyntaxNode,
        map: SpanMap,
        append: FxHashMap<SyntaxElement, Vec<tt::Token<S>>>,
        remove: FxHashSet<SyntaxElement>,
        call_site: S,
        mode: DocCommentDesugarMode,
    ) -> Self {
        let mut this = Converter {
            current: None,
            preorder: node.preorder_with_tokens(),
            range: node.text_range(),
            punct_offset: None,
            map,
            append,
            remove,
            call_site,
            current_leaves: vec![],
            mode,
        };
        let first = this.next_token();
        this.current = first;
        this
    }

    fn next_token(&mut self) -> Option<SyntaxToken> {
        while let Some(ev) = self.preorder.next() {
            match ev {
                WalkEvent::Enter(token) => {
                    if self.remove.contains(&token) {
                        match token {
                            syntax::NodeOrToken::Token(_) => {
                                continue;
                            }
                            node => {
                                self.preorder.skip_subtree();
                                if let Some(mut v) = self.append.remove(&node) {
                                    v.reverse();
                                    self.current_leaves.extend(v);
                                    return None;
                                }
                            }
                        }
                    } else if let syntax::NodeOrToken::Token(token) = token {
                        return Some(token);
                    }
                }
                WalkEvent::Leave(ele) => {
                    if let Some(mut v) = self.append.remove(&ele) {
                        v.reverse();
                        self.current_leaves.extend(v);
                        return None;
                    }
                }
            }
        }
        None
    }
}

impl<S, SpanMap> TokenConverter<S> for Converter<SpanMap, S>
where
    S: Copy,
    SpanMap: SpanMapper<S>,
{
    // fn convert_doc_comment(&self, token: &Self::Token, span: S) -> Option<Vec<tt::TokenTree<S>>> {
    //     convert_doc_comment(token.token(), span, self.mode)
    // }

    fn bump(&mut self) -> Option<(SyntaxKind, TextRange, &str)> {
        if let Some((punct, offset)) = self.punct_offset.clone() {
            if usize::from(offset) + 1 < punct.text().len() {
                let offset = offset + TextSize::of('.');
                let range = punct.text_range();
                self.punct_offset = Some((punct.clone(), offset));
                let range = TextRange::at(range.start() + offset, TextSize::of('.'));
                return Some((
                    SyntaxKind::from_char(punct.text().chars().nth(offset.into()).unwrap())
                        .unwrap(),
                    range,
                    "",
                ));
            }
        }

        // FIXME bring this back
        // if let Some(leaf) = self.current_leaves.pop() {
        //     if self.current_leaves.is_empty() {
        //         self.current = self.next_token();
        //     }
        //     return Some((SynToken::Leaf(leaf), TextRange::empty(TextSize::new(0))));
        // }

        let curr = self.current.clone()?;
        if !self.range.contains_range(curr.text_range()) {
            return None;
        }

        self.current = self.next_token();
        let token = if curr.kind().is_punct() {
            self.punct_offset = Some((curr.clone(), 0.into()));
            let range = curr.text_range();
            let range = TextRange::at(range.start(), TextSize::of('.'));
            (SyntaxKind::from_char(curr.text().chars().next().unwrap()).unwrap(), range, "")
        } else {
            self.punct_offset = None;
            let range = curr.text_range();
            (
                curr.kind(),
                range,
                // FIXME: lifetimes begone
                unsafe { std::mem::transmute::<&str, &str>(curr.text()) },
            )
        };

        Some(token)
    }

    fn peek(&self) -> Option<SyntaxKind> {
        if let Some((punct, mut offset)) = self.punct_offset.clone() {
            offset += TextSize::of('.');
            if usize::from(offset) < punct.text().len() {
                return Some(
                    SyntaxKind::from_char(punct.text().chars().nth(offset.into()).unwrap())
                        .unwrap(),
                );
            }
        }

        let curr = self.current.clone()?;
        if !self.range.contains_range(curr.text_range()) {
            return None;
        }

        let token = if curr.kind().is_punct() {
            SyntaxKind::from_char(curr.text().chars().next().unwrap()).unwrap()
        } else {
            curr.kind()
        };
        Some(token)
    }

    fn span_for(&self, range: TextRange) -> S {
        self.map.span_for(range)
    }
    fn call_site(&self) -> S {
        self.call_site
    }
}

struct TtTreeSink<'a, Ctx>
where
    SpanData<Ctx>: Copy,
{
    buf: String,
    cursor: tt::RefTokenCursor<'a, SpanData<Ctx>>,
    text_pos: TextSize,
    inner: SyntaxTreeBuilder,
    token_map: SpanMap<Ctx>,
    needs_spacing_if_punct: bool,
}

impl<'a, Ctx> TtTreeSink<'a, Ctx>
where
    SpanData<Ctx>: Copy,
{
    fn new(cursor: tt::RefTokenCursor<'a, SpanData<Ctx>>) -> Self {
        TtTreeSink {
            buf: String::new(),
            cursor,
            text_pos: 0.into(),
            inner: SyntaxTreeBuilder::default(),
            token_map: SpanMap::empty(),
            needs_spacing_if_punct: false,
        }
    }

    fn finish(mut self) -> (Parse<SyntaxNode>, SpanMap<Ctx>) {
        self.token_map.finish();
        (self.inner.finish(), self.token_map)
    }
}

fn delim_to_str(d: tt::Delimiter, closing: bool) -> Option<&'static str> {
    let texts = match d {
        tt::Delimiter::Parenthesis => "()",
        tt::Delimiter::Brace => "{}",
        tt::Delimiter::Bracket => "[]",
        tt::Delimiter::Invisible => return None,
    };

    let idx = closing as usize;
    Some(&texts[idx..texts.len() - (1 - idx)])
}

impl<Ctx> TtTreeSink<'_, Ctx>
where
    SpanData<Ctx>: Copy + fmt::Debug,
    Ctx: PartialEq,
{
    // /// Parses a float literal as if it was a one to two name ref nodes with a dot inbetween.
    // /// This occurs when a float literal is used as a field access.
    // fn float_split(&mut self, has_pseudo_dot: bool) {
    //     let (text, span) = match self.cursor.token_tree() {
    //         Some(tt::buffer::TokenTreeRef::Leaf(
    //             tt::Leaf::Literal(tt::Literal {
    //                 symbol: text,
    //                 span,
    //                 kind: tt::LitKind::Float,
    //                 suffix: _,
    //             }),
    //             _,
    //         )) => (text.as_str(), *span),
    //         tt => unreachable!("{tt:?}"),
    //     };
    //     // FIXME: Span splitting
    //     match text.split_once('.') {
    //         Some((left, right)) => {
    //             assert!(!left.is_empty());

    //             self.inner.start_node(SyntaxKind::NAME_REF);
    //             self.inner.token(SyntaxKind::INT_NUMBER, left);
    //             self.inner.finish_node();
    //             self.token_map.push(self.text_pos + TextSize::of(left), span);

    //             // here we move the exit up, the original exit has been deleted in process
    //             self.inner.finish_node();

    //             self.inner.token(SyntaxKind::DOT, ".");
    //             self.token_map.push(self.text_pos + TextSize::of(left) + TextSize::of("."), span);

    //             if has_pseudo_dot {
    //                 assert!(right.is_empty(), "{left}.{right}");
    //             } else {
    //                 assert!(!right.is_empty(), "{left}.{right}");
    //                 self.inner.start_node(SyntaxKind::NAME_REF);
    //                 self.inner.token(SyntaxKind::INT_NUMBER, right);
    //                 self.token_map.push(self.text_pos + TextSize::of(text), span);
    //                 self.inner.finish_node();

    //                 // the parser creates an unbalanced start node, we are required to close it here
    //                 self.inner.finish_node();
    //             }
    //             self.text_pos += TextSize::of(text);
    //         }
    //         None => unreachable!(),
    //     }
    //     self.cursor = self.cursor.bump();
    // }

    fn token(&mut self, kind: SyntaxKind, mut n_tokens: u8) {
        if kind == LIFETIME_IDENT {
            n_tokens = 2;
        }

        let mut combined_span = None;
        let mut last_spacing = Spacing::Joint;
        'tokens: for _ in 0..n_tokens {
            let Some((t, spacing)) = self.cursor.next() else { break };
            last_spacing = spacing;
            let text = match t.kind {
                TokenKind::Eq => "=",
                TokenKind::Lt => "<",
                TokenKind::Le => "<=",
                TokenKind::EqEq => "==",
                TokenKind::Ne => "!=",
                TokenKind::Ge => ">=",
                TokenKind::Gt => ">",
                TokenKind::AndAnd => "&&",
                TokenKind::OrOr => "||",
                TokenKind::Not => "!",
                TokenKind::Tilde => "~",
                TokenKind::BinOp(b) => match b {
                    tt::BinOpToken::Plus => "+",
                    tt::BinOpToken::Minus => "-",
                    tt::BinOpToken::Star => "*",
                    tt::BinOpToken::Slash => "/",
                    tt::BinOpToken::Percent => "%",
                    tt::BinOpToken::Caret => "^",
                    tt::BinOpToken::And => "&",
                    tt::BinOpToken::Or => "|",
                    tt::BinOpToken::Shl => "<<",
                    tt::BinOpToken::Shr => ">>",
                },
                TokenKind::BinOpEq(b) => match b {
                    tt::BinOpToken::Plus => "+=",
                    tt::BinOpToken::Minus => "-=",
                    tt::BinOpToken::Star => "*=",
                    tt::BinOpToken::Slash => "/=",
                    tt::BinOpToken::Percent => "%=",
                    tt::BinOpToken::Caret => "^=",
                    tt::BinOpToken::And => "&=",
                    tt::BinOpToken::Or => "|=",
                    tt::BinOpToken::Shl => "<<=",
                    tt::BinOpToken::Shr => ">>=",
                },
                TokenKind::At => "@",
                TokenKind::Dot => ".",
                TokenKind::DotDot => "..",
                TokenKind::DotDotDot => "...",
                TokenKind::DotDotEq => "..=",
                TokenKind::Comma => ",",
                TokenKind::Semi => ";",
                TokenKind::Colon => ":",
                TokenKind::PathSep => "::",
                TokenKind::RArrow => "->",
                TokenKind::LArrow => "<-",
                TokenKind::FatArrow => "=>",
                TokenKind::Pound => "#",
                TokenKind::Dollar => "$",
                TokenKind::Question => "?",
                TokenKind::SingleQuote => "'",
                TokenKind::OpenDelim(d) => match delim_to_str(d, false) {
                    Some(it) => it,
                    None => continue,
                },
                TokenKind::CloseDelim(d) => match delim_to_str(d, true) {
                    Some(it) => it,
                    None => continue,
                },
                TokenKind::Literal(lit) => {
                    let buf_l = self.buf.len();
                    format_to!(self.buf, "{lit}");
                    debug_assert_ne!(self.buf.len() - buf_l, 0);
                    self.text_pos += TextSize::new((self.buf.len() - buf_l) as u32);
                    ""
                }
                TokenKind::Ident(ref ident, raw) => {
                    if raw.yes() {
                        self.buf.push_str("r#");
                        self.text_pos += TextSize::of("r#");
                    }
                    ident.as_str()
                }
                TokenKind::Lifetime(_) => todo!(),
                TokenKind::DocComment(_, _, _) => todo!(),
                TokenKind::Eof => todo!(),
            };
            self.buf += text;
            self.text_pos += TextSize::of(text);
            combined_span = match combined_span {
                None => Some(t.span),
                Some(prev_span) => Some(Self::merge_spans(prev_span, t.span)),
            }
        }

        self.token_map.push(self.text_pos, combined_span.expect("expected at least one token"));
        self.inner.token(kind, self.buf.as_str());
        self.buf.clear();
        // FIXME: Emitting whitespace for this is really just a hack, we should get rid of it.
        // Add whitespace between adjoint puncts
        // let next = last.bump();
        // if let (
        //     Some(tt::buffer::TokenTreeRef::Leaf(tt::Leaf::Punct(curr), _)),
        //     Some(tt::buffer::TokenTreeRef::Leaf(tt::Leaf::Punct(next), _)),
        // ) = (last.token_tree(), next.token_tree())
        // {
        //     // Note: We always assume the semi-colon would be the last token in
        //     // other parts of RA such that we don't add whitespace here.
        //     //
        //     // When `next` is a `Punct` of `'`, that's a part of a lifetime identifier so we don't
        //     // need to add whitespace either.
        //     if curr.spacing == tt::Spacing::Alone && curr.char != ';' && next.char != '\'' {
        //         self.inner.token(WHITESPACE, " ");
        //         self.text_pos += TextSize::of(' ');
        //         self.token_map.push(self.text_pos, curr.span);
        //     }
        // }
        // let is_punct = SyntaxKind::is_punct(kind);
        // if self.needs_spacing_if_punct && is_punct &&  {

        // }
        // self.needs_spacing_if_punct = last_spacing == Spacing::Alone
        //     && is_punct
        //     && kind != SyntaxKind::SEMICOLON;
    }

    fn start_node(&mut self, kind: SyntaxKind) {
        self.inner.start_node(kind);
    }

    fn finish_node(&mut self) {
        self.inner.finish_node();
    }

    fn error(&mut self, error: String) {
        self.inner.error(error, self.text_pos)
    }

    fn merge_spans(a: SpanData<Ctx>, b: SpanData<Ctx>) -> SpanData<Ctx> {
        // We don't do what rustc does exactly, rustc does something clever when the spans have different syntax contexts
        // but this runs afoul of our separation between `span` and `hir-expand`.
        SpanData {
            range: if a.ctx == b.ctx {
                TextRange::new(
                    std::cmp::min(a.range.start(), b.range.start()),
                    std::cmp::max(a.range.end(), b.range.end()),
                )
            } else {
                // Combining ranges make no sense when they come from different syntax contexts.
                a.range
            },
            anchor: a.anchor,
            ctx: a.ctx,
        }
    }
}
