#![cfg(target_arch = "wasm32")]
#![allow(non_snake_case)]

use ra_ide_api::{Analysis, FileId, FilePosition, LineCol, Severity};
use ra_syntax::{SyntaxKind, TextRange};
use wasm_bindgen::prelude::*;

mod return_types;
use return_types::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Warn).expect("could not install logging hook");
    log::info!("worker initialized")
}

#[wasm_bindgen]
pub struct WorldState {
    analysis: Analysis,
    file_id: FileId,
}

#[wasm_bindgen]
impl WorldState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let (analysis, file_id) = Analysis::from_single_file("".to_owned());
        Self { analysis, file_id }
    }

    pub fn update(&mut self, code: String) -> JsValue {
        let (analysis, file_id) = Analysis::from_single_file(code);
        self.analysis = analysis;
        self.file_id = file_id;

        let highlights: Vec<_> = self
            .analysis
            .highlight(file_id)
            .unwrap()
            .into_iter()
            .map(|hl| Highlight { tag: Some(hl.tag), range: self.range(hl.range) })
            .collect();

        let diagnostics: Vec<_> = self
            .analysis
            .diagnostics(self.file_id)
            .unwrap()
            .into_iter()
            .map(|d| {
                let Range { startLineNumber, startColumn, endLineNumber, endColumn } =
                    self.range(d.range);
                Diagnostic {
                    message: d.message,
                    severity: match d.severity {
                        Severity::Error => 8,       // monaco MarkerSeverity.Error
                        Severity::WeakWarning => 1, // monaco MarkerSeverity.Hint
                    },
                    startLineNumber,
                    startColumn,
                    endLineNumber,
                    endColumn,
                }
            })
            .collect();

        serde_wasm_bindgen::to_value(&UpdateResult { diagnostics, highlights }).unwrap()
    }

    fn file_pos(&self, line: u32, col_utf16: u32) -> FilePosition {
        // monaco doesn't work zero-based
        let line_col = LineCol { line: line - 1, col_utf16: col_utf16 - 1 };
        let offset = self.analysis.file_line_index(self.file_id).unwrap().offset(line_col);
        FilePosition { file_id: self.file_id, offset }
    }

    fn range(&self, text_range: TextRange) -> Range {
        let line_index = self.analysis.file_line_index(self.file_id).unwrap();
        let start = line_index.line_col(text_range.start());
        let end = line_index.line_col(text_range.end());

        Range {
            startLineNumber: start.line + 1,
            startColumn: start.col_utf16 + 1,
            endLineNumber: end.line + 1,
            endColumn: end.col_utf16 + 1,
        }
    }

    pub fn on_dot_typed(&self, line_number: u32, column: u32) {
        let pos = self.file_pos(line_number, column);
        log::warn!("on_dot_typed");
        let res = self.analysis.on_dot_typed(pos).unwrap();

        log::debug!("{:?}", res);
    }

    pub fn hover(&self, line_number: u32, column: u32) -> JsValue {
        let pos = self.file_pos(line_number, column);
        log::warn!("hover");
        let info = match self.analysis.hover(pos).unwrap() {
            Some(info) => info,
            _ => return JsValue::NULL,
        };

        let value = info.info.to_markup();
        let hover =
            Hover { contents: vec![MarkdownString { value }], range: self.range(info.range) };

        serde_wasm_bindgen::to_value(&hover).unwrap()
    }

    pub fn code_lenses(&self) -> JsValue {
        log::warn!("code_lenses");

        let results: Vec<_> = self
            .analysis
            .file_structure(self.file_id)
            .unwrap()
            .into_iter()
            .filter(|it| match it.kind {
                SyntaxKind::TRAIT_DEF | SyntaxKind::STRUCT_DEF | SyntaxKind::ENUM_DEF => true,
                _ => false,
            })
            .filter_map(|it| {
                let position =
                    FilePosition { file_id: self.file_id, offset: it.node_range.start() };
                let nav_info = self.analysis.goto_implementation(position).unwrap()?;

                let title = if nav_info.info.len() == 1 {
                    "1 implementation".into()
                } else {
                    format!("{} implementations", nav_info.info.len())
                };

                let positions = nav_info
                    .info
                    .iter()
                    .map(|target| target.focus_range().unwrap_or(target.full_range()))
                    .map(|range| self.range(range))
                    .collect();

                Some(CodeLensSymbol {
                    range: self.range(it.node_range),
                    command: Some(Command {
                        id: "editor.action.showReferences".into(),
                        title,
                        positions,
                    }),
                })
            })
            .collect();

        serde_wasm_bindgen::to_value(&results).unwrap()
    }

    pub fn references(&self, line_number: u32, column: u32) -> JsValue {
        let pos = self.file_pos(line_number, column);
        log::warn!("references");
        let info = match self.analysis.find_all_refs(pos).unwrap() {
            Some(info) => info,
            _ => return JsValue::NULL,
        };

        let res: Vec<_> =
            info.into_iter().map(|r| Highlight { tag: None, range: self.range(r.range) }).collect();
        serde_wasm_bindgen::to_value(&res).unwrap()
    }

    pub fn prepare_rename(&self, line_number: u32, column: u32) -> JsValue {
        let pos = self.file_pos(line_number, column);
        log::warn!("prepare_rename");
        let refs = match self.analysis.find_all_refs(pos).unwrap() {
            None => return JsValue::NULL,
            Some(refs) => refs,
        };

        let declaration = refs.declaration();
        let range = self.range(declaration.range());
        let text = declaration.name().to_string();

        serde_wasm_bindgen::to_value(&RenameLocation { range, text }).unwrap()
    }

    pub fn rename(&self, line_number: u32, column: u32, new_name: &str) -> JsValue {
        let pos = self.file_pos(line_number, column);
        log::warn!("rename");
        let change = match self.analysis.rename(pos, new_name) {
            Ok(Some(change)) => change,
            _ => return JsValue::NULL,
        };

        let result: Vec<_> = change
            .source_file_edits
            .iter()
            .flat_map(|sfe| sfe.edit.as_atoms())
            .map(|atom| TextEdit { range: self.range(atom.delete), text: atom.insert.clone() })
            .collect();

        serde_wasm_bindgen::to_value(&result).unwrap()
    }
}
