//! Serialization-friendly representation of `tt::TopSubtree`.
//!
//! It is possible to serialize `TopSubtree` recursively, as a tree, but using
//! arbitrary-nested trees in JSON is problematic, as they can cause the JSON
//! parser to overflow the stack.
//!
//! Additionally, such implementation would be pretty verbose, and we do care
//! about performance here a bit.
//!
//! So what this module does is dumping a `tt::TopSubtree` into a bunch of flat
//! array of numbers.
//!
//! ```json
//!  {
//!    // Array of subtrees, each subtree is represented by 4 numbers:
//!    // id of delimiter, delimiter kind, index of first child in `token_tree`,
//!    // index of last child in `token_tree`
//!    "subtree":[4294967295,0,0,5,2,2,5,5],
//!    // 2 ints per literal: [token id, index into `text`]
//!    "literal":[4294967295,1],
//!    // 3 ints per punct: [token id, char, spacing]
//!    "punct":[4294967295,64,1],
//!    // 2 ints per ident: [token id, index into `text`]
//!    "ident":   [0,0,1,1],
//!    // children of all subtrees, concatenated. Each child is represented as `index << shift_indices_by | tag`
//!    // where tag denotes one of subtree, literal, punct or ident.
//!    "token_tree":[3,7,1,4],
//!    // Strings shared by idents and literals
//!    "text": ["struct","Foo"]
//!  }
//! ```
//!
//! We probably should replace most of the code here with bincode someday, but,
//! as we don't have bincode in Cargo.toml yet, let's stick with serde_json for
//! the time being.

#[cfg(feature = "in-proc-macro-srv")]
mod proc_macro_srv_side;
#[cfg(feature = "in-ra")]
mod ra_side;

use std::{borrow::Borrow, cell::Cell, collections::VecDeque, marker::PhantomData};

use intern::{Symbol, sym};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use span::{EditionedFileId, ErasedFileAstId, Span, SpanAnchor, SyntaxContext, TextRange};
use stdx::always;

use crate::{
    legacy_protocol::SpanId,
    version::{DOC_COMMENT_LEAF, ENCODE_CLOSE_SPAN_VERSION, EXTENDED_LEAF_DATA},
};

pub type SpanDataIndexMap =
    indexmap::IndexSet<Span, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

pub fn serialize_span_data_index_map(map: &SpanDataIndexMap) -> Vec<u32> {
    map.iter()
        .map(|span| {
            [
                span.anchor.file_id.as_u32(),
                span.anchor.ast_id.into_raw(),
                span.range.start().into(),
                span.range.end().into(),
                span.ctx.into_u32(),
            ]
        })
        .collect::<Vec<_>>()
        .into_flattened()
}

pub fn deserialize_span_data_index_map(map: &[u32]) -> SpanDataIndexMap {
    let (chunks, remainder) = map.as_chunks();
    assert!(remainder.is_empty());
    chunks
        .iter()
        .map(|&[file_id, ast_id, start, end, e]| {
            Span {
                anchor: SpanAnchor {
                    file_id: EditionedFileId::from_raw(file_id),
                    ast_id: ErasedFileAstId::from_raw(ast_id),
                },
                range: TextRange::new(start.into(), end.into()),
                // SAFETY: We only receive spans from the server. If someone mess up the communication UB can happen,
                // but that will be their problem.
                ctx: unsafe { SyntaxContext::from_u32(e) },
            }
        })
        .collect()
}

fn tag_bit_width(version: u32) -> u32 {
    if version >= DOC_COMMENT_LEAF { 3 } else { 2 }
}

/// [`FlatTree`] when `version < DOC_COMMENT_LEAF`, because `postcard` is non-self-describing and does not support `skip_serializing_if`.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct FlatTreePreDocCommentLeafParts {
    subtree: Vec<u32>,
    literal: Vec<u32>,
    punct: Vec<u32>,
    ident: Vec<u32>,
    token_tree: Vec<u32>,
    text: Vec<String>,
}

