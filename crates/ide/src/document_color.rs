#![allow(warnings)]

use hir::{EditionedFileId, Semantics};
use ide_db::RootDatabase;
use span::FileId;
use syntax::AstNode as _;

pub struct DocumentColor {
    pub start_line: u32,
    pub start_char: u32,
    pub end_char: u32,
    pub end_line: u32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Document colors let IDEs annotate regions of source code as having a particular color.
///
/// The advantage over `textDocument/documentHighlight` is that IDEs can create virtual text
/// to show swatches, representing the color.
///
/// # Basic Plan
///
/// 1. Visit every expression that is known at compile time (is `const`)
/// 1. If the type `struct` of the expression contains
///    an attribute `#[rust_analyzer::color::rgb]`, proceed
/// 1. If there are 3 `f32` fields with attribute `#[rust_analyzer::color::rgb::{r,g,b}]`,
///    then we have all the necessary information to construct the type itself
pub(crate) fn document_color(db: &RootDatabase, file_id: FileId) -> Vec<DocumentColor> {
    let sema = Semantics::new(db);
    let file_id = sema
        .attach_first_edition(file_id)
        .unwrap_or_else(|| EditionedFileId::current_edition(db, file_id));

    let file = sema.parse(file_id);
    let file = file.syntax();

    for event in file.preorder() {
        let syntax::WalkEvent::Leave(syntax_node) = event else { continue };
        let Some(expr) = syntax::ast::Expr::cast(syntax_node) else { continue };
        dbg!(&expr);

        // `type_of_expr` fails... but why?
        let Some(ty) = sema.type_of_expr(&expr) else { continue };
        dbg!(ty);
    }

    vec![DocumentColor {
        start_line: 1,
        start_char: 1,
        end_char: 1,
        end_line: 1,
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    }]
}

// ^^^^^ INFO: `sema.scope(file)` is NONE
// let Some(scope) = sema.scope(file) else {
//     return None;
// };
