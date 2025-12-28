//! `tt` crate defines a `TokenTree` data structure: this is the interface (both
//! input and output) of macros.
//!
//! The `TokenTree` is semantically a tree, but for performance reasons it is stored as a flat structure.
//!
//! A token tree piece has spans, but it does not carry them alone (well, it does, but it won't in the future).
//! so you always work with the `Spanned` variants: [`SpannedLeaf`], [`SpannedSubtree`], [`SpannedTokenTree`]
//! and [`SpannedLeafKind`]. They reconstruct the span from its pieces.

#![cfg_attr(feature = "in-rust-tree", feature(rustc_private))]

#[cfg(feature = "in-rust-tree")]
extern crate rustc_driver as _;

#[cfg(not(feature = "in-rust-tree"))]
extern crate ra_ap_rustc_lexer as rustc_lexer;
#[cfg(feature = "in-rust-tree")]
extern crate rustc_lexer;

pub mod buffer;
pub mod iter;

use std::{fmt, ops::Deref};

use buffer::Cursor;
use intern::Symbol;
use stdx::{impl_from, itertools::Itertools as _};

pub use span::Span;
pub use text_size::{TextRange, TextSize};

pub use self::iter::{TtElement, TtIter};

pub const MAX_GLUED_PUNCT_LEN: usize = 3;

