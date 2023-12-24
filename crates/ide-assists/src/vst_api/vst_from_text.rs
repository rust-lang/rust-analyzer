use std::{process::Command, collections::hash_map::DefaultHasher, time::Instant, env::{self, Args}, path::Path, hash::{Hasher, Hash}, fs::File, io::Write};

use crate::{AssistContext, verus_error::*, tests::CHANHEE_VERUS_PATH};
use hir::Semantics;
use syntax::{
    ast::{self, vst, HasModuleItem, HasName},
    AstNode, SyntaxToken, SyntaxKind,
};

impl<'a> AssistContext<'a> {
    pub fn vst_path_from_text(&self, text: &str) -> Option<vst::Path> {
        let path = ast::make::path_from_text(text);
        let vst_path = vst::Path::try_from(path).ok()?;
        return Some(vst_path);
    }

    pub fn vst_call_expr_from_text(&self, fn_name: &str, arglist: vst::ArgList) -> Option<vst::CallExpr> {
        let fn_name_as_path: vst::Path = self.vst_path_from_text(fn_name)?;
        let fn_name_as_pathexpr: vst::PathExpr = vst::PathExpr::new(fn_name_as_path);
        let vst_call_expr = vst::CallExpr::new(fn_name_as_pathexpr, arglist);
        return Some(vst_call_expr);
    }

    pub fn vst_nameref_from_text(&self, s: &str) -> Option<vst::NameRef> {
        let mut name = vst::NameRef::new();
        name.ident_token = Some(String::from(s));
        Some(name)
    }

}