/// [`FlatTree`] when `version >= DOC_COMMENT_LEAF`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FlatTreePostDocCommentLeafParts {
    pre_doc_comment_leaf: FlatTreePreDocCommentLeafParts,
    doc_comments: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct FlatTree(pub FlatTreePostDocCommentLeafParts);

impl FlatTree {
    fn from_pre_doc_comments(value: FlatTreePreDocCommentLeafParts) -> Self {
        Self(FlatTreePostDocCommentLeafParts {
            pre_doc_comment_leaf: value,
            doc_comments: Vec::new(),
        })
    }

    fn from_post_doc_comments(value: FlatTreePostDocCommentLeafParts) -> Self {
        Self(value)
    }

    fn as_pre_doc_comments(&self) -> &FlatTreePreDocCommentLeafParts {
        let FlatTreePostDocCommentLeafParts { pre_doc_comment_leaf: _, doc_comments } = &self.0;
        always!(doc_comments.is_empty());
        &self.0.pre_doc_comment_leaf
    }

    fn as_post_doc_comments(&self) -> &FlatTreePostDocCommentLeafParts {
        &self.0
    }
}

thread_local! {
    static IN_FLIGHT_SERIALIZATION_VERSION: Cell<Option<u32>> = const { Cell::new(None) };
}

/// We need to see the version during serialization, because it impacts the shape of the `FlatTree`
/// and postcard is non-self-describing.
///
/// `serde` provides `DeserializeSeed` to pass data to deserializers, but not to serializers and there is no derive for
/// it (external crates have but we don't want to import them just for this). So we smuggle it in a thread local instead.
///
/// Note: while the proc macro server side always has the same version, the r-a side might handle multiple servers with
/// different versions.
pub fn with_serialization_version<T>(version: u32, f: impl FnOnce() -> T) -> T {
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            IN_FLIGHT_SERIALIZATION_VERSION.set(None);
        }
    }

    let _guard = Guard;

    std::assert_matches!(
        IN_FLIGHT_SERIALIZATION_VERSION.replace(Some(version)),
        None,
        "cannot set serialization version mid-[de]serialization",
    );

    f()
}

fn serialization_version() -> u32 {
    IN_FLIGHT_SERIALIZATION_VERSION
        .get()
        .expect("`FlatTree` serialization version must be set during [de]serialization")
}

impl Serialize for FlatTree {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match serialization_version() {
            (..DOC_COMMENT_LEAF) => self.as_pre_doc_comments().serialize(serializer),
            (DOC_COMMENT_LEAF..) => self.as_post_doc_comments().serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for FlatTree {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match serialization_version() {
            (..DOC_COMMENT_LEAF) => FlatTreePreDocCommentLeafParts::deserialize(deserializer)
                .map(Self::from_pre_doc_comments),
            (DOC_COMMENT_LEAF..) => FlatTreePostDocCommentLeafParts::deserialize(deserializer)
                .map(Self::from_post_doc_comments),
        }
    }
}

impl FlatTree {
    fn deserialize<'a, ST: SpanTransformer, W: WriterTrait<'a, ST::Span>>(
        top_subtree: W::Subtree,
        version: u32,
        span_data_table: &mut ST::Table,
    ) -> FlatTree {
        let mut w = Writer::<ST, W> {
            string_table: FxHashMap::default(),
            work: VecDeque::new(),
            span_data_table,
            tag_bit_width: tag_bit_width(version),

            subtree: Vec::new(),
            literal: Vec::new(),
            punct: Vec::new(),
            ident: Vec::new(),
            doc_comment: Vec::new(),
            token_tree: Vec::new(),
            text: Vec::new(),
            version,
        };
        w.write_subtree(top_subtree);

        FlatTree(FlatTreePostDocCommentLeafParts {
            pre_doc_comment_leaf: FlatTreePreDocCommentLeafParts {
                subtree: if version >= ENCODE_CLOSE_SPAN_VERSION {
                    write_vec(w.subtree, SubtreeRepr::write_with_close_span)
                } else {
                    write_vec(w.subtree, SubtreeRepr::write)
                },
                literal: if version >= EXTENDED_LEAF_DATA {
                    write_vec(w.literal, LiteralRepr::write_with_kind)
                } else {
                    write_vec(w.literal, LiteralRepr::write)
                },
                punct: write_vec(w.punct, PunctRepr::write),
                ident: if version >= EXTENDED_LEAF_DATA {
                    write_vec(w.ident, IdentRepr::write_with_rawness)
                } else {
                    write_vec(w.ident, IdentRepr::write)
                },
                token_tree: w.token_tree,
                text: w.text,
            },
            doc_comments: write_vec(w.doc_comment, DocCommentRepr::write),
        })
    }

