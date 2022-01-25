//! Find dependencies of a function

use rustc_hash::{FxHashMap, FxHashSet};
use std::iter;

use crate::{Adt, AdtId, Enum, HasSource, Struct, Variant, Trait};
use hir_def::{
    adt::VariantData,
    body::Body,
    expr::{Expr, ExprId, Pat, PatId},
    path::PathKind,
    resolver::{resolver_for_expr, HasResolver, ResolveValueResult, Resolver, ValueNs},
    type_ref::TypeRef,
    DefWithBodyId, FunctionId, ModuleId, Lookup, TraitId,
};
use hir_expand::name::Name;
use hir_ty::{db::HirDatabase, display::HirDisplay, Interner, Ty, TyKind};
use stdx::{format_to, impl_from};

use crate::Function;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Dependency {
    AdtId(AdtId),
    FunctionId(FunctionId),
    TraitId(TraitId),
    ImplOfTrait(Ty, TraitId),
    Ty(Ty),
}

impl_from!(AdtId, Ty, FunctionId, TraitId for Dependency);

fn struct_dependencies<'a>(db: &'a dyn HirDatabase, t: Struct) -> Vec<Dependency> {
    t.fields(db).into_iter().map(|x| x.ty(db).ty.into()).collect()
}

fn enum_dependencies<'a>(db: &'a dyn HirDatabase, t: Enum) -> Vec<Dependency> {
    t.variants(db)
        .into_iter()
        .flat_map(|var| var.fields(db).into_iter().map(|x| x.ty(db).ty.into()))
        .collect()
}

fn type_dependencies(t: Ty) -> Vec<Dependency> {
    match t.kind(Interner) {
        TyKind::Adt(adt_id, args) => args
            .type_parameters(Interner)
            .into_iter()
            .map(|x| x.into())
            .chain(iter::once(adt_id.0.into()))
            .collect(),
        _ => vec![],
    }
}

fn signature_dependencies(db: &dyn HirDatabase, func: Function) -> Vec<Dependency> {
    let params = func.assoc_fn_params(db);
    params.into_iter().map(|x| x.ty.ty.into()).collect()
}

fn expr_dependencies(
    db: &dyn HirDatabase,
    expr_id: ExprId,
    body: &Body,
    owner: DefWithBodyId,
    success: &mut bool,
    result: &mut Vec<Dependency>,
) {
    let expr = &body.exprs[expr_id];
    match expr {
        Expr::Path(path) => {
            let resolver = resolver_for_expr(db.upcast(), owner, expr_id);
            let value_or_partial = resolver.resolve_path_in_value_ns(db.upcast(), path.mod_path());
            if let Some(ResolveValueResult::ValueNs(v)) = value_or_partial {
                match v {
                    ValueNs::FunctionId(f) => result.push(f.into()),
                    ValueNs::EnumVariantId(ev) => {
                        let adt_id: AdtId = ev.parent.into();
                        result.push(adt_id.into());
                    }
                    _ => {}
                }
            }
        }
        Expr::MethodCall { receiver, .. } => {
            let infer = db.infer(owner);
            let (func, _) = match infer.method_resolution(expr_id) {
                Some(x) => x,
                None => todo!(), // FIXME: ???
            };
            match func.lookup(db.upcast()).container {
                hir_def::ItemContainerId::ExternBlockId(_) => todo!(),
                hir_def::ItemContainerId::ModuleId(_) => {
                    // what?
                    result.push(func.into());
                }
                hir_def::ItemContainerId::ImplId(_) => todo!(),
                hir_def::ItemContainerId::TraitId(tr) => {
                    let ty = infer.type_of_expr[*receiver].clone();
                    result.push(Dependency::ImplOfTrait(ty, tr));
                },
            }
        }
        _ => {
            expr.walk_child_exprs(|x| expr_dependencies(db, x, body, owner, success, result));
        }
    }
}

fn function_dependencies(db: &dyn HirDatabase, func: Function) -> Vec<Dependency> {
    let mut result = signature_dependencies(db, func);
    let def = func.id.into();
    let body = db.body(def);
    let mut success = true;
    expr_dependencies(db, body.body_expr, &body, def, &mut success, &mut result);
    result
}

fn recursive_dependencies(db: &dyn HirDatabase, top_level: Vec<Dependency>) -> Vec<Dependency> {
    fn f(
        db: &dyn HirDatabase,
        dep: Dependency,
        set: &mut FxHashSet<Dependency>,
        result: &mut Vec<Dependency>,
    ) {
        if set.contains(&dep) {
            return;
        }
        set.insert(dep.clone());
        let deps = match &dep {
            Dependency::AdtId(x) => match x {
                AdtId::StructId(x) => struct_dependencies(db, (*x).into()),
                AdtId::UnionId(_) => vec![], // FIXME: this is incomplete
                AdtId::EnumId(x) => enum_dependencies(db, (*x).into()),
            },
            Dependency::Ty(x) => type_dependencies(x.clone()),
            Dependency::FunctionId(x) => signature_dependencies(db, (*x).into()),
            Dependency::TraitId(_) => vec![],
            Dependency::ImplOfTrait(ty, tr) => vec![ty.clone().into(), (*tr).into()],
        };
        for d in deps {
            f(db, d, set, result);
        }
        result.push(dep);
    }
    let mut result = vec![];
    let mut set = FxHashSet::<Dependency>::default();
    for dep in top_level {
        f(db, dep, &mut set, &mut result);
    }
    result
}

