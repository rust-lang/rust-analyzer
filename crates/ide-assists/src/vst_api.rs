use crate::AssistContext;
use hir::Semantics;
use syntax::{
    ast::{self, vst, HasModuleItem},
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

    pub(crate) fn vst_find_fn(&self, call: vst::CallExpr) -> Option<vst::Fn> {
        for item in self.source_file.items() {
            let v_item: ast::generated::vst_nodes::Item = item.try_into().unwrap();
            match v_item {
                ast::generated::vst_nodes::Item::Fn(f) => {
                    if call.expr.to_string().trim() == f.name.to_string().trim() {
                        return Some(*f);
                    }
                },
                _ => {},
            }
        }
        return None;
    }

    /// inline function call
    /// for now, assume one file only
    pub fn vst_inline_call(&self, name_ref: vst::NameRef, expr_to_inline: vst::Expr) -> Option<vst::Expr> {
        use crate::handlers::inline_call::*;
        let name_ref: ast::NameRef = name_ref.cst?;
        let call_info = CallInfo::from_name_ref(name_ref.clone())?;
        let (function, label) = match &call_info.node {
            ast::CallableExpr::Call(call) => {
                let path = match call.expr()? {
                    ast::Expr::PathExpr(path) => path.path(),
                    _ => None,
                }?;
                let function = match self.sema.resolve_path(&path)? {
                    hir::PathResolution::Def(hir::ModuleDef::Function(f)) => f,
                    _ => return None,
                };
                (function, format!("Inline `{path}`"))
            }
            ast::CallableExpr::MethodCall(call) => {
                (self.sema.resolve_method_call(call)?, format!("Inline `{name_ref}`"))
            }
        };
    
        let fn_source: hir::InFile<ast::Fn> = self.sema.source(function)?;
        
        // let fn_body = fn_source.value.body()?;
        let fn_body = expr_to_inline.cst()?;
        let fn_body = ast::make::tail_only_block_expr(fn_body);

        // construct a function (this is a hack -- should properly register req/ens in semantics)
        let temp_fn = fn_source.value.clone_for_update();
        syntax::ted::replace(temp_fn.body()?.syntax(), fn_body.syntax().clone_for_update());

        // for the above function, construct a temporaty semantic database
        let mut temp_fn_str = temp_fn.to_string();
        temp_fn_str.insert_str(0, "$0");
        let (mut db, file_with_caret_id, range_or_offset) =
            <ide_db::RootDatabase as ide_db::base_db::fixture::WithFixture>::with_range_or_offset(&temp_fn_str);
        db.enable_proc_attr_macros();
        let frange = ide_db::base_db::FileRange { file_id: file_with_caret_id, range: range_or_offset.into() };
        let sema: Semantics<'_, ide_db::RootDatabase> = Semantics::new(&db);
        let config = crate::tests::TEST_CONFIG;
        let tmp_ctx = AssistContext::new(sema, &config, frange);
        let tmp_foo = tmp_ctx.find_node_at_offset::<ast::Fn>()?;
        let tmp_body = tmp_foo.body()?;
        let tmp_param_list = tmp_foo.param_list()?;
        let tmp_function = tmp_ctx.sema.to_def(&tmp_foo)?;
        let tmp_params = get_fn_params(tmp_ctx.db(), tmp_function, &tmp_param_list)?;
        let replacement = inline(
            &tmp_ctx.sema,
            file_with_caret_id,
            tmp_function,
            &tmp_body,
            &tmp_params,
            &call_info,
        );
        let vst_replacement = vst::Expr::try_from(replacement).ok()?;
        return Some(vst_replacement);
    }

       
    
}