    fn serialize<ST: SpanTransformer, R: ReaderTrait<ST::Span>>(
        self,
        version: u32,
        span_data_table: &ST::Table,
    ) -> (tt::Delimiter<ST::Span>, Vec<R::TokenTree>) {
        let tag_bit_width = tag_bit_width(version);
        Reader::<ST, R> {
            subtree: if version >= ENCODE_CLOSE_SPAN_VERSION {
                read_vec(self.0.pre_doc_comment_leaf.subtree, SubtreeRepr::read_with_close_span)
            } else {
                read_vec(self.0.pre_doc_comment_leaf.subtree, SubtreeRepr::read)
            },
            literal: if version >= EXTENDED_LEAF_DATA {
                read_vec(self.0.pre_doc_comment_leaf.literal, LiteralRepr::read_with_kind)
            } else {
                read_vec(self.0.pre_doc_comment_leaf.literal, LiteralRepr::read)
            },
            punct: read_vec(self.0.pre_doc_comment_leaf.punct, PunctRepr::read),
            ident: if version >= EXTENDED_LEAF_DATA {
                read_vec(self.0.pre_doc_comment_leaf.ident, IdentRepr::read_with_rawness)
            } else {
                read_vec(self.0.pre_doc_comment_leaf.ident, IdentRepr::read)
            },
            doc_comment: read_vec(self.0.doc_comments, DocCommentRepr::read),
            token_tree: self.0.pre_doc_comment_leaf.token_tree,
            text: self.0.pre_doc_comment_leaf.text,
            span_data_table,
            version,
            tag_bit_width,
            tag_mask: (1 << tag_bit_width) - 1,
            _marker: PhantomData,
        }
        .read()
    }
}

#[derive(Debug)]
struct SubtreeRepr {
    open: SpanId,
    close: SpanId,
    kind: tt::DelimiterKind,
    tt: [u32; 2],
}

#[derive(Debug)]
struct LiteralRepr {
    id: SpanId,
    text: u32,
    suffix: u32,
    kind: u16,
}

#[derive(Debug)]
struct PunctRepr {
    id: SpanId,
    char: char,
    spacing: tt::Spacing,
}

#[derive(Debug)]
struct IdentRepr {
    id: SpanId,
    text: u32,
    is_raw: bool,
}

#[derive(Debug)]
struct DocCommentRepr {
    id: SpanId,
    text_with_comment_signs: u32,
    is_inner: bool,
    is_block: bool,
}

fn read_vec<T, F: Fn([u32; N]) -> T, const N: usize>(xs: Vec<u32>, f: F) -> Vec<T> {
    let (chunks, remainder) = xs.as_chunks();
    assert!(remainder.is_empty());
    chunks.iter().map(|chunk| f(*chunk)).collect()
}

fn write_vec<T, F: Fn(T) -> [u32; N], const N: usize>(xs: Vec<T>, f: F) -> Vec<u32> {
    xs.into_iter().map(f).collect::<Vec<_>>().into_flattened()
}

impl SubtreeRepr {
    fn write(self) -> [u32; 4] {
        let kind = match self.kind {
            tt::DelimiterKind::Invisible => 0,
            tt::DelimiterKind::Parenthesis => 1,
            tt::DelimiterKind::Brace => 2,
            tt::DelimiterKind::Bracket => 3,
        };
        [self.open.0, kind, self.tt[0], self.tt[1]]
    }
    fn read([open, kind, lo, len]: [u32; 4]) -> SubtreeRepr {
        let kind = match kind {
            0 => tt::DelimiterKind::Invisible,
            1 => tt::DelimiterKind::Parenthesis,
            2 => tt::DelimiterKind::Brace,
            3 => tt::DelimiterKind::Bracket,
            other => panic!("bad kind {other}"),
        };
        SubtreeRepr { open: SpanId(open), close: SpanId(!0), kind, tt: [lo, len] }
    }
    fn write_with_close_span(self) -> [u32; 5] {
        let kind = match self.kind {
            tt::DelimiterKind::Invisible => 0,
            tt::DelimiterKind::Parenthesis => 1,
            tt::DelimiterKind::Brace => 2,
            tt::DelimiterKind::Bracket => 3,
        };
        [self.open.0, self.close.0, kind, self.tt[0], self.tt[1]]
    }
    fn read_with_close_span([open, close, kind, lo, len]: [u32; 5]) -> SubtreeRepr {
        let kind = match kind {
            0 => tt::DelimiterKind::Invisible,
            1 => tt::DelimiterKind::Parenthesis,
            2 => tt::DelimiterKind::Brace,
            3 => tt::DelimiterKind::Bracket,
            other => panic!("bad kind {other}"),
        };
        SubtreeRepr { open: SpanId(open), close: SpanId(close), kind, tt: [lo, len] }
    }
}

impl LiteralRepr {
    fn write(self) -> [u32; 2] {
        [self.id.0, self.text]
    }
    fn read([id, text]: [u32; 2]) -> LiteralRepr {
        LiteralRepr { id: SpanId(id), text, kind: 0, suffix: !0 }
    }
    fn write_with_kind(self) -> [u32; 4] {
        [self.id.0, self.text, self.kind as u32, self.suffix]
    }
    fn read_with_kind([id, text, kind, suffix]: [u32; 4]) -> LiteralRepr {
        LiteralRepr { id: SpanId(id), text, kind: kind as u16, suffix }
    }
}

impl PunctRepr {
    fn write(self) -> [u32; 3] {
        let spacing = match self.spacing {
            tt::Spacing::Alone | tt::Spacing::JointHidden => 0,
            tt::Spacing::Joint => 1,
        };
        [self.id.0, self.char as u32, spacing]
    }
    fn read([id, char, spacing]: [u32; 3]) -> PunctRepr {
        let spacing = match spacing {
            0 => tt::Spacing::Alone,
            1 => tt::Spacing::Joint,
            other => panic!("bad spacing {other}"),
        };
        PunctRepr { id: SpanId(id), char: char.try_into().unwrap(), spacing }
    }
}

impl IdentRepr {
    fn write(self) -> [u32; 2] {
        [self.id.0, self.text]
    }
    fn read(data: [u32; 2]) -> IdentRepr {
        IdentRepr { id: SpanId(data[0]), text: data[1], is_raw: false }
    }
    fn write_with_rawness(self) -> [u32; 3] {
        [self.id.0, self.text, self.is_raw as u32]
    }
    fn read_with_rawness([id, text, is_raw]: [u32; 3]) -> IdentRepr {
        IdentRepr { id: SpanId(id), text, is_raw: is_raw == 1 }
    }
}

impl DocCommentRepr {
    fn write(self) -> [u32; 3] {
        [
            self.id.0,
            self.text_with_comment_signs,
            u16::from_le_bytes([self.is_inner.into(), self.is_block.into()]).into(),
        ]
    }

