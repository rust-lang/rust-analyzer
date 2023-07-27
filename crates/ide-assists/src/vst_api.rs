use std::{process::Command, collections::hash_map::DefaultHasher, time::Instant, env, path::Path, hash::{Hasher, Hash}, fs::File, io::Write};

use crate::{AssistContext, verus_error::*, tests::CHANHEE_VERUS_PATH};
use hir::Semantics;
use syntax::{
    ast::{self, vst, HasModuleItem, HasName},
    AstNode, SyntaxToken, SyntaxKind,
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
                }
                _ => {}
            }
        }
        return None;
    }

    /// inline function call
    /// for now, assume one file only
    pub fn vst_inline_call(
        &self,
        name_ref: vst::NameRef,
        expr_to_inline: vst::Expr,
    ) -> Option<vst::Expr> {
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

    pub fn vst_path_from_text(&self, text: &str) -> Option<vst::Path> {
        let path = ast::make::path_from_text(text);
        let vst_path = vst::Path::try_from(path).ok()?;
        return Some(vst_path);
    }

    pub(crate) fn verus_errors(&self) -> Vec<VerusError> {
        self.verus_errors.clone()
    }
    pub(crate) fn pre_failures(&self) -> Vec<PreFailure> {
        filter_pre_failuires(&self.verus_errors)
    }
    pub(crate) fn post_failures(&self) -> Vec<PostFailure> {
        filter_post_failuires(&self.verus_errors)
    }
    pub(crate) fn expr_from_pre_failure(&self, pre: PreFailure) -> Option<vst::Expr> {
        self.find_node_at_given_range::<syntax::ast::Expr>(pre.failing_pre)?.try_into().ok()
    }
    pub(crate) fn expr_from_post_failure(&self, post: PostFailure) -> Option<vst::Expr> {
        self.find_node_at_given_range::<syntax::ast::Expr>(post.failing_post)?.try_into().ok()
    }


    // for now, assume one file only
    // 1) copy the file to a temporary file
    // 2) replace out the function with this VST Fn 
    // 3) run verus on the temporary file
    // run Verus on the `vst::Fn` node
    // assume running verus inside vs-code
    // TODO: pass the whole project to verus, instead of this single file
    // TODO: projects with multiple file/module -- `verify-module` flag --verify-function flag
    // output: None -> compile error
    pub(crate) fn try_verus(
        &self,
        vst_fn: &vst::Fn,
    ) -> Option<bool> {
        let source_file = &self.source_file;
        let verus_exec_path = &self.config.verus_path;

        if verus_exec_path.len() == 0 {
            dbg!("verus path not set");
        }

        #[cfg(test)] 
        let verus_exec_path = CHANHEE_VERUS_PATH.to_string(); // TODO: maybe move this to test config


        if verus_exec_path.len() == 0 {
            dbg!("verus path not set");
        }
        let mut text_string  = String::new();

        // in VST, we should also be able to "print" and verify
        // display for VST should be correct modulo whitespace
        for it in source_file.items() {
            match it {
                ast::Item::Fn(f) => {
                    text_string += "\nverus!{\n";
                    if f.name()?.to_string().trim() == vst_fn.name.to_string().trim() {
                        text_string += &vst_fn.to_string();
                    } else {
                        // review: f.cst.to_string?
                        text_string += &f.to_string();
                    }
                    text_string += "\n}\n";
                },
                _ => {
                    // review: it.cst.to_string?
                    text_string += &it.to_string();
                },
            }
        }
        dbg!(&text_string);

        
        // let verify_func_flag = "--verify-function";
        // let verify_root_flag = "--verify-root"; // TODO: figure out the surrounding module of `token`
        // let func_name = vst_fn.name.to_string();



        // REIVEW: instead of writing to a file in the tmp directory, consider using `memfd_create` for an anonymous file
        // refer to `man memfd_create` or `dev/shm`
        let mut hasher = DefaultHasher::new();
        let now = Instant::now();
        now.hash(&mut hasher);
        // in linux, set env TMPDIR to set the tmp directory. Otherwise, it fails
        let tmp_dir = env::temp_dir();
        let tmp_name = format!("{}/_verus_assert_comment_{:?}_.rs", tmp_dir.display(), hasher.finish());
        dbg!(&tmp_name);
        let path = Path::new(&tmp_name);
        let display = path.display();

        // Open a file in write-only mode, returns `io::Result<File>`
        let mut file = match File::create(&path) {
            Err(why) => {
                dbg!("couldn't create {}: {}", display, why);
                return None;
            }
            Ok(file) => file,
        };

        // Write the modified verus program to `file`, returns `io::Result<()>`
        match file.write_all(text_string.as_bytes()) {
            Err(why) => {
                dbg!("couldn't write to {}: {}", display, why);
                return None;
            }
            Ok(_) => dbg!("successfully wrote to {}", display),
        };

        let output = 
            Command::new(verus_exec_path)
                .arg(path)
                .output();

        // match std::fs::remove_file(path) {
        //     Err(why) => {
        //         dbg!("couldn't remove file {}: {}", path.display(), why);
        //     }
        //     Ok(_) => {
        //         dbg!("successfully removed {}", path.display());
        //     }
        // };

        let output = output.ok()?;
        dbg!(&output);
        if output.status.success() {
            return Some(true);
        } else {
            // disambiguate verification failure     VS    compile error etc
            match std::str::from_utf8(&output.stdout) {
                Ok(out) => {
                    if out.contains("verification results:: verified: 0 errors: 0") {
                        // failure from other errors. (e.g. compile error)
                        return None;
                    } else {
                        // verification failure
                        return Some(false);
                    }
                }
                Err(_) => return None,
            }
        }
    }
}
