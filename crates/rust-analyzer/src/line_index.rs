//! Enhances `ide::LineIndex` with additional info required to convert offsets
//! into lsp positions.
//!
//! The VFS normalizes UTF-8 file contents to use `\n` as the line separator.
//! We retain the detected line-ending convention here so outgoing LSP edits
//! can restore `\r\n` when needed.

use ide_db::line_index::WideEncoding;
use triomphe::Arc;
use vfs::LineEndings;

#[derive(Clone, Copy)]
pub enum PositionEncoding {
    Utf8,
    Wide(WideEncoding),
}

pub(crate) struct LineIndex {
    pub(crate) index: Arc<ide::LineIndex>,
    pub(crate) endings: LineEndings,
    pub(crate) encoding: PositionEncoding,
}