    fn read([id, text_with_comment_signs, is_inner_and_is_block]: [u32; 3]) -> DocCommentRepr {
        let [is_inner, is_block] = (is_inner_and_is_block as u16).to_le_bytes();
        DocCommentRepr {
            id: SpanId(id),
            text_with_comment_signs,
            is_inner: is_inner != 0,
            is_block: is_block != 0,
        }
    }
}

pub trait SpanTransformer {
    type Table;
    type Span: Copy + 'static;
    fn token_id_of(table: &mut Self::Table, s: Self::Span) -> SpanId;
    fn span_for_token_id(table: &Self::Table, id: SpanId) -> Self::Span;
}
impl SpanTransformer for SpanId {
    type Table = ();
    type Span = Self;
    fn token_id_of((): &mut Self::Table, token_id: Self::Span) -> SpanId {
        token_id
    }

    fn span_for_token_id((): &Self::Table, id: SpanId) -> Self::Span {
        id
    }
}
impl SpanTransformer for Span {
    type Table = SpanDataIndexMap;
    type Span = Self;
    fn token_id_of(table: &mut Self::Table, span: Self::Span) -> SpanId {
        SpanId(table.insert_full(span).0 as u32)
    }
    fn span_for_token_id(table: &Self::Table, id: SpanId) -> Self::Span {
        *table.get_index(id.0 as usize).unwrap_or_else(|| &table[0])
    }
}

enum SubtreeOrLeafRef<'a, Span, W: WriterTrait<'a, Span>> {
    Subtree(W::Subtree),
    Leaf(W::Leaf),
}

enum WorkItem<'a, Span, W: WriterTrait<'a, Span>> {
    Subtree(W::SubtreeIter),
    DesugaredDocCommentSubtree(tt::DocComment<Span>),
}

trait WriterTrait<'a, Span>: Sized {
    type Subtree;
    type Leaf: Borrow<tt::Leaf<Span>>;

