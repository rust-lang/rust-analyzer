/// Machinery for macro expansion.
///
/// One of the more complicated things about macros is managing the source code
/// that is produced after expansion. See `HirFileId` and `MacroCallId` for how
/// do we do that.
///
/// When the file-management question is resolved, all that is left is a
/// token-tree-to-token-tree transformation plus hygiene. We don't have either of
/// those yet, so all macros are string based at the moment!
use std::sync::Arc;

use ra_syntax::{
    TextRange, TextUnit, SourceFile, AstNode, SyntaxNode, TreeArc, SyntaxNodePtr,
    ast::{self, NameOwner, ModuleItemOwner},
};
use rustc_hash::FxHashMap;
use mbe::{MacroRules, RangesMap};

use crate::{PersistentHirDatabase, MacroCallId, Name, AsName, Module, ModuleSource};

// Hard-coded defs for now :-(
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MacroDef {
    Vec,
    QueryGroup,
    MacroRules(Arc<MacroRules>),
}

impl MacroDef {
    /// Expands macro call, returning the expansion and offset to be used to
    /// convert ranges between expansion and original source.
    pub fn ast_expand(macro_call: &ast::MacroCall) -> Option<(TextUnit, MacroExpansion)> {
        let (def, input) = MacroDef::from_call(macro_call)?;
        let exp = def.expand(input)?;
        let off = macro_call.token_tree()?.syntax().range().start();
        Some((off, exp))
    }

    pub(crate) fn from_macro_rules(macro_call: &ast::MacroCall) -> Option<(Name, MacroDef)> {
        if macro_name(macro_call)?.text() != "macro_rules" {
            return None;
        }
        let name = macro_call.name()?.as_name();
        let (tt, _) = token_tree(macro_call)?;
        let rules = MacroRules::parse(&tt)?;
        Some((name, MacroDef::MacroRules(Arc::new(rules))))
    }

    fn from_call(macro_call: &ast::MacroCall) -> Option<(MacroDef, MacroInput)> {
        let def = match macro_name(macro_call)?.text().as_str() {
            "vec" => MacroDef::Vec,
            "query_group" => MacroDef::QueryGroup,
            _ => return None,
        };

        let input = MacroInput::from_ast(macro_call)?;
        Some((def, input))
    }

