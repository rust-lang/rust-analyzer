//! Eager expansion related utils

use crate::{
    ast::{self, AstNode},
    MacroDefId, MacroDefIdWithAst, MacroDefKind,
};

use ra_parser::FragmentKind;
use ra_syntax::SyntaxNode;
use ra_text_edit::AtomTextEdit;

fn to_subtree(text: &str) -> Option<tt::Subtree> {
    // FIXME: It definitely is a hack
    let source_file = ast::SourceFile::parse(&format!("m!({});", text));
    let tt = source_file.syntax_node().descendants().find_map(ast::TokenTree::cast)?;

    let mut subtree = mbe::ast_to_token_tree(&tt)?.0;
    if subtree.token_trees.len() == 1 {
        if let tt::TokenTree::Subtree(child) = &subtree.token_trees[0] {
            return Some(child.clone());
        }
    }
    subtree.delimiter = tt::Delimiter::None;
    Some(subtree)
}

struct EagerMacro {
    pub def: MacroDefId,
    pub macro_call: ast::MacroCall,
}

fn usual_expand(def: &MacroDefIdWithAst, macro_call: ast::MacroCall) -> Option<SyntaxNode> {
    let tt = macro_call.token_tree()?;
    let macro_arg = mbe::ast_to_token_tree(&tt)?.0;

    let tt = def.ast.token_tree()?;
    let def_subtree = mbe::ast_to_token_tree(&tt)?.0;
    let rules = mbe::MacroRules::parse(&def_subtree).ok()?;

    let tt = rules.expand(&macro_arg).ok()?;
    // Set a hard limit for the expanded tt
    let count = tt.count();
    if count > 65536 {
        return None;
    }

    Some(mbe::token_tree_to_syntax_node(&tt, FragmentKind::Expr).ok()?.0.syntax_node())
}

fn eager_macro_recur(
    curr: SyntaxNode,
    macro_resolver: &dyn Fn(ast::Path) -> Option<MacroDefIdWithAst>,
    file_resolver: &dyn Fn(&str) -> Option<String>,
) -> Option<String> {
    let mut args_text = curr.text().to_string();

    // Get expanded arguments macro ast
    // We need `Vec` here for `rev`
    let children: Vec<_> = curr.descendants().filter_map(ast::MacroCall::cast).collect();

    // Apply replacement reversely to prevent text edit position shifted
    for child in children.into_iter().rev() {
        let def = macro_resolver(child.path()?)?;
        let insert = if def.id.is_eager_expansion() {
            match expand_eager_macro(child.clone(), def.id, macro_resolver, file_resolver)? {
                EagerResult::Syntax(syn) => syn.text().to_string(),
                EagerResult::IncludeFile(file) => file_resolver(&file)?,
                EagerResult::IncludeString(syn, _) => syn.text().to_string(),
                EagerResult::IncludeBytes(syn, _) => syn.text().to_string(),
            }
        } else {
            let expanded = usual_expand(&def, child.clone())?;
            // replace macro inside
            eager_macro_recur(expanded, macro_resolver, file_resolver)?
        };

        let edit = AtomTextEdit { delete: child.syntax().text_range(), insert };
        args_text = edit.apply(args_text);
    }

    Some(args_text)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EagerResult {
    Syntax(SyntaxNode),
    IncludeFile(String),
    IncludeString(SyntaxNode, String),
    IncludeBytes(SyntaxNode, String),
}

pub fn expand_eager_macro(
    macro_call: ast::MacroCall,
    def: MacroDefId,
    macro_resolver: &dyn Fn(ast::Path) -> Option<MacroDefIdWithAst>,
    file_resolver: &dyn Fn(&str) -> Option<String>,
) -> Option<EagerResult> {
    let curr = EagerMacro { macro_call, def };

    let args = curr.macro_call.token_tree()?;
    let parsed_args = mbe::ast_to_token_tree(&args)?.0;
    let parsed_args = mbe::token_tree_to_syntax_node(&parsed_args, FragmentKind::Expr).ok()?.0;
    let result = eager_macro_recur(parsed_args.syntax_node(), macro_resolver, file_resolver)?;

    // remove args parenthesis
    let result = result[1..result.len() - 1].to_string();
    let subtree = to_subtree(&result)?;

    if let MacroDefKind::BuiltIn(builtin) = def.kind {
        let eager_expand = builtin.eager()?;
        eager_expand(&subtree).ok()
    } else {
        None
    }
}