    type SubtreeIter: Clone;

    fn subtree_data(subtree: &Self::Subtree) -> (tt::Delimiter<Span>, Self::SubtreeIter);
    fn subtree_len(subtree: &Self::Subtree) -> usize;

    fn subtree_iter_next(iter: &mut Self::SubtreeIter) -> Option<SubtreeOrLeafRef<'a, Span, Self>>;
}

struct Writer<'a, 'span, ST: SpanTransformer, W: WriterTrait<'a, ST::Span>> {
    work: VecDeque<(usize, usize, WorkItem<'a, ST::Span, W>)>,
    string_table: FxHashMap<std::borrow::Cow<'a, str>, u32>,
    span_data_table: &'span mut ST::Table,
    version: u32,
    tag_bit_width: u32,

    subtree: Vec<SubtreeRepr>,
    literal: Vec<LiteralRepr>,
    punct: Vec<PunctRepr>,
    ident: Vec<IdentRepr>,
    doc_comment: Vec<DocCommentRepr>,
    token_tree: Vec<u32>,
    text: Vec<String>,
}

impl<'a, ST: SpanTransformer, W: WriterTrait<'a, ST::Span>> Writer<'a, '_, ST, W> {
    fn write_subtree(&mut self, root: W::Subtree) {
        self.enqueue(root);
        while let Some((idx, len, subtree)) = self.work.pop_front() {
            self.subtree(idx, len, subtree);
        }
    }

