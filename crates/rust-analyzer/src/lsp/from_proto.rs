//! Conversion lsp_types types to rust-analyzer specific ones.
use anyhow::format_err;
use hir::{HirFilePosition, HirFileRange, MacroCallId};
use ide::{Annotation, AnnotationKind, AssistKind, LineCol};
use ide_db::{FilePosition, FileRange, base_db::salsa, line_index::WideLineCol};
use paths::Utf8PathBuf;
use syntax::{TextRange, TextSize};
use vfs::AbsPathBuf;

use crate::{
    global_state::GlobalStateSnapshot,
    line_index::{LineIndex, PositionEncoding},
    lsp_ext, try_default,
};

#[derive(Clone)]
pub(crate) enum VfsOrMacroPath {
    Vfs(vfs::VfsPath),
    Macro(MacroCallId),
}

impl VfsOrMacroPath {
    pub(crate) fn into_vfs(self) -> Option<vfs::VfsPath> {
        if let Self::Vfs(v) = self { Some(v) } else { None }
    }
}

pub(crate) fn url_to_vfs_path(url: &lsp_types::Url) -> anyhow::Result<VfsOrMacroPath> {
    if url.scheme() == "rust-macro-file" {
        // rust-macro-file:/id.macro-file.rs
        let macro_call = url
            .path()
            .strip_suffix(".macro-file.rs")
            .and_then(|it| it.parse::<u64>().ok())
            .ok_or_else(|| format_err!("Invalid `rust-macro-file` url: {url:?}"))?;

        return Ok(VfsOrMacroPath::Macro(unsafe {
            salsa::plumbing::FromId::from_id(salsa::Id::from_bits(macro_call))
        }));
    }
    let path = url.to_file_path().map_err(|()| anyhow::format_err!("url is not a file"))?;
    let path = AbsPathBuf::try_from(Utf8PathBuf::from_path_buf(path).unwrap()).unwrap();
    Ok(VfsOrMacroPath::Vfs(vfs::VfsPath::from(path)))
}

pub(crate) fn offset(
    line_index: &LineIndex,
    position: lsp_types::Position,
) -> anyhow::Result<TextSize> {
    let line_col = match line_index.encoding {
        PositionEncoding::Utf8 => LineCol { line: position.line, col: position.character },
        PositionEncoding::Wide(enc) => {
            let line_col = WideLineCol { line: position.line, col: position.character };
            line_index
                .index
                .to_utf8(enc, line_col)
                .ok_or_else(|| format_err!("Invalid wide col offset"))?
        }
    };
    line_index.offset(line_col).ok_or_else(|| {
        format_err!("Invalid offset {line_col:?} (line index length: {:?})", line_index.index.len())
    })
}

pub(crate) fn text_range(
    line_index: &LineIndex,
    range: lsp_types::Range,
) -> anyhow::Result<TextRange> {
    let start = offset(line_index, range.start)?;
    let end = offset(line_index, range.end)?;
    match end < start {
        true => Err(format_err!("Invalid Range")),
        false => Ok(TextRange::new(start, end)),
    }
}

/// Returns `None` if the file was excluded.
pub(crate) fn file_position(
    snap: &GlobalStateSnapshot,
    tdpp: lsp_types::TextDocumentPositionParams,
) -> anyhow::Result<Option<FilePosition>> {
    let file_id = try_default!(snap.url_to_file_id(&tdpp.text_document.uri)?);
    let line_index = snap.file_line_index(file_id)?;
    let offset = offset(&line_index, tdpp.position)?;
    Ok(Some(FilePosition { file_id, offset }))
}

/// Returns `None` if the file was excluded.
pub(crate) fn hir_file_position(
    snap: &GlobalStateSnapshot,
    tdpp: lsp_types::TextDocumentPositionParams,
) -> anyhow::Result<Option<HirFilePosition>> {
    let file_id = try_default!(snap.url_to_hir_file_id(&tdpp.text_document.uri)?);
    let line_index = snap.hir_line_index(file_id)?;
    let offset = offset(&line_index, tdpp.position)?;
    Ok(Some(HirFilePosition { file_id, offset }))
}

/// Returns `None` if the file was excluded.
pub(crate) fn file_range(
    snap: &GlobalStateSnapshot,
    text_document_identifier: &lsp_types::TextDocumentIdentifier,
    range: lsp_types::Range,
) -> anyhow::Result<Option<FileRange>> {
    file_range_uri(snap, &text_document_identifier.uri, range)
}

/// Returns `None` if the file was excluded.
pub(crate) fn hir_file_range(
    snap: &GlobalStateSnapshot,
    text_document_identifier: &lsp_types::TextDocumentIdentifier,
    range: lsp_types::Range,
) -> anyhow::Result<Option<HirFileRange>> {
    let file_id = try_default!(snap.url_to_hir_file_id(&text_document_identifier.uri)?);
    let line_index = snap.hir_line_index(file_id)?;
    let range = text_range(&line_index, range)?;
    Ok(Some(HirFileRange { file_id, range }))
}

pub(crate) fn file_range_uri(
    snap: &GlobalStateSnapshot,
    document: &lsp_types::Url,
    range: lsp_types::Range,
) -> anyhow::Result<Option<FileRange>> {
    let file_id = try_default!(snap.url_to_file_id(document)?);
    let line_index = snap.file_line_index(file_id)?;
    let range = text_range(&line_index, range)?;
    Ok(Some(FileRange { file_id, range }))
}

pub(crate) fn assist_kind(kind: lsp_types::CodeActionKind) -> Option<AssistKind> {
    let assist_kind = match &kind {
        k if k == &lsp_types::CodeActionKind::EMPTY => AssistKind::Generate,
        k if k == &lsp_types::CodeActionKind::QUICKFIX => AssistKind::QuickFix,
        k if k == &lsp_types::CodeActionKind::REFACTOR => AssistKind::Refactor,
        k if k == &lsp_types::CodeActionKind::REFACTOR_EXTRACT => AssistKind::RefactorExtract,
        k if k == &lsp_types::CodeActionKind::REFACTOR_INLINE => AssistKind::RefactorInline,
        k if k == &lsp_types::CodeActionKind::REFACTOR_REWRITE => AssistKind::RefactorRewrite,
        _ => return None,
    };

    Some(assist_kind)
}

/// Returns `None` if the file was excluded.
pub(crate) fn annotation(
    snap: &GlobalStateSnapshot,
    range: lsp_types::Range,
    data: lsp_ext::CodeLensResolveData,
) -> anyhow::Result<Option<Annotation>> {
    match data.kind {
        lsp_ext::CodeLensResolveDataKind::Impls(params) => {
            if snap.url_file_version(&params.text_document_position_params.text_document.uri)
                != Some(data.version)
            {
                return Ok(None);
            }
            let pos @ HirFilePosition { file_id, .. } =
                try_default!(hir_file_position(snap, params.text_document_position_params)?);
            let line_index = snap.hir_line_index(file_id)?;

            Ok(Annotation {
                range: text_range(&line_index, range)?,
                kind: AnnotationKind::HasImpls { pos, data: None },
            })
        }
        lsp_ext::CodeLensResolveDataKind::References(params) => {
            if snap.url_file_version(&params.text_document.uri) != Some(data.version) {
                return Ok(None);
            }
            let pos @ HirFilePosition { file_id, .. } =
                try_default!(hir_file_position(snap, params)?);
            let line_index = snap.hir_line_index(file_id)?;

            Ok(Annotation {
                range: text_range(&line_index, range)?,
                kind: AnnotationKind::HasReferences { pos, data: None },
            })
        }
    }
    .map(Some)
}
