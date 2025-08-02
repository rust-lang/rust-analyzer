//! Compute the binary representation of a type

use hir_def::{
    AdtId, LocalFieldId, StructId,
    layout::{
        Float, Integer, LayoutCalculator, Primitive, ReprOptions, Scalar, StructKind,
        TargetDataLayout, WrappingRange,
    },
};
use rustc_abi::{AddressSpace, LayoutData};
use rustc_index::IndexVec;

use rustc_type_ir::{
    FloatTy, IntTy, TyKind, UintTy,
    inherent::{IntoKind, SliceLike},
};
use triomphe::Arc;

use crate::{
    TraitEnvironment,
    consteval_nextsolver::try_const_usize,
    db::HirDatabase,
    layout::{Layout, LayoutError},
    next_solver::{
        DbInterner, GenericArgs, SolverDefId, Ty,
        mapping::{ChalkToNextSolver, convert_binder_to_early_binder},
    },
};

pub(crate) use adt::layout_of_adt_cycle_result;
pub use adt::layout_of_adt_query;

mod adt;

struct LayoutCx<'a> {
    calc: LayoutCalculator<&'a TargetDataLayout>,
}

impl<'a> LayoutCx<'a> {
    fn new(target: &'a TargetDataLayout) -> Self {
        Self { calc: LayoutCalculator::new(target) }
    }
}

// FIXME: move this to the `rustc_abi`.
fn layout_of_simd_ty<'db>(
    db: &'db dyn HirDatabase,
    id: StructId,
    repr_packed: bool,
    args: &GenericArgs<'db>,
    env: Arc<TraitEnvironment>,
    dl: &TargetDataLayout,
) -> Result<Arc<Layout>, LayoutError> {
    // Supported SIMD vectors are homogeneous ADTs with exactly one array field:
    //
    // * #[repr(simd)] struct S([T; 4])
    //
    // where T is a primitive scalar (integer/float/pointer).
    let fields = db.field_types_ns(id.into());
    let mut fields = fields.iter();
    let Some(TyKind::Array(e_ty, e_len)) = fields
        .next()
        .filter(|_| fields.next().is_none())
        .map(|f| f.1.clone().instantiate(DbInterner::new_with(db, None, None), args).kind())
    else {
        return Err(LayoutError::InvalidSimdType);
    };

    let e_len = try_const_usize(db, &e_len).ok_or(LayoutError::HasErrorConst)? as u64;
    let e_ly = db.layout_of_ty_ns(e_ty, env)?;

    let cx = LayoutCx::new(dl);
    Ok(Arc::new(cx.calc.simd_type(e_ly, e_len, repr_packed)?))
}

