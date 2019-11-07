//! This module contains not generated ast builders and shortcut methods.
use crate::ast::*;
use crate::{AstNode, SmolStr, SyntaxKind, SyntaxTreeBuilder};
use std::marker::PhantomData;

pub trait AstMake {
    type Node;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder);
    fn finish_make(&mut self, _builder: &mut SyntaxTreeBuilder);
}

pub trait AstBuild<N> {
    fn build(self) -> N;
}

impl<M: AstMake> AstBuild<M::Node> for M
where
    M::Node: AstNode,
{
    fn build(mut self) -> M::Node {
        let mut builder = SyntaxTreeBuilder::default();
        self.make(&mut builder);
        self.finish_make(&mut builder);
        M::Node::cast(builder.finish().syntax_node()).unwrap()
    }
}

pub(crate) struct TokenMake {
    kind: SyntaxKind,
    token: &'static str,
}

impl TokenMake {
    pub(crate) fn new(kind: SyntaxKind, token: &'static str) -> Self {
        TokenMake { kind, token }
    }
}

pub struct Make<A, B> {
    a: A,
    b: B,
    end_token: Option<TokenMake>,
}

impl<A, B> Make<A, B> {
    pub(crate) fn new(a: A, b: B, end_token: Option<TokenMake>) -> Self {
        Make { a, b, end_token }
    }
}

impl<AN, BN, A: AstMake<Node = AN>, B: AstMake<Node = BN>> AstMake for Make<A, B> {
    type Node = AN;
    fn make(&mut self, b: &mut SyntaxTreeBuilder) {
        self.a.make(b);
        self.b.make(b);
        self.b.finish_make(b);
        if let Some(tm) = &self.end_token {
            b.token(tm.kind, SmolStr::new(tm.token));
        }
    }

    fn finish_make(&mut self, b: &mut SyntaxTreeBuilder) {
        self.a.finish_make(b);
    }
}

// FIXME: this nodes haven't specified children in ASDL file
// This builders should be implemented by hands or specified in ASDL
pub trait RangeExprMake {}
pub trait SlicePatMake {}
pub trait RangePatMake {}
pub trait DotDotPatMake {}
pub trait PlaceholderPatMake {}
pub trait NeverTypeMake {}
pub trait PlaceholderTypeMake {}
pub trait LifetimeArgMake {}
pub trait TokenTreeMake {}
pub trait VisibilityMake {}

pub struct OneTokenNodeAstMake<T> {
    node_kind: SyntaxKind,
    token_kind: SyntaxKind,
    token: std::string::String,
    _phantom: PhantomData<T>,
}

impl<T> AstMake for OneTokenNodeAstMake<T> {
    type Node = T;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(self.node_kind);
        builder.token(self.token_kind, SmolStr::new(&self.token));
        builder.finish_node();
    }

    fn finish_make(&mut self, _builder: &mut SyntaxTreeBuilder) {}
}

pub trait NameRefMake: AstMake {}
pub type NameRefBase = OneTokenNodeAstMake<crate::ast::NameRef>;
impl NameRefMake for NameRefBase {}

impl NameRefBase {
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
    pub fn new(ident: &str) -> NameRefBase {
        NameRefBase::new(ident)
    }
}

pub trait NameMake: AstMake {}
pub type NameBase = OneTokenNodeAstMake<crate::ast::Name>;
impl NameMake for NameBase {}

impl NameBase {
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
    pub fn new(ident: &str) -> NameBase {
        NameBase::new(ident)
    }
}

pub trait LabelMake: AstMake {}
pub type LabelBase = OneTokenNodeAstMake<crate::ast::Label>;
impl LabelMake for LabelBase {}

impl LabelBase {
    pub fn new(label: &str) -> Self {
        Self {
            node_kind: SyntaxKind::LABEL,
            token_kind: SyntaxKind::LIFETIME,
            token: label.to_string(),
            _phantom: PhantomData,
        }
    }
}

pub trait LiteralMake: ExprMake + AstMake {}
pub type LiteralBase = OneTokenNodeAstMake<crate::ast::Literal>;
impl LiteralMake for LiteralBase {}
impl ExprMake for LiteralBase {}

impl LiteralBase {
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
    pub fn new_int(value: &str) -> impl LiteralMake {
        LiteralBase::new(SyntaxKind::INT_NUMBER, value)
    }

    pub fn new_float(value: &str) -> impl LiteralMake {
        LiteralBase::new(SyntaxKind::FLOAT_NUMBER, value)
    }
}

pub struct NoOpAstMake<N: AstNode> {
    _phantom: PhantomData<N>,
}

impl<N: AstNode> AstMake for NoOpAstMake<N> {
    type Node = N;
    fn make(&mut self, _builder: &mut SyntaxTreeBuilder) {}
    fn finish_make(&mut self, _builder: &mut SyntaxTreeBuilder) {}
}

impl TypeRef {
    pub fn new_unit() -> impl TypeRefMake {
        TupleType::new()
    }

    pub fn new_i32() -> impl TypeRefMake {
        PathType::new().path(Path::new().segment(PathSegment::new().name_ref(NameRef::new("i32"))))
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
            .lh(Literal::new_int("1"))
            .op(BinOp::LesserTest)
            .rh(Literal::new_int("2"))
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
