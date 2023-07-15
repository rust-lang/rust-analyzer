use crate::AssistContext;
use hir::Semantics;
use syntax::{
    ast::{self, vst},
    AstNode,
};

impl<'a> AssistContext<'a> {
    /// From an VST Expr, get the definition VST Adt of that type
    pub fn type_of_expr_adt(&self, expr: &vst::Expr) -> Option<vst::Adt> {
        let sema: &Semantics<'_, ide_db::RootDatabase> = &self.sema;
        let expr = expr.cst()?;
        let hir_ty: Vec<hir::Type> =
            sema.type_of_expr(&expr)?.adjusted().autoderef(sema.db).collect::<Vec<_>>();
        let hir_ty = hir_ty.first()?;
        if let Some(t) = hir_ty.as_adt() {
            let ast_ty: ast::Adt = sema.source(t)?.value;
            return ast_ty.try_into().ok();
        }
        None
    }

    /// From an VST Expr, get the definition VST enum of that type
    pub fn type_of_expr_enum(&self, expr: &vst::Expr) -> Option<vst::Enum> {
        let typename = self.type_of_expr_adt(expr)?;
        if let vst::Adt::Enum(e) = typename {
            return Some(*e.clone());
        }
        None
    }

    /// Get VST node from the current cursor position
    /// This is a wrapper around `find_node_at_offset` that returns a VST node
    /// REVIEW: to remove type annotation, consider auto-generating all sorts of this function
    pub(crate) fn vst_find_node_at_offset<VSTT, CSTT>(&self) -> Option<VSTT>
    where
        VSTT: TryFrom<CSTT>,
        CSTT: AstNode,
    {
        let cst_node: CSTT = self.find_node_at_offset()?;
        VSTT::try_from(cst_node).ok()
    }
}
