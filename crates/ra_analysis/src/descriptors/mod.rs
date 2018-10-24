pub(crate) mod module;

use ra_editor::resolve_local_name;

use ra_syntax::{
    ast::{self, AstNode, NameOwner},
    text_utils::is_subrange,
    TextRange
};

#[derive(Debug, Clone)]
pub struct FnDescriptor {
    pub name: String,
    pub label: String,
    pub ret_type: Option<String>,
    pub params: Vec<String>,
}

impl FnDescriptor {
    pub fn new(node: ast::FnDef) -> Option<Self> {
        let name = node.name()?.text().to_string();

        // Strip the body out for the label.
        let label: String = if let Some(body) = node.body() {
            let body_range = body.syntax().range();
            let label: String = node
                .syntax()
                .children()
                .filter(|child| !is_subrange(body_range, child.range()))
                .map(|node| node.text().to_string())
                .collect();
            label
        } else {
            node.syntax().text().to_string()
        };

        let params = FnDescriptor::param_list(node);
        let ret_type = node.ret_type().map(|r| r.syntax().text().to_string());

        Some(FnDescriptor {
            name,
            ret_type,
            params,
            label,
        })
    }

    fn param_list(node: ast::FnDef) -> Vec<String> {
        let mut res = vec![];
        if let Some(param_list) = node.param_list() {
            if let Some(self_param) = param_list.self_param() {
                res.push(self_param.syntax().text().to_string())
            }

            // Maybe use param.pat here? See if we can just extract the name?
            //res.extend(param_list.params().map(|p| p.syntax().text().to_string()));
            res.extend(
                param_list
                    .params()
                    .filter_map(|p| p.pat())
                    .map(|pat| pat.syntax().text().to_string()),
            );
        }
        res
    }
}

#[derive(Debug)]
pub struct ReferenceDescriptor {
    pub range: TextRange,
    pub name: String
}

#[derive(Debug)]
pub struct DeclarationDescriptor<'a> {
    pat: ast::BindPat<'a>,
    pub range: TextRange
}

impl<'a> DeclarationDescriptor<'a> {
    pub fn new(pat: ast::BindPat) -> DeclarationDescriptor {
        let range = pat.syntax().range();

        DeclarationDescriptor {
            pat,
            range
        }
    }

    pub fn find_all_refs(&self) -> Vec<ReferenceDescriptor> {
        let name = match self.pat.name() {
            Some(name) => name,
            None => return Default::default()
        };

        let fn_def = match name.syntax().ancestors().find_map(ast::FnDef::cast) {
            Some(def) => def,
            None => return Default::default()
        };

        let refs : Vec<_> = fn_def.syntax().descendants()
            .filter_map(ast::NameRef::cast)
            .filter(|name_ref| resolve_local_name(*name_ref) == Some((name.text(), name.syntax().range())))
            .map(|name_ref| ReferenceDescriptor {
                name: name_ref.syntax().text().to_string(),
                range : name_ref.syntax().range(),
            })
            .collect();

        refs
    }
}