#[derive(Clone, PartialEq, Debug)]
pub struct Lit {
    pub kind: LitKind,
    pub symbol: Symbol,
    pub suffix: Option<Symbol>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IdentIsRaw {
    No,
    Yes,
}
impl IdentIsRaw {
    pub fn yes(self) -> bool {
        matches!(self, IdentIsRaw::Yes)
    }
    pub fn no(&self) -> bool {
        matches!(self, IdentIsRaw::No)
    }
    pub fn as_str(self) -> &'static str {
        match self {
            IdentIsRaw::No => "",
            IdentIsRaw::Yes => "r#",
        }
    }
    pub fn split_from_symbol(sym: &str) -> (Self, &str) {
        if let Some(sym) = sym.strip_prefix("r#") {
            (IdentIsRaw::Yes, sym)
        } else {
            (IdentIsRaw::No, sym)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum LitKind {
    Byte,
    Char,
    Integer, // e.g. `1`, `1u8`, `1f32`
    Float,   // e.g. `1.`, `1.0`, `1e3f32`
    Str,
    StrRaw(u8), // raw string delimited by `n` hash symbols
    ByteStr,
    ByteStrRaw(u8), // raw byte string delimited by `n` hash symbols
    CStr,
    CStrRaw(u8),
    Err(()),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenTree {
    Leaf(Leaf),
    Subtree(Subtree),
}
impl_from!(Leaf, Subtree for TokenTree);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Leaf {
    Literal(Literal),
    Punct(Punct),
    Ident(Ident),
}

impl_from!(Literal, Punct, Ident for Leaf);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Subtree {
    pub delimiter: Delimiter,
    /// Number of following token trees that belong to this subtree, excluding this subtree.
    pub len: u32,
}

impl Subtree {
    #[inline]
    pub fn usize_len(&self) -> usize {
        self.len as usize
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TopSubtree(Box<[TokenTree]>);

impl TopSubtree {
    pub fn empty(span: DelimSpan) -> Self {
        Self(Box::new([TokenTree::Subtree(Subtree {
            delimiter: Delimiter {
                open: span.open,
                close: span.close,
                kind: DelimiterKind::Invisible,
            },
            len: 0,
        })]))
    }

    pub fn invisible_from_leaves<const N: usize>(
        delim_span: Span,
        leaves: [SpannedLeaf<impl Into<Leaf>>; N],
    ) -> Self {
        let mut builder =
            TopSubtreeBuilder::new(DelimiterKind::Invisible, DelimSpan::from_single(delim_span));
        builder.extend(leaves);
        builder.build()
    }

    pub fn from_token_trees(
        delimiter_kind: DelimiterKind,
        delimiter_span: DelimSpan,
        token_trees: TokenTreesView<'_>,
    ) -> Self {
        let mut builder = TopSubtreeBuilder::new(delimiter_kind, delimiter_span);
        builder.extend_with_tt(token_trees);
        builder.build()
    }

    pub fn from_subtree(subtree: SubtreeView<'_>) -> Self {
        Self(subtree.0.into())
    }

    pub fn from_serialized(tt: impl IntoIterator<Item = SerializedTokenTree>) -> Self {
        Self(
            tt.into_iter()
                .map(|tt| match tt {
                    SerializedTokenTree::Subtree { delimiter, len } => {
                        TokenTree::Subtree(Subtree {
                            delimiter: Delimiter {
                                open: delimiter.open,
                                close: delimiter.close,
                                kind: delimiter.kind,
                            },
                            len,
                        })
                    }
                    SerializedTokenTree::Leaf(leaf) => TokenTree::Leaf(leaf.leaf),
                })
                .collect(),
        )
    }

    pub fn view(&self) -> SubtreeView<'_> {
        SubtreeView::new(&self.0)
    }

    pub fn iter(&self) -> TtIter<'_> {
        self.view().iter()
    }

    pub fn top_subtree(&self) -> SpannedSubtree<'_> {
        self.view().top_subtree()
    }

    pub fn set_top_subtree_delimiter(&mut self, kind: DelimiterKind, span: DelimSpan) {
        self.top_subtree_mut().delimiter = Delimiter { open: span.open, close: span.close, kind };
    }

    pub fn set_top_subtree_delimiter_kind(&mut self, kind: DelimiterKind) {
        self.top_subtree_mut().delimiter.kind = kind;
    }

    fn top_subtree_mut(&mut self) -> &mut Subtree {
        let TokenTree::Subtree(subtree) = &mut self.0[0] else {
            unreachable!("the first token tree is always the top subtree");
        };
        subtree
    }

    pub fn flat_tokens_mut(&mut self) -> &mut [TokenTree] {
        &mut self.0
    }

    pub fn token_trees(&self) -> TokenTreesView<'_> {
        self.view().token_trees()
    }

    pub fn as_token_trees(&self) -> TokenTreesView<'_> {
        self.view().as_token_trees()
    }

    pub fn change_every_ast_id(&mut self, mut callback: impl FnMut(&mut span::ErasedFileAstId)) {
        for tt in &mut self.0 {
            match tt {
                TokenTree::Leaf(Leaf::Ident(Ident { span, .. }))
                | TokenTree::Leaf(Leaf::Literal(Literal { span, .. }))
                | TokenTree::Leaf(Leaf::Punct(Punct { span, .. })) => {
                    callback(&mut span.anchor.ast_id);
                }
                TokenTree::Subtree(subtree) => {
                    callback(&mut subtree.delimiter.open.anchor.ast_id);
                    callback(&mut subtree.delimiter.close.anchor.ast_id);
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter_mut_dangerous(&mut self) -> impl Iterator<Item = &mut TokenTree> {
        self.0.iter_mut()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopSubtreeBuilder {
    unclosed_subtree_indices: Vec<usize>,
    token_trees: Vec<TokenTree>,
    last_closed_subtree: Option<usize>,
}

impl TopSubtreeBuilder {
    pub fn new(top_delimiter_kind: DelimiterKind, top_delimiter_span: DelimSpan) -> Self {
        let mut result = Self {
            unclosed_subtree_indices: Vec::new(),
            token_trees: Vec::new(),
            last_closed_subtree: None,
        };
        let top_subtree = TokenTree::Subtree(Subtree {
            delimiter: Delimiter {
                open: top_delimiter_span.open,
                close: top_delimiter_span.close,
                kind: top_delimiter_kind,
            },
            len: 0,
        });
        result.token_trees.push(top_subtree);
        result
    }

    pub fn open(&mut self, delimiter_kind: DelimiterKind, open_span: Span) {
        self.unclosed_subtree_indices.push(self.token_trees.len());
        self.token_trees.push(TokenTree::Subtree(Subtree {
            delimiter: Delimiter {
                open: open_span,
                close: open_span, // Will be overwritten on close.
                kind: delimiter_kind,
            },
            len: 0,
        }));
    }

    pub fn close(&mut self, close_span: Span) {
        let last_unclosed_index = self
            .unclosed_subtree_indices
            .pop()
            .expect("attempt to close a `tt::Subtree` when none is open");
        let subtree_len = (self.token_trees.len() - last_unclosed_index - 1) as u32;
        let TokenTree::Subtree(subtree) = &mut self.token_trees[last_unclosed_index] else {
            unreachable!("unclosed token tree is always a subtree");
        };
        subtree.len = subtree_len;
        subtree.delimiter.close = close_span;
        self.last_closed_subtree = Some(last_unclosed_index);
    }

    /// You cannot call this consecutively, it will only work once after close.
    pub fn remove_last_subtree_if_invisible(&mut self) {
        let Some(last_subtree_idx) = self.last_closed_subtree else { return };
        if let TokenTree::Subtree(Subtree {
            delimiter: Delimiter { kind: DelimiterKind::Invisible, .. },
            ..
        }) = self.token_trees[last_subtree_idx]
        {
            self.token_trees.remove(last_subtree_idx);
            self.last_closed_subtree = None;
        }
    }

    pub fn push(&mut self, leaf: SpannedLeaf<impl Into<Leaf>>) {
        self.token_trees.push(TokenTree::Leaf(leaf.leaf.into()));
    }

    pub fn extend(&mut self, leaves: impl IntoIterator<Item = SpannedLeaf<impl Into<Leaf>>>) {
        self.token_trees.extend(leaves.into_iter().map(|leaf| TokenTree::Leaf(leaf.leaf.into())));
    }

    pub fn extend_with_tt(&mut self, tt: TokenTreesView<'_>) {
        self.token_trees.extend(tt.0.iter().cloned());
    }

    /// Like [`Self::extend_with_tt()`], but makes sure the new tokens will never be
    /// joint with whatever comes after them.
    pub fn extend_with_tt_alone(&mut self, tt: TokenTreesView<'_>) {
        if let Some((last, before_last)) = tt.0.split_last() {
            self.token_trees.reserve(tt.0.len());
            self.token_trees.extend(before_last.iter().cloned());
            let last = if let TokenTree::Leaf(Leaf::Punct(last)) = last {
                let mut last = *last;
                last.spacing = Spacing::Alone;
                TokenTree::Leaf(Leaf::Punct(last))
            } else {
                last.clone()
            };
            self.token_trees.push(last);
        }
    }

    pub fn expected_delimiters(&self) -> impl Iterator<Item = &Delimiter> {
        self.unclosed_subtree_indices.iter().rev().map(|&subtree_idx| {
            let TokenTree::Subtree(subtree) = &self.token_trees[subtree_idx] else {
                unreachable!("unclosed token tree is always a subtree")
            };
            &subtree.delimiter
        })
    }

    /// Builds, and remove the top subtree if it has only one subtree child.
    pub fn build_skip_top_subtree(mut self) -> TopSubtree {
        let top_tts = TokenTreesView::new(&self.token_trees[1..]);
        match top_tts.try_into_subtree() {
            Some(_) => {
                assert!(
                    self.unclosed_subtree_indices.is_empty(),
                    "attempt to build an unbalanced `TopSubtreeBuilder`"
                );
                TopSubtree(self.token_trees.drain(1..).collect())
            }
            None => self.build(),
        }
    }

    pub fn build(mut self) -> TopSubtree {
        assert!(
            self.unclosed_subtree_indices.is_empty(),
            "attempt to build an unbalanced `TopSubtreeBuilder`"
        );
        let total_len = self.token_trees.len() as u32;
        let TokenTree::Subtree(top_subtree) = &mut self.token_trees[0] else {
            unreachable!("first token tree is always a subtree");
        };
        top_subtree.len = total_len - 1;
        TopSubtree(self.token_trees.into_boxed_slice())
    }

    pub fn restore_point(&self) -> SubtreeBuilderRestorePoint {
        SubtreeBuilderRestorePoint {
            unclosed_subtree_indices_len: self.unclosed_subtree_indices.len(),
            token_trees_len: self.token_trees.len(),
            last_closed_subtree: self.last_closed_subtree,
        }
    }

    pub fn restore(&mut self, restore_point: SubtreeBuilderRestorePoint) {
        self.unclosed_subtree_indices.truncate(restore_point.unclosed_subtree_indices_len);
        self.token_trees.truncate(restore_point.token_trees_len);
        self.last_closed_subtree = restore_point.last_closed_subtree;
    }
}

#[derive(Clone, Copy)]
pub struct SubtreeBuilderRestorePoint {
    unclosed_subtree_indices_len: usize,
    token_trees_len: usize,
    last_closed_subtree: Option<usize>,
}

#[derive(Clone, Copy)]
pub struct TokenTreesView<'a>(&'a [TokenTree]);

impl<'a> TokenTreesView<'a> {
    #[inline]
    pub fn empty() -> Self {
        Self(&[])
    }

    #[inline]
    fn new(tts: &'a [TokenTree]) -> Self {
        if cfg!(debug_assertions) {
            tts.iter().enumerate().for_each(|(idx, tt)| {
                if let TokenTree::Subtree(tt) = &tt {
                    // `<` and not `<=` because `Subtree.len` does not include the subtree node itself.
                    debug_assert!(
                        idx + tt.usize_len() < tts.len(),
                        "`TokenTreeView::new()` was given a cut-in-half list"
                    );
                }
            });
        }
        Self(tts)
    }

    pub fn iter(&self) -> TtIter<'a> {
        TtIter::new(self.0)
    }

    pub fn cursor(&self) -> Cursor<'a> {
        Cursor::new(*self)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn try_into_subtree(self) -> Option<SubtreeView<'a>> {
        if let Some(TokenTree::Subtree(subtree)) = self.0.first()
            && subtree.usize_len() == (self.0.len() - 1)
        {
            return Some(SubtreeView::new(self.0));
        }
        None
    }

    pub fn strip_invisible(self) -> TokenTreesView<'a> {
        self.try_into_subtree().map(|subtree| subtree.strip_invisible()).unwrap_or(self)
    }

    /// This returns a **flat** structure of tokens (subtrees will be represented by a single node
    /// preceding their children), so it isn't suited for most use cases, only for matching leaves
    /// at the beginning/end with no subtrees before them. If you need a structured pass, use [`TtIter`].
    pub fn flat_tokens(&self) -> &'a [TokenTree] {
        self.0
    }

    pub fn split(
        self,
        mut split_fn: impl FnMut(TtElement<'a>) -> bool,
    ) -> impl Iterator<Item = TokenTreesView<'a>> {
        let mut subtree_iter = self.iter();
        let mut need_to_yield_even_if_empty = true;

        std::iter::from_fn(move || {
            if subtree_iter.is_empty() && !need_to_yield_even_if_empty {
                return None;
            };

            need_to_yield_even_if_empty = false;
            let savepoint = subtree_iter.savepoint();
            let mut result = subtree_iter.from_savepoint(savepoint);
            while let Some(tt) = subtree_iter.next() {
                if split_fn(tt) {
                    need_to_yield_even_if_empty = true;
                    break;
                }
                result = subtree_iter.from_savepoint(savepoint);
            }
            Some(result)
        })
    }

    #[inline]
    pub fn first_span(&self) -> Option<Span> {
        Some(match self.0.first()? {
            TokenTree::Leaf(leaf) => leaf.span(),
            TokenTree::Subtree(subtree) => subtree.delimiter.open,
        })
    }

    #[inline]
    pub fn last_span(&self) -> Option<Span> {
        Some(match self.0.last()? {
            TokenTree::Leaf(leaf) => leaf.span(),
            TokenTree::Subtree(subtree) => subtree.delimiter.close,
        })
    }
}

impl fmt::Debug for TokenTreesView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.iter();
        while let Some(tt) = iter.next() {
            print_debug_token(f, 0, tt)?;
            if !iter.is_empty() {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for TokenTreesView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return token_trees_display(f, self.iter());

        fn subtree_display(
            subtree: &Subtree,
            f: &mut fmt::Formatter<'_>,
            iter: TtIter<'_>,
        ) -> fmt::Result {
            let (l, r) = match subtree.delimiter.kind {
                DelimiterKind::Parenthesis => ("(", ")"),
                DelimiterKind::Brace => ("{", "}"),
                DelimiterKind::Bracket => ("[", "]"),
                DelimiterKind::Invisible => ("", ""),
            };
            f.write_str(l)?;
            token_trees_display(f, iter)?;
            f.write_str(r)?;
            Ok(())
        }

        fn token_trees_display(f: &mut fmt::Formatter<'_>, iter: TtIter<'_>) -> fmt::Result {
            let mut needs_space = false;
            for child in iter {
                if needs_space {
                    f.write_str(" ")?;
                }
                needs_space = true;

                match child {
                    TtElement::Leaf(SpannedLeafKind::Punct(p)) => {
                        needs_space = p.spacing == Spacing::Alone;
                        fmt::Display::fmt(*p, f)?;
                    }
                    TtElement::Leaf(SpannedLeafKind::Literal(leaf)) => fmt::Display::fmt(*leaf, f)?,
                    TtElement::Leaf(SpannedLeafKind::Ident(leaf)) => fmt::Display::fmt(*leaf, f)?,
                    TtElement::Subtree(subtree, subtree_iter) => {
                        subtree_display(&subtree, f, subtree_iter)?
                    }
                }
            }
            Ok(())
        }
    }
}

#[derive(Clone, Copy)]
// Invariant: always starts with `Subtree` that covers the entire thing.
pub struct SubtreeView<'a>(&'a [TokenTree]);

impl<'a> SubtreeView<'a> {
    fn new(tts: &'a [TokenTree]) -> Self {
        if cfg!(debug_assertions) {
            let TokenTree::Subtree(subtree) = &tts[0] else {
                panic!("first token tree must be a subtree in `SubtreeView`");
            };
            assert_eq!(
                subtree.usize_len(),
                tts.len() - 1,
                "subtree must cover the entire `SubtreeView`"
            );
        }
        Self(tts)
    }

