//! Various helper functions related to type resolution to work with TOST Nodes (VST Nodes)
//! 
//! Utilizes Rust-analyzer's type `hir` implementation
//! 
#![allow(dead_code)]
use crate::AssistContext;
use hir::Semantics;
use syntax::ast::{self, vst, HasModuleItem};

impl<'a> AssistContext<'a> {
    /// From an VST Expr, get the definition VST Adt of that type
    pub fn type_of_expr_adt(&self, expr: &vst::Expr) -> Option<vst::Adt> {
        let sema: &Semantics<'_, ide_db::RootDatabase> = &self.sema;
        let expr = expr.cst()?;
        dbg!("call type_of_expr");
        let hir_ty: Vec<hir::Type> =
            sema.type_of_expr(&expr)?.adjusted().autoderef(sema.db).collect::<Vec<_>>();
            
        dbg!("end type_of_expr");
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

    /// From a pattern, get the Enum definition of that pat.
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

    /// Get the Struct definition of an expression
    pub fn type_of_expr_struct(&self, expr: &vst::Expr) -> Option<vst::Struct> {
        let typename = self.type_of_expr_adt(expr)?;
        if let vst::Adt::Struct(e) = typename {
            return Some(*e.clone());
        }
        None
    }

    /// Get the struct deinition of a pat
    pub fn type_of_pat_struct(&self, pat: &vst::Pat) -> Option<vst::Struct> {
        let sema: &Semantics<'_, ide_db::RootDatabase> = &self.sema;
        let hir_ty: Vec<hir::Type> =
            sema.type_of_pat(&pat.cst()?)?.adjusted().autoderef(sema.db).collect::<Vec<_>>();
        let hir_ty = hir_ty.first()?;
        if let Some(t) = hir_ty.as_adt() {
            let ast_ty: ast::Adt = sema.source(t)?.value;
            let typename = ast_ty.try_into().ok()?;
            if let vst::Adt::Struct(s) = typename {
                return Some(*s.clone());
            }
        }
        None
    }

    /// From a Type usage, get the definition of the enum
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

    /// Get the NameRef at the callsite
    pub fn name_ref_from_call_expr(&self, call: &vst::CallExpr) -> Option<vst::NameRef> {
        let path = match &*call.expr {
            vst::Expr::PathExpr(path) => &path.path,
            _ => return None,
        };
        let name_ref =  &path.segment.name_ref;
        Some(*name_ref.clone())
    }

    /// Get function definition from the callsite
    pub fn vst_find_fn(&self, call: &vst::CallExpr) -> Option<vst::Fn> {
        for item in self.source_file.items() {
            let v_item: ast::generated::vst_nodes::Item = 
                match item.try_into() {
                    Ok(ii) => ii,
                    Err(err_msg) => {
                        dbg!("into_vst failed: {}", err_msg);
                        continue;
                    },
                };
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

    /// Query if this function is opaque (non-visible to the solver)
    pub fn is_opaque(&self, func: &vst::Fn) -> bool {
        for attr in &func.attrs {
            if attr.to_string().contains("opaque") {
                return true;
            }
        }
        return false;
    }

}
