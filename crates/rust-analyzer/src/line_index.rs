//! Enhances `ide::LineIndex` with additional info required to convert offsets
//! into lsp positions.
//!
//! We maintain invariant that all internal strings use `\n` as line separator.
//! This module does line ending conversion and detection (so that we can
//! convert back to `\r\n` on the way out).

use ide::{TextRange, TextSize};
use ide_db::line_index::WideEncoding;
use itertools::Itertools;
use memchr::memmem;
use syntax_bridge::prettify_macro_expansion::PrettifyWsKind;
use triomphe::Arc;

#[derive(Clone, Copy)]
pub enum PositionEncoding {
    Utf8,
    Wide(WideEncoding),
}

pub(crate) struct LineIndex {
    pub(crate) index: Arc<ide::LineIndex>,
    pub(crate) endings: LineEndings,
    pub(crate) encoding: PositionEncoding,
    pub(crate) transform: PositionTransform,
}

impl LineIndex {
    pub(crate) fn line_col(&self, mut offset: TextSize) -> ide::LineCol {
        if !self.transform.insertions.is_empty() {
            offset += TextSize::new(
                self.transform
                    .insertions
                    .iter()
                    .copied()
                    .take_while(|&(off, _)| off < offset)
                    .map(|(_, len)| ws_kind_width(len))
                    .sum::<u32>(),
            );
        }
        self.index.line_col(offset)
    }

    pub(crate) fn offset(&self, line_col: ide::LineCol) -> Option<TextSize> {
        let mut offset = self.index.offset(line_col)?;
        if !self.transform.insertions.is_empty() {
            let mut iter = self.transform.insertions.iter();
            let overall_sub = TextSize::new(if line_col.line == 0 {
                0
            } else {
                // collect all ws insertions until the line `line` starts
                // we need to offset our range by this value
                let mut nl_seen = 0;
                iter.peeking_take_while(|&&(_p, ws)| {
                    let m = nl_seen != line_col.line;
                    if ws == PrettifyWsKind::Newline {
                        nl_seen += 1;
                    }
                    m
                })
                .copied()
                .map(|(_, len)| ws_kind_width(len))
                .sum::<u32>()
            });
            offset -= overall_sub;

            for (pos, ws) in iter.copied().take_while(|&(_, ws)| ws != PrettifyWsKind::Newline) {
                if offset < pos {
                    break;
                }
                offset -= TextSize::new(ws_kind_width(ws));
            }
        }
        Some(offset)
    }

    #[allow(dead_code)]
    pub(crate) fn line(&self, line: u32) -> Option<ide::TextRange> {
        let mut range = self.index.line(line)?;
        if !self.transform.insertions.is_empty() {
            let mut iter = self.transform.insertions.iter();
            let overall_sub = TextSize::new(if line == 0 {
                0
            } else {
                // collect all ws insertions until the line `line` starts
                // we need to offset our range by this value
                let mut nl_seen = 0;
                iter.peeking_take_while(|&&(_p, ws)| {
                    let m = nl_seen != line;
                    if ws == PrettifyWsKind::Newline {
                        nl_seen += 1;
                    }
                    m
                })
                .copied()
                .map(|(_, len)| ws_kind_width(len))
                .sum::<u32>()
            });

            // collect all ws insertions within the line `line`
            // we need to deduct this from range end by this value
            let end_sub = TextSize::new(
                iter.copied()
                    .take_while(|&(_, ws)| ws != PrettifyWsKind::Newline)
                    .map(|(_, len)| ws_kind_width(len))
                    .sum::<u32>(),
            );
            range =
                TextRange::new(range.start() - overall_sub, range.end() - overall_sub - end_sub);
        }
        Some(range)
    }
}

#[derive(Default)]
pub(crate) struct PositionTransform {
    pub insertions: Vec<(TextSize, PrettifyWsKind)>,
}

fn ws_kind_width(ws: PrettifyWsKind) -> u32 {
    match ws {
        PrettifyWsKind::Space => 1,
        PrettifyWsKind::Indent(indent) => 4 * (indent as u32),
        PrettifyWsKind::Newline => 1,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum LineEndings {
    Unix,
    Dos,
}

impl LineEndings {
    /// Replaces `\r\n` with `\n` in-place in `src`.
    pub(crate) fn normalize(src: String) -> (String, LineEndings) {
        // We replace `\r\n` with `\n` in-place, which doesn't break utf-8 encoding.
        // While we *can* call `as_mut_vec` and do surgery on the live string
        // directly, let's rather steal the contents of `src`. This makes the code
        // safe even if a panic occurs.

        let mut buf = src.into_bytes();
        let mut gap_len = 0;
        let mut tail = buf.as_mut_slice();
        let mut crlf_seen = false;

        let finder = memmem::Finder::new(b"\r\n");

        loop {
            let idx = match finder.find(&tail[gap_len..]) {
                None if crlf_seen => tail.len(),
                // SAFETY: buf is unchanged and therefore still contains utf8 data
                None => return (unsafe { String::from_utf8_unchecked(buf) }, LineEndings::Unix),
                Some(idx) => {
                    crlf_seen = true;
                    idx + gap_len
                }
            };
            tail.copy_within(gap_len..idx, 0);
            tail = &mut tail[idx - gap_len..];
            if tail.len() == gap_len {
                break;
            }
            gap_len += 1;
        }

        // Account for removed `\r`.
        // After `set_len`, `buf` is guaranteed to contain utf-8 again.
        let src = unsafe {
            let new_len = buf.len() - gap_len;
            buf.set_len(new_len);
            String::from_utf8_unchecked(buf)
        };
        (src, LineEndings::Dos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix() {
        let src = "a\nb\nc\n\n\n\n";
        let (res, endings) = LineEndings::normalize(src.into());
        assert_eq!(endings, LineEndings::Unix);
        assert_eq!(res, src);
    }

    #[test]
    fn dos() {
        let src = "\r\na\r\n\r\nb\r\nc\r\n\r\n\r\n\r\n";
        let (res, endings) = LineEndings::normalize(src.into());
        assert_eq!(endings, LineEndings::Dos);
        assert_eq!(res, "\na\n\nb\nc\n\n\n\n");
    }

    #[test]
    fn mixed() {
        let src = "a\r\nb\r\nc\r\n\n\r\n\n";
        let (res, endings) = LineEndings::normalize(src.into());
        assert_eq!(endings, LineEndings::Dos);
        assert_eq!(res, "a\nb\nc\n\n\n\n");
    }

    #[test]
    fn none() {
        let src = "abc";
        let (res, endings) = LineEndings::normalize(src.into());
        assert_eq!(endings, LineEndings::Unix);
        assert_eq!(res, src);
    }
}