    fn subtree(&mut self, idx: usize, n_tt: usize, subtree: WorkItem<'a, ST::Span, W>) {
        let mut first_tt = self.token_tree.len();
        self.token_tree.resize(first_tt + n_tt, !0);

        self.subtree[idx].tt = [first_tt as u32, (first_tt + n_tt) as u32];

        let mut push_tt = |this: &mut Self, idx_tag| {
            this.token_tree[first_tt] = idx_tag;
            first_tt += 1;
        };

        let mut subtree = match subtree {
            WorkItem::Subtree(it) => it,
            WorkItem::DesugaredDocCommentSubtree(doc_comment) => {
                let doc_ident = self.ident(&tt::Ident {
                    sym: sym::doc,
                    span: doc_comment.span,
                    is_raw: tt::IdentIsRaw::No,
                });
                push_tt(self, doc_ident);
                let eq_punct = self.punct(&tt::Punct {
                    char: '=',
                    spacing: tt::Spacing::Alone,
                    span: doc_comment.span,
                });
                push_tt(self, eq_punct);
                let doc_literal = self.literal(&doc_comment.literal_for_proc_macros());
                push_tt(self, doc_literal);
                return;
            }
        };

        while let Some(child) = W::subtree_iter_next(&mut subtree) {
            let idx_tag = match child {
                SubtreeOrLeafRef::Subtree(subtree) => {
                    let idx = self.enqueue(subtree);
                    idx << self.tag_bit_width
                }
                SubtreeOrLeafRef::Leaf(leaf) => match leaf.borrow() {
                    tt::Leaf::Literal(lit) => self.literal(lit),
                    tt::Leaf::Punct(punct) => self.punct(punct),
                    tt::Leaf::Ident(ident) => self.ident(ident),
                    tt::Leaf::DocComment(doc_comment) => {
                        if self.version >= DOC_COMMENT_LEAF {
                            let idx = self.doc_comment.len() as u32;
                            let id = self.token_id_of(doc_comment.span);
                            let text = self.intern_owned(
                                doc_comment.text_with_comment_signs.as_str().to_owned(),
                            );
                            let is_inner = doc_comment.doc_style == tt::DocCommentStyle::Inner;
                            let is_block = doc_comment.comment_style == tt::CommentStyle::Block;
                            self.doc_comment.push(DocCommentRepr {
                                id,
                                text_with_comment_signs: text,
                                is_inner,
                                is_block,
                            });
                            (idx << self.tag_bit_width) | 0b100
                        } else {
                            let hash_punct = self.punct(&tt::Punct {
                                char: '#',
                                spacing: tt::Spacing::Alone,
                                span: doc_comment.span,
                            });
                            push_tt(self, hash_punct);
                            if doc_comment.doc_style == tt::DocCommentStyle::Inner {
                                let bang_punct = self.punct(&tt::Punct {
                                    char: '!',
                                    spacing: tt::Spacing::Alone,
                                    span: doc_comment.span,
                                });
                                push_tt(self, bang_punct);
                            }

                            /// `doc`, `=`, and the literal.
                            const DESUGARED_DOC_COMMENT_SUBTREE_LEN: usize = 3;
                            let idx = self.subtree.len();
                            let kind = tt::DelimiterKind::Bracket;
                            let span = self.token_id_of(doc_comment.span);
                            self.subtree.push(SubtreeRepr {
                                open: span,
                                close: span,
                                kind,
                                tt: [!0, !0],
                            });
                            self.work.push_back((
                                idx,
                                DESUGARED_DOC_COMMENT_SUBTREE_LEN,
                                WorkItem::DesugaredDocCommentSubtree(doc_comment.clone()),
                            ));
                            push_tt(self, idx as u32);

                            return;
                        }
                    }
                },
            };
            push_tt(self, idx_tag);
        }
    }

    fn ident(&mut self, ident: &tt::Ident<ST::Span>) -> u32 {
        let idx = self.ident.len() as u32;
        let id = self.token_id_of(ident.span);
        let text = if self.version >= EXTENDED_LEAF_DATA {
            self.intern_owned(ident.sym.as_str().to_owned())
        } else if ident.is_raw.yes() {
            self.intern_owned(format!("r#{}", ident.sym.as_str(),))
        } else {
            self.intern_owned(ident.sym.as_str().to_owned())
        };
        self.ident.push(IdentRepr { id, text, is_raw: ident.is_raw.yes() });
        (idx << self.tag_bit_width) | 0b011
    }

    fn punct(&mut self, punct: &tt::Punct<ST::Span>) -> u32 {
        let idx = self.punct.len() as u32;
        let id = self.token_id_of(punct.span);
        self.punct.push(PunctRepr { char: punct.char, spacing: punct.spacing, id });
        (idx << self.tag_bit_width) | 0b010
    }