    pub fn as_token_trees(self) -> TokenTreesView<'a> {
        TokenTreesView::new(self.0)
    }

    pub fn iter(&self) -> TtIter<'a> {
        TtIter::new(&self.0[1..])
    }

    pub fn top_subtree(&self) -> SpannedSubtree<'a> {
        let TokenTree::Subtree(subtree) = &self.0[0] else {
            unreachable!("the first token tree is always the top subtree");
        };
        SpannedSubtree { _open_span: (), _close_span: (), subtree }
    }

    pub fn strip_invisible(&self) -> TokenTreesView<'a> {
        if self.top_subtree().delimiter.kind == DelimiterKind::Invisible {
            TokenTreesView::new(&self.0[1..])
        } else {
            TokenTreesView::new(self.0)
        }
    }

    pub fn token_trees(&self) -> TokenTreesView<'a> {
        TokenTreesView::new(&self.0[1..])
    }
}

impl fmt::Debug for SubtreeView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&TokenTreesView(self.0), f)
    }
}

impl fmt::Display for SubtreeView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&TokenTreesView(self.0), f)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DelimSpan {
    pub open: Span,
    pub close: Span,
}

impl DelimSpan {
    pub fn from_single(sp: Span) -> Self {
        DelimSpan { open: sp, close: sp }
    }

