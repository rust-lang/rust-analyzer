use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Hover {
    pub range: Range,
    pub contents: Vec<MarkdownString>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Range {
    pub startLineNumber: u32,
    pub startColumn: u32,
    pub endLineNumber: u32,
    pub endColumn: u32,
}

#[derive(Serialize)]
pub struct MarkdownString {
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct CodeLensSymbol {
    pub range: Range,
    pub command: Option<Command>,
}

#[derive(Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub title: String,
    pub positions: Vec<Range>, // customized
}

#[derive(Serialize)]
pub struct Highlight {
    pub tag: Option<&'static str>,
    pub range: Range,
}

#[derive(Serialize)]
pub struct TextEdit {
    pub range: Range,
    pub text: String,
}

#[derive(Serialize)]
pub struct UpdateResult {
    pub diagnostics: Vec<Diagnostic>,
    pub highlights: Vec<Highlight>,
}

#[derive(Serialize)]
pub struct Diagnostic {
    pub message: String,
    pub startLineNumber: u32,
    pub startColumn: u32,
    pub endLineNumber: u32,
    pub endColumn: u32,
    pub severity: u32,
}

#[derive(Serialize)]
pub struct RenameLocation {
    pub range: Range,
    pub text: String,
}