pub fn layout_of_ty_query<'db>(
    db: &'db dyn HirDatabase,
    ty: Ty<'db>,
    trait_env: Arc<TraitEnvironment>,
) -> Result<Arc<Layout>, LayoutError> {
    let krate = trait_env.krate;
    let interner = DbInterner::new_with(db, Some(krate), None);
    let Ok(target) = db.target_data_layout(krate) else {
        return Err(LayoutError::TargetLayoutNotAvailable);
    };
    let dl = &*target;
    let cx = LayoutCx::new(dl);
    //let ty = normalize(db, trait_env.clone(), ty);
    let result = match ty.clone().kind() {
        TyKind::Adt(def, args) => {
            match def.inner().id {
                hir_def::AdtId::StructId(s) => {
                    let data = db.struct_signature(s);
                    let repr = data.repr.unwrap_or_default();
                    if repr.simd() {
                        return layout_of_simd_ty(db, s, repr.packed(), &args, trait_env, &target);
                    }
                }
                _ => {}
            }
            return db.layout_of_adt_ns(def.inner().id, args, trait_env);
        }
        TyKind::Bool => Layout::scalar(
            dl,
            Scalar::Initialized {
                value: Primitive::Int(Integer::I8, false),
                valid_range: WrappingRange { start: 0, end: 1 },
            },
        ),
        TyKind::Char => Layout::scalar(
            dl,
            Scalar::Initialized {
                value: Primitive::Int(Integer::I32, false),
                valid_range: WrappingRange { start: 0, end: 0x10FFFF },
            },
        ),
        TyKind::Int(i) => Layout::scalar(
            dl,
            scalar_unit(
                dl,
                Primitive::Int(
                    match i {
                        IntTy::Isize => dl.ptr_sized_integer(),
                        IntTy::I8 => Integer::I8,
                        IntTy::I16 => Integer::I16,
                        IntTy::I32 => Integer::I32,
                        IntTy::I64 => Integer::I64,
                        IntTy::I128 => Integer::I128,
                    },
                    true,
                ),
            ),
        ),
        TyKind::Uint(i) => Layout::scalar(
            dl,
            scalar_unit(
                dl,
                Primitive::Int(
                    match i {
                        UintTy::Usize => dl.ptr_sized_integer(),
                        UintTy::U8 => Integer::I8,
                        UintTy::U16 => Integer::I16,
                        UintTy::U32 => Integer::I32,
                        UintTy::U64 => Integer::I64,
                        UintTy::U128 => Integer::I128,
                    },
                    false,
                ),
            ),
        ),
        TyKind::Float(f) => Layout::scalar(
            dl,
            scalar_unit(
                dl,
                Primitive::Float(match f {
                    FloatTy::F16 => Float::F16,
                    FloatTy::F32 => Float::F32,
                    FloatTy::F64 => Float::F64,
                    FloatTy::F128 => Float::F128,
                }),
            ),
        ),
        TyKind::Tuple(tys) => {
            let kind =
                if tys.len() == 0 { StructKind::AlwaysSized } else { StructKind::MaybeUnsized };

            let fields = tys
                .iter()
                .map(|k| db.layout_of_ty_ns(k, trait_env.clone()))
                .collect::<Result<Vec<_>, _>>()?;
            let fields = fields.iter().map(|it| &**it).collect::<Vec<_>>();
            let fields = fields.iter().collect::<IndexVec<_, _>>();
            cx.calc.univariant(&fields, &ReprOptions::default(), kind)?
        }
        TyKind::Array(element, count) => {
            let count = try_const_usize(db, &count).ok_or(LayoutError::HasErrorConst)? as u64;
            let element = db.layout_of_ty_ns(element, trait_env)?;
            cx.calc.array_like::<_, _, ()>(&element, Some(count))?
        }
        TyKind::Slice(element) => {
            let element = db.layout_of_ty_ns(element, trait_env)?;
            cx.calc.array_like::<_, _, ()>(&element, None)?
        }
        TyKind::Str => {
            let element = scalar_unit(dl, Primitive::Int(Integer::I8, false));
            cx.calc.array_like::<_, _, ()>(&Layout::scalar(dl, element), None)?
        }
        // Potentially-wide pointers.
        TyKind::Ref(_, pointee, _) | TyKind::RawPtr(pointee, _) => {
            let mut data_ptr = scalar_unit(dl, Primitive::Pointer(AddressSpace::ZERO));
            if matches!(ty.clone().kind(), TyKind::Ref(..)) {
                data_ptr.valid_range_mut().start = 1;
            }

            // FIXME(next-solver)
            // let pointee = tcx.normalize_erasing_regions(param_env, pointee);
            // if pointee.is_sized(tcx.at(DUMMY_SP), param_env) {
            //     return Ok(tcx.mk_layout(LayoutS::scalar(cx, data_ptr)));
            // }

            let unsized_part = struct_tail_erasing_lifetimes(db, pointee.clone());
            // FIXME(next-solver)
            /*
            if let TyKind::AssociatedType(id, subst) = unsized_part.kind(Interner) {
                unsized_part = TyKind::Alias(chalk_ir::AliasTy::Projection(ProjectionTy {
                    associated_ty_id: *id,
                    substitution: subst.clone(),
                }))
                .intern(Interner);
            }
            unsized_part = normalize(db, trait_env, unsized_part);
            */
            let metadata = match unsized_part.kind() {
                TyKind::Slice(_) | TyKind::Str => {
                    scalar_unit(dl, Primitive::Int(dl.ptr_sized_integer(), false))
                }
                TyKind::Dynamic(..) => {
                    let mut vtable = scalar_unit(dl, Primitive::Pointer(AddressSpace::ZERO));
                    vtable.valid_range_mut().start = 1;
                    vtable
                }
                _ => {
                    // pointee is sized
                    return Ok(Arc::new(Layout::scalar(dl, data_ptr)));
                }
            };

            // Effectively a (ptr, meta) tuple.
            LayoutData::scalar_pair(dl, data_ptr, metadata)
        }
        TyKind::Never => LayoutData::never_type(dl),
        TyKind::FnDef(..) => LayoutData::unit(dl, true),
        TyKind::Dynamic(..) | TyKind::Foreign(_) => LayoutData::unit(dl, false),
        TyKind::FnPtr(..) => {
            let mut ptr = scalar_unit(dl, Primitive::Pointer(dl.instruction_address_space));
            ptr.valid_range_mut().start = 1;
            Layout::scalar(dl, ptr)
        }
        TyKind::Alias(_, ty) => match ty.def_id {
            SolverDefId::TypeAliasId(_) => {
                return Err(LayoutError::HasPlaceholder);
            }
            SolverDefId::InternedOpaqueTyId(opaque) => {
                let impl_trait_id = db.lookup_intern_impl_trait_id(opaque);
                match impl_trait_id {
                    crate::ImplTraitId::ReturnTypeImplTrait(func, idx) => {
                        let infer = db.infer(func.into());
                        return db.layout_of_ty(infer.type_of_rpit[idx].clone(), trait_env);
                    }
                    crate::ImplTraitId::TypeAliasImplTrait(..) => {
                        return Err(LayoutError::NotImplemented);
                    }
                    crate::ImplTraitId::AsyncBlockTypeImplTrait(_, _) => {
                        return Err(LayoutError::NotImplemented);
                    }
                }
            }
            _ => unreachable!(),
        },
        TyKind::Closure(c, args) => {
            let id = match c {
                SolverDefId::InternedClosureId(id) => id,
                _ => unreachable!(),
            };
            let def = db.lookup_intern_closure(id);
            let infer = db.infer(def.0);
            let (captures, _) = infer.closure_info(&id.into());
            let fields = captures
                .iter()
                .map(|it| {
                    let ty = convert_binder_to_early_binder(
                        interner,
                        it.ty.to_nextsolver(interner).clone(),
                    )
                    .instantiate(interner, args.clone());
                    db.layout_of_ty_ns(ty, trait_env.clone())
                })
                .collect::<Result<Vec<_>, _>>()?;
            let fields = fields.iter().map(|it| &**it).collect::<Vec<_>>();
            let fields = fields.iter().collect::<IndexVec<_, _>>();
            cx.calc.univariant(&fields, &ReprOptions::default(), StructKind::AlwaysSized)?
        }
        TyKind::Coroutine(_, _) | TyKind::CoroutineWitness(_, _) => {
            return Err(LayoutError::NotImplemented);
        }
        TyKind::Error(_) => return Err(LayoutError::HasErrorType),
        TyKind::Placeholder(_) | TyKind::Bound(..) | TyKind::Infer(..) | TyKind::Param(..) => {
            return Err(LayoutError::HasPlaceholder);
        }
        TyKind::Pat(..) | TyKind::CoroutineClosure(..) => todo!(),
        TyKind::UnsafeBinder(..) => todo!(),
    };
    Ok(Arc::new(result))
}