    pub fn from_pair(open: Span, close: Span) -> Self {
        DelimSpan { open, close }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Delimiter {
    open: Span,
    close: Span,
    pub kind: DelimiterKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpannedDelimiter {
    pub open: Span,
    pub close: Span,
    pub kind: DelimiterKind,
}

impl SpannedDelimiter {
    pub const fn invisible_spanned(span: Span) -> Self {
        SpannedDelimiter { open: span, close: span, kind: DelimiterKind::Invisible }
    }

    pub const fn invisible_delim_spanned(span: DelimSpan) -> Self {
        SpannedDelimiter { open: span.open, close: span.close, kind: DelimiterKind::Invisible }
    }

    pub fn delim_span(&self) -> DelimSpan {
        DelimSpan { open: self.open, close: self.close }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DelimiterKind {
    Parenthesis,
    Brace,
    Bracket,
    Invisible,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Literal {
    // escaped
    pub symbol: Symbol,
    span: Span,
    pub kind: LitKind,
    pub suffix: Option<Symbol>,
}

impl Literal {
    #[inline]
    pub fn new(
        symbol: Symbol,
        span: Span,
        kind: LitKind,
        suffix: Option<Symbol>,
    ) -> SpannedLeaf<Self> {
        SpannedLeaf { _span: (), leaf: Self { symbol, span, kind, suffix } }
    }
}

pub fn token_to_literal(text: &str, span: Span) -> SpannedLeaf<Literal> {
    use rustc_lexer::LiteralKind;

    let token = rustc_lexer::tokenize(text, rustc_lexer::FrontmatterAllowed::No).next_tuple();
    let Some((rustc_lexer::Token {
        kind: rustc_lexer::TokenKind::Literal { kind, suffix_start },
        ..
    },)) = token
    else {
        return Literal::new(Symbol::intern(text), span, LitKind::Err(()), None);
    };

    let (kind, start_offset, end_offset) = match kind {
        LiteralKind::Int { .. } => (LitKind::Integer, 0, 0),
        LiteralKind::Float { .. } => (LitKind::Float, 0, 0),
        LiteralKind::Char { terminated } => (LitKind::Char, 1, terminated as usize),
        LiteralKind::Byte { terminated } => (LitKind::Byte, 2, terminated as usize),
        LiteralKind::Str { terminated } => (LitKind::Str, 1, terminated as usize),
        LiteralKind::ByteStr { terminated } => (LitKind::ByteStr, 2, terminated as usize),
        LiteralKind::CStr { terminated } => (LitKind::CStr, 2, terminated as usize),
        LiteralKind::RawStr { n_hashes } => (
            LitKind::StrRaw(n_hashes.unwrap_or_default()),
            2 + n_hashes.unwrap_or_default() as usize,
            1 + n_hashes.unwrap_or_default() as usize,
        ),
        LiteralKind::RawByteStr { n_hashes } => (
            LitKind::ByteStrRaw(n_hashes.unwrap_or_default()),
            3 + n_hashes.unwrap_or_default() as usize,
            1 + n_hashes.unwrap_or_default() as usize,
        ),
        LiteralKind::RawCStr { n_hashes } => (
            LitKind::CStrRaw(n_hashes.unwrap_or_default()),
            3 + n_hashes.unwrap_or_default() as usize,
            1 + n_hashes.unwrap_or_default() as usize,
        ),
    };

    let (lit, suffix) = text.split_at(suffix_start as usize);
    let lit = &lit[start_offset..lit.len() - end_offset];
    let suffix = match suffix {
        "" | "_" => None,
        // ill-suffixed literals
        _ if !matches!(kind, LitKind::Integer | LitKind::Float | LitKind::Err(_)) => {
            return Literal::new(Symbol::intern(text), span, LitKind::Err(()), None);
        }
        suffix => Some(Symbol::intern(suffix)),
    };

    Literal::new(Symbol::intern(lit), span, kind, suffix)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Punct {
    pub char: char,
    pub spacing: Spacing,
    span: Span,
}

impl Punct {
    #[inline]
    pub fn new(char: char, spacing: Spacing, span: Span) -> SpannedLeaf<Self> {
        SpannedLeaf { _span: (), leaf: Self { char, spacing, span } }
    }
}

/// Indicates whether a token can join with the following token to form a
/// compound token. Used for conversions to `proc_macro::Spacing`. Also used to
/// guide pretty-printing, which is where the `JointHidden` value (which isn't
/// part of `proc_macro::Spacing`) comes in useful.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Spacing {
    /// The token cannot join with the following token to form a compound
    /// token.
    ///
    /// In token streams parsed from source code, the compiler will use `Alone`
    /// for any token immediately followed by whitespace, a non-doc comment, or
    /// EOF.
    ///
    /// When constructing token streams within the compiler, use this for each
    /// token that (a) should be pretty-printed with a space after it, or (b)
    /// is the last token in the stream. (In the latter case the choice of
    /// spacing doesn't matter because it is never used for the last token. We
    /// arbitrarily use `Alone`.)
    ///
    /// Converts to `proc_macro::Spacing::Alone`, and
    /// `proc_macro::Spacing::Alone` converts back to this.
    Alone,

    /// The token can join with the following token to form a compound token.
    ///
    /// In token streams parsed from source code, the compiler will use `Joint`
    /// for any token immediately followed by punctuation (as determined by
    /// `Token::is_punct`).
    ///
    /// When constructing token streams within the compiler, use this for each
    /// token that (a) should be pretty-printed without a space after it, and
    /// (b) is followed by a punctuation token.
    ///
    /// Converts to `proc_macro::Spacing::Joint`, and
    /// `proc_macro::Spacing::Joint` converts back to this.
    Joint,

    /// The token can join with the following token to form a compound token,
    /// but this will not be visible at the proc macro level. (This is what the
    /// `Hidden` means; see below.)
    ///
    /// In token streams parsed from source code, the compiler will use
    /// `JointHidden` for any token immediately followed by anything not
    /// covered by the `Alone` and `Joint` cases: an identifier, lifetime,
    /// literal, delimiter, doc comment.
    ///
    /// When constructing token streams, use this for each token that (a)
    /// should be pretty-printed without a space after it, and (b) is followed
    /// by a non-punctuation token.
    ///
    /// Converts to `proc_macro::Spacing::Alone`, but
    /// `proc_macro::Spacing::Alone` converts back to `token::Spacing::Alone`.
    /// Because of that, pretty-printing of `TokenStream`s produced by proc
    /// macros is unavoidably uglier (with more whitespace between tokens) than
    /// pretty-printing of `TokenStream`'s produced by other means (i.e. parsed
    /// source code, internally constructed token streams, and token streams
    /// produced by declarative macros).
    JointHidden,
}

/// Identifier or keyword.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub sym: Symbol,
    span: Span,
    pub is_raw: IdentIsRaw,
}

impl Ident {
    pub fn new(text: &str, span: Span) -> SpannedLeaf<Self> {
        // let raw_stripped = IdentIsRaw::split_from_symbol(text.as_ref());
        let (is_raw, text) = IdentIsRaw::split_from_symbol(text);
        Ident::new_sym(Symbol::intern(text), is_raw, span)
    }

    #[inline]
    pub fn new_sym(sym: Symbol, is_raw: IdentIsRaw, span: Span) -> SpannedLeaf<Self> {
        SpannedLeaf { _span: (), leaf: Self { sym, span, is_raw } }
    }
}

fn print_debug_subtree(
    f: &mut fmt::Formatter<'_>,
    subtree: &Subtree,
    level: usize,
    iter: TtIter<'_>,
) -> fmt::Result {
    let align = "  ".repeat(level);

    let Delimiter { kind, open, close } = &subtree.delimiter;
    let delim = match kind {
        DelimiterKind::Invisible => "$$",
        DelimiterKind::Parenthesis => "()",
        DelimiterKind::Brace => "{}",
        DelimiterKind::Bracket => "[]",
    };

    write!(f, "{align}SUBTREE {delim} ",)?;
    write!(f, "{open:#?}")?;
    write!(f, " ")?;
    write!(f, "{close:#?}")?;
    for child in iter {
        writeln!(f)?;
        print_debug_token(f, level + 1, child)?;
    }

    Ok(())
}

fn print_debug_token(f: &mut fmt::Formatter<'_>, level: usize, tt: TtElement<'_>) -> fmt::Result {
    let align = "  ".repeat(level);

    match tt {
        TtElement::Leaf(leaf) => match leaf {
            SpannedLeafKind::Literal(lit) => {
                write!(
                    f,
                    "{}LITERAL {:?} {}{} {:#?}",
                    align,
                    lit.kind,
                    lit.symbol,
                    lit.suffix.as_ref().map(|it| it.as_str()).unwrap_or(""),
                    lit.span
                )?;
            }
            SpannedLeafKind::Punct(punct) => {
                write!(
                    f,
                    "{}PUNCH   {} [{}] {:#?}",
                    align,
                    punct.char,
                    if punct.spacing == Spacing::Alone { "alone" } else { "joint" },
                    punct.span
                )?;
            }
            SpannedLeafKind::Ident(ident) => {
                write!(
                    f,
                    "{}IDENT   {}{} {:#?}",
                    align,
                    ident.is_raw.as_str(),
                    ident.sym,
                    ident.span
                )?;
            }
        },
        TtElement::Subtree(subtree, subtree_iter) => {
            print_debug_subtree(f, &subtree, level, subtree_iter)?;
        }
    }

    Ok(())
}

impl fmt::Debug for TopSubtree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.view(), f)
    }
}

impl fmt::Display for TopSubtree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.view(), f)
    }
}

