//! It can be cumbersom to generate a new TOST node using `new`
//! Below are some helper functions in this case.
//! We can generate commonly used TOST nodes from text using the following APIs.
//!

use crate::AssistContext;
use syntax::ast::{self, vst};

impl<'a> AssistContext<'a> {
    /// Generate Path from text
    pub fn vst_path_from_text(&self, text: &str) -> Option<vst::Path> {
        let path = ast::make::path_from_text(text);
        let vst_path = vst::Path::try_from(path).ok()?;
        return Some(vst_path);
    }

    /// Generate CallExpr from Text and ArgList
    pub fn vst_call_expr_from_text(
        &self,
        fn_name: &str,
        arglist: vst::ArgList,
    ) -> Option<vst::CallExpr> {
        let fn_name_as_path: vst::Path = self.vst_path_from_text(fn_name)?;
        let fn_name_as_pathexpr: vst::PathExpr = vst::PathExpr::new(fn_name_as_path);
        let vst_call_expr = vst::CallExpr::new(fn_name_as_pathexpr, arglist);
        return Some(vst_call_expr);
    }

    /// Generate NameRef from text
    pub fn vst_nameref_from_text(&self, s: &str) -> Option<vst::NameRef> {
        let mut name = vst::NameRef::new();
        name.ident_token = Some(String::from(s));
        Some(name)
    }

    /// Generate Expr from text
    pub fn vst_expr_from_text(&self, s: &str) -> Option<vst::Expr> {
        let ret: vst::Expr = vst::Literal::new(s.to_string()).into();
        Some(ret)
    }
}