    fn literal(&mut self, lit: &tt::Literal<ST::Span>) -> u32 {
        let idx = self.literal.len() as u32;
        let id = self.token_id_of(lit.span);
        let (text, suffix) = if self.version >= EXTENDED_LEAF_DATA {
            let (text, suffix) = lit.text_and_suffix();
            (
                self.intern_owned(text.to_owned()),
                if suffix.is_empty() { !0 } else { self.intern_owned(suffix.to_owned()) },
            )
        } else {
            (self.intern_owned(format!("{lit}")), !0)
        };
        self.literal.push(LiteralRepr {
            id,
            text,
            kind: u16::from_le_bytes(match lit.kind {
                tt::LitKind::Err(_) => [0, 0],
                tt::LitKind::Byte => [1, 0],
                tt::LitKind::Char => [2, 0],
                tt::LitKind::Integer => [3, 0],
                tt::LitKind::Float => [4, 0],
                tt::LitKind::Str => [5, 0],
                tt::LitKind::StrRaw(r) => [6, r],
                tt::LitKind::ByteStr => [7, 0],
                tt::LitKind::ByteStrRaw(r) => [8, r],
                tt::LitKind::CStr => [9, 0],
                tt::LitKind::CStrRaw(r) => [10, r],
            }),
            suffix,
        });
        (idx << self.tag_bit_width) | 0b001
    }

    fn enqueue(&mut self, subtree: W::Subtree) -> u32 {
        let idx = self.subtree.len();
        let (delimiter, contents) = W::subtree_data(&subtree);
        let len = if self.version >= DOC_COMMENT_LEAF {
            W::subtree_len(&subtree)
        } else {
            // We need to count doc comments as multiple items.
            let mut contents = contents.clone();
            let contents = std::iter::from_fn(move || W::subtree_iter_next(&mut contents));
            contents
                .map(|item| {
                    if let SubtreeOrLeafRef::Leaf(leaf) = item
                        && let tt::Leaf::DocComment(doc_comment) = leaf.borrow()
                    {
                        // `#`, `!` if inner, and `[...]`.
                        2 + usize::from(doc_comment.doc_style == tt::DocCommentStyle::Inner)
                    } else {
                        1
                    }
                })
                .sum()
        };
        let open = self.token_id_of(delimiter.open);
        let close = self.token_id_of(delimiter.close);
        let delimiter_kind = delimiter.kind;
        self.subtree.push(SubtreeRepr { open, close, kind: delimiter_kind, tt: [!0, !0] });
        self.work.push_back((idx, len, WorkItem::Subtree(contents)));
        idx as u32
    }

    fn token_id_of(&mut self, span: ST::Span) -> SpanId {
        ST::token_id_of(self.span_data_table, span)
    }

    fn intern_owned(&mut self, text: String) -> u32 {
        let table = &mut self.text;
        *self.string_table.entry(text.clone().into()).or_insert_with(|| {
            let idx = table.len();
            table.push(text);
            idx as u32
        })
    }
}

trait ReaderTrait<Span> {
    type TokenTree;

    fn leaf(leaf: tt::Leaf<Span>) -> Self::TokenTree;

    fn append_subtree(
        delimiter: tt::Delimiter<Span>,
        children: Vec<Self::TokenTree>,
        insert_into: &mut Vec<Self::TokenTree>,
    );
}

struct Reader<'span, ST: SpanTransformer, R: ReaderTrait<ST::Span>> {
    version: u32,
    tag_bit_width: u32,
    tag_mask: u32,
    subtree: Vec<SubtreeRepr>,
    literal: Vec<LiteralRepr>,
    punct: Vec<PunctRepr>,
    ident: Vec<IdentRepr>,
    doc_comment: Vec<DocCommentRepr>,
    token_tree: Vec<u32>,
    text: Vec<String>,
    span_data_table: &'span ST::Table,
    _marker: PhantomData<R>,
}