impl fmt::Display for Leaf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Leaf::Ident(it) => fmt::Display::fmt(it, f),
            Leaf::Literal(it) => fmt::Display::fmt(it, f),
            Leaf::Punct(it) => fmt::Display::fmt(it, f),
        }
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.is_raw.as_str(), f)?;
        fmt::Display::fmt(&self.sym, f)
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            LitKind::Byte => write!(f, "b'{}'", self.symbol),
            LitKind::Char => write!(f, "'{}'", self.symbol),
            LitKind::Integer | LitKind::Float | LitKind::Err(_) => write!(f, "{}", self.symbol),
            LitKind::Str => write!(f, "\"{}\"", self.symbol),
            LitKind::ByteStr => write!(f, "b\"{}\"", self.symbol),
            LitKind::CStr => write!(f, "c\"{}\"", self.symbol),
            LitKind::StrRaw(num_of_hashes) => {
                let num_of_hashes = num_of_hashes as usize;
                write!(
                    f,
                    r#"r{0:#<num_of_hashes$}"{text}"{0:#<num_of_hashes$}"#,
                    "",
                    text = self.symbol
                )
            }
            LitKind::ByteStrRaw(num_of_hashes) => {
                let num_of_hashes = num_of_hashes as usize;
                write!(
                    f,
                    r#"br{0:#<num_of_hashes$}"{text}"{0:#<num_of_hashes$}"#,
                    "",
                    text = self.symbol
                )
            }
            LitKind::CStrRaw(num_of_hashes) => {
                let num_of_hashes = num_of_hashes as usize;
                write!(
                    f,
                    r#"cr{0:#<num_of_hashes$}"{text}"{0:#<num_of_hashes$}"#,
                    "",
                    text = self.symbol
                )
            }
        }?;
        if let Some(suffix) = &self.suffix {
            write!(f, "{suffix}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Punct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.char, f)
    }
}