fn function_recursive_dependencies(db: &dyn HirDatabase, func: Function) -> Vec<Dependency> {
    recursive_dependencies(db, function_dependencies(db, func))
}

fn dependencies_to_string(db: &dyn HirDatabase, deps: Vec<Dependency>) -> Option<String> {
    fn variant_data(
        db: &dyn HirDatabase,
        data: &VariantData,
        module_id: ModuleId,
        r: &mut String,
    ) -> Option<()> {
        match data {
            VariantData::Record(_) => (),
            VariantData::Tuple(x) => {
                *r += "(";
                for (_, field) in x.iter() {
                    let resolver = module_id.resolver(db.upcast());
                    *r += &field
                        .type_ref
                        .display_machine_source_code(db, &mut Some(resolver))
                        .ok()?;
                    *r += ",";
                }
                *r += ")";
            }
            VariantData::Unit => (),
        }
        Some(())
    }
    let mut r = String::new();
    for dep in deps {
        match dep {
            Dependency::AdtId(id) => {
                let x: Adt = id.into();
                let module_id = x.module(db).id;
                r += &x.display_machine_source_code(db, &mut None).ok()?;
                match id {
                    AdtId::StructId(id) => {
                        let data = db.struct_data(id);
                        variant_data(db, &data.variant_data, module_id, &mut r)?;
                        match data.variant_data.as_ref() {
                            VariantData::Record(_) => (),
                            VariantData::Tuple(_) | VariantData::Unit => r += ";",
                        }
                    }
                    AdtId::UnionId(_) => (),
                    AdtId::EnumId(id) => {
                        let data = db.enum_data(id);
                        r += "{";
                        for (_, variant) in data.variants.iter() {
                            r += &variant.name.to_string();
                            variant_data(db, &variant.variant_data, module_id, &mut r)?;
                            r += ",";
                        }
                        r += "}";
                    }
                }
            }
            Dependency::FunctionId(x) => {
                let x: Function = x.into();
                r += &x.display_machine_source_code(db, &mut None).ok()?;
                r += "{loop{}}"
            },
            Dependency::Ty(_) => (), // types don't need code for their own
            Dependency::TraitId(tr) => {
                let data = db.trait_data(tr);
                let tr: Trait = tr.into();
                r += &tr.display_machine_source_code(db, &mut None).ok()?;
                r += "{";
                for (_, item) in &data.items {
                    match item {
                        hir_def::AssocItemId::FunctionId(f) => {
                            let f: Function = (*f).into();
                            r += &f.display_machine_source_code(db, &mut None).ok()?;
                            r += "{loop{}}";           
                        },
                        hir_def::AssocItemId::ConstId(_) => todo!(),
                        hir_def::AssocItemId::TypeAliasId(_) => todo!(),
                    }
                }
                r += "}";
            }
            Dependency::ImplOfTrait(ty, tr) => {
                format_to!(r, "impl {} for {} {{}}", tr.machine_name(), ty.display_machine_source_code(db, &mut None).ok()?);
            }
        }
    }
    Some(r)
}

enum UseTree {
    Mod { children: UseTreeRoot },
    Item { machine_name: String },
}

type UseTreeRoot = FxHashMap<Name, UseTree>;

fn insert_path<'a>(mut root: &'a mut UseTreeRoot, path: &[Name]) -> Option<&'a mut UseTreeRoot> {
    for s in path {
        if !root.contains_key(s) {
            root.insert(s.clone(), UseTree::Mod { children: UseTreeRoot::default() });
        }
        let x = root.get_mut(s)?;
        if let UseTree::Mod { children } = x {
            root = children;
        } else {
            return None; // User code is broken in this case
        }
    }
    Some(root)
}

