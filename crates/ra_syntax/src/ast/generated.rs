//! Generated file, do not edit by hand, see `crate/ra_tools/src/codegen`

use crate::{
    ast::{self, builders::*, traits::CommentIter, AstChildren, AstNode},
    SmolStr,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, SyntaxTreeBuilder, T_STR,
};
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BinOp {
    BooleanOr,
    BooleanAnd,
    EqualityTest,
    NegatedEqualityTest,
    LesserEqualTest,
    GreaterEqualTest,
    LesserTest,
    GreaterTest,
    Addition,
    Multiplication,
    Subtraction,
    Division,
    Remainder,
    LeftShift,
    RightShift,
    BitwiseXor,
    BitwiseOr,
    BitwiseAnd,
    Assignment,
    AddAssign,
    DivAssign,
    MulAssign,
    RemAssign,
    ShrAssign,
    ShlAssign,
    SubAssign,
    BitOrAssign,
    BitAndAssign,
    BitXorAssign,
}
impl BinOp {
    fn from_token(t: &SyntaxToken) -> Option<BinOp> {
        match t.kind() {
            PIPEPIPE => Some(BinOp::BooleanOr),
            AMPAMP => Some(BinOp::BooleanAnd),
            EQEQ => Some(BinOp::EqualityTest),
            NEQ => Some(BinOp::NegatedEqualityTest),
            LTEQ => Some(BinOp::LesserEqualTest),
            GTEQ => Some(BinOp::GreaterEqualTest),
            L_ANGLE => Some(BinOp::LesserTest),
            R_ANGLE => Some(BinOp::GreaterTest),
            PLUS => Some(BinOp::Addition),
            STAR => Some(BinOp::Multiplication),
            MINUS => Some(BinOp::Subtraction),
            SLASH => Some(BinOp::Division),
            PERCENT => Some(BinOp::Remainder),
            SHL => Some(BinOp::LeftShift),
            SHR => Some(BinOp::RightShift),
            CARET => Some(BinOp::BitwiseXor),
            PIPE => Some(BinOp::BitwiseOr),
            AMP => Some(BinOp::BitwiseAnd),
            EQ => Some(BinOp::Assignment),
            PLUSEQ => Some(BinOp::AddAssign),
            SLASHEQ => Some(BinOp::DivAssign),
            STAREQ => Some(BinOp::MulAssign),
            PERCENTEQ => Some(BinOp::RemAssign),
            SHREQ => Some(BinOp::ShrAssign),
            SHLEQ => Some(BinOp::ShlAssign),
            MINUSEQ => Some(BinOp::SubAssign),
            PIPEEQ => Some(BinOp::BitOrAssign),
            AMPEQ => Some(BinOp::BitAndAssign),
            CARETEQ => Some(BinOp::BitXorAssign),
            _ => return None,
        }
    }
}
impl AstMake for BinOp {
    type Node = Self;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        let (kind, token) = match self {
            BinOp::BooleanOr => (PIPEPIPE, T_STR!(PIPEPIPE)),
            BinOp::BooleanAnd => (AMPAMP, T_STR!(AMPAMP)),
            BinOp::EqualityTest => (EQEQ, T_STR!(EQEQ)),
            BinOp::NegatedEqualityTest => (NEQ, T_STR!(NEQ)),
            BinOp::LesserEqualTest => (LTEQ, T_STR!(LTEQ)),
            BinOp::GreaterEqualTest => (GTEQ, T_STR!(GTEQ)),
            BinOp::LesserTest => (L_ANGLE, T_STR!(L_ANGLE)),
            BinOp::GreaterTest => (R_ANGLE, T_STR!(R_ANGLE)),
            BinOp::Addition => (PLUS, T_STR!(PLUS)),
            BinOp::Multiplication => (STAR, T_STR!(STAR)),
            BinOp::Subtraction => (MINUS, T_STR!(MINUS)),
            BinOp::Division => (SLASH, T_STR!(SLASH)),
            BinOp::Remainder => (PERCENT, T_STR!(PERCENT)),
            BinOp::LeftShift => (SHL, T_STR!(SHL)),
            BinOp::RightShift => (SHR, T_STR!(SHR)),
            BinOp::BitwiseXor => (CARET, T_STR!(CARET)),
            BinOp::BitwiseOr => (PIPE, T_STR!(PIPE)),
            BinOp::BitwiseAnd => (AMP, T_STR!(AMP)),
            BinOp::Assignment => (EQ, T_STR!(EQ)),
            BinOp::AddAssign => (PLUSEQ, T_STR!(PLUSEQ)),
            BinOp::DivAssign => (SLASHEQ, T_STR!(SLASHEQ)),
            BinOp::MulAssign => (STAREQ, T_STR!(STAREQ)),
            BinOp::RemAssign => (PERCENTEQ, T_STR!(PERCENTEQ)),
            BinOp::ShrAssign => (SHREQ, T_STR!(SHREQ)),
            BinOp::ShlAssign => (SHLEQ, T_STR!(SHLEQ)),
            BinOp::SubAssign => (MINUSEQ, T_STR!(MINUSEQ)),
            BinOp::BitOrAssign => (PIPEEQ, T_STR!(PIPEEQ)),
            BinOp::BitAndAssign => (AMPEQ, T_STR!(AMPEQ)),
            BinOp::BitXorAssign => (CARETEQ, T_STR!(CARETEQ)),
        };
        builder.token(kind, SmolStr::new(token));
    }
    fn finish_make(&mut self, _builder: &mut SyntaxTreeBuilder) {}
}
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrefixOp {
    Deref,
    Not,
    Neg,
}
impl PrefixOp {
    fn from_token(t: &SyntaxToken) -> Option<PrefixOp> {
        match t.kind() {
            STAR => Some(PrefixOp::Deref),
            EXCL => Some(PrefixOp::Not),
            MINUS => Some(PrefixOp::Neg),
            _ => return None,
        }
    }
}
impl AstMake for PrefixOp {
    type Node = Self;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        let (kind, token) = match self {
            PrefixOp::Deref => (STAR, T_STR!(STAR)),
            PrefixOp::Not => (EXCL, T_STR!(EXCL)),
            PrefixOp::Neg => (MINUS, T_STR!(MINUS)),
        };
        builder.token(kind, SmolStr::new(token));
    }
    fn finish_make(&mut self, _builder: &mut SyntaxTreeBuilder) {}
}
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
pub trait ExprMake: AstMake {}
impl<A: ExprMake, B: AstMake> ExprMake for Make<A, B> {}
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
pub trait TupleExprMake: ExprMake + AstMake {
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: TupleExprMake, B: AstMake> TupleExprMake for Make<A, B> {}
pub struct TupleExprBase {}
impl TupleExprMake for TupleExprBase {}
impl ExprMake for TupleExprBase {}
impl AstMake for TupleExprBase {
    type Node = TupleExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TUPLE_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl TupleExpr {
    pub fn new() -> TupleExprBase {
        TupleExprBase {}
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
pub trait ArrayExprMake: ExprMake + AstMake {
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ArrayExprMake, B: AstMake> ArrayExprMake for Make<A, B> {}
pub struct ArrayExprBase {}
impl ArrayExprMake for ArrayExprBase {}
impl ExprMake for ArrayExprBase {}
impl AstMake for ArrayExprBase {
    type Node = ArrayExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ARRAY_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl ArrayExpr {
    pub fn new() -> ArrayExprBase {
        ArrayExprBase {}
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
pub trait ParenExprMake: ExprMake + AstMake {
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ParenExprMake, B: AstMake> ParenExprMake for Make<A, B> {}
pub struct ParenExprBase {}
impl ParenExprMake for ParenExprBase {}
impl ExprMake for ParenExprBase {}
impl AstMake for ParenExprBase {
    type Node = ParenExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PAREN_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl ParenExpr {
    pub fn new() -> ParenExprBase {
        ParenExprBase {}
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
pub trait PathExprMake: ExprMake + AstMake {
    fn path<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PathMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: PathExprMake, B: AstMake> PathExprMake for Make<A, B> {}
pub struct PathExprBase {}
impl PathExprMake for PathExprBase {}
impl ExprMake for PathExprBase {}
impl AstMake for PathExprBase {
    type Node = PathExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PATH_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl PathExpr {
    pub fn new() -> PathExprBase {
        PathExprBase {}
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
pub trait LambdaExprMake: ExprMake + AstMake {
    fn param_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ParamListMake,
    {
        Make::new(self, b, None)
    }
    fn body<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: LambdaExprMake, B: AstMake> LambdaExprMake for Make<A, B> {}
pub struct LambdaExprBase {}
impl LambdaExprMake for LambdaExprBase {}
impl ExprMake for LambdaExprBase {}
impl AstMake for LambdaExprBase {
    type Node = LambdaExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::LAMBDA_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl LambdaExpr {
    pub fn new() -> LambdaExprBase {
        LambdaExprBase {}
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
pub trait IfExprMake: ExprMake + AstMake {
    fn condition<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ConditionMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: IfExprMake, B: AstMake> IfExprMake for Make<A, B> {}
pub struct IfExprBase {}
impl IfExprMake for IfExprBase {}
impl ExprMake for IfExprBase {}
impl AstMake for IfExprBase {
    type Node = IfExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::IF_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl IfExpr {
    pub fn new() -> IfExprBase {
        IfExprBase {}
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
pub trait LoopExprMake: ExprMake + AstMake {
    fn loop_body<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: BlockExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: LoopExprMake, B: AstMake> LoopExprMake for Make<A, B> {}
pub struct LoopExprBase {}
impl LoopExprMake for LoopExprBase {}
impl ExprMake for LoopExprBase {}
impl AstMake for LoopExprBase {
    type Node = LoopExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::LOOP_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl LoopExpr {
    pub fn new() -> LoopExprBase {
        LoopExprBase {}
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
pub trait ForExprMake: ExprMake + AstMake {
    fn pat<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PatMake,
    {
        Make::new(self, b, None)
    }
    fn iterable<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
    fn loop_body<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: BlockExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ForExprMake, B: AstMake> ForExprMake for Make<A, B> {}
pub struct ForExprBase {}
impl ForExprMake for ForExprBase {}
impl ExprMake for ForExprBase {}
impl AstMake for ForExprBase {
    type Node = ForExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::FOR_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl ForExpr {
    pub fn new() -> ForExprBase {
        ForExprBase {}
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
pub trait WhileExprMake: ExprMake + AstMake {
    fn condition<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ConditionMake,
    {
        Make::new(self, b, None)
    }
    fn loop_body<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: BlockExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: WhileExprMake, B: AstMake> WhileExprMake for Make<A, B> {}
pub struct WhileExprBase {}
impl WhileExprMake for WhileExprBase {}
impl ExprMake for WhileExprBase {}
impl AstMake for WhileExprBase {
    type Node = WhileExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::WHILE_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl WhileExpr {
    pub fn new() -> WhileExprBase {
        WhileExprBase {}
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
pub trait ContinueExprMake: ExprMake + AstMake {}
impl<A: ContinueExprMake, B: AstMake> ContinueExprMake for Make<A, B> {}
pub struct ContinueExprBase {}
impl ContinueExprMake for ContinueExprBase {}
impl ExprMake for ContinueExprBase {}
impl AstMake for ContinueExprBase {
    type Node = ContinueExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::CONTINUE_EXPR);
        builder.token(SyntaxKind::CONTINUE_KW, SmolStr::new(T_STR!(CONTINUE_KW)));
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl ContinueExpr {
    pub fn new() -> ContinueExprBase {
        ContinueExprBase {}
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
pub trait BreakExprMake: ExprMake + AstMake {
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: BreakExprMake, B: AstMake> BreakExprMake for Make<A, B> {}
pub struct BreakExprBase {}
impl BreakExprMake for BreakExprBase {}
impl ExprMake for BreakExprBase {}
impl AstMake for BreakExprBase {
    type Node = BreakExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::BREAK_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl BreakExpr {
    pub fn new() -> BreakExprBase {
        BreakExprBase {}
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
pub trait ReturnExprMake: ExprMake + AstMake {
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ReturnExprMake, B: AstMake> ReturnExprMake for Make<A, B> {}
pub struct ReturnExprBase {}
impl ReturnExprMake for ReturnExprBase {}
impl ExprMake for ReturnExprBase {}
impl AstMake for ReturnExprBase {
    type Node = ReturnExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RETURN_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl ReturnExpr {
    pub fn new() -> ReturnExprBase {
        ReturnExprBase {}
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
pub trait MatchExprMake: ExprMake + AstMake {
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
    fn match_arm_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: MatchArmListMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: MatchExprMake, B: AstMake> MatchExprMake for Make<A, B> {}
pub struct MatchExprBase {}
impl MatchExprMake for MatchExprBase {}
impl ExprMake for MatchExprBase {}
impl AstMake for MatchExprBase {
    type Node = MatchExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MATCH_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl MatchExpr {
    pub fn new() -> MatchExprBase {
        MatchExprBase {}
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
pub trait RecordLitMake: ExprMake + AstMake {
    fn path<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PathMake,
    {
        Make::new(self, b, None)
    }
    fn record_field_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: RecordFieldListMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: RecordLitMake, B: AstMake> RecordLitMake for Make<A, B> {}
pub struct RecordLitBase {}
impl RecordLitMake for RecordLitBase {}
impl ExprMake for RecordLitBase {}
impl AstMake for RecordLitBase {
    type Node = RecordLit;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_LIT);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl RecordLit {
    pub fn new() -> RecordLitBase {
        RecordLitBase {}
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
pub trait CallExprMake: ExprMake + AstMake {
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
    fn arg_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ArgListMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: CallExprMake, B: AstMake> CallExprMake for Make<A, B> {}
pub struct CallExprBase {}
impl CallExprMake for CallExprBase {}
impl ExprMake for CallExprBase {}
impl AstMake for CallExprBase {
    type Node = CallExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::CALL_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl CallExpr {
    pub fn new() -> CallExprBase {
        CallExprBase {}
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
impl IndexExpr {
    pub fn base(&self) -> Option<Expr> {
        super::children(self).nth(0usize)
    }
    pub fn index(&self) -> Option<Expr> {
        super::children(self).nth(1usize)
    }
}
pub trait IndexExprMake: ExprMake + AstMake {
    fn base<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, Some(TokenMake::new(SyntaxKind::L_BRACK, T_STR!(L_BRACK))))
    }
    fn index<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: IndexExprMake, B: AstMake> IndexExprMake for Make<A, B> {}
pub struct IndexExprBase {}
impl IndexExprMake for IndexExprBase {}
impl ExprMake for IndexExprBase {}
impl AstMake for IndexExprBase {
    type Node = IndexExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::INDEX_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.token(SyntaxKind::R_BRACK, SmolStr::new(T_STR!(R_BRACK)));
        builder.finish_node();
    }
}
impl IndexExpr {
    pub fn new() -> IndexExprBase {
        IndexExprBase {}
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
pub trait MethodCallExprMake: ExprMake + AstMake {
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
    fn name_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameRefMake,
    {
        Make::new(self, b, None)
    }
    fn type_arg_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeArgListMake,
    {
        Make::new(self, b, None)
    }
    fn arg_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ArgListMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: MethodCallExprMake, B: AstMake> MethodCallExprMake for Make<A, B> {}
pub struct MethodCallExprBase {}
impl MethodCallExprMake for MethodCallExprBase {}
impl ExprMake for MethodCallExprBase {}
impl AstMake for MethodCallExprBase {
    type Node = MethodCallExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::METHOD_CALL_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl MethodCallExpr {
    pub fn new() -> MethodCallExprBase {
        MethodCallExprBase {}
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
pub trait FieldExprMake: ExprMake + AstMake {
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
    fn name_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameRefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: FieldExprMake, B: AstMake> FieldExprMake for Make<A, B> {}
pub struct FieldExprBase {}
impl FieldExprMake for FieldExprBase {}
impl ExprMake for FieldExprBase {}
impl AstMake for FieldExprBase {
    type Node = FieldExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::FIELD_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl FieldExpr {
    pub fn new() -> FieldExprBase {
        FieldExprBase {}
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
pub trait AwaitExprMake: ExprMake + AstMake {
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: AwaitExprMake, B: AstMake> AwaitExprMake for Make<A, B> {}
pub struct AwaitExprBase {}
impl AwaitExprMake for AwaitExprBase {}
impl ExprMake for AwaitExprBase {}
impl AstMake for AwaitExprBase {
    type Node = AwaitExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::AWAIT_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl AwaitExpr {
    pub fn new() -> AwaitExprBase {
        AwaitExprBase {}
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
pub trait TryExprMake: ExprMake + AstMake {
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: TryExprMake, B: AstMake> TryExprMake for Make<A, B> {}
pub struct TryExprBase {}
impl TryExprMake for TryExprBase {}
impl ExprMake for TryExprBase {}
impl AstMake for TryExprBase {
    type Node = TryExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TRY_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl TryExpr {
    pub fn new() -> TryExprBase {
        TryExprBase {}
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
pub trait TryBlockExprMake: ExprMake + AstMake {
    fn body<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: BlockExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: TryBlockExprMake, B: AstMake> TryBlockExprMake for Make<A, B> {}
pub struct TryBlockExprBase {}
impl TryBlockExprMake for TryBlockExprBase {}
impl ExprMake for TryBlockExprBase {}
impl AstMake for TryBlockExprBase {
    type Node = TryBlockExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TRY_BLOCK_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl TryBlockExpr {
    pub fn new() -> TryBlockExprBase {
        TryBlockExprBase {}
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
pub trait CastExprMake: ExprMake + AstMake {
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
    fn type_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: CastExprMake, B: AstMake> CastExprMake for Make<A, B> {}
pub struct CastExprBase {}
impl CastExprMake for CastExprBase {}
impl ExprMake for CastExprBase {}
impl AstMake for CastExprBase {
    type Node = CastExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::CAST_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl CastExpr {
    pub fn new() -> CastExprBase {
        CastExprBase {}
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
pub trait RefExprMake: ExprMake + AstMake {
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: RefExprMake, B: AstMake> RefExprMake for Make<A, B> {}
pub struct RefExprBase {}
impl RefExprMake for RefExprBase {}
impl ExprMake for RefExprBase {}
impl AstMake for RefExprBase {
    type Node = RefExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::REF_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl RefExpr {
    pub fn new() -> RefExprBase {
        RefExprBase {}
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
    pub fn op_details(&self) -> Option<(SyntaxToken, PrefixOp)> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .find_map(|tok| PrefixOp::from_token(&tok).map(|ty| (tok, ty)))
    }
    pub fn op_kind(&self) -> Option<PrefixOp> {
        self.op_details().map(|t| t.1)
    }
    pub fn op_token(&self) -> Option<SyntaxToken> {
        self.op_details().map(|t| t.0)
    }
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
}
pub trait PrefixExprMake: ExprMake + AstMake {
    fn op(self, ts: PrefixOp) -> Make<Self, PrefixOp>
    where
        Self: Sized,
    {
        Make::new(self, ts, None)
    }
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: PrefixExprMake, B: AstMake> PrefixExprMake for Make<A, B> {}
pub struct PrefixExprBase {}
impl PrefixExprMake for PrefixExprBase {}
impl ExprMake for PrefixExprBase {}
impl AstMake for PrefixExprBase {
    type Node = PrefixExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PREFIX_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl PrefixExpr {
    pub fn new() -> PrefixExprBase {
        PrefixExprBase {}
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
pub trait BoxExprMake: ExprMake + AstMake {
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: BoxExprMake, B: AstMake> BoxExprMake for Make<A, B> {}
pub struct BoxExprBase {}
impl BoxExprMake for BoxExprBase {}
impl ExprMake for BoxExprBase {}
impl AstMake for BoxExprBase {
    type Node = BoxExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::BOX_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl BoxExpr {
    pub fn new() -> BoxExprBase {
        BoxExprBase {}
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
impl BinExpr {
    pub fn lhs(&self) -> Option<Expr> {
        super::children(self).nth(0usize)
    }
    pub fn rhs(&self) -> Option<Expr> {
        super::children(self).nth(1usize)
    }
    pub fn op_details(&self) -> Option<(SyntaxToken, BinOp)> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .find_map(|tok| BinOp::from_token(&tok).map(|ty| (tok, ty)))
    }
    pub fn op_kind(&self) -> Option<BinOp> {
        self.op_details().map(|t| t.1)
    }
    pub fn op_token(&self) -> Option<SyntaxToken> {
        self.op_details().map(|t| t.0)
    }
}
pub trait BinExprMake: ExprMake + AstMake {
    fn lh<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
    fn op(self, ts: BinOp) -> Make<Self, BinOp>
    where
        Self: Sized,
    {
        Make::new(self, ts, None)
    }
    fn rh<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: BinExprMake, B: AstMake> BinExprMake for Make<A, B> {}
pub struct BinExprBase {}
impl BinExprMake for BinExprBase {}
impl ExprMake for BinExprBase {}
impl AstMake for BinExprBase {
    type Node = BinExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::BIN_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl BinExpr {
    pub fn new() -> BinExprBase {
        BinExprBase {}
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
pub trait MacroCallMake: ExprMake + AstMake {
    fn token_tree<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TokenTreeMake,
    {
        Make::new(self, b, None)
    }
    fn path<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PathMake,
    {
        Make::new(self, b, None)
    }
    fn name<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: MacroCallMake, B: AstMake> MacroCallMake for Make<A, B> {}
pub struct MacroCallBase {}
impl MacroCallMake for MacroCallBase {}
impl ExprMake for MacroCallBase {}
impl AstMake for MacroCallBase {
    type Node = MacroCall;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MACRO_CALL);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl MacroCall {
    pub fn new() -> MacroCallBase {
        MacroCallBase {}
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImplItem {
    FnDef(FnDef),
    TypeAliasDef(TypeAliasDef),
    ConstDef(ConstDef),
}
pub trait ImplItemMake: AstMake {}
impl<A: ImplItemMake, B: AstMake> ImplItemMake for Make<A, B> {}
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
pub trait ModuleItemMake: AstMake {}
impl<A: ModuleItemMake, B: AstMake> ModuleItemMake for Make<A, B> {}
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
pub trait TraitDefMake: ModuleItemMake + AstMake {
    fn item_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ItemListMake,
    {
        Make::new(self, b, None)
    }
    fn visibility<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: VisibilityMake,
    {
        Make::new(self, b, None)
    }
    fn name<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
    fn type_param_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeParamListMake,
    {
        Make::new(self, b, None)
    }
    fn where_clause<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: WhereClauseMake,
    {
        Make::new(self, b, None)
    }
    fn type_bound_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeBoundListMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: TraitDefMake, B: AstMake> TraitDefMake for Make<A, B> {}
pub struct TraitDefBase {}
impl TraitDefMake for TraitDefBase {}
impl ModuleItemMake for TraitDefBase {}
impl AstMake for TraitDefBase {
    type Node = TraitDef;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TRAIT_DEF);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl TraitDef {
    pub fn new() -> TraitDefBase {
        TraitDefBase {}
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
pub trait ImplBlockMake: ModuleItemMake + AstMake {
    fn item_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ItemListMake,
    {
        Make::new(self, b, None)
    }
    fn type_param_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeParamListMake,
    {
        Make::new(self, b, None)
    }
    fn where_clause<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: WhereClauseMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ImplBlockMake, B: AstMake> ImplBlockMake for Make<A, B> {}
pub struct ImplBlockBase {}
impl ImplBlockMake for ImplBlockBase {}
impl ModuleItemMake for ImplBlockBase {}
impl AstMake for ImplBlockBase {
    type Node = ImplBlock;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::IMPL_BLOCK);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl ImplBlock {
    pub fn new() -> ImplBlockBase {
        ImplBlockBase {}
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
pub trait UseItemMake: ModuleItemMake + AstMake {
    fn use_tree<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: UseTreeMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: UseItemMake, B: AstMake> UseItemMake for Make<A, B> {}
pub struct UseItemBase {}
impl UseItemMake for UseItemBase {}
impl ModuleItemMake for UseItemBase {}
impl AstMake for UseItemBase {
    type Node = UseItem;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::USE_ITEM);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl UseItem {
    pub fn new() -> UseItemBase {
        UseItemBase {}
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
pub trait ExternCrateItemMake: ModuleItemMake + AstMake {
    fn name_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameRefMake,
    {
        Make::new(self, b, None)
    }
    fn alia<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AliasMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ExternCrateItemMake, B: AstMake> ExternCrateItemMake for Make<A, B> {}
pub struct ExternCrateItemBase {}
impl ExternCrateItemMake for ExternCrateItemBase {}
impl ModuleItemMake for ExternCrateItemBase {}
impl AstMake for ExternCrateItemBase {
    type Node = ExternCrateItem;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::EXTERN_CRATE_ITEM);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl ExternCrateItem {
    pub fn new() -> ExternCrateItemBase {
        ExternCrateItemBase {}
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
pub trait StaticDefMake: ModuleItemMake + AstMake {
    fn body<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
    fn visibility<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: VisibilityMake,
    {
        Make::new(self, b, None)
    }
    fn name<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameMake,
    {
        Make::new(self, b, None)
    }
    fn type_param_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeParamListMake,
    {
        Make::new(self, b, None)
    }
    fn where_clause<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: WhereClauseMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
    fn ascribed_type<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: StaticDefMake, B: AstMake> StaticDefMake for Make<A, B> {}
pub struct StaticDefBase {}
impl StaticDefMake for StaticDefBase {}
impl ModuleItemMake for StaticDefBase {}
impl AstMake for StaticDefBase {
    type Node = StaticDef;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::STATIC_DEF);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl StaticDef {
    pub fn new() -> StaticDefBase {
        StaticDefBase {}
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
pub trait ModuleMake: ModuleItemMake + AstMake {
    fn item_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ItemListMake,
    {
        Make::new(self, b, None)
    }
    fn visibility<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: VisibilityMake,
    {
        Make::new(self, b, None)
    }
    fn name<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ModuleMake, B: AstMake> ModuleMake for Make<A, B> {}
pub struct ModuleBase {}
impl ModuleMake for ModuleBase {}
impl ModuleItemMake for ModuleBase {}
impl AstMake for ModuleBase {
    type Node = Module;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MODULE);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl Module {
    pub fn new() -> ModuleBase {
        ModuleBase {}
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NominalDef {
    StructDef(StructDef),
    UnionDef(UnionDef),
    EnumDef(EnumDef),
}
pub trait NominalDefMake: AstMake {}
impl<A: NominalDefMake, B: AstMake> NominalDefMake for Make<A, B> {}
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
pub trait BlockExprMake: ExprMake + AstMake {
    fn block<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: BlockMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: BlockExprMake, B: AstMake> BlockExprMake for Make<A, B> {}
pub struct BlockExprBase {}
impl BlockExprMake for BlockExprBase {}
impl ExprMake for BlockExprBase {}
impl AstMake for BlockExprBase {
    type Node = BlockExpr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::BLOCK_EXPR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl BlockExpr {
    pub fn new() -> BlockExprBase {
        BlockExprBase {}
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
pub trait StructDefMake: ModuleItemMake + NominalDefMake + AstMake {
    fn visibility<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: VisibilityMake,
    {
        Make::new(self, b, None)
    }
    fn name<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameMake,
    {
        Make::new(self, b, None)
    }
    fn type_param_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeParamListMake,
    {
        Make::new(self, b, None)
    }
    fn where_clause<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: WhereClauseMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: StructDefMake, B: AstMake> StructDefMake for Make<A, B> {}
pub struct StructDefBase {}
impl StructDefMake for StructDefBase {}
impl ModuleItemMake for StructDefBase {}
impl NominalDefMake for StructDefBase {}
impl AstMake for StructDefBase {
    type Node = StructDef;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::STRUCT_DEF);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl StructDef {
    pub fn new() -> StructDefBase {
        StructDefBase {}
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
pub trait UnionDefMake: ModuleItemMake + NominalDefMake + AstMake {
    fn visibility<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: VisibilityMake,
    {
        Make::new(self, b, None)
    }
    fn name<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameMake,
    {
        Make::new(self, b, None)
    }
    fn type_param_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeParamListMake,
    {
        Make::new(self, b, None)
    }
    fn where_clause<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: WhereClauseMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
    fn record_field_def_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: RecordFieldDefListMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: UnionDefMake, B: AstMake> UnionDefMake for Make<A, B> {}
pub struct UnionDefBase {}
impl UnionDefMake for UnionDefBase {}
impl ModuleItemMake for UnionDefBase {}
impl NominalDefMake for UnionDefBase {}
impl AstMake for UnionDefBase {
    type Node = UnionDef;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::UNION_DEF);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl UnionDef {
    pub fn new() -> UnionDefBase {
        UnionDefBase {}
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
pub trait EnumDefMake: ModuleItemMake + NominalDefMake + AstMake {
    fn variant_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: EnumVariantListMake,
    {
        Make::new(self, b, None)
    }
    fn visibility<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: VisibilityMake,
    {
        Make::new(self, b, None)
    }
    fn name<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameMake,
    {
        Make::new(self, b, None)
    }
    fn type_param_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeParamListMake,
    {
        Make::new(self, b, None)
    }
    fn where_clause<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: WhereClauseMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: EnumDefMake, B: AstMake> EnumDefMake for Make<A, B> {}
pub struct EnumDefBase {}
impl EnumDefMake for EnumDefBase {}
impl ModuleItemMake for EnumDefBase {}
impl NominalDefMake for EnumDefBase {}
impl AstMake for EnumDefBase {
    type Node = EnumDef;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ENUM_DEF);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl EnumDef {
    pub fn new() -> EnumDefBase {
        EnumDefBase {}
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
pub trait FnDefMake: ImplItemMake + ModuleItemMake + AstMake {
    fn param_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ParamListMake,
    {
        Make::new(self, b, None)
    }
    fn body<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: BlockExprMake,
    {
        Make::new(self, b, None)
    }
    fn ret_type<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: RetTypeMake,
    {
        Make::new(self, b, None)
    }
    fn visibility<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: VisibilityMake,
    {
        Make::new(self, b, None)
    }
    fn name<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameMake,
    {
        Make::new(self, b, None)
    }
    fn type_param_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeParamListMake,
    {
        Make::new(self, b, None)
    }
    fn where_clause<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: WhereClauseMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: FnDefMake, B: AstMake> FnDefMake for Make<A, B> {}
pub struct FnDefBase {}
impl FnDefMake for FnDefBase {}
impl ImplItemMake for FnDefBase {}
impl ModuleItemMake for FnDefBase {}
impl AstMake for FnDefBase {
    type Node = FnDef;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::FN_DEF);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl FnDef {
    pub fn new() -> FnDefBase {
        FnDefBase {}
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
pub trait TypeAliasDefMake: ImplItemMake + ModuleItemMake + AstMake {
    fn type_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
    fn visibility<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: VisibilityMake,
    {
        Make::new(self, b, None)
    }
    fn name<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameMake,
    {
        Make::new(self, b, None)
    }
    fn type_param_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeParamListMake,
    {
        Make::new(self, b, None)
    }
    fn where_clause<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: WhereClauseMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
    fn type_bound_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeBoundListMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: TypeAliasDefMake, B: AstMake> TypeAliasDefMake for Make<A, B> {}
pub struct TypeAliasDefBase {}
impl TypeAliasDefMake for TypeAliasDefBase {}
impl ImplItemMake for TypeAliasDefBase {}
impl ModuleItemMake for TypeAliasDefBase {}
impl AstMake for TypeAliasDefBase {
    type Node = TypeAliasDef;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TYPE_ALIAS_DEF);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl TypeAliasDef {
    pub fn new() -> TypeAliasDefBase {
        TypeAliasDefBase {}
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
pub trait ConstDefMake: ImplItemMake + ModuleItemMake + AstMake {
    fn body<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
    fn visibility<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: VisibilityMake,
    {
        Make::new(self, b, None)
    }
    fn name<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameMake,
    {
        Make::new(self, b, None)
    }
    fn type_param_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeParamListMake,
    {
        Make::new(self, b, None)
    }
    fn where_clause<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: WhereClauseMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
    fn ascribed_type<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ConstDefMake, B: AstMake> ConstDefMake for Make<A, B> {}
pub struct ConstDefBase {}
impl ConstDefMake for ConstDefBase {}
impl ImplItemMake for ConstDefBase {}
impl ModuleItemMake for ConstDefBase {}
impl AstMake for ConstDefBase {
    type Node = ConstDef;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::CONST_DEF);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl ConstDef {
    pub fn new() -> ConstDefBase {
        ConstDefBase {}
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
pub trait PatMake: AstMake {}
impl<A: PatMake, B: AstMake> PatMake for Make<A, B> {}
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
pub trait RefPatMake: PatMake + AstMake {
    fn pat<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PatMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: RefPatMake, B: AstMake> RefPatMake for Make<A, B> {}
pub struct RefPatBase {}
impl RefPatMake for RefPatBase {}
impl PatMake for RefPatBase {}
impl AstMake for RefPatBase {
    type Node = RefPat;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::REF_PAT);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl RefPat {
    pub fn new() -> RefPatBase {
        RefPatBase {}
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
pub trait BoxPatMake: PatMake + AstMake {
    fn pat<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PatMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: BoxPatMake, B: AstMake> BoxPatMake for Make<A, B> {}
pub struct BoxPatBase {}
impl BoxPatMake for BoxPatBase {}
impl PatMake for BoxPatBase {}
impl AstMake for BoxPatBase {
    type Node = BoxPat;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::BOX_PAT);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl BoxPat {
    pub fn new() -> BoxPatBase {
        BoxPatBase {}
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
pub trait PathPatMake: PatMake + AstMake {
    fn path<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PathMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: PathPatMake, B: AstMake> PathPatMake for Make<A, B> {}
pub struct PathPatBase {}
impl PathPatMake for PathPatBase {}
impl PatMake for PathPatBase {}
impl AstMake for PathPatBase {
    type Node = PathPat;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PATH_PAT);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl PathPat {
    pub fn new() -> PathPatBase {
        PathPatBase {}
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
pub trait RecordPatMake: PatMake + AstMake {
    fn record_field_pat_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: RecordFieldPatListMake,
    {
        Make::new(self, b, None)
    }
    fn path<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PathMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: RecordPatMake, B: AstMake> RecordPatMake for Make<A, B> {}
pub struct RecordPatBase {}
impl RecordPatMake for RecordPatBase {}
impl PatMake for RecordPatBase {}
impl AstMake for RecordPatBase {
    type Node = RecordPat;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_PAT);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl RecordPat {
    pub fn new() -> RecordPatBase {
        RecordPatBase {}
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
pub trait TupleStructPatMake: PatMake + AstMake {
    fn arg<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PatMake,
    {
        Make::new(self, b, None)
    }
    fn path<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PathMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: TupleStructPatMake, B: AstMake> TupleStructPatMake for Make<A, B> {}
pub struct TupleStructPatBase {}
impl TupleStructPatMake for TupleStructPatBase {}
impl PatMake for TupleStructPatBase {}
impl AstMake for TupleStructPatBase {
    type Node = TupleStructPat;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TUPLE_STRUCT_PAT);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl TupleStructPat {
    pub fn new() -> TupleStructPatBase {
        TupleStructPatBase {}
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
pub trait TuplePatMake: PatMake + AstMake {
    fn arg<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PatMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: TuplePatMake, B: AstMake> TuplePatMake for Make<A, B> {}
pub struct TuplePatBase {}
impl TuplePatMake for TuplePatBase {}
impl PatMake for TuplePatBase {}
impl AstMake for TuplePatBase {
    type Node = TuplePat;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TUPLE_PAT);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl TuplePat {
    pub fn new() -> TuplePatBase {
        TuplePatBase {}
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
pub trait LiteralPatMake: PatMake + AstMake {
    fn literal<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: LiteralMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: LiteralPatMake, B: AstMake> LiteralPatMake for Make<A, B> {}
pub struct LiteralPatBase {}
impl LiteralPatMake for LiteralPatBase {}
impl PatMake for LiteralPatBase {}
impl AstMake for LiteralPatBase {
    type Node = LiteralPat;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::LITERAL_PAT);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl LiteralPat {
    pub fn new() -> LiteralPatBase {
        LiteralPatBase {}
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stmt {
    ExprStmt(ExprStmt),
    LetStmt(LetStmt),
}
pub trait StmtMake: AstMake {}
impl<A: StmtMake, B: AstMake> StmtMake for Make<A, B> {}
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
pub trait ExprStmtMake: StmtMake + AstMake {
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ExprStmtMake, B: AstMake> ExprStmtMake for Make<A, B> {}
pub struct ExprStmtBase {}
impl ExprStmtMake for ExprStmtBase {}
impl StmtMake for ExprStmtBase {}
impl AstMake for ExprStmtBase {
    type Node = ExprStmt;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::EXPR_STMT);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl ExprStmt {
    pub fn new() -> ExprStmtBase {
        ExprStmtBase {}
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
pub trait LetStmtMake: StmtMake + AstMake {
    fn pat<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PatMake,
    {
        Make::new(self, b, None)
    }
    fn initializer<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
    fn ascribed_type<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: LetStmtMake, B: AstMake> LetStmtMake for Make<A, B> {}
pub struct LetStmtBase {}
impl LetStmtMake for LetStmtBase {}
impl StmtMake for LetStmtBase {}
impl AstMake for LetStmtBase {
    type Node = LetStmt;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::LET_STMT);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl LetStmt {
    pub fn new() -> LetStmtBase {
        LetStmtBase {}
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
pub trait TypeRefMake: AstMake {}
impl<A: TypeRefMake, B: AstMake> TypeRefMake for Make<A, B> {}
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
pub trait ParenTypeMake: TypeRefMake + AstMake {
    fn type_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ParenTypeMake, B: AstMake> ParenTypeMake for Make<A, B> {}
pub struct ParenTypeBase {}
impl ParenTypeMake for ParenTypeBase {}
impl TypeRefMake for ParenTypeBase {}
impl AstMake for ParenTypeBase {
    type Node = ParenType;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PAREN_TYPE);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl ParenType {
    pub fn new() -> ParenTypeBase {
        ParenTypeBase {}
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
pub trait TupleTypeMake: TypeRefMake + AstMake {
    fn field<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: TupleTypeMake, B: AstMake> TupleTypeMake for Make<A, B> {}
pub struct TupleTypeBase {}
impl TupleTypeMake for TupleTypeBase {}
impl TypeRefMake for TupleTypeBase {}
impl AstMake for TupleTypeBase {
    type Node = TupleType;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TUPLE_TYPE);
        builder.token(SyntaxKind::L_PAREN, SmolStr::new(T_STR!(L_PAREN)));
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.token(SyntaxKind::R_PAREN, SmolStr::new(T_STR!(R_PAREN)));
        builder.finish_node();
    }
}
impl TupleType {
    pub fn new() -> TupleTypeBase {
        TupleTypeBase {}
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
pub trait PointerTypeMake: TypeRefMake + AstMake {
    fn type_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: PointerTypeMake, B: AstMake> PointerTypeMake for Make<A, B> {}
pub struct PointerTypeBase {}
impl PointerTypeMake for PointerTypeBase {}
impl TypeRefMake for PointerTypeBase {}
impl AstMake for PointerTypeBase {
    type Node = PointerType;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::POINTER_TYPE);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl PointerType {
    pub fn new() -> PointerTypeBase {
        PointerTypeBase {}
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
pub trait ArrayTypeMake: TypeRefMake + AstMake {
    fn type_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ArrayTypeMake, B: AstMake> ArrayTypeMake for Make<A, B> {}
pub struct ArrayTypeBase {}
impl ArrayTypeMake for ArrayTypeBase {}
impl TypeRefMake for ArrayTypeBase {}
impl AstMake for ArrayTypeBase {
    type Node = ArrayType;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ARRAY_TYPE);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl ArrayType {
    pub fn new() -> ArrayTypeBase {
        ArrayTypeBase {}
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
pub trait SliceTypeMake: TypeRefMake + AstMake {
    fn type_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: SliceTypeMake, B: AstMake> SliceTypeMake for Make<A, B> {}
pub struct SliceTypeBase {}
impl SliceTypeMake for SliceTypeBase {}
impl TypeRefMake for SliceTypeBase {}
impl AstMake for SliceTypeBase {
    type Node = SliceType;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::SLICE_TYPE);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl SliceType {
    pub fn new() -> SliceTypeBase {
        SliceTypeBase {}
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
pub trait ReferenceTypeMake: TypeRefMake + AstMake {
    fn type_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ReferenceTypeMake, B: AstMake> ReferenceTypeMake for Make<A, B> {}
pub struct ReferenceTypeBase {}
impl ReferenceTypeMake for ReferenceTypeBase {}
impl TypeRefMake for ReferenceTypeBase {}
impl AstMake for ReferenceTypeBase {
    type Node = ReferenceType;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::REFERENCE_TYPE);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl ReferenceType {
    pub fn new() -> ReferenceTypeBase {
        ReferenceTypeBase {}
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
pub trait FnPointerTypeMake: TypeRefMake + AstMake {
    fn param_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ParamListMake,
    {
        Make::new(self, b, None)
    }
    fn ret_type<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: RetTypeMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: FnPointerTypeMake, B: AstMake> FnPointerTypeMake for Make<A, B> {}
pub struct FnPointerTypeBase {}
impl FnPointerTypeMake for FnPointerTypeBase {}
impl TypeRefMake for FnPointerTypeBase {}
impl AstMake for FnPointerTypeBase {
    type Node = FnPointerType;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::FN_POINTER_TYPE);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl FnPointerType {
    pub fn new() -> FnPointerTypeBase {
        FnPointerTypeBase {}
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
pub trait ForTypeMake: TypeRefMake + AstMake {
    fn type_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ForTypeMake, B: AstMake> ForTypeMake for Make<A, B> {}
pub struct ForTypeBase {}
impl ForTypeMake for ForTypeBase {}
impl TypeRefMake for ForTypeBase {}
impl AstMake for ForTypeBase {
    type Node = ForType;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::FOR_TYPE);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl ForType {
    pub fn new() -> ForTypeBase {
        ForTypeBase {}
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
pub trait ImplTraitTypeMake: TypeRefMake + AstMake {
    fn type_bound_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeBoundListMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ImplTraitTypeMake, B: AstMake> ImplTraitTypeMake for Make<A, B> {}
pub struct ImplTraitTypeBase {}
impl ImplTraitTypeMake for ImplTraitTypeBase {}
impl TypeRefMake for ImplTraitTypeBase {}
impl AstMake for ImplTraitTypeBase {
    type Node = ImplTraitType;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::IMPL_TRAIT_TYPE);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl ImplTraitType {
    pub fn new() -> ImplTraitTypeBase {
        ImplTraitTypeBase {}
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
pub trait DynTraitTypeMake: TypeRefMake + AstMake {
    fn type_bound_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeBoundListMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: DynTraitTypeMake, B: AstMake> DynTraitTypeMake for Make<A, B> {}
pub struct DynTraitTypeBase {}
impl DynTraitTypeMake for DynTraitTypeBase {}
impl TypeRefMake for DynTraitTypeBase {}
impl AstMake for DynTraitTypeBase {
    type Node = DynTraitType;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::DYN_TRAIT_TYPE);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl DynTraitType {
    pub fn new() -> DynTraitTypeBase {
        DynTraitTypeBase {}
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttrInput {
    Literal(Literal),
    TokenTree(TokenTree),
}
pub trait AttrInputMake: AstMake {}
impl<A: AttrInputMake, B: AstMake> AttrInputMake for Make<A, B> {}
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
pub trait AliasMake: AstMake {
    fn name<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: AliasMake, B: AstMake> AliasMake for Make<A, B> {}
pub struct AliasBase {}
impl AliasMake for AliasBase {}
impl AstMake for AliasBase {
    type Node = Alias;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ALIAS);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl Alias {
    pub fn new() -> AliasBase {
        AliasBase {}
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
pub trait ArgListMake: AstMake {
    fn arg<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ArgListMake, B: AstMake> ArgListMake for Make<A, B> {}
pub struct ArgListBase {}
impl ArgListMake for ArgListBase {}
impl AstMake for ArgListBase {
    type Node = ArgList;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ARG_LIST);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl ArgList {
    pub fn new() -> ArgListBase {
        ArgListBase {}
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
pub trait BindPatMake: PatMake + AstMake {
    fn pat<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PatMake,
    {
        Make::new(self, b, None)
    }
    fn name<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: BindPatMake, B: AstMake> BindPatMake for Make<A, B> {}
pub struct BindPatBase {}
impl BindPatMake for BindPatBase {}
impl PatMake for BindPatBase {}
impl AstMake for BindPatBase {
    type Node = BindPat;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::BIND_PAT);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl BindPat {
    pub fn new() -> BindPatBase {
        BindPatBase {}
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
pub trait AssocTypeArgMake: AstMake {
    fn name_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameRefMake,
    {
        Make::new(self, b, None)
    }
    fn type_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: AssocTypeArgMake, B: AstMake> AssocTypeArgMake for Make<A, B> {}
pub struct AssocTypeArgBase {}
impl AssocTypeArgMake for AssocTypeArgBase {}
impl AstMake for AssocTypeArgBase {
    type Node = AssocTypeArg;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ASSOC_TYPE_ARG);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl AssocTypeArg {
    pub fn new() -> AssocTypeArgBase {
        AssocTypeArgBase {}
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
pub trait AttrMake: AstMake {
    fn path<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PathMake,
    {
        Make::new(self, b, None)
    }
    fn input<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrInputMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: AttrMake, B: AstMake> AttrMake for Make<A, B> {}
pub struct AttrBase {}
impl AttrMake for AttrBase {}
impl AstMake for AttrBase {
    type Node = Attr;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ATTR);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl Attr {
    pub fn new() -> AttrBase {
        AttrBase {}
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
pub trait BlockMake: AstMake {
    fn statement<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: StmtMake,
    {
        Make::new(self, b, None)
    }
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: BlockMake, B: AstMake> BlockMake for Make<A, B> {}
pub struct BlockBase {}
impl BlockMake for BlockBase {}
impl AstMake for BlockBase {
    type Node = Block;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::BLOCK);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl Block {
    pub fn new() -> BlockBase {
        BlockBase {}
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
pub trait ConditionMake: ExprMake + AstMake {
    fn pat<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PatMake,
    {
        Make::new(self, b, None)
    }
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ConditionMake, B: AstMake> ConditionMake for Make<A, B> {}
pub struct ConditionBase {}
impl ConditionMake for ConditionBase {}
impl ExprMake for ConditionBase {}
impl AstMake for ConditionBase {
    type Node = Condition;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::CONDITION);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl Condition {
    pub fn new() -> ConditionBase {
        ConditionBase {}
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
pub trait EnumVariantMake: AstMake {
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
    fn name<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: EnumVariantMake, B: AstMake> EnumVariantMake for Make<A, B> {}
pub struct EnumVariantBase {}
impl EnumVariantMake for EnumVariantBase {}
impl AstMake for EnumVariantBase {
    type Node = EnumVariant;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ENUM_VARIANT);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl EnumVariant {
    pub fn new() -> EnumVariantBase {
        EnumVariantBase {}
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
pub trait EnumVariantListMake: AstMake {
    fn variant<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: EnumVariantMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: EnumVariantListMake, B: AstMake> EnumVariantListMake for Make<A, B> {}
pub struct EnumVariantListBase {}
impl EnumVariantListMake for EnumVariantListBase {}
impl AstMake for EnumVariantListBase {
    type Node = EnumVariantList;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ENUM_VARIANT_LIST);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl EnumVariantList {
    pub fn new() -> EnumVariantListBase {
        EnumVariantListBase {}
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
pub trait RecordFieldPatMake: AstMake {
    fn pat<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PatMake,
    {
        Make::new(self, b, None)
    }
    fn name<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: RecordFieldPatMake, B: AstMake> RecordFieldPatMake for Make<A, B> {}
pub struct RecordFieldPatBase {}
impl RecordFieldPatMake for RecordFieldPatBase {}
impl AstMake for RecordFieldPatBase {
    type Node = RecordFieldPat;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_FIELD_PAT);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl RecordFieldPat {
    pub fn new() -> RecordFieldPatBase {
        RecordFieldPatBase {}
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
pub trait RecordFieldPatListMake: AstMake {
    fn record_field_pat<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: RecordFieldPatMake,
    {
        Make::new(self, b, None)
    }
    fn bind_pat<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: BindPatMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: RecordFieldPatListMake, B: AstMake> RecordFieldPatListMake for Make<A, B> {}
pub struct RecordFieldPatListBase {}
impl RecordFieldPatListMake for RecordFieldPatListBase {}
impl AstMake for RecordFieldPatListBase {
    type Node = RecordFieldPatList;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_FIELD_PAT_LIST);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl RecordFieldPatList {
    pub fn new() -> RecordFieldPatListBase {
        RecordFieldPatListBase {}
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
pub trait ItemListMake: AstMake {
    fn impl_item<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ImplItemMake,
    {
        Make::new(self, b, None)
    }
    fn function<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: FnDefMake,
    {
        Make::new(self, b, None)
    }
    fn item<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ModuleItemMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ItemListMake, B: AstMake> ItemListMake for Make<A, B> {}
pub struct ItemListBase {}
impl ItemListMake for ItemListBase {}
impl AstMake for ItemListBase {
    type Node = ItemList;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ITEM_LIST);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl ItemList {
    pub fn new() -> ItemListBase {
        ItemListBase {}
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
pub trait LifetimeParamMake: AstMake {
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: LifetimeParamMake, B: AstMake> LifetimeParamMake for Make<A, B> {}
pub struct LifetimeParamBase {}
impl LifetimeParamMake for LifetimeParamBase {}
impl AstMake for LifetimeParamBase {
    type Node = LifetimeParam;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::LIFETIME_PARAM);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl LifetimeParam {
    pub fn new() -> LifetimeParamBase {
        LifetimeParamBase {}
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
pub trait MacroItemsMake: AstMake {
    fn item<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ModuleItemMake,
    {
        Make::new(self, b, None)
    }
    fn function<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: FnDefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: MacroItemsMake, B: AstMake> MacroItemsMake for Make<A, B> {}
pub struct MacroItemsBase {}
impl MacroItemsMake for MacroItemsBase {}
impl AstMake for MacroItemsBase {
    type Node = MacroItems;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MACRO_ITEMS);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl MacroItems {
    pub fn new() -> MacroItemsBase {
        MacroItemsBase {}
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
pub trait MacroStmtsMake: AstMake {
    fn statement<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: StmtMake,
    {
        Make::new(self, b, None)
    }
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: MacroStmtsMake, B: AstMake> MacroStmtsMake for Make<A, B> {}
pub struct MacroStmtsBase {}
impl MacroStmtsMake for MacroStmtsBase {}
impl AstMake for MacroStmtsBase {
    type Node = MacroStmts;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MACRO_STMTS);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl MacroStmts {
    pub fn new() -> MacroStmtsBase {
        MacroStmtsBase {}
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
pub trait MatchArmMake: AstMake {
    fn pat<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PatMake,
    {
        Make::new(self, b, None)
    }
    fn guard<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: MatchGuardMake,
    {
        Make::new(self, b, None)
    }
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: MatchArmMake, B: AstMake> MatchArmMake for Make<A, B> {}
pub struct MatchArmBase {}
impl MatchArmMake for MatchArmBase {}
impl AstMake for MatchArmBase {
    type Node = MatchArm;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MATCH_ARM);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl MatchArm {
    pub fn new() -> MatchArmBase {
        MatchArmBase {}
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
pub trait MatchArmListMake: AstMake {
    fn arm<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: MatchArmMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: MatchArmListMake, B: AstMake> MatchArmListMake for Make<A, B> {}
pub struct MatchArmListBase {}
impl MatchArmListMake for MatchArmListBase {}
impl AstMake for MatchArmListBase {
    type Node = MatchArmList;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MATCH_ARM_LIST);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl MatchArmList {
    pub fn new() -> MatchArmListBase {
        MatchArmListBase {}
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
pub trait MatchGuardMake: AstMake {
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: MatchGuardMake, B: AstMake> MatchGuardMake for Make<A, B> {}
pub struct MatchGuardBase {}
impl MatchGuardMake for MatchGuardBase {}
impl AstMake for MatchGuardBase {
    type Node = MatchGuard;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MATCH_GUARD);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl MatchGuard {
    pub fn new() -> MatchGuardBase {
        MatchGuardBase {}
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
pub trait RecordFieldMake: AstMake {
    fn name_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameRefMake,
    {
        Make::new(self, b, None)
    }
    fn expr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: RecordFieldMake, B: AstMake> RecordFieldMake for Make<A, B> {}
pub struct RecordFieldBase {}
impl RecordFieldMake for RecordFieldBase {}
impl AstMake for RecordFieldBase {
    type Node = RecordField;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_FIELD);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl RecordField {
    pub fn new() -> RecordFieldBase {
        RecordFieldBase {}
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
pub trait RecordFieldDefMake: AstMake {
    fn visibility<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: VisibilityMake,
    {
        Make::new(self, b, None)
    }
    fn name<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameMake,
    {
        Make::new(self, b, Some(TokenMake::new(SyntaxKind::COLON, T_STR!(COLON))))
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
    fn ascribed_type<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: RecordFieldDefMake, B: AstMake> RecordFieldDefMake for Make<A, B> {}
pub struct RecordFieldDefBase {}
impl RecordFieldDefMake for RecordFieldDefBase {}
impl AstMake for RecordFieldDefBase {
    type Node = RecordFieldDef;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_FIELD_DEF);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl RecordFieldDef {
    pub fn new() -> RecordFieldDefBase {
        RecordFieldDefBase {}
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
pub trait RecordFieldDefListMake: AstMake {
    fn field<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: RecordFieldDefMake,
    {
        Make::new(self, b, Some(TokenMake::new(SyntaxKind::COMMA, T_STR!(COMMA))))
    }
}
impl<A: RecordFieldDefListMake, B: AstMake> RecordFieldDefListMake for Make<A, B> {}
pub struct RecordFieldDefListBase {}
impl RecordFieldDefListMake for RecordFieldDefListBase {}
impl AstMake for RecordFieldDefListBase {
    type Node = RecordFieldDefList;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_FIELD_DEF_LIST);
        builder.token(SyntaxKind::L_CURLY, SmolStr::new(T_STR!(L_CURLY)));
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.token(SyntaxKind::R_CURLY, SmolStr::new(T_STR!(R_CURLY)));
        builder.finish_node();
    }
}
impl RecordFieldDefList {
    pub fn new() -> RecordFieldDefListBase {
        RecordFieldDefListBase {}
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
pub trait RecordFieldListMake: AstMake {
    fn field<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: RecordFieldMake,
    {
        Make::new(self, b, None)
    }
    fn spread<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ExprMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: RecordFieldListMake, B: AstMake> RecordFieldListMake for Make<A, B> {}
pub struct RecordFieldListBase {}
impl RecordFieldListMake for RecordFieldListBase {}
impl AstMake for RecordFieldListBase {
    type Node = RecordFieldList;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_FIELD_LIST);
        builder.token(SyntaxKind::L_CURLY, SmolStr::new(T_STR!(L_CURLY)));
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.token(SyntaxKind::R_CURLY, SmolStr::new(T_STR!(R_CURLY)));
        builder.finish_node();
    }
}
impl RecordFieldList {
    pub fn new() -> RecordFieldListBase {
        RecordFieldListBase {}
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
pub trait ParamMake: AstMake {
    fn pat<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PatMake,
    {
        Make::new(self, b, None)
    }
    fn ascribed_type<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ParamMake, B: AstMake> ParamMake for Make<A, B> {}
pub struct ParamBase {}
impl ParamMake for ParamBase {}
impl AstMake for ParamBase {
    type Node = Param;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PARAM);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl Param {
    pub fn new() -> ParamBase {
        ParamBase {}
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
pub trait ParamListMake: AstMake {
    fn param<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ParamMake,
    {
        Make::new(self, b, None)
    }
    fn self_param<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: SelfParamMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: ParamListMake, B: AstMake> ParamListMake for Make<A, B> {}
pub struct ParamListBase {}
impl ParamListMake for ParamListBase {}
impl AstMake for ParamListBase {
    type Node = ParamList;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PARAM_LIST);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl ParamList {
    pub fn new() -> ParamListBase {
        ParamListBase {}
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
pub trait PathTypeMake: TypeRefMake + AstMake {
    fn path<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PathMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: PathTypeMake, B: AstMake> PathTypeMake for Make<A, B> {}
pub struct PathTypeBase {}
impl PathTypeMake for PathTypeBase {}
impl TypeRefMake for PathTypeBase {}
impl AstMake for PathTypeBase {
    type Node = PathType;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PATH_TYPE);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl PathType {
    pub fn new() -> PathTypeBase {
        PathTypeBase {}
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
pub trait PathMake: ExprMake + PatMake + AstMake {
    fn segment<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PathSegmentMake,
    {
        Make::new(self, b, None)
    }
    fn qualifier<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PathMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: PathMake, B: AstMake> PathMake for Make<A, B> {}
pub struct PathBase {}
impl PathMake for PathBase {}
impl ExprMake for PathBase {}
impl PatMake for PathBase {}
impl AstMake for PathBase {
    type Node = Path;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PATH);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl Path {
    pub fn new() -> PathBase {
        PathBase {}
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
pub trait PathSegmentMake: AstMake {
    fn name_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameRefMake,
    {
        Make::new(self, b, None)
    }
    fn type_arg_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeArgListMake,
    {
        Make::new(self, b, None)
    }
    fn param_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ParamListMake,
    {
        Make::new(self, b, None)
    }
    fn ret_type<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: RetTypeMake,
    {
        Make::new(self, b, None)
    }
    fn path_type<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PathTypeMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: PathSegmentMake, B: AstMake> PathSegmentMake for Make<A, B> {}
pub struct PathSegmentBase {}
impl PathSegmentMake for PathSegmentBase {}
impl AstMake for PathSegmentBase {
    type Node = PathSegment;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PATH_SEGMENT);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl PathSegment {
    pub fn new() -> PathSegmentBase {
        PathSegmentBase {}
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
pub trait TupleFieldDefMake: AstMake {
    fn type_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
    fn visibility<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: VisibilityMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: TupleFieldDefMake, B: AstMake> TupleFieldDefMake for Make<A, B> {}
pub struct TupleFieldDefBase {}
impl TupleFieldDefMake for TupleFieldDefBase {}
impl AstMake for TupleFieldDefBase {
    type Node = TupleFieldDef;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TUPLE_FIELD_DEF);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl TupleFieldDef {
    pub fn new() -> TupleFieldDefBase {
        TupleFieldDefBase {}
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
pub trait TupleFieldDefListMake: AstMake {
    fn field<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TupleFieldDefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: TupleFieldDefListMake, B: AstMake> TupleFieldDefListMake for Make<A, B> {}
pub struct TupleFieldDefListBase {}
impl TupleFieldDefListMake for TupleFieldDefListBase {}
impl AstMake for TupleFieldDefListBase {
    type Node = TupleFieldDefList;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TUPLE_FIELD_DEF_LIST);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl TupleFieldDefList {
    pub fn new() -> TupleFieldDefListBase {
        TupleFieldDefListBase {}
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
pub trait RetTypeMake: AstMake {
    fn type_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: RetTypeMake, B: AstMake> RetTypeMake for Make<A, B> {}
pub struct RetTypeBase {}
impl RetTypeMake for RetTypeBase {}
impl AstMake for RetTypeBase {
    type Node = RetType;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RET_TYPE);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl RetType {
    pub fn new() -> RetTypeBase {
        RetTypeBase {}
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
pub trait SelfParamMake: AstMake {
    fn ascribed_type<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: SelfParamMake, B: AstMake> SelfParamMake for Make<A, B> {}
pub struct SelfParamBase {}
impl SelfParamMake for SelfParamBase {}
impl AstMake for SelfParamBase {
    type Node = SelfParam;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::SELF_PARAM);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl SelfParam {
    pub fn new() -> SelfParamBase {
        SelfParamBase {}
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
pub trait SourceFileMake: AstMake {
    fn item<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: ModuleItemMake,
    {
        Make::new(self, b, None)
    }
    fn function<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: FnDefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: SourceFileMake, B: AstMake> SourceFileMake for Make<A, B> {}
pub struct SourceFileBase {}
impl SourceFileMake for SourceFileBase {}
impl AstMake for SourceFileBase {
    type Node = SourceFile;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::SOURCE_FILE);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl SourceFile {
    pub fn new() -> SourceFileBase {
        SourceFileBase {}
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
pub trait TypeArgMake: AstMake {
    fn type_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: TypeArgMake, B: AstMake> TypeArgMake for Make<A, B> {}
pub struct TypeArgBase {}
impl TypeArgMake for TypeArgBase {}
impl AstMake for TypeArgBase {
    type Node = TypeArg;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TYPE_ARG);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl TypeArg {
    pub fn new() -> TypeArgBase {
        TypeArgBase {}
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
pub trait TypeArgListMake: AstMake {
    fn type_arg<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeArgMake,
    {
        Make::new(self, b, None)
    }
    fn lifetime_arg<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: LifetimeArgMake,
    {
        Make::new(self, b, None)
    }
    fn assoc_type_arg<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AssocTypeArgMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: TypeArgListMake, B: AstMake> TypeArgListMake for Make<A, B> {}
pub struct TypeArgListBase {}
impl TypeArgListMake for TypeArgListBase {}
impl AstMake for TypeArgListBase {
    type Node = TypeArgList;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TYPE_ARG_LIST);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl TypeArgList {
    pub fn new() -> TypeArgListBase {
        TypeArgListBase {}
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
pub trait TypeBoundMake: AstMake {
    fn type_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: TypeBoundMake, B: AstMake> TypeBoundMake for Make<A, B> {}
pub struct TypeBoundBase {}
impl TypeBoundMake for TypeBoundBase {}
impl AstMake for TypeBoundBase {
    type Node = TypeBound;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TYPE_BOUND);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl TypeBound {
    pub fn new() -> TypeBoundBase {
        TypeBoundBase {}
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
pub trait TypeBoundListMake: TypeRefMake + AstMake {
    fn bound<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeBoundMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: TypeBoundListMake, B: AstMake> TypeBoundListMake for Make<A, B> {}
pub struct TypeBoundListBase {}
impl TypeBoundListMake for TypeBoundListBase {}
impl TypeRefMake for TypeBoundListBase {}
impl AstMake for TypeBoundListBase {
    type Node = TypeBoundList;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TYPE_BOUND_LIST);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl TypeBoundList {
    pub fn new() -> TypeBoundListBase {
        TypeBoundListBase {}
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
pub trait TypeParamMake: AstMake {
    fn name<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: NameMake,
    {
        Make::new(self, b, None)
    }
    fn attr<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AttrMake,
    {
        Make::new(self, b, None)
    }
    fn type_bound_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeBoundListMake,
    {
        Make::new(self, b, None)
    }
    fn default_type<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: TypeParamMake, B: AstMake> TypeParamMake for Make<A, B> {}
pub struct TypeParamBase {}
impl TypeParamMake for TypeParamBase {}
impl AstMake for TypeParamBase {
    type Node = TypeParam;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TYPE_PARAM);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl TypeParam {
    pub fn new() -> TypeParamBase {
        TypeParamBase {}
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
pub trait TypeParamListMake: AstMake {
    fn type_param<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeParamMake,
    {
        Make::new(self, b, None)
    }
    fn lifetime_param<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: LifetimeParamMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: TypeParamListMake, B: AstMake> TypeParamListMake for Make<A, B> {}
pub struct TypeParamListBase {}
impl TypeParamListMake for TypeParamListBase {}
impl AstMake for TypeParamListBase {
    type Node = TypeParamList;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TYPE_PARAM_LIST);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl TypeParamList {
    pub fn new() -> TypeParamListBase {
        TypeParamListBase {}
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
pub trait UseTreeMake: AstMake {
    fn path<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: PathMake,
    {
        Make::new(self, b, None)
    }
    fn use_tree_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: UseTreeListMake,
    {
        Make::new(self, b, None)
    }
    fn alia<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: AliasMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: UseTreeMake, B: AstMake> UseTreeMake for Make<A, B> {}
pub struct UseTreeBase {}
impl UseTreeMake for UseTreeBase {}
impl AstMake for UseTreeBase {
    type Node = UseTree;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::USE_TREE);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl UseTree {
    pub fn new() -> UseTreeBase {
        UseTreeBase {}
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
pub trait UseTreeListMake: AstMake {
    fn use_tree<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: UseTreeMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: UseTreeListMake, B: AstMake> UseTreeListMake for Make<A, B> {}
pub struct UseTreeListBase {}
impl UseTreeListMake for UseTreeListBase {}
impl AstMake for UseTreeListBase {
    type Node = UseTreeList;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::USE_TREE_LIST);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl UseTreeList {
    pub fn new() -> UseTreeListBase {
        UseTreeListBase {}
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
pub trait WhereClauseMake: AstMake {
    fn predicate<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: WherePredMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: WhereClauseMake, B: AstMake> WhereClauseMake for Make<A, B> {}
pub struct WhereClauseBase {}
impl WhereClauseMake for WhereClauseBase {}
impl AstMake for WhereClauseBase {
    type Node = WhereClause;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::WHERE_CLAUSE);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl WhereClause {
    pub fn new() -> WhereClauseBase {
        WhereClauseBase {}
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
pub trait WherePredMake: AstMake {
    fn type_ref<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeRefMake,
    {
        Make::new(self, b, None)
    }
    fn type_bound_list<B>(self, b: B) -> Make<Self, B>
    where
        Self: Sized,
        B: TypeBoundListMake,
    {
        Make::new(self, b, None)
    }
}
impl<A: WherePredMake, B: AstMake> WherePredMake for Make<A, B> {}
pub struct WherePredBase {}
impl WherePredMake for WherePredBase {}
impl AstMake for WherePredBase {
    type Node = WherePred;
    fn make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::WHERE_PRED);
    }
    fn finish_make(&mut self, builder: &mut SyntaxTreeBuilder) {
        builder.finish_node();
    }
}
impl WherePred {
    pub fn new() -> WherePredBase {
        WherePredBase {}
    }
}