impl Subtree {
    /// Count the number of tokens recursively
    pub fn count(&self) -> usize {
        self.usize_len()
    }
}

pub trait HasLeafSpan {
    #[doc(hidden)]
    fn span(&self) -> Span;
}

impl<T: HasLeafSpan> HasLeafSpan for &T {
    #[inline]
    fn span(&self) -> Span {
        T::span(*self)
    }
}

impl HasLeafSpan for Literal {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl HasLeafSpan for Punct {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl HasLeafSpan for Ident {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl HasLeafSpan for Leaf {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Leaf::Literal(it) => it.span,
            Leaf::Punct(it) => it.span,
            Leaf::Ident(it) => it.span,
        }
    }
}

/// A leaf (parameterized by the generic parameter), but with its full span.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[expect(clippy::manual_non_exhaustive, reason = "those are placeholders for compressed spans")]
pub struct SpannedLeaf<T> {
    _span: (),
    pub leaf: T,
}

impl<T: HasLeafSpan> SpannedLeaf<T> {
    #[inline]
    pub fn span(&self) -> Span {
        self.leaf.span()
    }

    #[inline]
    pub fn as_ref(&self) -> SpannedLeaf<&T> {
        SpannedLeaf { _span: self._span, leaf: &self.leaf }
    }

    #[inline]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> SpannedLeaf<U> {
        SpannedLeaf { _span: self._span, leaf: f(self.leaf) }
    }
}

impl<T: Clone> SpannedLeaf<&T> {
    #[inline]
    pub fn cloned(self) -> SpannedLeaf<T> {
        SpannedLeaf { _span: self._span, leaf: self.leaf.clone() }
    }
}

impl<T> Deref for SpannedLeaf<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.leaf
    }
}

pub trait HasOwnedLeafSpan {
    #[doc(hidden)]
    fn set_span(&mut self, span: Span);
}

