//! This module contains not generated ast builders and shortcut methods.
use crate::ast::*;
use crate::{AstNode, SmolStr, SyntaxKind, SyntaxTreeBuilder};
use std::marker::PhantomData;

pub trait AstNodeBuilder {
    type Node: AstNode + Sized;

    fn make(self, builder: &mut SyntaxTreeBuilder);

    fn build(self) -> Self::Node
    where
        Self: Sized,
    {
        let mut builder = SyntaxTreeBuilder::default();
        self.make(&mut builder);
        Self::Node::cast(builder.finish().syntax_node()).unwrap()
    }
}

pub struct OneTokenNodeAstBuilder<N: AstNode + Sized> {
    node_kind: SyntaxKind,
    token_kind: SyntaxKind,
    token: std::string::String,
    _phantom: PhantomData<N>,
}

impl<N: AstNode + Sized> AstNodeBuilder for OneTokenNodeAstBuilder<N> {
    type Node = N;

    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(self.node_kind);
        builder.token(self.token_kind, SmolStr::new(&self.token));
        builder.finish_node();
    }
}

pub type NameRefBuilder = OneTokenNodeAstBuilder<crate::ast::NameRef>;

impl NameRefBuilder {
    pub fn new(ident: &str) -> Self {
        Self {
            node_kind: SyntaxKind::NAME_REF,
            token_kind: SyntaxKind::IDENT,
            token: ident.to_string(),
            _phantom: PhantomData,
        }
    }
}

impl NameRef {
    pub fn new(ident: &str) -> NameRefBuilder {
        NameRefBuilder::new(ident)
    }
}

pub type NameBuilder = OneTokenNodeAstBuilder<crate::ast::Name>;

impl NameBuilder {
    pub fn new(ident: &str) -> Self {
        Self {
            node_kind: SyntaxKind::NAME,
            token_kind: SyntaxKind::IDENT,
            token: ident.to_string(),
            _phantom: PhantomData,
        }
    }
}

impl Name {
    pub fn new(ident: &str) -> NameBuilder {
        NameBuilder::new(ident)
    }
}

pub type LabelBuilder = OneTokenNodeAstBuilder<crate::ast::Label>;

impl LabelBuilder {
    pub fn new(label: &str) -> Self {
        Self {
            node_kind: SyntaxKind::LABEL,
            token_kind: SyntaxKind::LIFETIME,
            token: label.to_string(),
            _phantom: PhantomData,
        }
    }
}

pub struct NoOpAstBuilder<N: AstNode + Sized> {
    _phantom: PhantomData<N>,
}

impl<N: AstNode + Sized> AstNodeBuilder for NoOpAstBuilder<N> {
    type Node = N;

    fn make(self, _builder: &mut SyntaxTreeBuilder) {}
}

// FIXME: this nodes haven't specified children in ASDL file
// This builders should be implemented by hands or specified in ASDL
pub type IndexExprBuilder = NoOpAstBuilder<crate::ast::IndexExpr>;
pub type RangeExprBuilder = NoOpAstBuilder<crate::ast::RangeExpr>;
pub type BinExprBuilder = NoOpAstBuilder<crate::ast::BinExpr>;
pub type LiteralBuilder = NoOpAstBuilder<crate::ast::Literal>;
pub type SlicePatBuilder = NoOpAstBuilder<crate::ast::SlicePat>;
pub type RangePatBuilder = NoOpAstBuilder<crate::ast::RangePat>;
pub type DotDotPatBuilder = NoOpAstBuilder<crate::ast::DotDotPat>;
pub type PlaceholderPatBuilder = NoOpAstBuilder<crate::ast::PlaceholderPat>;
pub type NeverTypeBuilder = NoOpAstBuilder<crate::ast::NeverType>;
pub type PlaceholderTypeBuilder = NoOpAstBuilder<crate::ast::PlaceholderType>;
pub type LifetimeArgBuilder = NoOpAstBuilder<crate::ast::LifetimeArg>;
pub type TokenTreeBuilder = NoOpAstBuilder<crate::ast::TokenTree>;
pub type VisibilityBuilder = NoOpAstBuilder<crate::ast::Visibility>;

impl TypeRef {
    pub fn new_unit() -> TypeRefBuilder {
        TupleType::new().into()
    }

    pub fn new_i32() -> TypeRefBuilder {
        PathType::new()
            .path(Path::new().segment(PathSegment::new().name_ref(NameRef::new("i32"))))
            .into()
    }
}

#[cfg(test)]
mod tests {

    use crate::ast::*;
    use test_utils::assert_eq_text;

    #[test]
    fn test_record_field_def_list_builder() {
        let fields = RecordFieldDefList::new()
            .field(RecordFieldDef::new().name(Name::new("foo")).ascribed_type(TypeRef::new_unit()))
            .field(RecordFieldDef::new().name(Name::new("bar")).ascribed_type(TypeRef::new_i32()))
            .build();
        let dump = format!("{:#?}", fields.syntax());
        assert_eq_text!(
            dump.trim(),
            r#"
RECORD_FIELD_DEF_LIST@[0; 17)
  L_CURLY@[0; 1) "{"
  RECORD_FIELD_DEF@[1; 7)
    NAME@[1; 4)
      IDENT@[1; 4) "foo"
    COLON@[4; 5) ":"
    TUPLE_TYPE@[5; 7)
      L_PAREN@[5; 6) "("
      R_PAREN@[6; 7) ")"
  COMMA@[7; 8) ","
  RECORD_FIELD_DEF@[8; 15)
    NAME@[8; 11)
      IDENT@[8; 11) "bar"
    COLON@[11; 12) ":"
    PATH_TYPE@[12; 15)
      PATH@[12; 15)
        PATH_SEGMENT@[12; 15)
          NAME_REF@[12; 15)
            IDENT@[12; 15) "i32"
  COMMA@[15; 16) ","
  R_CURLY@[16; 17) "}"
            "#
            .trim(),
        );
    }
}
