//! This module contains not generated ast builders and shortcut methods.
use crate::ast::*;
use crate::{AstNode, SmolStr, SyntaxKind, SyntaxTreeBuilder};
use std::marker::PhantomData;

pub trait AstMake {
    type I;
    fn make(self, builder: &mut SyntaxTreeBuilder);
}

pub trait AstBuild<N> {
    fn build(self) -> N;
}

impl<M: AstMake> AstBuild<M::I> for M
where
    M::I: AstNode,
{
    fn build(self) -> M::I {
        let mut builder = SyntaxTreeBuilder::default();
        self.make(&mut builder);
        M::I::cast(builder.finish().syntax_node()).unwrap()
    }
}

pub struct OneTokenNodeAstMake<T> {
    node_kind: SyntaxKind,
    token_kind: SyntaxKind,
    token: std::string::String,
    _phantom: PhantomData<T>,
}

impl<T> AstMake for OneTokenNodeAstMake<T> {
    type I = T;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(self.node_kind);
        builder.token(self.token_kind, SmolStr::new(&self.token));
        builder.finish_node();
    }
}

pub type NameRefMake = OneTokenNodeAstMake<crate::ast::NameRef>;

impl NameRefMake {
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
    pub fn new(ident: &str) -> NameRefMake {
        NameRefMake::new(ident)
    }
}

pub type NameMake = OneTokenNodeAstMake<crate::ast::Name>;

impl NameMake {
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
    pub fn new(ident: &str) -> NameMake {
        NameMake::new(ident)
    }
}

pub type LabelMake = OneTokenNodeAstMake<crate::ast::Label>;

impl LabelMake {
    pub fn new(label: &str) -> Self {
        Self {
            node_kind: SyntaxKind::LABEL,
            token_kind: SyntaxKind::LIFETIME,
            token: label.to_string(),
            _phantom: PhantomData,
        }
    }
}

pub type LiteralMake = OneTokenNodeAstMake<crate::ast::Literal>;

impl LiteralMake {
    pub fn new(token_kind: SyntaxKind, value: &str) -> Self {
        Self {
            node_kind: SyntaxKind::LITERAL,
            token_kind,
            token: value.to_string(),
            _phantom: PhantomData,
        }
    }
}

impl Literal {
    pub fn new_int(value: &str) -> LiteralMake {
        LiteralMake::new(SyntaxKind::INT_NUMBER, value)
    }

    pub fn new_float(value: &str) -> LiteralMake {
        LiteralMake::new(SyntaxKind::FLOAT_NUMBER, value)
    }
}

pub struct NoOpAstMake<N: AstNode> {
    _phantom: PhantomData<N>,
}

impl<N: AstNode> AstMake for NoOpAstMake<N> {
    type I = N;
    fn make(self, _builder: &mut SyntaxTreeBuilder) {}
}

// FIXME: this nodes haven't specified children in ASDL file
// This builders should be implemented by hands or specified in ASDL
pub type RangeExprMake = NoOpAstMake<crate::ast::RangeExpr>;
pub type SlicePatMake = NoOpAstMake<crate::ast::SlicePat>;
pub type RangePatMake = NoOpAstMake<crate::ast::RangePat>;
pub type DotDotPatMake = NoOpAstMake<crate::ast::DotDotPat>;
pub type PlaceholderPatMake = NoOpAstMake<crate::ast::PlaceholderPat>;
pub type NeverTypeMake = NoOpAstMake<crate::ast::NeverType>;
pub type PlaceholderTypeMake = NoOpAstMake<crate::ast::PlaceholderType>;
pub type LifetimeArgMake = NoOpAstMake<crate::ast::LifetimeArg>;
pub type TokenTreeMake = NoOpAstMake<crate::ast::TokenTree>;
pub type VisibilityMake = NoOpAstMake<crate::ast::Visibility>;

impl TypeRef {
    pub fn new_unit() -> TypeRefMake {
        TupleType::new().into()
    }

    pub fn new_i32() -> TypeRefMake {
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

    #[test]
    fn test_bin_expr_builder() {
        let expr = BinExpr::new()
            .lh(Literal::new_int("1").into())
            .op(BinOp::LesserTest)
            .rh(Literal::new_int("2").into())
            .build();
        let dump = format!("{:#?}", expr.syntax());
        assert_eq_text!(
            dump.trim(),
            r#"
BIN_EXPR@[0; 3)
  LITERAL@[0; 1)
    INT_NUMBER@[0; 1) "1"
  L_ANGLE@[1; 2) "<"
  LITERAL@[2; 3)
    INT_NUMBER@[2; 3) "2""#
                .trim(),
        );
    }
}