impl HasOwnedLeafSpan for Literal {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}

impl HasOwnedLeafSpan for Punct {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}

impl HasOwnedLeafSpan for Ident {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}

impl HasOwnedLeafSpan for Leaf {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Leaf::Literal(it) => it.span = span,
            Leaf::Punct(it) => it.span = span,
            Leaf::Ident(it) => it.span = span,
        }
    }
}

impl<T: HasOwnedLeafSpan> SpannedLeaf<T> {
    #[inline]
    pub fn set_span(&mut self, span: Span) {
        self.leaf.set_span(span);
    }
}

impl<'a> SpannedLeaf<&'a Leaf> {
    #[inline]
    pub fn kind(&self) -> SpannedLeafKind<'a> {
        match **self {
            Leaf::Literal(leaf) => SpannedLeafKind::Literal(SpannedLeaf { _span: (), leaf }),
            Leaf::Punct(leaf) => SpannedLeafKind::Punct(SpannedLeaf { _span: (), leaf }),
            Leaf::Ident(leaf) => SpannedLeafKind::Ident(SpannedLeaf { _span: (), leaf }),
        }
    }
}

impl From<SpannedLeaf<Literal>> for SpannedLeaf<Leaf> {
    #[inline]
    fn from(value: SpannedLeaf<Literal>) -> Self {
        value.map(Leaf::Literal)
    }
}

impl From<SpannedLeaf<Punct>> for SpannedLeaf<Leaf> {
    #[inline]
    fn from(value: SpannedLeaf<Punct>) -> Self {
        value.map(Leaf::Punct)
    }
}

impl From<SpannedLeaf<Ident>> for SpannedLeaf<Leaf> {
    #[inline]
    fn from(value: SpannedLeaf<Ident>) -> Self {
        value.map(Leaf::Ident)
    }
}

impl<T: fmt::Display> fmt::Display for SpannedLeaf<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.leaf, f)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SpannedLeafKind<'a> {
    Literal(SpannedLeaf<&'a Literal>),
    Punct(SpannedLeaf<&'a Punct>),
    Ident(SpannedLeaf<&'a Ident>),
}

impl SpannedLeafKind<'_> {
    #[inline]
    pub fn span(&self) -> Span {
        match self {
            SpannedLeafKind::Literal(it) => it.span(),
            SpannedLeafKind::Punct(it) => it.span(),
            SpannedLeafKind::Ident(it) => it.span(),
        }
    }
}

/// A leaf (parameterized by the generic parameter), but with its full span.
#[derive(Debug, Clone, Copy)]
pub struct SpannedSubtree<'a> {
    _open_span: (),
    _close_span: (),
    subtree: &'a Subtree,
}

impl SpannedSubtree<'_> {
    #[inline]
    pub fn open_span(&self) -> Span {
        self.subtree.delimiter.open
    }

    #[inline]
    pub fn close_span(&self) -> Span {
        self.subtree.delimiter.close
    }

    #[inline]
    pub fn delim_span(&self) -> DelimSpan {
        DelimSpan { open: self.open_span(), close: self.close_span() }
    }

    #[inline]
    pub fn delimiter(&self) -> SpannedDelimiter {
        SpannedDelimiter {
            open: self.open_span(),
            close: self.close_span(),
            kind: self.delimiter.kind,
        }
    }
}

