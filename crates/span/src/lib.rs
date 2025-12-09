//! File and span related types.

#![cfg_attr(feature = "in-rust-tree", feature(rustc_private))]

#[cfg(feature = "in-rust-tree")]
extern crate rustc_driver as _;

use std::fmt::{self, Write};

mod ast_id;
mod file_id;
mod hygiene;
mod map;

pub use self::{
    ast_id::{
        AstIdMap, AstIdNode, ErasedFileAstId, FIXUP_ERASED_FILE_AST_ID_MARKER, FileAstId,
        NO_DOWNMAP_ERASED_FILE_AST_ID_MARKER, ROOT_ERASED_FILE_AST_ID,
    },
    file_id::{EditionedFileId, File},
    hygiene::{SyntaxContext, Transparency},
    map::{RealSpanMap, SpanMap},
};

pub use syntax::Edition;
pub use text_size::{TextRange, TextSize};
// TODO: Remove this re-export once all usages are migrated to EditionedFileId
pub use vfs::FileId;

pub type Span = SpanData<SyntaxContext>;

impl Span {
    pub fn cover(self, other: Span) -> Span {
        if self.anchor != other.anchor {
            return self;
        }
        let range = self.range.cover(other.range);
        Span { range, ..self }
    }

    pub fn join(
        self,
        other: Span,
        differing_anchor: impl FnOnce(Span, Span) -> Option<Span>,
    ) -> Option<Span> {
        // We can't modify the span range for fixup spans, those are meaningful to fixup, so just
        // prefer the non-fixup span.
        if self.anchor.ast_id == FIXUP_ERASED_FILE_AST_ID_MARKER {
            return Some(other);
        }
        if other.anchor.ast_id == FIXUP_ERASED_FILE_AST_ID_MARKER {
            return Some(self);
        }
        if self.anchor != other.anchor {
            return differing_anchor(self, other);
        }
        // Differing context, we can't merge these so prefer the one that's root
        if self.ctx != other.ctx {
            if self.ctx.is_root() {
                return Some(other);
            } else if other.ctx.is_root() {
                return Some(self);
            }
        }
        Some(Span { range: self.range.cover(other.range), anchor: other.anchor, ctx: other.ctx })
    }
}

/// Spans represent a region of code, used by the IDE to be able link macro inputs and outputs
/// together. Positions in spans are relative to some [`SpanAnchor`] to make them more incremental
/// friendly.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpanData<Ctx> {
    /// The text range of this span, relative to the anchor.
    /// We need the anchor for incrementality, as storing absolute ranges will require
    /// recomputation on every change in a file at all times.
    pub range: TextRange,
    /// The anchor this span is relative to.
    pub anchor: SpanAnchor,
    /// The syntax context of the span.
    pub ctx: Ctx,
}

impl<Ctx: fmt::Debug> fmt::Debug for SpanData<Ctx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            fmt::Debug::fmt(&self.anchor.file_id.as_u32(), f)?;
            f.write_char(':')?;
            write!(f, "{:#?}", self.anchor.ast_id)?;
            f.write_char('@')?;
            fmt::Debug::fmt(&self.range, f)?;
            f.write_char('#')?;
            self.ctx.fmt(f)
        } else {
            f.debug_struct("SpanData")
                .field("range", &self.range)
                .field("anchor", &self.anchor)
                .field("ctx", &self.ctx)
                .finish()
        }
    }
}

impl<Ctx: Copy> SpanData<Ctx> {
    pub fn eq_ignoring_ctx(self, other: Self) -> bool {
        self.anchor == other.anchor && self.range == other.range
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.anchor.file_id.as_u32(), f)?;
        f.write_char(':')?;
        write!(f, "{:#?}", self.anchor.ast_id)?;
        f.write_char('@')?;
        fmt::Debug::fmt(&self.range, f)?;
        f.write_char('#')?;
        self.ctx.fmt(f)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct SpanAnchor {
    pub file_id: EditionedFileId,
    pub ast_id: ErasedFileAstId,
}

impl fmt::Debug for SpanAnchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SpanAnchor").field(&self.file_id).field(&self.ast_id).finish()
    }
}

#[cfg(not(feature = "salsa"))]
mod salsa {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Id(u32);
}

/// Input to the analyzer is a set of files, where each file is identified by
/// `FileId` and contains source code. However, another source of source code in
/// Rust are macros: each macro can be thought of as producing a "temporary
/// file". To assign an id to such a file, we use the id of the macro call that
/// produced the file. So, a `HirFileId` is either a `FileId` (source code
/// written by user), or a `MacroCallId` (source code produced by macro).
///
/// What is a `MacroCallId`? Simplifying, it's a `HirFileId` of a file
/// containing the call plus the offset of the macro call in the file. Note that
/// this is a recursive definition! However, the size_of of `HirFileId` is
/// finite (because everything bottoms out at the real `FileId`) and small
/// (`MacroCallId` uses the location interning. You can check details here:
/// <https://en.wikipedia.org/wiki/String_interning>).
///
/// Internally this holds a `salsa::Id`, but we cannot use this definition here
/// as it references things from base-db and hir-expand.
// FIXME: Give this a better fitting name
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HirFileId(pub salsa::Id);

/// `MacroCallId` identifies a particular macro invocation, like
/// `println!("Hello, {}", world)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MacroCallId(pub salsa::Id);
