//! This module contains the implementation details of the HIR for ADTs, i.e.
//! structs and enums (and unions).

use std::sync::Arc;

use ra_arena::{impl_arena_id, Arena, RawId};
use ra_syntax::ast::{self, NameOwner, StructKind, TypeAscriptionOwner};

use crate::{
    db::{AstDatabase, DefDatabase},
    ids::{AstItemId, EnumId, StructId},
    name::AsName,
    type_ref::TypeRef,
    Name,
    // EnumVariant, FieldSource, HasSource, Module, Struct, StructField,
};

/// Note that we use `StructData` for unions as well!
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructData {
    pub(crate) name: Option<Name>,
    pub(crate) variant_data: Arc<VariantData>,
}

impl StructData {
    fn new(struct_def: &ast::StructDef) -> StructData {
        let name = struct_def.name().map(|n| n.as_name());
        let variant_data = VariantData::new(struct_def.kind());
        let variant_data = Arc::new(variant_data);
        StructData { name, variant_data }
    }

    pub(crate) fn struct_data_query(
        db: &(impl DefDatabase + AstDatabase),
        struct_: StructId,
    ) -> Arc<StructData> {
        let src = struct_.source(db);
        Arc::new(StructData::new(&src.ast))
    }
}

fn variants(enum_def: &ast::EnumDef) -> impl Iterator<Item = ast::EnumVariant> {
    enum_def.variant_list().into_iter().flat_map(|it| it.variants())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumData {
    pub(crate) name: Option<Name>,
    pub(crate) variants: Arena<LocalEnumVariantId, EnumVariantData>,
}

impl EnumData {
    pub(crate) fn enum_data_query(
        db: &(impl DefDatabase + AstDatabase),
        e: EnumId,
    ) -> Arc<EnumData> {
        let src = e.source(db);
        let name = src.ast.name().map(|n| n.as_name());
        let variants = variants(&src.ast)
            .map(|var| EnumVariantData {
                name: var.name().map(|it| it.as_name()),
                variant_data: Arc::new(VariantData::new(var.kind())),
            })
            .collect();
        Arc::new(EnumData { name, variants })
    }

    pub(crate) fn lookup(&self, name: &Name) -> Option<LocalEnumVariantId> {
        self.variants.iter().find_map(|(id, data)| {
            if data.name.as_ref() == Some(name) {
                Some(id)
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct LocalEnumVariantId(RawId);
impl_arena_id!(LocalEnumVariantId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EnumVariantData {
    pub(crate) name: Option<Name>,
    variant_data: Arc<VariantData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct StructFieldId(RawId);
impl_arena_id!(StructFieldId);

/// A single field of an enum variant or struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldData {
    pub(crate) name: Name,
    pub(crate) type_ref: TypeRef,
}

/// Fields of an enum variant or struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VariantData(VariantDataInner);

#[derive(Debug, Clone, PartialEq, Eq)]
enum VariantDataInner {
    Struct(Arena<StructFieldId, StructFieldData>),
    Tuple(Arena<StructFieldId, StructFieldData>),
    Unit,
}

impl VariantData {
    pub(crate) fn fields(&self) -> Option<&Arena<StructFieldId, StructFieldData>> {
        match &self.0 {
            VariantDataInner::Struct(fields) | VariantDataInner::Tuple(fields) => Some(fields),
            _ => None,
        }
    }
}

impl VariantData {
    fn new(flavor: StructKind) -> Self {
        let inner = match flavor {
            ast::StructKind::Tuple(fl) => {
                let fields = fl
                    .fields()
                    .enumerate()
                    .map(|(i, fd)| StructFieldData {
                        name: Name::new_tuple_field(i),
                        type_ref: TypeRef::from_ast_opt(fd.type_ref()),
                    })
                    .collect();
                VariantDataInner::Tuple(fields)
            }
            ast::StructKind::Named(fl) => {
                let fields = fl
                    .fields()
                    .map(|fd| StructFieldData {
                        name: fd.name().map(|n| n.as_name()).unwrap_or_else(Name::missing),
                        type_ref: TypeRef::from_ast_opt(fd.ascribed_type()),
                    })
                    .collect();
                VariantDataInner::Struct(fields)
            }
            ast::StructKind::Unit => VariantDataInner::Unit,
        };
        VariantData(inner)
    }
}