impl<ST: SpanTransformer, R: ReaderTrait<ST::Span>> Reader<'_, ST, R> {
    pub(crate) fn read(self) -> (tt::Delimiter<ST::Span>, Vec<R::TokenTree>) {
        let mut res: Vec<Option<(tt::Delimiter<ST::Span>, Vec<R::TokenTree>)>> =
            (0..self.subtree.len()).map(|_| None).collect();
        let read_span = |id| ST::span_for_token_id(self.span_data_table, id);
        for i in (0..self.subtree.len()).rev() {
            let repr = &self.subtree[i];
            let token_trees = &self.token_tree[repr.tt[0] as usize..repr.tt[1] as usize];
            let delimiter = tt::Delimiter {
                open: read_span(repr.open),
                close: read_span(repr.close),
                kind: repr.kind,
            };
            let mut s = Vec::new();
            for &idx_tag in token_trees {
                let tag = idx_tag & self.tag_mask;
                let idx = (idx_tag >> self.tag_bit_width) as usize;
                match tag {
                    // XXX: we iterate subtrees in reverse to guarantee
                    // that this unwrap doesn't fire.
                    0b000 => {
                        let (delimiter, subtree) = res[idx].take().unwrap();
                        R::append_subtree(delimiter, subtree, &mut s);
                    }
                    0b001 => {
                        use tt::LitKind::*;
                        let repr = &self.literal[idx];
                        let text = self.text[repr.text as usize].as_str();
                        let span = read_span(repr.id);
                        s.push(R::leaf(tt::Leaf::Literal(if self.version >= EXTENDED_LEAF_DATA {
                            tt::Literal::new(
                                text,
                                span,
                                match u16::to_le_bytes(repr.kind) {
                                    [0, _] => Err(()),
                                    [1, _] => Byte,
                                    [2, _] => Char,
                                    [3, _] => Integer,
                                    [4, _] => Float,
                                    [5, _] => Str,
                                    [6, r] => StrRaw(r),
                                    [7, _] => ByteStr,
                                    [8, r] => ByteStrRaw(r),
                                    [9, _] => CStr,
                                    [10, r] => CStrRaw(r),
                                    _ => unreachable!(),
                                },
                                if repr.suffix != !0 {
                                    self.text[repr.suffix as usize].as_str()
                                } else {
                                    ""
                                },
                            )
                        } else {
                            tt::literal_from_str_or_err(text, span)
                        })))
                    }
                    0b010 => {
                        let repr = &self.punct[idx];
                        s.push(R::leaf(tt::Leaf::Punct(tt::Punct {
                            char: repr.char,
                            spacing: repr.spacing,
                            span: read_span(repr.id),
                        })))
                    }
                    0b011 => {
                        let repr = &self.ident[idx];
                        let text = self.text[repr.text as usize].as_str();
                        let (is_raw, text) = if self.version >= EXTENDED_LEAF_DATA {
                            (
                                if repr.is_raw { tt::IdentIsRaw::Yes } else { tt::IdentIsRaw::No },
                                text,
                            )
                        } else {
                            tt::IdentIsRaw::split_from_symbol(text)
                        };
                        s.push(R::leaf(tt::Leaf::Ident(tt::Ident {
                            sym: Symbol::intern(text),
                            span: read_span(repr.id),
                            is_raw,
                        })))
                    }
                    0b100 => {
                        let repr = &self.doc_comment[idx];
                        let text_with_comment_signs =
                            self.text[repr.text_with_comment_signs as usize].as_str();
                        let doc_style = if repr.is_inner {
                            tt::DocCommentStyle::Inner
                        } else {
                            tt::DocCommentStyle::Outer
                        };
                        let comment_style = if repr.is_block {
                            tt::CommentStyle::Block
                        } else {
                            tt::CommentStyle::Line
                        };
                        s.push(R::leaf(tt::Leaf::DocComment(tt::DocComment {
                            text_with_comment_signs: Symbol::intern(text_with_comment_signs),
                            span: read_span(repr.id),
                            doc_style,
                            comment_style,
                        })))
                    }
                    other => panic!("bad tag: {other}"),
                }
            }
            res[i] = Some((delimiter, s));
        }

        res[0].take().unwrap()
    }
}
