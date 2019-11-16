//! Builtin macro
use crate::db::AstDatabase;
use crate::quote;
use crate::{
    ast::{self, AstNode},
    eager::EagerResult,
    name,
    util::unquote,
    AstId, CrateId, HirFileId, MacroCallId, MacroDefId, MacroDefKind, MacroFileKind, TextUnit,
};
use once_cell::sync::Lazy;
use ra_parser::FragmentKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinExpander {
    Line,
    Concat,
}

struct BuiltInMacroInfo {
    name: name::Name,
    kind: BuiltinExpander,
    expand: fn(
        db: &dyn AstDatabase,
        id: MacroCallId,
        _tt: &tt::Subtree,
    ) -> Result<tt::Subtree, mbe::ExpandError>,
    eager: Option<fn(&tt::Subtree) -> Result<EagerResult, mbe::ExpandError>>,
}

impl BuiltInMacroInfo {
    fn new(
        name: name::Name,
        kind: BuiltinExpander,
        expand: fn(
            db: &dyn AstDatabase,
            id: MacroCallId,
            _tt: &tt::Subtree,
        ) -> Result<tt::Subtree, mbe::ExpandError>,
        eager: Option<fn(&tt::Subtree) -> Result<EagerResult, mbe::ExpandError>>,
    ) -> Self {
        BuiltInMacroInfo { name, kind, expand, eager }
    }
}

const BUILTIN_MACROS: Lazy<Vec<BuiltInMacroInfo>> = Lazy::new(|| {
    vec![
        BuiltInMacroInfo::new(name::LINE_MACRO, BuiltinExpander::Line, line_expand, None),
        BuiltInMacroInfo::new(
            name::CONCAT_MACRO,
            BuiltinExpander::Concat,
            eager_expand_error,
            Some(concat_expand),
        ),
    ]
});

impl BuiltinExpander {
    pub fn expand(
        &self,
        db: &dyn AstDatabase,
        id: MacroCallId,
        tt: &tt::Subtree,
    ) -> Result<tt::Subtree, mbe::ExpandError> {
        BUILTIN_MACROS
            .iter()
            .find(|info| info.kind == *self)
            .map(|info| (info.expand)(db, id, tt))
            .unwrap()
    }

    pub fn eager(&self) -> Option<fn(&tt::Subtree) -> Result<EagerResult, mbe::ExpandError>> {
        BUILTIN_MACROS.iter().find(|info| info.kind == *self).and_then(|info| info.eager)
    }
}

pub fn find_builtin_macro(
    ident: &name::Name,
    krate: CrateId,
    ast_id: AstId<ast::MacroCall>,
) -> Option<MacroDefId> {
    let expander =
        BUILTIN_MACROS.iter().find_map(
            |info| {
                if *ident == info.name {
                    Some(info.kind)
                } else {
                    None
                }
            },
        )?;

    Some(MacroDefId { krate, ast_id, kind: MacroDefKind::BuiltIn(expander) })
}

fn to_line_number(db: &dyn AstDatabase, file: HirFileId, pos: TextUnit) -> usize {
    // FIXME: Use expansion info
    let file_id = file.original_file(db);
    let text = db.file_text(file_id);
    let mut line_num = 1;

    // Count line end
    for (i, c) in text.chars().enumerate() {
        if i == pos.to_usize() {
            break;
        }
        if c == '\n' {
            line_num += 1;
        }
    }

    line_num
}

fn line_expand(
    db: &dyn AstDatabase,
    id: MacroCallId,
    _tt: &tt::Subtree,
) -> Result<tt::Subtree, mbe::ExpandError> {
    let loc = db.lookup_intern_macro(id);
    let macro_call = loc.ast_id.to_node(db);

    let arg = macro_call.token_tree().ok_or_else(|| mbe::ExpandError::UnexpectedToken)?;
    let arg_start = arg.syntax().text_range().start();

    let file = id.as_file(MacroFileKind::Expr);
    let line_num = to_line_number(db, file, arg_start);

    let expanded = quote! {
        #line_num
    };

    Ok(expanded)
}
