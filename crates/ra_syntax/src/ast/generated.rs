//! Generated file, do not edit by hand, see `crate/ra_tools/src/codegen`

use crate::{
    ast::{self, traits::CommentIter, AstChildren, AstNode},
    SyntaxKind::{self, *},
    SyntaxNode,
};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    TupleExpr(TupleExpr),
    ArrayExpr(ArrayExpr),
    ParenExpr(ParenExpr),
    PathExpr(PathExpr),
    LambdaExpr(LambdaExpr),
    IfExpr(IfExpr),
    LoopExpr(LoopExpr),
    ForExpr(ForExpr),
    WhileExpr(WhileExpr),
    ContinueExpr(ContinueExpr),
    BreakExpr(BreakExpr),
    Label(Label),
    BlockExpr(BlockExpr),
    ReturnExpr(ReturnExpr),
    MatchExpr(MatchExpr),
    RecordLit(RecordLit),
    CallExpr(CallExpr),
    IndexExpr(IndexExpr),
    MethodCallExpr(MethodCallExpr),
    FieldExpr(FieldExpr),
    AwaitExpr(AwaitExpr),
    TryExpr(TryExpr),
    TryBlockExpr(TryBlockExpr),
    CastExpr(CastExpr),
    RefExpr(RefExpr),
    PrefixExpr(PrefixExpr),
    BoxExpr(BoxExpr),
    RangeExpr(RangeExpr),
    BinExpr(BinExpr),
    Literal(Literal),
    MacroCall(MacroCall),
}
impl From<TupleExpr> for Expr {
    fn from(node: TupleExpr) -> Expr {
        Expr::TupleExpr(node)
    }
}
impl From<ArrayExpr> for Expr {
    fn from(node: ArrayExpr) -> Expr {
        Expr::ArrayExpr(node)
    }
}
impl From<ParenExpr> for Expr {
    fn from(node: ParenExpr) -> Expr {
        Expr::ParenExpr(node)
    }
}
impl From<PathExpr> for Expr {
    fn from(node: PathExpr) -> Expr {
        Expr::PathExpr(node)
    }
}
impl From<LambdaExpr> for Expr {
    fn from(node: LambdaExpr) -> Expr {
        Expr::LambdaExpr(node)
    }
}
impl From<IfExpr> for Expr {
    fn from(node: IfExpr) -> Expr {
        Expr::IfExpr(node)
    }
}
impl From<LoopExpr> for Expr {
    fn from(node: LoopExpr) -> Expr {
        Expr::LoopExpr(node)
    }
}
impl From<ForExpr> for Expr {
    fn from(node: ForExpr) -> Expr {
        Expr::ForExpr(node)
    }
}
impl From<WhileExpr> for Expr {
    fn from(node: WhileExpr) -> Expr {
        Expr::WhileExpr(node)
    }
}
impl From<ContinueExpr> for Expr {
    fn from(node: ContinueExpr) -> Expr {
        Expr::ContinueExpr(node)
    }
}
impl From<BreakExpr> for Expr {
    fn from(node: BreakExpr) -> Expr {
        Expr::BreakExpr(node)
    }
}
impl From<Label> for Expr {
    fn from(node: Label) -> Expr {
        Expr::Label(node)
    }
}
impl From<BlockExpr> for Expr {
    fn from(node: BlockExpr) -> Expr {
        Expr::BlockExpr(node)
    }
}
impl From<ReturnExpr> for Expr {
    fn from(node: ReturnExpr) -> Expr {
        Expr::ReturnExpr(node)
    }
}
impl From<MatchExpr> for Expr {
    fn from(node: MatchExpr) -> Expr {
        Expr::MatchExpr(node)
    }
}
impl From<RecordLit> for Expr {
    fn from(node: RecordLit) -> Expr {
        Expr::RecordLit(node)
    }
}
impl From<CallExpr> for Expr {
    fn from(node: CallExpr) -> Expr {
        Expr::CallExpr(node)
    }
}
impl From<IndexExpr> for Expr {
    fn from(node: IndexExpr) -> Expr {
        Expr::IndexExpr(node)
    }
}
impl From<MethodCallExpr> for Expr {
    fn from(node: MethodCallExpr) -> Expr {
        Expr::MethodCallExpr(node)
    }
}
impl From<FieldExpr> for Expr {
    fn from(node: FieldExpr) -> Expr {
        Expr::FieldExpr(node)
    }
}
impl From<AwaitExpr> for Expr {
    fn from(node: AwaitExpr) -> Expr {
        Expr::AwaitExpr(node)
    }
}
impl From<TryExpr> for Expr {
    fn from(node: TryExpr) -> Expr {
        Expr::TryExpr(node)
    }
}
impl From<TryBlockExpr> for Expr {
    fn from(node: TryBlockExpr) -> Expr {
        Expr::TryBlockExpr(node)
    }
}
impl From<CastExpr> for Expr {
    fn from(node: CastExpr) -> Expr {
        Expr::CastExpr(node)
    }
}
impl From<RefExpr> for Expr {
    fn from(node: RefExpr) -> Expr {
        Expr::RefExpr(node)
    }
}
impl From<PrefixExpr> for Expr {
    fn from(node: PrefixExpr) -> Expr {
        Expr::PrefixExpr(node)
    }
}
impl From<BoxExpr> for Expr {
    fn from(node: BoxExpr) -> Expr {
        Expr::BoxExpr(node)
    }
}
impl From<RangeExpr> for Expr {
    fn from(node: RangeExpr) -> Expr {
        Expr::RangeExpr(node)
    }
}
impl From<BinExpr> for Expr {
    fn from(node: BinExpr) -> Expr {
        Expr::BinExpr(node)
    }
}
impl From<Literal> for Expr {
    fn from(node: Literal) -> Expr {
        Expr::Literal(node)
    }
}
impl From<MacroCall> for Expr {
    fn from(node: MacroCall) -> Expr {
        Expr::MacroCall(node)
    }
}
impl AstNode for Expr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_EXPR | ARRAY_EXPR | PAREN_EXPR | PATH_EXPR | LAMBDA_EXPR | IF_EXPR
            | LOOP_EXPR | FOR_EXPR | WHILE_EXPR | CONTINUE_EXPR | BREAK_EXPR | LABEL
            | BLOCK_EXPR | RETURN_EXPR | MATCH_EXPR | RECORD_LIT | CALL_EXPR | INDEX_EXPR
            | METHOD_CALL_EXPR | FIELD_EXPR | AWAIT_EXPR | TRY_EXPR | TRY_BLOCK_EXPR
            | CAST_EXPR | REF_EXPR | PREFIX_EXPR | BOX_EXPR | RANGE_EXPR | BIN_EXPR | LITERAL
            | MACRO_CALL => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            TUPLE_EXPR => Expr::TupleExpr(TupleExpr { syntax }),
            ARRAY_EXPR => Expr::ArrayExpr(ArrayExpr { syntax }),
            PAREN_EXPR => Expr::ParenExpr(ParenExpr { syntax }),
            PATH_EXPR => Expr::PathExpr(PathExpr { syntax }),
            LAMBDA_EXPR => Expr::LambdaExpr(LambdaExpr { syntax }),
            IF_EXPR => Expr::IfExpr(IfExpr { syntax }),
            LOOP_EXPR => Expr::LoopExpr(LoopExpr { syntax }),
            FOR_EXPR => Expr::ForExpr(ForExpr { syntax }),
            WHILE_EXPR => Expr::WhileExpr(WhileExpr { syntax }),
            CONTINUE_EXPR => Expr::ContinueExpr(ContinueExpr { syntax }),
            BREAK_EXPR => Expr::BreakExpr(BreakExpr { syntax }),
            LABEL => Expr::Label(Label { syntax }),
            BLOCK_EXPR => Expr::BlockExpr(BlockExpr { syntax }),
            RETURN_EXPR => Expr::ReturnExpr(ReturnExpr { syntax }),
            MATCH_EXPR => Expr::MatchExpr(MatchExpr { syntax }),
            RECORD_LIT => Expr::RecordLit(RecordLit { syntax }),
            CALL_EXPR => Expr::CallExpr(CallExpr { syntax }),
            INDEX_EXPR => Expr::IndexExpr(IndexExpr { syntax }),
            METHOD_CALL_EXPR => Expr::MethodCallExpr(MethodCallExpr { syntax }),
            FIELD_EXPR => Expr::FieldExpr(FieldExpr { syntax }),
            AWAIT_EXPR => Expr::AwaitExpr(AwaitExpr { syntax }),
            TRY_EXPR => Expr::TryExpr(TryExpr { syntax }),
            TRY_BLOCK_EXPR => Expr::TryBlockExpr(TryBlockExpr { syntax }),
            CAST_EXPR => Expr::CastExpr(CastExpr { syntax }),
            REF_EXPR => Expr::RefExpr(RefExpr { syntax }),
            PREFIX_EXPR => Expr::PrefixExpr(PrefixExpr { syntax }),
            BOX_EXPR => Expr::BoxExpr(BoxExpr { syntax }),
            RANGE_EXPR => Expr::RangeExpr(RangeExpr { syntax }),
            BIN_EXPR => Expr::BinExpr(BinExpr { syntax }),
            LITERAL => Expr::Literal(Literal { syntax }),
            MACRO_CALL => Expr::MacroCall(MacroCall { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Expr::TupleExpr(it) => &it.syntax,
            Expr::ArrayExpr(it) => &it.syntax,
            Expr::ParenExpr(it) => &it.syntax,
            Expr::PathExpr(it) => &it.syntax,
            Expr::LambdaExpr(it) => &it.syntax,
            Expr::IfExpr(it) => &it.syntax,
            Expr::LoopExpr(it) => &it.syntax,
            Expr::ForExpr(it) => &it.syntax,
            Expr::WhileExpr(it) => &it.syntax,
            Expr::ContinueExpr(it) => &it.syntax,
            Expr::BreakExpr(it) => &it.syntax,
            Expr::Label(it) => &it.syntax,
            Expr::BlockExpr(it) => &it.syntax,
            Expr::ReturnExpr(it) => &it.syntax,
            Expr::MatchExpr(it) => &it.syntax,
            Expr::RecordLit(it) => &it.syntax,
            Expr::CallExpr(it) => &it.syntax,
            Expr::IndexExpr(it) => &it.syntax,
            Expr::MethodCallExpr(it) => &it.syntax,
            Expr::FieldExpr(it) => &it.syntax,
            Expr::AwaitExpr(it) => &it.syntax,
            Expr::TryExpr(it) => &it.syntax,
            Expr::TryBlockExpr(it) => &it.syntax,
            Expr::CastExpr(it) => &it.syntax,
            Expr::RefExpr(it) => &it.syntax,
            Expr::PrefixExpr(it) => &it.syntax,
            Expr::BoxExpr(it) => &it.syntax,
            Expr::RangeExpr(it) => &it.syntax,
            Expr::BinExpr(it) => &it.syntax,
            Expr::Literal(it) => &it.syntax,
            Expr::MacroCall(it) => &it.syntax,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TupleExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for TupleExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl TupleExpr {
    pub fn exprs(&self) -> AstChildren<Expr> {
        super::children(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ArrayExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ARRAY_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl ArrayExpr {
    pub fn exprs(&self) -> AstChildren<Expr> {
        super::children(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParenExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ParenExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PAREN_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl ParenExpr {
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for PathExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PATH_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl PathExpr {
    pub fn path(&self) -> Option<Path> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LambdaExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for LambdaExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LAMBDA_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl LambdaExpr {
    pub fn param_list(&self) -> Option<ParamList> {
        super::child_opt(self)
    }
    pub fn body(&self) -> Option<Expr> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IfExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for IfExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            IF_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl IfExpr {
    pub fn condition(&self) -> Option<Condition> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoopExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for LoopExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LOOP_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl LoopExpr {
    pub fn loop_body(&self) -> Option<BlockExpr> {
        super::child_opt(self)
    }
}
impl ast::LoopBodyOwner for LoopExpr {
    fn loop_body(&self) -> Option<BlockExpr> {
        self.loop_body()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ForExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            FOR_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl ForExpr {
    pub fn pat(&self) -> Option<Pat> {
        super::child_opt(self)
    }
    pub fn iterable(&self) -> Option<Expr> {
        super::child_opt(self)
    }
    pub fn loop_body(&self) -> Option<BlockExpr> {
        super::child_opt(self)
    }
}
impl ast::LoopBodyOwner for ForExpr {
    fn loop_body(&self) -> Option<BlockExpr> {
        self.loop_body()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WhileExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for WhileExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            WHILE_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl WhileExpr {
    pub fn condition(&self) -> Option<Condition> {
        super::child_opt(self)
    }
    pub fn loop_body(&self) -> Option<BlockExpr> {
        super::child_opt(self)
    }
}
impl ast::LoopBodyOwner for WhileExpr {
    fn loop_body(&self) -> Option<BlockExpr> {
        self.loop_body()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContinueExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ContinueExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CONTINUE_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BreakExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for BreakExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BREAK_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl BreakExpr {
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Label {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LABEL => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReturnExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ReturnExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RETURN_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl ReturnExpr {
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MatchExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for MatchExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MATCH_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl MatchExpr {
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
    pub fn match_arm_list(&self) -> Option<MatchArmList> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordLit {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for RecordLit {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_LIT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl RecordLit {
    pub fn path(&self) -> Option<Path> {
        super::child_opt(self)
    }
    pub fn record_field_list(&self) -> Option<RecordFieldList> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CallExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for CallExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CALL_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl CallExpr {
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
    pub fn arg_list(&self) -> Option<ArgList> {
        super::child_opt(self)
    }
}
impl ast::ArgListOwner for CallExpr {
    fn arg_list(&self) -> Option<ArgList> {
        self.arg_list()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndexExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for IndexExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            INDEX_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MethodCallExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for MethodCallExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            METHOD_CALL_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl MethodCallExpr {
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
    pub fn name_ref(&self) -> Option<NameRef> {
        super::child_opt(self)
    }
    pub fn type_arg_list(&self) -> Option<TypeArgList> {
        super::child_opt(self)
    }
    pub fn arg_list(&self) -> Option<ArgList> {
        super::child_opt(self)
    }
}
impl ast::ArgListOwner for MethodCallExpr {
    fn arg_list(&self) -> Option<ArgList> {
        self.arg_list()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for FieldExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            FIELD_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FieldExpr {
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
    pub fn name_ref(&self) -> Option<NameRef> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AwaitExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for AwaitExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            AWAIT_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AwaitExpr {
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TryExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for TryExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TRY_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl TryExpr {
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TryBlockExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for TryBlockExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TRY_BLOCK_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl TryBlockExpr {
    pub fn body(&self) -> Option<BlockExpr> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CastExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for CastExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CAST_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl CastExpr {
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
    pub fn type_ref(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RefExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for RefExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            REF_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl RefExpr {
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrefixExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for PrefixExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PREFIX_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl PrefixExpr {
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoxExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for BoxExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BOX_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl BoxExpr {
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RangeExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for RangeExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RANGE_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for BinExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BIN_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MacroCall {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for MacroCall {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MACRO_CALL => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl MacroCall {
    pub fn token_tree(&self) -> Option<TokenTree> {
        super::child_opt(self)
    }
    pub fn path(&self) -> Option<Path> {
        super::child_opt(self)
    }
    pub fn name(&self) -> Option<Name> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
    pub fn doc_comments(&self) -> CommentIter {
        CommentIter::new(self.syntax().children_with_tokens())
    }
}
impl ast::NameOwner for MacroCall {
    fn name(&self) -> Option<Name> {
        self.name()
    }
}
impl ast::DocCommentsOwner for MacroCall {
    fn doc_comments(&self) -> CommentIter {
        self.doc_comments()
    }
}
impl ast::AttrsOwner for MacroCall {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImplItem {
    FnDef(FnDef),
    TypeAliasDef(TypeAliasDef),
    ConstDef(ConstDef),
}
impl From<FnDef> for ImplItem {
    fn from(node: FnDef) -> ImplItem {
        ImplItem::FnDef(node)
    }
}
impl From<TypeAliasDef> for ImplItem {
    fn from(node: TypeAliasDef) -> ImplItem {
        ImplItem::TypeAliasDef(node)
    }
}
impl From<ConstDef> for ImplItem {
    fn from(node: ConstDef) -> ImplItem {
        ImplItem::ConstDef(node)
    }
}
impl AstNode for ImplItem {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            FN_DEF | TYPE_ALIAS_DEF | CONST_DEF => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            FN_DEF => ImplItem::FnDef(FnDef { syntax }),
            TYPE_ALIAS_DEF => ImplItem::TypeAliasDef(TypeAliasDef { syntax }),
            CONST_DEF => ImplItem::ConstDef(ConstDef { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            ImplItem::FnDef(it) => &it.syntax,
            ImplItem::TypeAliasDef(it) => &it.syntax,
            ImplItem::ConstDef(it) => &it.syntax,
        }
    }
}
impl ImplItem {
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
}
impl ast::AttrsOwner for ImplItem {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModuleItem {
    StructDef(StructDef),
    UnionDef(UnionDef),
    EnumDef(EnumDef),
    FnDef(FnDef),
    TraitDef(TraitDef),
    TypeAliasDef(TypeAliasDef),
    ImplBlock(ImplBlock),
    UseItem(UseItem),
    ExternCrateItem(ExternCrateItem),
    ConstDef(ConstDef),
    StaticDef(StaticDef),
    Module(Module),
}
impl From<StructDef> for ModuleItem {
    fn from(node: StructDef) -> ModuleItem {
        ModuleItem::StructDef(node)
    }
}
impl From<UnionDef> for ModuleItem {
    fn from(node: UnionDef) -> ModuleItem {
        ModuleItem::UnionDef(node)
    }
}
impl From<EnumDef> for ModuleItem {
    fn from(node: EnumDef) -> ModuleItem {
        ModuleItem::EnumDef(node)
    }
}
impl From<FnDef> for ModuleItem {
    fn from(node: FnDef) -> ModuleItem {
        ModuleItem::FnDef(node)
    }
}
impl From<TraitDef> for ModuleItem {
    fn from(node: TraitDef) -> ModuleItem {
        ModuleItem::TraitDef(node)
    }
}
impl From<TypeAliasDef> for ModuleItem {
    fn from(node: TypeAliasDef) -> ModuleItem {
        ModuleItem::TypeAliasDef(node)
    }
}
impl From<ImplBlock> for ModuleItem {
    fn from(node: ImplBlock) -> ModuleItem {
        ModuleItem::ImplBlock(node)
    }
}
impl From<UseItem> for ModuleItem {
    fn from(node: UseItem) -> ModuleItem {
        ModuleItem::UseItem(node)
    }
}
impl From<ExternCrateItem> for ModuleItem {
    fn from(node: ExternCrateItem) -> ModuleItem {
        ModuleItem::ExternCrateItem(node)
    }
}
impl From<ConstDef> for ModuleItem {
    fn from(node: ConstDef) -> ModuleItem {
        ModuleItem::ConstDef(node)
    }
}
impl From<StaticDef> for ModuleItem {
    fn from(node: StaticDef) -> ModuleItem {
        ModuleItem::StaticDef(node)
    }
}
impl From<Module> for ModuleItem {
    fn from(node: Module) -> ModuleItem {
        ModuleItem::Module(node)
    }
}
impl AstNode for ModuleItem {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            STRUCT_DEF | UNION_DEF | ENUM_DEF | FN_DEF | TRAIT_DEF | TYPE_ALIAS_DEF
            | IMPL_BLOCK | USE_ITEM | EXTERN_CRATE_ITEM | CONST_DEF | STATIC_DEF | MODULE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            STRUCT_DEF => ModuleItem::StructDef(StructDef { syntax }),
            UNION_DEF => ModuleItem::UnionDef(UnionDef { syntax }),
            ENUM_DEF => ModuleItem::EnumDef(EnumDef { syntax }),
            FN_DEF => ModuleItem::FnDef(FnDef { syntax }),
            TRAIT_DEF => ModuleItem::TraitDef(TraitDef { syntax }),
            TYPE_ALIAS_DEF => ModuleItem::TypeAliasDef(TypeAliasDef { syntax }),
            IMPL_BLOCK => ModuleItem::ImplBlock(ImplBlock { syntax }),
            USE_ITEM => ModuleItem::UseItem(UseItem { syntax }),
            EXTERN_CRATE_ITEM => ModuleItem::ExternCrateItem(ExternCrateItem { syntax }),
            CONST_DEF => ModuleItem::ConstDef(ConstDef { syntax }),
            STATIC_DEF => ModuleItem::StaticDef(StaticDef { syntax }),
            MODULE => ModuleItem::Module(Module { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            ModuleItem::StructDef(it) => &it.syntax,
            ModuleItem::UnionDef(it) => &it.syntax,
            ModuleItem::EnumDef(it) => &it.syntax,
            ModuleItem::FnDef(it) => &it.syntax,
            ModuleItem::TraitDef(it) => &it.syntax,
            ModuleItem::TypeAliasDef(it) => &it.syntax,
            ModuleItem::ImplBlock(it) => &it.syntax,
            ModuleItem::UseItem(it) => &it.syntax,
            ModuleItem::ExternCrateItem(it) => &it.syntax,
            ModuleItem::ConstDef(it) => &it.syntax,
            ModuleItem::StaticDef(it) => &it.syntax,
            ModuleItem::Module(it) => &it.syntax,
        }
    }
}
impl ModuleItem {
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
}
impl ast::AttrsOwner for ModuleItem {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraitDef {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for TraitDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TRAIT_DEF => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl TraitDef {
    pub fn item_list(&self) -> Option<ItemList> {
        super::child_opt(self)
    }
    pub fn visibility(&self) -> Option<Visibility> {
        super::child_opt(self)
    }
    pub fn name(&self) -> Option<Name> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
    pub fn doc_comments(&self) -> CommentIter {
        CommentIter::new(self.syntax().children_with_tokens())
    }
    pub fn type_param_list(&self) -> Option<TypeParamList> {
        super::child_opt(self)
    }
    pub fn where_clause(&self) -> Option<WhereClause> {
        super::child_opt(self)
    }
    pub fn type_bound_list(&self) -> Option<TypeBoundList> {
        super::child_opt(self)
    }
}
impl ast::TypeParamsOwner for TraitDef {
    fn type_param_list(&self) -> Option<TypeParamList> {
        self.type_param_list()
    }
    fn where_clause(&self) -> Option<WhereClause> {
        self.where_clause()
    }
}
impl ast::NameOwner for TraitDef {
    fn name(&self) -> Option<Name> {
        self.name()
    }
}
impl ast::VisibilityOwner for TraitDef {
    fn visibility(&self) -> Option<Visibility> {
        self.visibility()
    }
}
impl ast::TypeBoundsOwner for TraitDef {
    fn type_bound_list(&self) -> Option<TypeBoundList> {
        self.type_bound_list()
    }
}
impl ast::DocCommentsOwner for TraitDef {
    fn doc_comments(&self) -> CommentIter {
        self.doc_comments()
    }
}
impl ast::AttrsOwner for TraitDef {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImplBlock {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ImplBlock {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            IMPL_BLOCK => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl ImplBlock {
    pub fn item_list(&self) -> Option<ItemList> {
        super::child_opt(self)
    }
    pub fn type_param_list(&self) -> Option<TypeParamList> {
        super::child_opt(self)
    }
    pub fn where_clause(&self) -> Option<WhereClause> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
}
impl ast::TypeParamsOwner for ImplBlock {
    fn type_param_list(&self) -> Option<TypeParamList> {
        self.type_param_list()
    }
    fn where_clause(&self) -> Option<WhereClause> {
        self.where_clause()
    }
}
impl ast::AttrsOwner for ImplBlock {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UseItem {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for UseItem {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            USE_ITEM => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl UseItem {
    pub fn use_tree(&self) -> Option<UseTree> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
}
impl ast::AttrsOwner for UseItem {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternCrateItem {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ExternCrateItem {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            EXTERN_CRATE_ITEM => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl ExternCrateItem {
    pub fn name_ref(&self) -> Option<NameRef> {
        super::child_opt(self)
    }
    pub fn alias(&self) -> Option<Alias> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
}
impl ast::AttrsOwner for ExternCrateItem {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StaticDef {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for StaticDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            STATIC_DEF => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl StaticDef {
    pub fn body(&self) -> Option<Expr> {
        super::child_opt(self)
    }
    pub fn visibility(&self) -> Option<Visibility> {
        super::child_opt(self)
    }
    pub fn name(&self) -> Option<Name> {
        super::child_opt(self)
    }
    pub fn type_param_list(&self) -> Option<TypeParamList> {
        super::child_opt(self)
    }
    pub fn where_clause(&self) -> Option<WhereClause> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
    pub fn doc_comments(&self) -> CommentIter {
        CommentIter::new(self.syntax().children_with_tokens())
    }
    pub fn ascribed_type(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
}
impl ast::TypeAscriptionOwner for StaticDef {
    fn ascribed_type(&self) -> Option<TypeRef> {
        self.ascribed_type()
    }
}
impl ast::TypeParamsOwner for StaticDef {
    fn type_param_list(&self) -> Option<TypeParamList> {
        self.type_param_list()
    }
    fn where_clause(&self) -> Option<WhereClause> {
        self.where_clause()
    }
}
impl ast::NameOwner for StaticDef {
    fn name(&self) -> Option<Name> {
        self.name()
    }
}
impl ast::VisibilityOwner for StaticDef {
    fn visibility(&self) -> Option<Visibility> {
        self.visibility()
    }
}
impl ast::DocCommentsOwner for StaticDef {
    fn doc_comments(&self) -> CommentIter {
        self.doc_comments()
    }
}
impl ast::AttrsOwner for StaticDef {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Module {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Module {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MODULE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Module {
    pub fn item_list(&self) -> Option<ItemList> {
        super::child_opt(self)
    }
    pub fn visibility(&self) -> Option<Visibility> {
        super::child_opt(self)
    }
    pub fn name(&self) -> Option<Name> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
    pub fn doc_comments(&self) -> CommentIter {
        CommentIter::new(self.syntax().children_with_tokens())
    }
}
impl ast::NameOwner for Module {
    fn name(&self) -> Option<Name> {
        self.name()
    }
}
impl ast::VisibilityOwner for Module {
    fn visibility(&self) -> Option<Visibility> {
        self.visibility()
    }
}
impl ast::DocCommentsOwner for Module {
    fn doc_comments(&self) -> CommentIter {
        self.doc_comments()
    }
}
impl ast::AttrsOwner for Module {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NominalDef {
    StructDef(StructDef),
    UnionDef(UnionDef),
    EnumDef(EnumDef),
}
impl From<StructDef> for NominalDef {
    fn from(node: StructDef) -> NominalDef {
        NominalDef::StructDef(node)
    }
}
impl From<UnionDef> for NominalDef {
    fn from(node: UnionDef) -> NominalDef {
        NominalDef::UnionDef(node)
    }
}
impl From<EnumDef> for NominalDef {
    fn from(node: EnumDef) -> NominalDef {
        NominalDef::EnumDef(node)
    }
}
impl AstNode for NominalDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            STRUCT_DEF | UNION_DEF | ENUM_DEF => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            STRUCT_DEF => NominalDef::StructDef(StructDef { syntax }),
            UNION_DEF => NominalDef::UnionDef(UnionDef { syntax }),
            ENUM_DEF => NominalDef::EnumDef(EnumDef { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            NominalDef::StructDef(it) => &it.syntax,
            NominalDef::UnionDef(it) => &it.syntax,
            NominalDef::EnumDef(it) => &it.syntax,
        }
    }
}
impl NominalDef {
    pub fn name(&self) -> Option<Name> {
        super::child_opt(self)
    }
    pub fn type_param_list(&self) -> Option<TypeParamList> {
        super::child_opt(self)
    }
    pub fn where_clause(&self) -> Option<WhereClause> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
}
impl ast::TypeParamsOwner for NominalDef {
    fn type_param_list(&self) -> Option<TypeParamList> {
        self.type_param_list()
    }
    fn where_clause(&self) -> Option<WhereClause> {
        self.where_clause()
    }
}
impl ast::NameOwner for NominalDef {
    fn name(&self) -> Option<Name> {
        self.name()
    }
}
impl ast::AttrsOwner for NominalDef {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pat {
    RefPat(RefPat),
    BoxPat(BoxPat),
    BindPat(BindPat),
    PlaceholderPat(PlaceholderPat),
    DotDotPat(DotDotPat),
    PathPat(PathPat),
    RecordPat(RecordPat),
    TupleStructPat(TupleStructPat),
    TuplePat(TuplePat),
    SlicePat(SlicePat),
    RangePat(RangePat),
    LiteralPat(LiteralPat),
}
impl From<RefPat> for Pat {
    fn from(node: RefPat) -> Pat {
        Pat::RefPat(node)
    }
}
impl From<BoxPat> for Pat {
    fn from(node: BoxPat) -> Pat {
        Pat::BoxPat(node)
    }
}
impl From<BindPat> for Pat {
    fn from(node: BindPat) -> Pat {
        Pat::BindPat(node)
    }
}
impl From<PlaceholderPat> for Pat {
    fn from(node: PlaceholderPat) -> Pat {
        Pat::PlaceholderPat(node)
    }
}
impl From<DotDotPat> for Pat {
    fn from(node: DotDotPat) -> Pat {
        Pat::DotDotPat(node)
    }
}
impl From<PathPat> for Pat {
    fn from(node: PathPat) -> Pat {
        Pat::PathPat(node)
    }
}
impl From<RecordPat> for Pat {
    fn from(node: RecordPat) -> Pat {
        Pat::RecordPat(node)
    }
}
impl From<TupleStructPat> for Pat {
    fn from(node: TupleStructPat) -> Pat {
        Pat::TupleStructPat(node)
    }
}
impl From<TuplePat> for Pat {
    fn from(node: TuplePat) -> Pat {
        Pat::TuplePat(node)
    }
}
impl From<SlicePat> for Pat {
    fn from(node: SlicePat) -> Pat {
        Pat::SlicePat(node)
    }
}
impl From<RangePat> for Pat {
    fn from(node: RangePat) -> Pat {
        Pat::RangePat(node)
    }
}
impl From<LiteralPat> for Pat {
    fn from(node: LiteralPat) -> Pat {
        Pat::LiteralPat(node)
    }
}
impl AstNode for Pat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            REF_PAT | BOX_PAT | BIND_PAT | PLACEHOLDER_PAT | DOT_DOT_PAT | PATH_PAT
            | RECORD_PAT | TUPLE_STRUCT_PAT | TUPLE_PAT | SLICE_PAT | RANGE_PAT | LITERAL_PAT => {
                true
            }
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            REF_PAT => Pat::RefPat(RefPat { syntax }),
            BOX_PAT => Pat::BoxPat(BoxPat { syntax }),
            BIND_PAT => Pat::BindPat(BindPat { syntax }),
            PLACEHOLDER_PAT => Pat::PlaceholderPat(PlaceholderPat { syntax }),
            DOT_DOT_PAT => Pat::DotDotPat(DotDotPat { syntax }),
            PATH_PAT => Pat::PathPat(PathPat { syntax }),
            RECORD_PAT => Pat::RecordPat(RecordPat { syntax }),
            TUPLE_STRUCT_PAT => Pat::TupleStructPat(TupleStructPat { syntax }),
            TUPLE_PAT => Pat::TuplePat(TuplePat { syntax }),
            SLICE_PAT => Pat::SlicePat(SlicePat { syntax }),
            RANGE_PAT => Pat::RangePat(RangePat { syntax }),
            LITERAL_PAT => Pat::LiteralPat(LiteralPat { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Pat::RefPat(it) => &it.syntax,
            Pat::BoxPat(it) => &it.syntax,
            Pat::BindPat(it) => &it.syntax,
            Pat::PlaceholderPat(it) => &it.syntax,
            Pat::DotDotPat(it) => &it.syntax,
            Pat::PathPat(it) => &it.syntax,
            Pat::RecordPat(it) => &it.syntax,
            Pat::TupleStructPat(it) => &it.syntax,
            Pat::TuplePat(it) => &it.syntax,
            Pat::SlicePat(it) => &it.syntax,
            Pat::RangePat(it) => &it.syntax,
            Pat::LiteralPat(it) => &it.syntax,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RefPat {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for RefPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            REF_PAT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl RefPat {
    pub fn pat(&self) -> Option<Pat> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoxPat {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for BoxPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BOX_PAT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl BoxPat {
    pub fn pat(&self) -> Option<Pat> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindPat {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for BindPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BIND_PAT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl BindPat {
    pub fn pat(&self) -> Option<Pat> {
        super::child_opt(self)
    }
    pub fn name(&self) -> Option<Name> {
        super::child_opt(self)
    }
}
impl ast::NameOwner for BindPat {
    fn name(&self) -> Option<Name> {
        self.name()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlaceholderPat {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for PlaceholderPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PLACEHOLDER_PAT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DotDotPat {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for DotDotPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            DOT_DOT_PAT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathPat {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for PathPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PATH_PAT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl PathPat {
    pub fn path(&self) -> Option<Path> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordPat {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for RecordPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_PAT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl RecordPat {
    pub fn record_field_pat_list(&self) -> Option<RecordFieldPatList> {
        super::child_opt(self)
    }
    pub fn path(&self) -> Option<Path> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TupleStructPat {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for TupleStructPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_STRUCT_PAT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl TupleStructPat {
    pub fn args(&self) -> AstChildren<Pat> {
        super::children(self)
    }
    pub fn path(&self) -> Option<Path> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TuplePat {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for TuplePat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_PAT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl TuplePat {
    pub fn args(&self) -> AstChildren<Pat> {
        super::children(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SlicePat {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for SlicePat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SLICE_PAT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RangePat {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for RangePat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RANGE_PAT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LiteralPat {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for LiteralPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LITERAL_PAT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl LiteralPat {
    pub fn literal(&self) -> Option<Literal> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stmt {
    ExprStmt(ExprStmt),
    LetStmt(LetStmt),
}
impl From<ExprStmt> for Stmt {
    fn from(node: ExprStmt) -> Stmt {
        Stmt::ExprStmt(node)
    }
}
impl From<LetStmt> for Stmt {
    fn from(node: LetStmt) -> Stmt {
        Stmt::LetStmt(node)
    }
}
impl AstNode for Stmt {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            EXPR_STMT | LET_STMT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            EXPR_STMT => Stmt::ExprStmt(ExprStmt { syntax }),
            LET_STMT => Stmt::LetStmt(LetStmt { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Stmt::ExprStmt(it) => &it.syntax,
            Stmt::LetStmt(it) => &it.syntax,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExprStmt {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ExprStmt {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            EXPR_STMT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl ExprStmt {
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LetStmt {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for LetStmt {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LET_STMT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl LetStmt {
    pub fn pat(&self) -> Option<Pat> {
        super::child_opt(self)
    }
    pub fn initializer(&self) -> Option<Expr> {
        super::child_opt(self)
    }
    pub fn ascribed_type(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
}
impl ast::TypeAscriptionOwner for LetStmt {
    fn ascribed_type(&self) -> Option<TypeRef> {
        self.ascribed_type()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeRef {
    ParenType(ParenType),
    TupleType(TupleType),
    NeverType(NeverType),
    PathType(PathType),
    PointerType(PointerType),
    ArrayType(ArrayType),
    SliceType(SliceType),
    ReferenceType(ReferenceType),
    PlaceholderType(PlaceholderType),
    FnPointerType(FnPointerType),
    ForType(ForType),
    ImplTraitType(ImplTraitType),
    DynTraitType(DynTraitType),
}
impl From<ParenType> for TypeRef {
    fn from(node: ParenType) -> TypeRef {
        TypeRef::ParenType(node)
    }
}
impl From<TupleType> for TypeRef {
    fn from(node: TupleType) -> TypeRef {
        TypeRef::TupleType(node)
    }
}
impl From<NeverType> for TypeRef {
    fn from(node: NeverType) -> TypeRef {
        TypeRef::NeverType(node)
    }
}
impl From<PathType> for TypeRef {
    fn from(node: PathType) -> TypeRef {
        TypeRef::PathType(node)
    }
}
impl From<PointerType> for TypeRef {
    fn from(node: PointerType) -> TypeRef {
        TypeRef::PointerType(node)
    }
}
impl From<ArrayType> for TypeRef {
    fn from(node: ArrayType) -> TypeRef {
        TypeRef::ArrayType(node)
    }
}
impl From<SliceType> for TypeRef {
    fn from(node: SliceType) -> TypeRef {
        TypeRef::SliceType(node)
    }
}
impl From<ReferenceType> for TypeRef {
    fn from(node: ReferenceType) -> TypeRef {
        TypeRef::ReferenceType(node)
    }
}
impl From<PlaceholderType> for TypeRef {
    fn from(node: PlaceholderType) -> TypeRef {
        TypeRef::PlaceholderType(node)
    }
}
impl From<FnPointerType> for TypeRef {
    fn from(node: FnPointerType) -> TypeRef {
        TypeRef::FnPointerType(node)
    }
}
impl From<ForType> for TypeRef {
    fn from(node: ForType) -> TypeRef {
        TypeRef::ForType(node)
    }
}
impl From<ImplTraitType> for TypeRef {
    fn from(node: ImplTraitType) -> TypeRef {
        TypeRef::ImplTraitType(node)
    }
}
impl From<DynTraitType> for TypeRef {
    fn from(node: DynTraitType) -> TypeRef {
        TypeRef::DynTraitType(node)
    }
}
impl AstNode for TypeRef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PAREN_TYPE | TUPLE_TYPE | NEVER_TYPE | PATH_TYPE | POINTER_TYPE | ARRAY_TYPE
            | SLICE_TYPE | REFERENCE_TYPE | PLACEHOLDER_TYPE | FN_POINTER_TYPE | FOR_TYPE
            | IMPL_TRAIT_TYPE | DYN_TRAIT_TYPE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            PAREN_TYPE => TypeRef::ParenType(ParenType { syntax }),
            TUPLE_TYPE => TypeRef::TupleType(TupleType { syntax }),
            NEVER_TYPE => TypeRef::NeverType(NeverType { syntax }),
            PATH_TYPE => TypeRef::PathType(PathType { syntax }),
            POINTER_TYPE => TypeRef::PointerType(PointerType { syntax }),
            ARRAY_TYPE => TypeRef::ArrayType(ArrayType { syntax }),
            SLICE_TYPE => TypeRef::SliceType(SliceType { syntax }),
            REFERENCE_TYPE => TypeRef::ReferenceType(ReferenceType { syntax }),
            PLACEHOLDER_TYPE => TypeRef::PlaceholderType(PlaceholderType { syntax }),
            FN_POINTER_TYPE => TypeRef::FnPointerType(FnPointerType { syntax }),
            FOR_TYPE => TypeRef::ForType(ForType { syntax }),
            IMPL_TRAIT_TYPE => TypeRef::ImplTraitType(ImplTraitType { syntax }),
            DYN_TRAIT_TYPE => TypeRef::DynTraitType(DynTraitType { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            TypeRef::ParenType(it) => &it.syntax,
            TypeRef::TupleType(it) => &it.syntax,
            TypeRef::NeverType(it) => &it.syntax,
            TypeRef::PathType(it) => &it.syntax,
            TypeRef::PointerType(it) => &it.syntax,
            TypeRef::ArrayType(it) => &it.syntax,
            TypeRef::SliceType(it) => &it.syntax,
            TypeRef::ReferenceType(it) => &it.syntax,
            TypeRef::PlaceholderType(it) => &it.syntax,
            TypeRef::FnPointerType(it) => &it.syntax,
            TypeRef::ForType(it) => &it.syntax,
            TypeRef::ImplTraitType(it) => &it.syntax,
            TypeRef::DynTraitType(it) => &it.syntax,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParenType {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ParenType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PAREN_TYPE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl ParenType {
    pub fn type_ref(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TupleType {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for TupleType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_TYPE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl TupleType {
    pub fn fields(&self) -> AstChildren<TypeRef> {
        super::children(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NeverType {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for NeverType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            NEVER_TYPE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PointerType {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for PointerType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            POINTER_TYPE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl PointerType {
    pub fn type_ref(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayType {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ArrayType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ARRAY_TYPE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl ArrayType {
    pub fn type_ref(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SliceType {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for SliceType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SLICE_TYPE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl SliceType {
    pub fn type_ref(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceType {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ReferenceType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            REFERENCE_TYPE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl ReferenceType {
    pub fn type_ref(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlaceholderType {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for PlaceholderType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PLACEHOLDER_TYPE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnPointerType {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for FnPointerType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            FN_POINTER_TYPE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FnPointerType {
    pub fn param_list(&self) -> Option<ParamList> {
        super::child_opt(self)
    }
    pub fn ret_type(&self) -> Option<RetType> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForType {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ForType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            FOR_TYPE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl ForType {
    pub fn type_ref(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImplTraitType {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ImplTraitType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            IMPL_TRAIT_TYPE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl ImplTraitType {
    pub fn type_bound_list(&self) -> Option<TypeBoundList> {
        super::child_opt(self)
    }
}
impl ast::TypeBoundsOwner for ImplTraitType {
    fn type_bound_list(&self) -> Option<TypeBoundList> {
        self.type_bound_list()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DynTraitType {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for DynTraitType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            DYN_TRAIT_TYPE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl DynTraitType {
    pub fn type_bound_list(&self) -> Option<TypeBoundList> {
        super::child_opt(self)
    }
}
impl ast::TypeBoundsOwner for DynTraitType {
    fn type_bound_list(&self) -> Option<TypeBoundList> {
        self.type_bound_list()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttrInput {
    Literal(Literal),
    TokenTree(TokenTree),
}
impl From<Literal> for AttrInput {
    fn from(node: Literal) -> AttrInput {
        AttrInput::Literal(node)
    }
}
impl From<TokenTree> for AttrInput {
    fn from(node: TokenTree) -> AttrInput {
        AttrInput::TokenTree(node)
    }
}
impl AstNode for AttrInput {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LITERAL | TOKEN_TREE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            LITERAL => AttrInput::Literal(Literal { syntax }),
            TOKEN_TREE => AttrInput::TokenTree(TokenTree { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            AttrInput::Literal(it) => &it.syntax,
            AttrInput::TokenTree(it) => &it.syntax,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for BlockExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BLOCK_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl BlockExpr {
    pub fn block(&self) -> Option<Block> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructDef {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for StructDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            STRUCT_DEF => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl StructDef {
    pub fn visibility(&self) -> Option<Visibility> {
        super::child_opt(self)
    }
    pub fn name(&self) -> Option<Name> {
        super::child_opt(self)
    }
    pub fn type_param_list(&self) -> Option<TypeParamList> {
        super::child_opt(self)
    }
    pub fn where_clause(&self) -> Option<WhereClause> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
    pub fn doc_comments(&self) -> CommentIter {
        CommentIter::new(self.syntax().children_with_tokens())
    }
}
impl ast::TypeParamsOwner for StructDef {
    fn type_param_list(&self) -> Option<TypeParamList> {
        self.type_param_list()
    }
    fn where_clause(&self) -> Option<WhereClause> {
        self.where_clause()
    }
}
impl ast::NameOwner for StructDef {
    fn name(&self) -> Option<Name> {
        self.name()
    }
}
impl ast::VisibilityOwner for StructDef {
    fn visibility(&self) -> Option<Visibility> {
        self.visibility()
    }
}
impl ast::DocCommentsOwner for StructDef {
    fn doc_comments(&self) -> CommentIter {
        self.doc_comments()
    }
}
impl ast::AttrsOwner for StructDef {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionDef {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for UnionDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            UNION_DEF => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl UnionDef {
    pub fn visibility(&self) -> Option<Visibility> {
        super::child_opt(self)
    }
    pub fn name(&self) -> Option<Name> {
        super::child_opt(self)
    }
    pub fn type_param_list(&self) -> Option<TypeParamList> {
        super::child_opt(self)
    }
    pub fn where_clause(&self) -> Option<WhereClause> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
    pub fn record_field_def_list(&self) -> Option<RecordFieldDefList> {
        super::child_opt(self)
    }
    pub fn doc_comments(&self) -> CommentIter {
        CommentIter::new(self.syntax().children_with_tokens())
    }
}
impl ast::TypeParamsOwner for UnionDef {
    fn type_param_list(&self) -> Option<TypeParamList> {
        self.type_param_list()
    }
    fn where_clause(&self) -> Option<WhereClause> {
        self.where_clause()
    }
}
impl ast::NameOwner for UnionDef {
    fn name(&self) -> Option<Name> {
        self.name()
    }
}
impl ast::VisibilityOwner for UnionDef {
    fn visibility(&self) -> Option<Visibility> {
        self.visibility()
    }
}
impl ast::DocCommentsOwner for UnionDef {
    fn doc_comments(&self) -> CommentIter {
        self.doc_comments()
    }
}
impl ast::AttrsOwner for UnionDef {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumDef {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for EnumDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ENUM_DEF => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl EnumDef {
    pub fn variant_list(&self) -> Option<EnumVariantList> {
        super::child_opt(self)
    }
    pub fn visibility(&self) -> Option<Visibility> {
        super::child_opt(self)
    }
    pub fn name(&self) -> Option<Name> {
        super::child_opt(self)
    }
    pub fn type_param_list(&self) -> Option<TypeParamList> {
        super::child_opt(self)
    }
    pub fn where_clause(&self) -> Option<WhereClause> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
    pub fn doc_comments(&self) -> CommentIter {
        CommentIter::new(self.syntax().children_with_tokens())
    }
}
impl ast::TypeParamsOwner for EnumDef {
    fn type_param_list(&self) -> Option<TypeParamList> {
        self.type_param_list()
    }
    fn where_clause(&self) -> Option<WhereClause> {
        self.where_clause()
    }
}
impl ast::NameOwner for EnumDef {
    fn name(&self) -> Option<Name> {
        self.name()
    }
}
impl ast::VisibilityOwner for EnumDef {
    fn visibility(&self) -> Option<Visibility> {
        self.visibility()
    }
}
impl ast::DocCommentsOwner for EnumDef {
    fn doc_comments(&self) -> CommentIter {
        self.doc_comments()
    }
}
impl ast::AttrsOwner for EnumDef {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnDef {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for FnDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            FN_DEF => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FnDef {
    pub fn param_list(&self) -> Option<ParamList> {
        super::child_opt(self)
    }
    pub fn body(&self) -> Option<BlockExpr> {
        super::child_opt(self)
    }
    pub fn ret_type(&self) -> Option<RetType> {
        super::child_opt(self)
    }
    pub fn visibility(&self) -> Option<Visibility> {
        super::child_opt(self)
    }
    pub fn name(&self) -> Option<Name> {
        super::child_opt(self)
    }
    pub fn type_param_list(&self) -> Option<TypeParamList> {
        super::child_opt(self)
    }
    pub fn where_clause(&self) -> Option<WhereClause> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
    pub fn doc_comments(&self) -> CommentIter {
        CommentIter::new(self.syntax().children_with_tokens())
    }
}
impl ast::TypeParamsOwner for FnDef {
    fn type_param_list(&self) -> Option<TypeParamList> {
        self.type_param_list()
    }
    fn where_clause(&self) -> Option<WhereClause> {
        self.where_clause()
    }
}
impl ast::NameOwner for FnDef {
    fn name(&self) -> Option<Name> {
        self.name()
    }
}
impl ast::VisibilityOwner for FnDef {
    fn visibility(&self) -> Option<Visibility> {
        self.visibility()
    }
}
impl ast::DocCommentsOwner for FnDef {
    fn doc_comments(&self) -> CommentIter {
        self.doc_comments()
    }
}
impl ast::AttrsOwner for FnDef {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeAliasDef {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for TypeAliasDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_ALIAS_DEF => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl TypeAliasDef {
    pub fn type_ref(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
    pub fn visibility(&self) -> Option<Visibility> {
        super::child_opt(self)
    }
    pub fn name(&self) -> Option<Name> {
        super::child_opt(self)
    }
    pub fn type_param_list(&self) -> Option<TypeParamList> {
        super::child_opt(self)
    }
    pub fn where_clause(&self) -> Option<WhereClause> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
    pub fn doc_comments(&self) -> CommentIter {
        CommentIter::new(self.syntax().children_with_tokens())
    }
    pub fn type_bound_list(&self) -> Option<TypeBoundList> {
        super::child_opt(self)
    }
}
impl ast::TypeParamsOwner for TypeAliasDef {
    fn type_param_list(&self) -> Option<TypeParamList> {
        self.type_param_list()
    }
    fn where_clause(&self) -> Option<WhereClause> {
        self.where_clause()
    }
}
impl ast::NameOwner for TypeAliasDef {
    fn name(&self) -> Option<Name> {
        self.name()
    }
}
impl ast::VisibilityOwner for TypeAliasDef {
    fn visibility(&self) -> Option<Visibility> {
        self.visibility()
    }
}
impl ast::TypeBoundsOwner for TypeAliasDef {
    fn type_bound_list(&self) -> Option<TypeBoundList> {
        self.type_bound_list()
    }
}
impl ast::DocCommentsOwner for TypeAliasDef {
    fn doc_comments(&self) -> CommentIter {
        self.doc_comments()
    }
}
impl ast::AttrsOwner for TypeAliasDef {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConstDef {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ConstDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CONST_DEF => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl ConstDef {
    pub fn body(&self) -> Option<Expr> {
        super::child_opt(self)
    }
    pub fn visibility(&self) -> Option<Visibility> {
        super::child_opt(self)
    }
    pub fn name(&self) -> Option<Name> {
        super::child_opt(self)
    }
    pub fn type_param_list(&self) -> Option<TypeParamList> {
        super::child_opt(self)
    }
    pub fn where_clause(&self) -> Option<WhereClause> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
    pub fn doc_comments(&self) -> CommentIter {
        CommentIter::new(self.syntax().children_with_tokens())
    }
    pub fn ascribed_type(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
}
impl ast::TypeAscriptionOwner for ConstDef {
    fn ascribed_type(&self) -> Option<TypeRef> {
        self.ascribed_type()
    }
}
impl ast::TypeParamsOwner for ConstDef {
    fn type_param_list(&self) -> Option<TypeParamList> {
        self.type_param_list()
    }
    fn where_clause(&self) -> Option<WhereClause> {
        self.where_clause()
    }
}
impl ast::NameOwner for ConstDef {
    fn name(&self) -> Option<Name> {
        self.name()
    }
}
impl ast::VisibilityOwner for ConstDef {
    fn visibility(&self) -> Option<Visibility> {
        self.visibility()
    }
}
impl ast::DocCommentsOwner for ConstDef {
    fn doc_comments(&self) -> CommentIter {
        self.doc_comments()
    }
}
impl ast::AttrsOwner for ConstDef {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Literal {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Literal {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LITERAL => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Alias {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Alias {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ALIAS => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Alias {
    pub fn name(&self) -> Option<Name> {
        super::child_opt(self)
    }
}
impl ast::NameOwner for Alias {
    fn name(&self) -> Option<Name> {
        self.name()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArgList {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ArgList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ARG_LIST => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl ArgList {
    pub fn args(&self) -> AstChildren<Expr> {
        super::children(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssocTypeArg {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for AssocTypeArg {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ASSOC_TYPE_ARG => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AssocTypeArg {
    pub fn name_ref(&self) -> Option<NameRef> {
        super::child_opt(self)
    }
    pub fn type_ref(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Attr {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Attr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ATTR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Attr {
    pub fn path(&self) -> Option<Path> {
        super::child_opt(self)
    }
    pub fn input(&self) -> Option<AttrInput> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Block {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BLOCK => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Block {
    pub fn statements(&self) -> AstChildren<Stmt> {
        super::children(self)
    }
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
}
impl ast::AttrsOwner for Block {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Condition {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Condition {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CONDITION => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Condition {
    pub fn pat(&self) -> Option<Pat> {
        super::child_opt(self)
    }
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumVariant {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for EnumVariant {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ENUM_VARIANT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl EnumVariant {
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
    pub fn name(&self) -> Option<Name> {
        super::child_opt(self)
    }
    pub fn doc_comments(&self) -> CommentIter {
        CommentIter::new(self.syntax().children_with_tokens())
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
}
impl ast::NameOwner for EnumVariant {
    fn name(&self) -> Option<Name> {
        self.name()
    }
}
impl ast::DocCommentsOwner for EnumVariant {
    fn doc_comments(&self) -> CommentIter {
        self.doc_comments()
    }
}
impl ast::AttrsOwner for EnumVariant {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumVariantList {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for EnumVariantList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ENUM_VARIANT_LIST => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl EnumVariantList {
    pub fn variants(&self) -> AstChildren<EnumVariant> {
        super::children(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordFieldPat {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for RecordFieldPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD_PAT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl RecordFieldPat {
    pub fn pat(&self) -> Option<Pat> {
        super::child_opt(self)
    }
    pub fn name(&self) -> Option<Name> {
        super::child_opt(self)
    }
}
impl ast::NameOwner for RecordFieldPat {
    fn name(&self) -> Option<Name> {
        self.name()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordFieldPatList {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for RecordFieldPatList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD_PAT_LIST => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl RecordFieldPatList {
    pub fn record_field_pats(&self) -> AstChildren<RecordFieldPat> {
        super::children(self)
    }
    pub fn bind_pats(&self) -> AstChildren<BindPat> {
        super::children(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemList {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ItemList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ITEM_LIST => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl ItemList {
    pub fn impl_items(&self) -> AstChildren<ImplItem> {
        super::children(self)
    }
    pub fn functions(&self) -> AstChildren<FnDef> {
        super::children(self)
    }
    pub fn items(&self) -> AstChildren<ModuleItem> {
        super::children(self)
    }
}
impl ast::FnDefOwner for ItemList {
    fn functions(&self) -> AstChildren<FnDef> {
        self.functions()
    }
}
impl ast::ModuleItemOwner for ItemList {
    fn items(&self) -> AstChildren<ModuleItem> {
        self.items()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LifetimeArg {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for LifetimeArg {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LIFETIME_ARG => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LifetimeParam {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for LifetimeParam {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LIFETIME_PARAM => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl LifetimeParam {
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
}
impl ast::AttrsOwner for LifetimeParam {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MacroItems {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for MacroItems {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MACRO_ITEMS => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl MacroItems {
    pub fn items(&self) -> AstChildren<ModuleItem> {
        super::children(self)
    }
    pub fn functions(&self) -> AstChildren<FnDef> {
        super::children(self)
    }
}
impl ast::FnDefOwner for MacroItems {
    fn functions(&self) -> AstChildren<FnDef> {
        self.functions()
    }
}
impl ast::ModuleItemOwner for MacroItems {
    fn items(&self) -> AstChildren<ModuleItem> {
        self.items()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MacroStmts {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for MacroStmts {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MACRO_STMTS => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl MacroStmts {
    pub fn statements(&self) -> AstChildren<Stmt> {
        super::children(self)
    }
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MatchArm {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for MatchArm {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MATCH_ARM => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl MatchArm {
    pub fn pats(&self) -> AstChildren<Pat> {
        super::children(self)
    }
    pub fn guard(&self) -> Option<MatchGuard> {
        super::child_opt(self)
    }
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
}
impl ast::AttrsOwner for MatchArm {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MatchArmList {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for MatchArmList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MATCH_ARM_LIST => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl MatchArmList {
    pub fn arms(&self) -> AstChildren<MatchArm> {
        super::children(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
}
impl ast::AttrsOwner for MatchArmList {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MatchGuard {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for MatchGuard {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MATCH_GUARD => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl MatchGuard {
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Name {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            NAME => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NameRef {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for NameRef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            NAME_REF => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordField {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for RecordField {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl RecordField {
    pub fn name_ref(&self) -> Option<NameRef> {
        super::child_opt(self)
    }
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordFieldDef {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for RecordFieldDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD_DEF => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl RecordFieldDef {
    pub fn visibility(&self) -> Option<Visibility> {
        super::child_opt(self)
    }
    pub fn name(&self) -> Option<Name> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
    pub fn doc_comments(&self) -> CommentIter {
        CommentIter::new(self.syntax().children_with_tokens())
    }
    pub fn ascribed_type(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
}
impl ast::TypeAscriptionOwner for RecordFieldDef {
    fn ascribed_type(&self) -> Option<TypeRef> {
        self.ascribed_type()
    }
}
impl ast::NameOwner for RecordFieldDef {
    fn name(&self) -> Option<Name> {
        self.name()
    }
}
impl ast::VisibilityOwner for RecordFieldDef {
    fn visibility(&self) -> Option<Visibility> {
        self.visibility()
    }
}
impl ast::DocCommentsOwner for RecordFieldDef {
    fn doc_comments(&self) -> CommentIter {
        self.doc_comments()
    }
}
impl ast::AttrsOwner for RecordFieldDef {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordFieldDefList {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for RecordFieldDefList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD_DEF_LIST => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl RecordFieldDefList {
    pub fn fields(&self) -> AstChildren<RecordFieldDef> {
        super::children(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordFieldList {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for RecordFieldList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD_LIST => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl RecordFieldList {
    pub fn fields(&self) -> AstChildren<RecordField> {
        super::children(self)
    }
    pub fn spread(&self) -> Option<Expr> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Param {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Param {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PARAM => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Param {
    pub fn pat(&self) -> Option<Pat> {
        super::child_opt(self)
    }
    pub fn ascribed_type(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
}
impl ast::TypeAscriptionOwner for Param {
    fn ascribed_type(&self) -> Option<TypeRef> {
        self.ascribed_type()
    }
}
impl ast::AttrsOwner for Param {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParamList {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for ParamList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PARAM_LIST => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl ParamList {
    pub fn params(&self) -> AstChildren<Param> {
        super::children(self)
    }
    pub fn self_param(&self) -> Option<SelfParam> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathType {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for PathType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PATH_TYPE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl PathType {
    pub fn path(&self) -> Option<Path> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Path {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PATH => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl Path {
    pub fn segment(&self) -> Option<PathSegment> {
        super::child_opt(self)
    }
    pub fn qualifier(&self) -> Option<Path> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathSegment {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for PathSegment {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PATH_SEGMENT => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl PathSegment {
    pub fn name_ref(&self) -> Option<NameRef> {
        super::child_opt(self)
    }
    pub fn type_arg_list(&self) -> Option<TypeArgList> {
        super::child_opt(self)
    }
    pub fn param_list(&self) -> Option<ParamList> {
        super::child_opt(self)
    }
    pub fn ret_type(&self) -> Option<RetType> {
        super::child_opt(self)
    }
    pub fn path_type(&self) -> Option<PathType> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TupleFieldDef {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for TupleFieldDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_FIELD_DEF => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl TupleFieldDef {
    pub fn type_ref(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
    pub fn visibility(&self) -> Option<Visibility> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
}
impl ast::VisibilityOwner for TupleFieldDef {
    fn visibility(&self) -> Option<Visibility> {
        self.visibility()
    }
}
impl ast::AttrsOwner for TupleFieldDef {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TupleFieldDefList {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for TupleFieldDefList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_FIELD_DEF_LIST => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl TupleFieldDefList {
    pub fn fields(&self) -> AstChildren<TupleFieldDef> {
        super::children(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RetType {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for RetType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RET_TYPE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl RetType {
    pub fn type_ref(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SelfParam {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for SelfParam {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SELF_PARAM => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl SelfParam {
    pub fn ascribed_type(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
}
impl ast::TypeAscriptionOwner for SelfParam {
    fn ascribed_type(&self) -> Option<TypeRef> {
        self.ascribed_type()
    }
}
impl ast::AttrsOwner for SelfParam {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceFile {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for SourceFile {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SOURCE_FILE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl SourceFile {
    pub fn modules(&self) -> AstChildren<Module> {
        super::children(self)
    }
    pub fn items(&self) -> AstChildren<ModuleItem> {
        super::children(self)
    }
    pub fn functions(&self) -> AstChildren<FnDef> {
        super::children(self)
    }
}
impl ast::FnDefOwner for SourceFile {
    fn functions(&self) -> AstChildren<FnDef> {
        self.functions()
    }
}
impl ast::ModuleItemOwner for SourceFile {
    fn items(&self) -> AstChildren<ModuleItem> {
        self.items()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenTree {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for TokenTree {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TOKEN_TREE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeArg {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for TypeArg {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_ARG => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl TypeArg {
    pub fn type_ref(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeArgList {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for TypeArgList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_ARG_LIST => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl TypeArgList {
    pub fn type_args(&self) -> AstChildren<TypeArg> {
        super::children(self)
    }
    pub fn lifetime_args(&self) -> AstChildren<LifetimeArg> {
        super::children(self)
    }
    pub fn assoc_type_args(&self) -> AstChildren<AssocTypeArg> {
        super::children(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeBound {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for TypeBound {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_BOUND => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl TypeBound {
    pub fn type_ref(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeBoundList {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for TypeBoundList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_BOUND_LIST => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl TypeBoundList {
    pub fn bounds(&self) -> AstChildren<TypeBound> {
        super::children(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeParam {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for TypeParam {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_PARAM => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl TypeParam {
    pub fn name(&self) -> Option<Name> {
        super::child_opt(self)
    }
    pub fn attrs(&self) -> AstChildren<Attr> {
        super::children(self)
    }
    pub fn type_bound_list(&self) -> Option<TypeBoundList> {
        super::child_opt(self)
    }
    pub fn default_type(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
}
impl ast::NameOwner for TypeParam {
    fn name(&self) -> Option<Name> {
        self.name()
    }
}
impl ast::TypeBoundsOwner for TypeParam {
    fn type_bound_list(&self) -> Option<TypeBoundList> {
        self.type_bound_list()
    }
}
impl ast::AttrsOwner for TypeParam {
    fn attrs(&self) -> AstChildren<Attr> {
        self.attrs()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeParamList {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for TypeParamList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_PARAM_LIST => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl TypeParamList {
    pub fn type_params(&self) -> AstChildren<TypeParam> {
        super::children(self)
    }
    pub fn lifetime_params(&self) -> AstChildren<LifetimeParam> {
        super::children(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UseTree {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for UseTree {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            USE_TREE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl UseTree {
    pub fn path(&self) -> Option<Path> {
        super::child_opt(self)
    }
    pub fn use_tree_list(&self) -> Option<UseTreeList> {
        super::child_opt(self)
    }
    pub fn alias(&self) -> Option<Alias> {
        super::child_opt(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UseTreeList {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for UseTreeList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            USE_TREE_LIST => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl UseTreeList {
    pub fn use_trees(&self) -> AstChildren<UseTree> {
        super::children(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Visibility {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for Visibility {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            VISIBILITY => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WhereClause {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for WhereClause {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            WHERE_CLAUSE => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl WhereClause {
    pub fn predicates(&self) -> AstChildren<WherePred> {
        super::children(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WherePred {
    pub(crate) syntax: SyntaxNode,
}
impl AstNode for WherePred {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            WHERE_PRED => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl WherePred {
    pub fn type_ref(&self) -> Option<TypeRef> {
        super::child_opt(self)
    }
    pub fn type_bound_list(&self) -> Option<TypeBoundList> {
        super::child_opt(self)
    }
}
impl ast::TypeBoundsOwner for WherePred {
    fn type_bound_list(&self) -> Option<TypeBoundList> {
        self.type_bound_list()
    }
}