pub(crate) fn layout_of_ty_cycle_result<'db>(
    _: &dyn HirDatabase,
    _: Ty<'db>,
    _: Arc<TraitEnvironment>,
) -> Result<Arc<Layout>, LayoutError> {
    Err(LayoutError::RecursiveTypeWithoutIndirection)
}

fn struct_tail_erasing_lifetimes<'a>(db: &'a dyn HirDatabase, pointee: Ty<'a>) -> Ty<'a> {
    match pointee.clone().kind() {
        TyKind::Adt(def, args) => {
            let struct_id = match def.inner().id {
                AdtId::StructId(id) => id,
                _ => return pointee,
            };
            let data = struct_id.fields(db);
            let mut it = data.fields().iter().rev();
            match it.next() {
                Some((f, _)) => {
                    let last_field_ty = field_ty(db, struct_id.into(), f, &args);
                    struct_tail_erasing_lifetimes(db, last_field_ty)
                }
                None => pointee,
            }
        }
        TyKind::Tuple(tys) => {
            if let Some(last_field_ty) = tys.iter().last() {
                struct_tail_erasing_lifetimes(db, last_field_ty.clone())
            } else {
                pointee
            }
        }
        _ => pointee,
    }
}

fn field_ty<'a>(
    db: &'a dyn HirDatabase,
    def: hir_def::VariantId,
    fd: LocalFieldId,
    args: &GenericArgs<'a>,
) -> Ty<'a> {
    db.field_types_ns(def)[fd].clone().instantiate(DbInterner::new(db), args)
}

fn scalar_unit(dl: &TargetDataLayout, value: Primitive) -> Scalar {
    Scalar::Initialized { value, valid_range: WrappingRange::full(value.size(dl)) }
}