fn type_ref_use_tree(
    db: &dyn HirDatabase,
    ty: &TypeRef,
    root: &mut UseTreeRoot,
    resolver: &Resolver,
) -> Option<()> {
    let mut success = true;
    ty.walk(&mut |x| {
        if !success {
            return;
        }
        match x {
            TypeRef::Path(p) => {
                let p = p.mod_path();
                let ty = if let Some(x) = resolver.resolve_path_in_type_ns_fully(db.upcast(), p) {
                    x
                } else {
                    success = false;
                    return;
                };
                match ty {
                    hir_def::resolver::TypeNs::AdtId(x) => {
                        insert_item_with_segments(
                            p.segments(),
                            root,
                            x.machine_name(),
                            &mut success,
                        );
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    });
    if success {
        Some(())
    } else {
        None
    }
}

fn expr_use_tree(
    db: &dyn HirDatabase,
    expr_id: ExprId,
    body: &Body,
    owner: DefWithBodyId,
    root: &mut UseTreeRoot,
    success: &mut bool,
) {
    if !*success {
        return;
    }
    let expr = &body.exprs[expr_id];
    let resolver = resolver_for_expr(db.upcast(), owner, expr_id);
    match expr {
        Expr::Match { arms, .. } => {
            for arm in arms.iter() {
                pat_use_tree(arm.pat, body, db, &resolver, success, root);
            }
        }
        Expr::Path(path) => {
            path_use_tree(db, &resolver, path, success, root);
        }
        _ => {}
    }
    expr.walk_child_exprs(|x| expr_use_tree(db, x, body, owner, root, success));
}

fn pat_use_tree(
    pat: PatId,
    body: &Body,
    db: &dyn HirDatabase,
    resolver: &Resolver,
    success: &mut bool,
    root: &mut UseTreeRoot,
) {
    let pat = &body.pats[pat];
    match pat {
        Pat::Path(path)
        | Pat::TupleStruct { path: Some(path), .. }
        | Pat::Record { path: Some(path), .. } => {
            path_use_tree(db, resolver, &path, success, root);
        }
        _ => (),
    }
    pat.walk_child_pats(|x| {
        pat_use_tree(x, body, db, resolver, success, root);
    });
}

fn path_use_tree(
    db: &dyn HirDatabase,
    resolver: &Resolver,
    path: &hir_def::path::Path,
    success: &mut bool,
    root: &mut UseTreeRoot,
) {
    let mod_path = path.mod_path();
    if let PathKind::Super(_) | PathKind::Abs = mod_path.kind {
        *success = false;
        return;
    }
    let value_or_partial = resolver.resolve_path_in_value_ns(db.upcast(), mod_path);
    let segments = mod_path.segments();
    if let Some(ResolveValueResult::ValueNs(v)) = value_or_partial {
        match v {
            ValueNs::FunctionId(f) => {
                insert_item_with_segments(segments, root, f.machine_name(), success);
            }
            ValueNs::EnumVariantId(ev) => {
                let var: Variant = ev.into();
                let var_name = var.name(db);
                if segments.len() == 1 {
                    root.insert(
                        segments[0].clone(),
                        UseTree::Item {
                            machine_name: format!("{}::{}", ev.parent.machine_name(), var_name),
                        },
                    );
                }
            }
            _ => {}
        }
    }
}

fn insert_item_with_segments(
    segments: &[Name],
    root: &mut UseTreeRoot,
    machine_name: String,
    success: &mut bool,
) {
    let original_name = &segments[segments.len() - 1];
    if let Some(owner) = insert_path(root, &segments[0..segments.len() - 1]) {
        if owner.contains_key(original_name) {
            return;
        }
        owner.insert(original_name.clone(), UseTree::Item { machine_name });
    } else {
        *success = false;
        return;
    }
}

fn signature_use_tree(db: &dyn HirDatabase, func: Function) -> Option<UseTreeRoot> {
    let mut root = UseTreeRoot::default();
    let resolver = func.id.resolver(db.upcast());
    for (_, ty) in &db.function_data(func.id).params {
        type_ref_use_tree(db, &ty, &mut root, &resolver)?;
    }
    Some(root)
}

fn function_use_tree(db: &dyn HirDatabase, func: Function) -> Option<UseTreeRoot> {
    let mut result = signature_use_tree(db, func)?;
    let def = func.id.into();
    let body = db.body(def);
    let mut success = true;
    expr_use_tree(db, body.body_expr, &body, def, &mut result, &mut success);
    if success {
        Some(result)
    } else {
        None
    }
}

fn use_tree_to_string(root: UseTreeRoot) -> String {
    fn inner(root: &UseTreeRoot, r: &mut String, depth: usize) {
        for (name, x) in root {
            match x {
                UseTree::Mod { children } => {
                    format_to!(r, "pub mod {} {{", name);
                    inner(children, r, depth + 1);
                    *r += "}";
                }
                UseTree::Item { machine_name } => {
                    format_to!(
                        r,
                        "pub(crate) use {}{} as {};",
                        "super::".repeat(depth),
                        machine_name,
                        name
                    );
                }
            }
        }
    }
    let mut r = String::new();
    inner(&root, &mut r, 0);
    r
}

pub fn function_to_checkable_code(db: &dyn HirDatabase, func: Function) -> Option<String> {
    let deps = function_recursive_dependencies(db, func);
    let mut r = dependencies_to_string(db, deps)?;
    r += "\n";
    r += &use_tree_to_string(function_use_tree(db, func)?);
    r += "\n";
    format_to!(r, "fn main() {{ let _ = {}; }}", func.name(db));
    r += "\n";
    r += &func.source(db)?.value.to_string();
    Some(r)
}
