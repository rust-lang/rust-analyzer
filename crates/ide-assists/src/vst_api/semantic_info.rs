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

    pub fn type_of_pat_enum(&self, pat: &vst::Pat) -> Option<vst::Enum> {
        let sema: &Semantics<'_, ide_db::RootDatabase> = &self.sema;
        let hir_ty: Vec<hir::Type> =
            sema.type_of_pat(&pat.cst()?)?.adjusted().autoderef(sema.db).collect::<Vec<_>>();
        let hir_ty = hir_ty.first()?;
        if let Some(t) = hir_ty.as_adt() {
            let ast_ty: ast::Adt = sema.source(t)?.value;
            let typename = ast_ty.try_into().ok()?;
            if let vst::Adt::Enum(e) = typename {
                return Some(*e.clone());
            }
        }
        None
    }

    pub fn resolve_type_enum(&self, ty: &vst::Type) ->  Option<vst::Enum> {
        let sema: &Semantics<'_, ide_db::RootDatabase> = &self.sema;
        let hir_ty: Vec<hir::Type> =
            sema.resolve_type(&ty.cst()?)?.autoderef(sema.db).collect::<Vec<_>>();
        let hir_ty = hir_ty.first()?;
        dbg!(&hir_ty);
        if let Some(t) = hir_ty.as_adt() {
            let ast_ty: ast::Adt = sema.source(t)?.value;
            let typename = ast_ty.try_into().ok()?;
            if let vst::Adt::Enum(e) = typename {
                return Some(*e.clone());
            }
        } 

        if let Some(t) = hir_ty.as_builtin() {
            dbg!(t);
        }
        None
    }

    pub fn name_ref_from_call_expr(&self, call: &vst::CallExpr) -> Option<vst::NameRef> {
        let path = match &*call.expr {
            vst::Expr::PathExpr(path) => &path.path,
            _ => return None,
        };
        let name_ref =  &path.segment.name_ref;
        Some(*name_ref.clone())
    }

    pub(crate) fn vst_find_fn(&self, call: &vst::CallExpr) -> Option<vst::Fn> {
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

}
