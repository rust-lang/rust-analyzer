//! Builtin macro
use crate::db::AstDatabase;
use crate::quote;
use crate::{
    ast::{self, AstNode},
    eager::EagerResult,
    name,
    util::unquote_str,
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

/// For all eager expansion, we handled all from outside of query database
/// So we panic here to indiciate it is a hard error.
fn eager_expand_error(
    _db: &dyn AstDatabase,
    _id: MacroCallId,
    _tt: &tt::Subtree,
) -> Result<tt::Subtree, mbe::ExpandError> {
    panic!("Eager expansion should handle outside db query");
}

fn concat_expand(tt: &tt::Subtree) -> Result<EagerResult, mbe::ExpandError> {
    let mut text = String::new();
    // FIXME: we should parse it using ra_parser::parse_expr
    for (i, t) in tt.token_trees.iter().enumerate() {
        match t {
            tt::TokenTree::Leaf(tt::Leaf::Literal(it)) if i % 2 == 0 => {
                text += &unquote_str(&it.to_string())
                    .ok_or_else(|| mbe::ExpandError::ConversionError)?;
            }
            tt::TokenTree::Leaf(tt::Leaf::Punct(punct)) if i % 2 == 1 && punct.char == ',' => (),
            _ => return Err(mbe::ExpandError::UnexpectedToken),
        }
    }

    let res = quote!(#text);
    Ok(EagerResult::Syntax(
        mbe::token_tree_to_syntax_node(&res, FragmentKind::Expr)?.0.syntax_node(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{eager, test_db::TestDB, MacroCallLoc};
    use ra_db::{fixture::WithFixture, SourceDatabase};

    fn expand_builtin_macro(s: &str, expander: BuiltinExpander) -> String {
        let (db, file_id) = TestDB::with_single_file(&s);
        let parsed = db.parse(file_id);
        let macro_calls: Vec<_> =
            parsed.syntax_node().descendants().filter_map(|it| ast::MacroCall::cast(it)).collect();

        let ast_id_map = db.ast_id_map(file_id.into());

        // the first one should be a macro_rules
        let def = MacroDefId {
            krate: CrateId(0),
            ast_id: AstId::new(file_id.into(), ast_id_map.ast_id(&macro_calls[0])),
            kind: MacroDefKind::BuiltIn(expander),
        };

        if def.is_eager_expansion() {
            let res = eager::expand_eager_macro(macro_calls[1].clone(), def, &|_| None, &|_| None)
                .unwrap();

            match res {
                EagerResult::Syntax(syn) => syn.text().to_string(),
                EagerResult::IncludeFile(file_name) => file_name,
                EagerResult::IncludeString(_, file_name) => file_name,
                EagerResult::IncludeBytes(_, file_name) => file_name,
            }
        } else {
            let loc = MacroCallLoc {
                def,
                ast_id: AstId::new(file_id.into(), ast_id_map.ast_id(&macro_calls[1])),
            };

            let id = db.intern_macro(loc);
            let parsed = db.parse_or_expand(id.as_file(MacroFileKind::Expr)).unwrap();

            parsed.text().to_string()
        }
    }

    #[test]
    fn test_line_expand() {
        let expanded = expand_builtin_macro(
            r#"
        #[rustc_builtin_macro]
        macro_rules! line {() => {}}
        line!()
"#,
            BuiltinExpander::Line,
        );

        assert_eq!(expanded, "4");
    }

    #[test]
    fn test_concat_expand() {
        let expanded = expand_builtin_macro(
            r#"
        #[rustc_builtin_macro]
        macro_rules! concat {() => {}}
        concat!("abc", "def")
"#,
            BuiltinExpander::Concat,
        );

        assert_eq!(expanded, "\"abcdef\"");
    }
}