impl Deref for SpannedSubtree<'_> {
    type Target = Subtree;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.subtree
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SpannedTokenTree<'a> {
    Subtree(SpannedSubtree<'a>),
    Leaf(SpannedLeafKind<'a>),
}

impl SpannedTokenTree<'_> {
    #[inline]
    pub fn first_span(&self) -> Span {
        match self {
            SpannedTokenTree::Leaf(l) => l.span(),
            SpannedTokenTree::Subtree(s) => s.open_span(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SerializedTokenTree {
    Subtree { delimiter: SpannedDelimiter, len: u32 },
    Leaf(SpannedLeaf<Leaf>),
}

impl TopSubtree {
    /// A simple line string used for debugging
    pub fn subtree_as_debug_string(&self, subtree_idx: usize) -> String {
        fn debug_subtree(
            output: &mut String,
            subtree: &Subtree,
            iter: &mut std::slice::Iter<'_, TokenTree>,
        ) {
            let delim = match subtree.delimiter.kind {
                DelimiterKind::Brace => ("{", "}"),
                DelimiterKind::Bracket => ("[", "]"),
                DelimiterKind::Parenthesis => ("(", ")"),
                DelimiterKind::Invisible => ("$", "$"),
            };

            output.push_str(delim.0);
            let mut last = None;
            let mut idx = 0;
            while idx < subtree.len {
                let child = iter.next().unwrap();
                debug_token_tree(output, child, last, iter);
                last = Some(child);
                idx += 1;
            }

            output.push_str(delim.1);
        }

        fn debug_token_tree(
            output: &mut String,
            tt: &TokenTree,
            last: Option<&TokenTree>,
            iter: &mut std::slice::Iter<'_, TokenTree>,
        ) {
            match tt {
                TokenTree::Leaf(it) => {
                    let s = match it {
                        Leaf::Literal(it) => it.symbol.to_string(),
                        Leaf::Punct(it) => it.char.to_string(),
                        Leaf::Ident(it) => format!("{}{}", it.is_raw.as_str(), it.sym),
                    };
                    match (it, last) {
                        (Leaf::Ident(_), Some(&TokenTree::Leaf(Leaf::Ident(_)))) => {
                            output.push(' ');
                            output.push_str(&s);
                        }
                        (Leaf::Punct(_), Some(TokenTree::Leaf(Leaf::Punct(punct)))) => {
                            if punct.spacing == Spacing::Alone {
                                output.push(' ');
                                output.push_str(&s);
                            } else {
                                output.push_str(&s);
                            }
                        }
                        _ => output.push_str(&s),
                    }
                }
                TokenTree::Subtree(it) => debug_subtree(output, it, iter),
            }
        }

        let mut res = String::new();
        debug_token_tree(
            &mut res,
            &self.0[subtree_idx],
            None,
            &mut self.0[subtree_idx + 1..].iter(),
        );
        res
    }
}

pub fn pretty(mut tkns: &[TokenTree]) -> String {
    fn tokentree_to_text(tkn: &TokenTree, tkns: &mut &[TokenTree]) -> String {
        match tkn {
            TokenTree::Leaf(Leaf::Ident(ident)) => {
                format!("{}{}", ident.is_raw.as_str(), ident.sym)
            }
            TokenTree::Leaf(Leaf::Literal(literal)) => format!("{literal}"),
            TokenTree::Leaf(Leaf::Punct(punct)) => format!("{}", punct.char),
            TokenTree::Subtree(subtree) => {
                let (subtree_content, rest) = tkns.split_at(subtree.usize_len());
                let content = pretty(subtree_content);
                *tkns = rest;
                let (open, close) = match subtree.delimiter.kind {
                    DelimiterKind::Brace => ("{", "}"),
                    DelimiterKind::Bracket => ("[", "]"),
                    DelimiterKind::Parenthesis => ("(", ")"),
                    DelimiterKind::Invisible => ("", ""),
                };
                format!("{open}{content}{close}")
            }
        }
    }

    let mut last = String::new();
    let mut last_to_joint = true;

    while let Some((tkn, rest)) = tkns.split_first() {
        tkns = rest;
        last = [last, tokentree_to_text(tkn, &mut tkns)].join(if last_to_joint { "" } else { " " });
        last_to_joint = false;
        if let TokenTree::Leaf(Leaf::Punct(punct)) = tkn
            && punct.spacing == Spacing::Joint
        {
            last_to_joint = true;
        }
    }
    last
}

#[derive(Debug)]
pub enum TransformTtAction<'a> {
    Keep,
    ReplaceWith(TokenTreesView<'a>),
}

impl TransformTtAction<'_> {
    #[inline]
    pub fn remove() -> Self {
        Self::ReplaceWith(TokenTreesView::empty())
    }
}

/// This function takes a token tree, and calls `callback` with each token tree in it.
/// Then it does what the callback says: keeps the tt or replaces it with a (possibly empty)
/// tts view.
pub fn transform_tt<'b>(
    tt: &mut TopSubtree,
    mut callback: impl FnMut(SpannedTokenTree<'_>) -> TransformTtAction<'b>,
) {
    let mut tt_vec = std::mem::take(&mut tt.0).into_vec();

    // We need to keep a stack of the currently open subtrees, because we need to update
    // them if we change the number of items in them.
    let mut subtrees_stack = Vec::new();
    let mut i = 0;
    while i < tt_vec.len() {
        'pop_finished_subtrees: while let Some(&subtree_idx) = subtrees_stack.last() {
            let TokenTree::Subtree(subtree) = &tt_vec[subtree_idx] else {
                unreachable!("non-subtree on subtrees stack");
            };
            if i >= subtree_idx + 1 + subtree.usize_len() {
                subtrees_stack.pop();
            } else {
                break 'pop_finished_subtrees;
            }
        }

        let current = match &tt_vec[i] {
            TokenTree::Leaf(leaf) => SpannedTokenTree::Leaf(match leaf {
                Leaf::Literal(leaf) => SpannedLeafKind::Literal(SpannedLeaf { _span: (), leaf }),
                Leaf::Punct(leaf) => SpannedLeafKind::Punct(SpannedLeaf { _span: (), leaf }),
                Leaf::Ident(leaf) => SpannedLeafKind::Ident(SpannedLeaf { _span: (), leaf }),
            }),
            TokenTree::Subtree(subtree) => SpannedTokenTree::Subtree(SpannedSubtree {
                _open_span: (),
                _close_span: (),
                subtree,
            }),
        };
        let action = callback(current);
        match action {
            TransformTtAction::Keep => {
                // This cannot be shared with the replaced case, because then we may push the same subtree
                // twice, and will update it twice which will lead to errors.
                if let TokenTree::Subtree(_) = &tt_vec[i] {
                    subtrees_stack.push(i);
                }

                i += 1;
            }
            TransformTtAction::ReplaceWith(replacement) => {
                let old_len = 1 + match &tt_vec[i] {
                    TokenTree::Leaf(_) => 0,
                    TokenTree::Subtree(subtree) => subtree.usize_len(),
                };
                let len_diff = replacement.len() as i64 - old_len as i64;
                tt_vec.splice(i..i + old_len, replacement.flat_tokens().iter().cloned());
                // Skip the newly inserted replacement, we don't want to visit it.
                i += replacement.len();

                for &subtree_idx in &subtrees_stack {
                    let TokenTree::Subtree(subtree) = &mut tt_vec[subtree_idx] else {
                        unreachable!("non-subtree on subtrees stack");
                    };
                    subtree.len = (i64::from(subtree.len) + len_diff).try_into().unwrap();
                }
            }
        }
    }

    tt.0 = tt_vec.into_boxed_slice();
}