    fn expand(&self, input: MacroInput) -> Option<MacroExpansion> {
        match self {
            MacroDef::Vec => self.expand_vec(input),
            MacroDef::QueryGroup => self.expand_query_group(input),
            MacroDef::MacroRules(rules) => {
                if let Some((tt, token_map)) = &input.tt {
                    if let Some(tt) = rules.expand(tt) {
                        let (file, ranges_map) = mbe::parse_token_tree(&tt, &token_map);
                        let ptr = SyntaxNodePtr::new(file.syntax());
                        return Some(MacroExpansion {
                            text: file.syntax().text().to_string(),
                            ranges_map,
                            ptr,
                        });
                    }
                }
                return None;
            }
        }
    }
    fn expand_vec(&self, input: MacroInput) -> Option<MacroExpansion> {
        let text = format!(r"fn dummy() {{ {}; }}", input.text);
        let file = SourceFile::parse(&text);
        let array_expr = file.syntax().descendants().find_map(ast::ArrayExpr::cast)?;
        let ptr = SyntaxNodePtr::new(array_expr.syntax());
        let src_range = TextRange::offset_len(0.into(), TextUnit::of_str(&input.text));
        let ranges_map = vec![(src_range, array_expr.syntax().range())].into();
        let res = MacroExpansion {
            text,
            ranges_map,
            ptr,
        };
        Some(res)
    }
    fn expand_query_group(&self, input: MacroInput) -> Option<MacroExpansion> {
        let anchor = "trait ";
        let pos = input.text.find(anchor)? + anchor.len();
        let trait_name = input.text[pos..]
            .chars()
            .take_while(|c| c.is_alphabetic())
            .collect::<String>();
        if trait_name.is_empty() {
            return None;
        }
        let src_range = TextRange::offset_len((pos as u32).into(), TextUnit::of_str(&trait_name));
        let text = format!(r"trait {} {{ }}", trait_name);
        let file = SourceFile::parse(&text);
        let trait_def = file.syntax().descendants().find_map(ast::TraitDef::cast)?;
        let name = trait_def.name()?;
        let ptr = SyntaxNodePtr::new(trait_def.syntax());
        let ranges_map = vec![(src_range, name.syntax().range())].into();
        let res = MacroExpansion {
            text,
            ranges_map,
            ptr,
        };
        Some(res)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct MacroInput {
    // FIXME: remove text
    text: String,
    tt: Option<(tt::Subtree, mbe::TokenMap)>,
}

impl MacroInput {
    pub(crate) fn from_ast(macro_call: &ast::MacroCall) -> Option<MacroInput> {
        let arg = macro_call.token_tree()?;
        let res = MacroInput {
            text: arg.syntax().text().to_string(),
            tt: token_tree(macro_call),
        };
        Some(res)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MacroExpansion {
    // FIXME: should be tt::Subtree
    text: String,
    /// Correspondence between ranges in the original source code and ranges in
    /// the macro.
    ranges_map: RangesMap,
    /// Implementation detail: internally, a macro is expanded to the whole file,
    /// even if it is an expression. This `ptr` selects the actual expansion from
    /// the expanded file.
    ptr: SyntaxNodePtr,
}

impl MacroExpansion {
    // FIXME: does not really make sense, macro expansion is not neccessary a
    // whole file. See `MacroExpansion::ptr` as well.
    pub(crate) fn file(&self) -> TreeArc<SourceFile> {
        SourceFile::parse(&self.text)
    }

    pub fn syntax(&self) -> TreeArc<SyntaxNode> {
        self.ptr.to_node(&self.file()).to_owned()
    }
    /// Maps range in the source code to the range in the expanded code.
    pub fn map_range_forward(&self, src_range: TextRange) -> Option<TextRange> {
        self.ranges_map.map_forward(src_range)
    }
    /// Maps range in the expanded code to the range in the source code.
    pub fn map_range_back(&self, tgt_range: TextRange) -> Option<TextRange> {
        self.ranges_map.map_back(tgt_range)
    }
}

pub(crate) fn expand_macro_invocation(
    db: &impl PersistentHirDatabase,
    invoc: MacroCallId,
) -> Option<Arc<MacroExpansion>> {
    let loc = invoc.loc(db);
    let syntax = db.file_item(loc.source_item_id);
    let macro_call = ast::MacroCall::cast(&syntax).unwrap();
    if let Some((def, input)) = MacroDef::from_call(macro_call) {
        return def.expand(input).map(Arc::new);
    }
    let defs = macro_definitions(db, loc.module);
    let def = defs.get(&macro_name(macro_call)?.as_name())?;
    let input = MacroInput::from_ast(macro_call)?;
    def.expand(input).map(Arc::new)
}

fn token_tree(call: &ast::MacroCall) -> Option<(tt::Subtree, mbe::TokenMap)> {
    call.token_tree().and_then(mbe::ast_to_token_tree)
}

fn macro_name(macro_call: &ast::MacroCall) -> Option<&ast::NameRef> {
    let path = macro_call.path()?;
    let name_ref = path.segment()?.name_ref()?;
    Some(name_ref)
}

pub(crate) fn macro_definitions(
    db: &impl PersistentHirDatabase,
    module: Module,
) -> Arc<FxHashMap<Name, MacroDef>> {
    let (_file_id, source) = module.definition_source(db);
    let mut res = FxHashMap::default();
    match source {
        ModuleSource::SourceFile(it) => fill(&mut res, &mut it.items_with_macros()),
        ModuleSource::Module(it) => {
            if let Some(item_list) = it.item_list() {
                fill(&mut res, &mut item_list.items_with_macros())
            }
        }
    };
    return Arc::new(res);

    fn fill(acc: &mut FxHashMap<Name, MacroDef>, items: &mut Iterator<Item = ast::ItemOrMacro>) {
        for item in items {
            match item {
                ast::ItemOrMacro::Item(_) => continue,
                ast::ItemOrMacro::Macro(macro_call) => {
                    if let Some((name, def)) = MacroDef::from_macro_rules(macro_call) {
                        acc.insert(name, def);
                    }
                }
            }
        }
    }
}
