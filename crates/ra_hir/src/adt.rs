use std::sync::Arc;

use ra_db::Cancelable;
use ra_syntax::ast::{self, NameOwner, StructFlavor, AstNode};

use crate::{
    DefId, Name, AsName, Struct, Enum, VariantData, StructField, HirDatabase, DefKind,
    type_ref::TypeRef,
};

impl Struct {
    pub(crate) fn new(def_id: DefId) -> Self {
        Struct { def_id }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructData {
    pub(crate) name: Option<Name>,
    pub(crate) variant_data: Arc<VariantData>,
}

impl StructData {
    fn new(struct_def: &ast::StructDef) -> StructData {
        let name = struct_def.name().map(|n| n.as_name());
        let variant_data = VariantData::new(struct_def.flavor());
        let variant_data = Arc::new(variant_data);
        StructData { name, variant_data }
    }

    pub(crate) fn struct_data_query(
        db: &impl HirDatabase,
        def_id: DefId,
    ) -> Cancelable<Arc<StructData>> {
        let def_loc = def_id.loc(db);
        assert!(def_loc.kind == DefKind::Struct);
        let syntax = db.file_item(def_loc.source_item_id);
        let struct_def =
            ast::StructDef::cast(&syntax).expect("struct def should point to StructDef node");
        Ok(Arc::new(StructData::new(struct_def)))
    }
}

impl Enum {
    pub(crate) fn new(def_id: DefId, variants: Vec<DefId>) -> Self {
        Enum { def_id, variants }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumData {
    pub(crate) name: Option<Name>,
}

impl EnumData {
    fn new(enum_def: &ast::EnumDef) -> Self {
        let name = enum_def.name().map(|n| n.as_name());
        EnumData { name }
    }

    pub(crate) fn enum_data_query(
        db: &impl HirDatabase,
        def_id: DefId,
    ) -> Cancelable<Arc<EnumData>> {
        let def_loc = def_id.loc(db);
        assert!(def_loc.kind == DefKind::Enum);
        let syntax = db.file_item(def_loc.source_item_id);
        let enum_def = ast::EnumDef::cast(&syntax).expect("enum def should point to EnumDef node");
        Ok(Arc::new(EnumData::new(enum_def)))
    }
}

// FIXME: simply reuse StructData?
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariantData {
    pub(crate) name: Option<Name>,
    pub(crate) variant_data: Arc<VariantData>,
}

impl EnumVariantData {
    fn new(variant_def: &ast::EnumVariant) -> EnumVariantData {
        let name = variant_def.name().map(|n| n.as_name());
        let variant_data = VariantData::new(variant_def.flavor());
        let variant_data = Arc::new(variant_data);
        EnumVariantData { name, variant_data }
    }

    pub(crate) fn enum_variant_data_query(
        db: &impl HirDatabase,
        def_id: DefId,
    ) -> Cancelable<Arc<EnumVariantData>> {
        let def_loc = def_id.loc(db);
        assert!(def_loc.kind == DefKind::EnumVariant);
        let syntax = db.file_item(def_loc.source_item_id);
        let variant_def = ast::EnumVariant::cast(&syntax)
            .expect("enum variant def should point to EnumVariant node");
        Ok(Arc::new(EnumVariantData::new(variant_def)))
    }
}

impl VariantData {
    fn new(flavor: StructFlavor) -> Self {
        match flavor {
            StructFlavor::Tuple(fl) => {
                let fields = fl
                    .fields()
                    .enumerate()
                    .map(|(i, fd)| StructField {
                        name: Name::tuple_field_name(i),
                        type_ref: TypeRef::from_ast_opt(fd.type_ref()),
                    })
                    .collect();
                VariantData::Tuple(fields)
            }
            StructFlavor::Named(fl) => {
                let fields = fl
                    .fields()
                    .map(|fd| StructField {
                        name: fd.name().map(|n| n.as_name()).unwrap_or_else(Name::missing),
                        type_ref: TypeRef::from_ast_opt(fd.type_ref()),
                    })
                    .collect();
                VariantData::Struct(fields)
            }
            StructFlavor::Unit => VariantData::Unit,
        }
    }

    pub(crate) fn get_field_type_ref(&self, field_name: &Name) -> Option<&TypeRef> {
        self.fields()
            .iter()
            .find(|f| f.name() == field_name)
            .map(|f| f.type_ref())
    }
}
