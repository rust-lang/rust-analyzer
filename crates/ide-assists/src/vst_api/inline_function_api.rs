
use crate::AssistContext;
use hir::Semantics;
use syntax::{
    ast::{self, vst},
    AstNode,
};

impl<'a> AssistContext<'a> {
    /// inline function call
    /// for now, assume one file only
    /// TODO: handle Verus builtin types -- for example, `type_of_expr` panics for `int`
    /// TODO: properly register req/ens/etc in semantics db
    /// TODO: currently inline can panic when the inlining expr does not fully use all the parameters
    pub fn vst_inline_call(
        &self,
        name_ref: vst::NameRef,     // the name of the function to inline **at the callsite**. from `name_ref`, we get its arguments
        expr_to_inline: vst::Expr,  // the expression to inline --- this expression will replace the function body
    ) -> Option<vst::Expr> {
        use crate::handlers::inline_call::*;
        dbg!("vst_inline_call");
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
        // We will use 'inline_call', making the requires clause as the function body
        let fn_body = expr_to_inline.cst()?;
        let fn_body = ast::make::tail_only_block_expr(fn_body);

        // construct a function (this is a hack -- should properly register req/ens in semantics)
        let temp_fn = fn_source.value.clone_for_update();
        syntax::ted::replace(temp_fn.body()?.syntax(), fn_body.syntax().clone_for_update());

        // for the above function, construct a temporaty semantic database
        let mut temp_fn_str = temp_fn.to_string();
        temp_fn_str.insert_str(0, "$0");
        let (mut db, file_with_caret_id, range_or_offset) =
            <ide_db::RootDatabase as ide_db::base_db::fixture::WithFixture>::with_range_or_offset(
                &temp_fn_str,
            );
        db.enable_proc_attr_macros();
        let frange = ide_db::base_db::FileRange {
            file_id: file_with_caret_id,
            range: range_or_offset.into(),
        };
        let sema: Semantics<'_, ide_db::RootDatabase> = Semantics::new(&db);
        let config = crate::tests::TEST_CONFIG;
        let tmp_ctx = AssistContext::new(sema, &config, frange, vec![]);
        let tmp_foo = tmp_ctx.find_node_at_offset::<ast::Fn>()?;
        let tmp_body = tmp_foo.body()?;
        let tmp_param_list = tmp_foo.param_list()?;
        let tmp_function = tmp_ctx.sema.to_def(&tmp_foo)?;
        let tmp_params = get_fn_params(tmp_ctx.db(), tmp_function, &tmp_param_list)?;
        let replacement = inline_simple(
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
