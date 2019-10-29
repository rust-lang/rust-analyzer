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
    type I = Self;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
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
    type I = Self;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        let (kind, token) = match self {
            PrefixOp::Deref => (STAR, T_STR!(STAR)),
            PrefixOp::Not => (EXCL, T_STR!(EXCL)),
            PrefixOp::Neg => (MINUS, T_STR!(MINUS)),
        };
        builder.token(kind, SmolStr::new(token));
    }
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
pub enum ExprMake {
    TupleExprMake(Box<TupleExprMake>),
    ArrayExprMake(Box<ArrayExprMake>),
    ParenExprMake(Box<ParenExprMake>),
    PathExprMake(Box<PathExprMake>),
    LambdaExprMake(Box<LambdaExprMake>),
    IfExprMake(Box<IfExprMake>),
    LoopExprMake(Box<LoopExprMake>),
    ForExprMake(Box<ForExprMake>),
    WhileExprMake(Box<WhileExprMake>),
    ContinueExprMake(Box<ContinueExprMake>),
    BreakExprMake(Box<BreakExprMake>),
    LabelMake(Box<LabelMake>),
    BlockExprMake(Box<BlockExprMake>),
    ReturnExprMake(Box<ReturnExprMake>),
    MatchExprMake(Box<MatchExprMake>),
    RecordLitMake(Box<RecordLitMake>),
    CallExprMake(Box<CallExprMake>),
    IndexExprMake(Box<IndexExprMake>),
    MethodCallExprMake(Box<MethodCallExprMake>),
    FieldExprMake(Box<FieldExprMake>),
    AwaitExprMake(Box<AwaitExprMake>),
    TryExprMake(Box<TryExprMake>),
    TryBlockExprMake(Box<TryBlockExprMake>),
    CastExprMake(Box<CastExprMake>),
    RefExprMake(Box<RefExprMake>),
    PrefixExprMake(Box<PrefixExprMake>),
    BoxExprMake(Box<BoxExprMake>),
    RangeExprMake(Box<RangeExprMake>),
    BinExprMake(Box<BinExprMake>),
    LiteralMake(Box<LiteralMake>),
    MacroCallMake(Box<MacroCallMake>),
}
impl From<TupleExprMake> for ExprMake {
    fn from(builder: TupleExprMake) -> ExprMake {
        ExprMake::TupleExprMake(Box::new(builder))
    }
}
impl From<ArrayExprMake> for ExprMake {
    fn from(builder: ArrayExprMake) -> ExprMake {
        ExprMake::ArrayExprMake(Box::new(builder))
    }
}
impl From<ParenExprMake> for ExprMake {
    fn from(builder: ParenExprMake) -> ExprMake {
        ExprMake::ParenExprMake(Box::new(builder))
    }
}
impl From<PathExprMake> for ExprMake {
    fn from(builder: PathExprMake) -> ExprMake {
        ExprMake::PathExprMake(Box::new(builder))
    }
}
impl From<LambdaExprMake> for ExprMake {
    fn from(builder: LambdaExprMake) -> ExprMake {
        ExprMake::LambdaExprMake(Box::new(builder))
    }
}
impl From<IfExprMake> for ExprMake {
    fn from(builder: IfExprMake) -> ExprMake {
        ExprMake::IfExprMake(Box::new(builder))
    }
}
impl From<LoopExprMake> for ExprMake {
    fn from(builder: LoopExprMake) -> ExprMake {
        ExprMake::LoopExprMake(Box::new(builder))
    }
}
impl From<ForExprMake> for ExprMake {
    fn from(builder: ForExprMake) -> ExprMake {
        ExprMake::ForExprMake(Box::new(builder))
    }
}
impl From<WhileExprMake> for ExprMake {
    fn from(builder: WhileExprMake) -> ExprMake {
        ExprMake::WhileExprMake(Box::new(builder))
    }
}
impl From<ContinueExprMake> for ExprMake {
    fn from(builder: ContinueExprMake) -> ExprMake {
        ExprMake::ContinueExprMake(Box::new(builder))
    }
}
impl From<BreakExprMake> for ExprMake {
    fn from(builder: BreakExprMake) -> ExprMake {
        ExprMake::BreakExprMake(Box::new(builder))
    }
}
impl From<LabelMake> for ExprMake {
    fn from(builder: LabelMake) -> ExprMake {
        ExprMake::LabelMake(Box::new(builder))
    }
}
impl From<BlockExprMake> for ExprMake {
    fn from(builder: BlockExprMake) -> ExprMake {
        ExprMake::BlockExprMake(Box::new(builder))
    }
}
impl From<ReturnExprMake> for ExprMake {
    fn from(builder: ReturnExprMake) -> ExprMake {
        ExprMake::ReturnExprMake(Box::new(builder))
    }
}
impl From<MatchExprMake> for ExprMake {
    fn from(builder: MatchExprMake) -> ExprMake {
        ExprMake::MatchExprMake(Box::new(builder))
    }
}
impl From<RecordLitMake> for ExprMake {
    fn from(builder: RecordLitMake) -> ExprMake {
        ExprMake::RecordLitMake(Box::new(builder))
    }
}
impl From<CallExprMake> for ExprMake {
    fn from(builder: CallExprMake) -> ExprMake {
        ExprMake::CallExprMake(Box::new(builder))
    }
}
impl From<IndexExprMake> for ExprMake {
    fn from(builder: IndexExprMake) -> ExprMake {
        ExprMake::IndexExprMake(Box::new(builder))
    }
}
impl From<MethodCallExprMake> for ExprMake {
    fn from(builder: MethodCallExprMake) -> ExprMake {
        ExprMake::MethodCallExprMake(Box::new(builder))
    }
}
impl From<FieldExprMake> for ExprMake {
    fn from(builder: FieldExprMake) -> ExprMake {
        ExprMake::FieldExprMake(Box::new(builder))
    }
}
impl From<AwaitExprMake> for ExprMake {
    fn from(builder: AwaitExprMake) -> ExprMake {
        ExprMake::AwaitExprMake(Box::new(builder))
    }
}
impl From<TryExprMake> for ExprMake {
    fn from(builder: TryExprMake) -> ExprMake {
        ExprMake::TryExprMake(Box::new(builder))
    }
}
impl From<TryBlockExprMake> for ExprMake {
    fn from(builder: TryBlockExprMake) -> ExprMake {
        ExprMake::TryBlockExprMake(Box::new(builder))
    }
}
impl From<CastExprMake> for ExprMake {
    fn from(builder: CastExprMake) -> ExprMake {
        ExprMake::CastExprMake(Box::new(builder))
    }
}
impl From<RefExprMake> for ExprMake {
    fn from(builder: RefExprMake) -> ExprMake {
        ExprMake::RefExprMake(Box::new(builder))
    }
}
impl From<PrefixExprMake> for ExprMake {
    fn from(builder: PrefixExprMake) -> ExprMake {
        ExprMake::PrefixExprMake(Box::new(builder))
    }
}
impl From<BoxExprMake> for ExprMake {
    fn from(builder: BoxExprMake) -> ExprMake {
        ExprMake::BoxExprMake(Box::new(builder))
    }
}
impl From<RangeExprMake> for ExprMake {
    fn from(builder: RangeExprMake) -> ExprMake {
        ExprMake::RangeExprMake(Box::new(builder))
    }
}
impl From<BinExprMake> for ExprMake {
    fn from(builder: BinExprMake) -> ExprMake {
        ExprMake::BinExprMake(Box::new(builder))
    }
}
impl From<LiteralMake> for ExprMake {
    fn from(builder: LiteralMake) -> ExprMake {
        ExprMake::LiteralMake(Box::new(builder))
    }
}
impl From<MacroCallMake> for ExprMake {
    fn from(builder: MacroCallMake) -> ExprMake {
        ExprMake::MacroCallMake(Box::new(builder))
    }
}
impl AstMake for ExprMake {
    type I = Expr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        match self {
            ExprMake::TupleExprMake(b) => b.make(builder),
            ExprMake::ArrayExprMake(b) => b.make(builder),
            ExprMake::ParenExprMake(b) => b.make(builder),
            ExprMake::PathExprMake(b) => b.make(builder),
            ExprMake::LambdaExprMake(b) => b.make(builder),
            ExprMake::IfExprMake(b) => b.make(builder),
            ExprMake::LoopExprMake(b) => b.make(builder),
            ExprMake::ForExprMake(b) => b.make(builder),
            ExprMake::WhileExprMake(b) => b.make(builder),
            ExprMake::ContinueExprMake(b) => b.make(builder),
            ExprMake::BreakExprMake(b) => b.make(builder),
            ExprMake::LabelMake(b) => b.make(builder),
            ExprMake::BlockExprMake(b) => b.make(builder),
            ExprMake::ReturnExprMake(b) => b.make(builder),
            ExprMake::MatchExprMake(b) => b.make(builder),
            ExprMake::RecordLitMake(b) => b.make(builder),
            ExprMake::CallExprMake(b) => b.make(builder),
            ExprMake::IndexExprMake(b) => b.make(builder),
            ExprMake::MethodCallExprMake(b) => b.make(builder),
            ExprMake::FieldExprMake(b) => b.make(builder),
            ExprMake::AwaitExprMake(b) => b.make(builder),
            ExprMake::TryExprMake(b) => b.make(builder),
            ExprMake::TryBlockExprMake(b) => b.make(builder),
            ExprMake::CastExprMake(b) => b.make(builder),
            ExprMake::RefExprMake(b) => b.make(builder),
            ExprMake::PrefixExprMake(b) => b.make(builder),
            ExprMake::BoxExprMake(b) => b.make(builder),
            ExprMake::RangeExprMake(b) => b.make(builder),
            ExprMake::BinExprMake(b) => b.make(builder),
            ExprMake::LiteralMake(b) => b.make(builder),
            ExprMake::MacroCallMake(b) => b.make(builder),
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
impl TupleExpr {
    pub fn new() -> TupleExprMake {
        TupleExprMake::default()
    }
}
#[derive(Default)]
pub struct TupleExprMake {
    exprs: Vec<Box<ExprMake>>,
}
impl TupleExprMake {
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.exprs.push(Box::new(f));
        self
    }
}
impl AstMake for TupleExprMake {
    type I = TupleExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TUPLE_EXPR);
        for b in self.exprs {
            b.make(builder);
        }
        builder.finish_node();
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
impl ArrayExpr {
    pub fn new() -> ArrayExprMake {
        ArrayExprMake::default()
    }
}
#[derive(Default)]
pub struct ArrayExprMake {
    exprs: Vec<Box<ExprMake>>,
}
impl ArrayExprMake {
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.exprs.push(Box::new(f));
        self
    }
}
impl AstMake for ArrayExprMake {
    type I = ArrayExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ARRAY_EXPR);
        for b in self.exprs {
            b.make(builder);
        }
        builder.finish_node();
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
impl ParenExpr {
    pub fn new() -> ParenExprMake {
        ParenExprMake::default()
    }
}
#[derive(Default)]
pub struct ParenExprMake {
    expr: Option<Box<ExprMake>>,
}
impl ParenExprMake {
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstMake for ParenExprMake {
    type I = ParenExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PAREN_EXPR);
        if let Some(b) = self.expr {
            b.make(builder);
        }
        builder.finish_node();
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
impl PathExpr {
    pub fn new() -> PathExprMake {
        PathExprMake::default()
    }
}
#[derive(Default)]
pub struct PathExprMake {
    path: Option<Box<PathMake>>,
}
impl PathExprMake {
    pub fn path(mut self, f: PathMake) -> Self {
        self.path = Some(Box::new(f));
        self
    }
}
impl AstMake for PathExprMake {
    type I = PathExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PATH_EXPR);
        if let Some(b) = self.path {
            b.make(builder);
        }
        builder.finish_node();
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
impl LambdaExpr {
    pub fn new() -> LambdaExprMake {
        LambdaExprMake::default()
    }
}
#[derive(Default)]
pub struct LambdaExprMake {
    param_list: Option<Box<ParamListMake>>,
    body: Option<Box<ExprMake>>,
}
impl LambdaExprMake {
    pub fn param_list(mut self, f: ParamListMake) -> Self {
        self.param_list = Some(Box::new(f));
        self
    }
    pub fn body(mut self, f: ExprMake) -> Self {
        self.body = Some(Box::new(f));
        self
    }
}
impl AstMake for LambdaExprMake {
    type I = LambdaExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::LAMBDA_EXPR);
        if let Some(b) = self.param_list {
            b.make(builder);
        }
        if let Some(b) = self.body {
            b.make(builder);
        }
        builder.finish_node();
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
impl IfExpr {
    pub fn new() -> IfExprMake {
        IfExprMake::default()
    }
}
#[derive(Default)]
pub struct IfExprMake {
    condition: Option<Box<ConditionMake>>,
}
impl IfExprMake {
    pub fn condition(mut self, f: ConditionMake) -> Self {
        self.condition = Some(Box::new(f));
        self
    }
}
impl AstMake for IfExprMake {
    type I = IfExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::IF_EXPR);
        if let Some(b) = self.condition {
            b.make(builder);
        }
        builder.finish_node();
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
impl LoopExpr {
    pub fn new() -> LoopExprMake {
        LoopExprMake::default()
    }
}
#[derive(Default)]
pub struct LoopExprMake {
    loop_body: Option<Box<BlockExprMake>>,
}
impl LoopExprMake {
    pub fn loop_body(mut self, f: BlockExprMake) -> Self {
        self.loop_body = Some(Box::new(f));
        self
    }
}
impl AstMake for LoopExprMake {
    type I = LoopExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::LOOP_EXPR);
        if let Some(b) = self.loop_body {
            b.make(builder);
        }
        builder.finish_node();
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
impl ForExpr {
    pub fn new() -> ForExprMake {
        ForExprMake::default()
    }
}
#[derive(Default)]
pub struct ForExprMake {
    pat: Option<Box<PatMake>>,
    iterable: Option<Box<ExprMake>>,
    loop_body: Option<Box<BlockExprMake>>,
}
impl ForExprMake {
    pub fn pat(mut self, f: PatMake) -> Self {
        self.pat = Some(Box::new(f));
        self
    }
    pub fn iterable(mut self, f: ExprMake) -> Self {
        self.iterable = Some(Box::new(f));
        self
    }
    pub fn loop_body(mut self, f: BlockExprMake) -> Self {
        self.loop_body = Some(Box::new(f));
        self
    }
}
impl AstMake for ForExprMake {
    type I = ForExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::FOR_EXPR);
        if let Some(b) = self.pat {
            b.make(builder);
        }
        if let Some(b) = self.iterable {
            b.make(builder);
        }
        if let Some(b) = self.loop_body {
            b.make(builder);
        }
        builder.finish_node();
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
impl WhileExpr {
    pub fn new() -> WhileExprMake {
        WhileExprMake::default()
    }
}
#[derive(Default)]
pub struct WhileExprMake {
    condition: Option<Box<ConditionMake>>,
    loop_body: Option<Box<BlockExprMake>>,
}
impl WhileExprMake {
    pub fn condition(mut self, f: ConditionMake) -> Self {
        self.condition = Some(Box::new(f));
        self
    }
    pub fn loop_body(mut self, f: BlockExprMake) -> Self {
        self.loop_body = Some(Box::new(f));
        self
    }
}
impl AstMake for WhileExprMake {
    type I = WhileExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::WHILE_EXPR);
        if let Some(b) = self.condition {
            b.make(builder);
        }
        if let Some(b) = self.loop_body {
            b.make(builder);
        }
        builder.finish_node();
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
impl ContinueExpr {
    pub fn new() -> ContinueExprMake {
        ContinueExprMake::default()
    }
}
#[derive(Default)]
pub struct ContinueExprMake {}
impl ContinueExprMake {}
impl AstMake for ContinueExprMake {
    type I = ContinueExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::CONTINUE_EXPR);
        builder.token(SyntaxKind::CONTINUE_KW, SmolStr::new(T_STR!(CONTINUE_KW)));
        builder.finish_node();
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
impl BreakExpr {
    pub fn new() -> BreakExprMake {
        BreakExprMake::default()
    }
}
#[derive(Default)]
pub struct BreakExprMake {
    expr: Option<Box<ExprMake>>,
}
impl BreakExprMake {
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstMake for BreakExprMake {
    type I = BreakExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::BREAK_EXPR);
        if let Some(b) = self.expr {
            b.make(builder);
        }
        builder.finish_node();
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
impl ReturnExpr {
    pub fn new() -> ReturnExprMake {
        ReturnExprMake::default()
    }
}
#[derive(Default)]
pub struct ReturnExprMake {
    expr: Option<Box<ExprMake>>,
}
impl ReturnExprMake {
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstMake for ReturnExprMake {
    type I = ReturnExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RETURN_EXPR);
        if let Some(b) = self.expr {
            b.make(builder);
        }
        builder.finish_node();
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
impl MatchExpr {
    pub fn new() -> MatchExprMake {
        MatchExprMake::default()
    }
}
#[derive(Default)]
pub struct MatchExprMake {
    expr: Option<Box<ExprMake>>,
    match_arm_list: Option<Box<MatchArmListMake>>,
}
impl MatchExprMake {
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
    pub fn match_arm_list(mut self, f: MatchArmListMake) -> Self {
        self.match_arm_list = Some(Box::new(f));
        self
    }
}
impl AstMake for MatchExprMake {
    type I = MatchExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MATCH_EXPR);
        if let Some(b) = self.expr {
            b.make(builder);
        }
        if let Some(b) = self.match_arm_list {
            b.make(builder);
        }
        builder.finish_node();
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
impl RecordLit {
    pub fn new() -> RecordLitMake {
        RecordLitMake::default()
    }
}
#[derive(Default)]
pub struct RecordLitMake {
    path: Option<Box<PathMake>>,
    record_field_list: Option<Box<RecordFieldListMake>>,
}
impl RecordLitMake {
    pub fn path(mut self, f: PathMake) -> Self {
        self.path = Some(Box::new(f));
        self
    }
    pub fn record_field_list(mut self, f: RecordFieldListMake) -> Self {
        self.record_field_list = Some(Box::new(f));
        self
    }
}
impl AstMake for RecordLitMake {
    type I = RecordLit;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_LIT);
        if let Some(b) = self.path {
            b.make(builder);
        }
        if let Some(b) = self.record_field_list {
            b.make(builder);
        }
        builder.finish_node();
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
impl CallExpr {
    pub fn new() -> CallExprMake {
        CallExprMake::default()
    }
}
#[derive(Default)]
pub struct CallExprMake {
    expr: Option<Box<ExprMake>>,
    arg_list: Option<Box<ArgListMake>>,
}
impl CallExprMake {
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
    pub fn arg_list(mut self, f: ArgListMake) -> Self {
        self.arg_list = Some(Box::new(f));
        self
    }
}
impl AstMake for CallExprMake {
    type I = CallExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::CALL_EXPR);
        if let Some(b) = self.expr {
            b.make(builder);
        }
        if let Some(b) = self.arg_list {
            b.make(builder);
        }
        builder.finish_node();
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
impl IndexExpr {
    pub fn new() -> IndexExprMake {
        IndexExprMake::default()
    }
}
#[derive(Default)]
pub struct IndexExprMake {
    base: Option<Box<ExprMake>>,
    index: Option<Box<ExprMake>>,
}
impl IndexExprMake {
    pub fn base(mut self, f: ExprMake) -> Self {
        self.base = Some(Box::new(f));
        self
    }
    pub fn index(mut self, f: ExprMake) -> Self {
        self.index = Some(Box::new(f));
        self
    }
}
impl AstMake for IndexExprMake {
    type I = IndexExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::INDEX_EXPR);
        if let Some(b) = self.base {
            b.make(builder);
        }
        if let Some(b) = self.index {
            b.make(builder);
        }
        builder.finish_node();
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
impl MethodCallExpr {
    pub fn new() -> MethodCallExprMake {
        MethodCallExprMake::default()
    }
}
#[derive(Default)]
pub struct MethodCallExprMake {
    expr: Option<Box<ExprMake>>,
    name_ref: Option<Box<NameRefMake>>,
    type_arg_list: Option<Box<TypeArgListMake>>,
    arg_list: Option<Box<ArgListMake>>,
}
impl MethodCallExprMake {
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
    pub fn name_ref(mut self, f: NameRefMake) -> Self {
        self.name_ref = Some(Box::new(f));
        self
    }
    pub fn type_arg_list(mut self, f: TypeArgListMake) -> Self {
        self.type_arg_list = Some(Box::new(f));
        self
    }
    pub fn arg_list(mut self, f: ArgListMake) -> Self {
        self.arg_list = Some(Box::new(f));
        self
    }
}
impl AstMake for MethodCallExprMake {
    type I = MethodCallExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::METHOD_CALL_EXPR);
        if let Some(b) = self.expr {
            b.make(builder);
        }
        if let Some(b) = self.name_ref {
            b.make(builder);
        }
        if let Some(b) = self.type_arg_list {
            b.make(builder);
        }
        if let Some(b) = self.arg_list {
            b.make(builder);
        }
        builder.finish_node();
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
impl FieldExpr {
    pub fn new() -> FieldExprMake {
        FieldExprMake::default()
    }
}
#[derive(Default)]
pub struct FieldExprMake {
    expr: Option<Box<ExprMake>>,
    name_ref: Option<Box<NameRefMake>>,
}
impl FieldExprMake {
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
    pub fn name_ref(mut self, f: NameRefMake) -> Self {
        self.name_ref = Some(Box::new(f));
        self
    }
}
impl AstMake for FieldExprMake {
    type I = FieldExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::FIELD_EXPR);
        if let Some(b) = self.expr {
            b.make(builder);
        }
        if let Some(b) = self.name_ref {
            b.make(builder);
        }
        builder.finish_node();
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
impl AwaitExpr {
    pub fn new() -> AwaitExprMake {
        AwaitExprMake::default()
    }
}
#[derive(Default)]
pub struct AwaitExprMake {
    expr: Option<Box<ExprMake>>,
}
impl AwaitExprMake {
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstMake for AwaitExprMake {
    type I = AwaitExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::AWAIT_EXPR);
        if let Some(b) = self.expr {
            b.make(builder);
        }
        builder.finish_node();
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
impl TryExpr {
    pub fn new() -> TryExprMake {
        TryExprMake::default()
    }
}
#[derive(Default)]
pub struct TryExprMake {
    expr: Option<Box<ExprMake>>,
}
impl TryExprMake {
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstMake for TryExprMake {
    type I = TryExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TRY_EXPR);
        if let Some(b) = self.expr {
            b.make(builder);
        }
        builder.finish_node();
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
impl TryBlockExpr {
    pub fn new() -> TryBlockExprMake {
        TryBlockExprMake::default()
    }
}
#[derive(Default)]
pub struct TryBlockExprMake {
    body: Option<Box<BlockExprMake>>,
}
impl TryBlockExprMake {
    pub fn body(mut self, f: BlockExprMake) -> Self {
        self.body = Some(Box::new(f));
        self
    }
}
impl AstMake for TryBlockExprMake {
    type I = TryBlockExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TRY_BLOCK_EXPR);
        if let Some(b) = self.body {
            b.make(builder);
        }
        builder.finish_node();
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
impl CastExpr {
    pub fn new() -> CastExprMake {
        CastExprMake::default()
    }
}
#[derive(Default)]
pub struct CastExprMake {
    expr: Option<Box<ExprMake>>,
    type_ref: Option<Box<TypeRefMake>>,
}
impl CastExprMake {
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
    pub fn type_ref(mut self, f: TypeRefMake) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstMake for CastExprMake {
    type I = CastExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::CAST_EXPR);
        if let Some(b) = self.expr {
            b.make(builder);
        }
        if let Some(b) = self.type_ref {
            b.make(builder);
        }
        builder.finish_node();
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
impl RefExpr {
    pub fn new() -> RefExprMake {
        RefExprMake::default()
    }
}
#[derive(Default)]
pub struct RefExprMake {
    expr: Option<Box<ExprMake>>,
}
impl RefExprMake {
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstMake for RefExprMake {
    type I = RefExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::REF_EXPR);
        if let Some(b) = self.expr {
            b.make(builder);
        }
        builder.finish_node();
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
impl PrefixExpr {
    pub fn new() -> PrefixExprMake {
        PrefixExprMake::default()
    }
}
#[derive(Default)]
pub struct PrefixExprMake {
    op: Option<PrefixOp>,
    expr: Option<Box<ExprMake>>,
}
impl PrefixExprMake {
    pub fn op(mut self, f: PrefixOp) -> Self {
        self.op = Some(f);
        self
    }
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstMake for PrefixExprMake {
    type I = PrefixExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PREFIX_EXPR);
        if let Some(b) = self.op {
            b.make(builder);
        }
        if let Some(b) = self.expr {
            b.make(builder);
        }
        builder.finish_node();
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
impl BoxExpr {
    pub fn new() -> BoxExprMake {
        BoxExprMake::default()
    }
}
#[derive(Default)]
pub struct BoxExprMake {
    expr: Option<Box<ExprMake>>,
}
impl BoxExprMake {
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstMake for BoxExprMake {
    type I = BoxExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::BOX_EXPR);
        if let Some(b) = self.expr {
            b.make(builder);
        }
        builder.finish_node();
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
impl BinExpr {
    pub fn new() -> BinExprMake {
        BinExprMake::default()
    }
}
#[derive(Default)]
pub struct BinExprMake {
    lhs: Option<Box<ExprMake>>,
    op: Option<BinOp>,
    rhs: Option<Box<ExprMake>>,
}
impl BinExprMake {
    pub fn lh(mut self, f: ExprMake) -> Self {
        self.lhs = Some(Box::new(f));
        self
    }
    pub fn op(mut self, f: BinOp) -> Self {
        self.op = Some(f);
        self
    }
    pub fn rh(mut self, f: ExprMake) -> Self {
        self.rhs = Some(Box::new(f));
        self
    }
}
impl AstMake for BinExprMake {
    type I = BinExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::BIN_EXPR);
        if let Some(b) = self.lhs {
            b.make(builder);
        }
        if let Some(b) = self.op {
            b.make(builder);
        }
        if let Some(b) = self.rhs {
            b.make(builder);
        }
        builder.finish_node();
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
impl MacroCall {
    pub fn new() -> MacroCallMake {
        MacroCallMake::default()
    }
}
#[derive(Default)]
pub struct MacroCallMake {
    token_tree: Option<Box<TokenTreeMake>>,
    path: Option<Box<PathMake>>,
    name: Option<Box<NameMake>>,
    attrs: Vec<Box<AttrMake>>,
}
impl MacroCallMake {
    pub fn token_tree(mut self, f: TokenTreeMake) -> Self {
        self.token_tree = Some(Box::new(f));
        self
    }
    pub fn path(mut self, f: PathMake) -> Self {
        self.path = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameMake) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstMake for MacroCallMake {
    type I = MacroCall;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MACRO_CALL);
        if let Some(b) = self.token_tree {
            b.make(builder);
        }
        if let Some(b) = self.path {
            b.make(builder);
        }
        if let Some(b) = self.name {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        builder.finish_node();
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
pub enum ImplItemMake {
    FnDefMake(Box<FnDefMake>),
    TypeAliasDefMake(Box<TypeAliasDefMake>),
    ConstDefMake(Box<ConstDefMake>),
}
impl From<FnDefMake> for ImplItemMake {
    fn from(builder: FnDefMake) -> ImplItemMake {
        ImplItemMake::FnDefMake(Box::new(builder))
    }
}
impl From<TypeAliasDefMake> for ImplItemMake {
    fn from(builder: TypeAliasDefMake) -> ImplItemMake {
        ImplItemMake::TypeAliasDefMake(Box::new(builder))
    }
}
impl From<ConstDefMake> for ImplItemMake {
    fn from(builder: ConstDefMake) -> ImplItemMake {
        ImplItemMake::ConstDefMake(Box::new(builder))
    }
}
impl AstMake for ImplItemMake {
    type I = ImplItem;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        match self {
            ImplItemMake::FnDefMake(b) => b.make(builder),
            ImplItemMake::TypeAliasDefMake(b) => b.make(builder),
            ImplItemMake::ConstDefMake(b) => b.make(builder),
        }
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
pub enum ModuleItemMake {
    StructDefMake(Box<StructDefMake>),
    UnionDefMake(Box<UnionDefMake>),
    EnumDefMake(Box<EnumDefMake>),
    FnDefMake(Box<FnDefMake>),
    TraitDefMake(Box<TraitDefMake>),
    TypeAliasDefMake(Box<TypeAliasDefMake>),
    ImplBlockMake(Box<ImplBlockMake>),
    UseItemMake(Box<UseItemMake>),
    ExternCrateItemMake(Box<ExternCrateItemMake>),
    ConstDefMake(Box<ConstDefMake>),
    StaticDefMake(Box<StaticDefMake>),
    ModuleMake(Box<ModuleMake>),
}
impl From<StructDefMake> for ModuleItemMake {
    fn from(builder: StructDefMake) -> ModuleItemMake {
        ModuleItemMake::StructDefMake(Box::new(builder))
    }
}
impl From<UnionDefMake> for ModuleItemMake {
    fn from(builder: UnionDefMake) -> ModuleItemMake {
        ModuleItemMake::UnionDefMake(Box::new(builder))
    }
}
impl From<EnumDefMake> for ModuleItemMake {
    fn from(builder: EnumDefMake) -> ModuleItemMake {
        ModuleItemMake::EnumDefMake(Box::new(builder))
    }
}
impl From<FnDefMake> for ModuleItemMake {
    fn from(builder: FnDefMake) -> ModuleItemMake {
        ModuleItemMake::FnDefMake(Box::new(builder))
    }
}
impl From<TraitDefMake> for ModuleItemMake {
    fn from(builder: TraitDefMake) -> ModuleItemMake {
        ModuleItemMake::TraitDefMake(Box::new(builder))
    }
}
impl From<TypeAliasDefMake> for ModuleItemMake {
    fn from(builder: TypeAliasDefMake) -> ModuleItemMake {
        ModuleItemMake::TypeAliasDefMake(Box::new(builder))
    }
}
impl From<ImplBlockMake> for ModuleItemMake {
    fn from(builder: ImplBlockMake) -> ModuleItemMake {
        ModuleItemMake::ImplBlockMake(Box::new(builder))
    }
}
impl From<UseItemMake> for ModuleItemMake {
    fn from(builder: UseItemMake) -> ModuleItemMake {
        ModuleItemMake::UseItemMake(Box::new(builder))
    }
}
impl From<ExternCrateItemMake> for ModuleItemMake {
    fn from(builder: ExternCrateItemMake) -> ModuleItemMake {
        ModuleItemMake::ExternCrateItemMake(Box::new(builder))
    }
}
impl From<ConstDefMake> for ModuleItemMake {
    fn from(builder: ConstDefMake) -> ModuleItemMake {
        ModuleItemMake::ConstDefMake(Box::new(builder))
    }
}
impl From<StaticDefMake> for ModuleItemMake {
    fn from(builder: StaticDefMake) -> ModuleItemMake {
        ModuleItemMake::StaticDefMake(Box::new(builder))
    }
}
impl From<ModuleMake> for ModuleItemMake {
    fn from(builder: ModuleMake) -> ModuleItemMake {
        ModuleItemMake::ModuleMake(Box::new(builder))
    }
}
impl AstMake for ModuleItemMake {
    type I = ModuleItem;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        match self {
            ModuleItemMake::StructDefMake(b) => b.make(builder),
            ModuleItemMake::UnionDefMake(b) => b.make(builder),
            ModuleItemMake::EnumDefMake(b) => b.make(builder),
            ModuleItemMake::FnDefMake(b) => b.make(builder),
            ModuleItemMake::TraitDefMake(b) => b.make(builder),
            ModuleItemMake::TypeAliasDefMake(b) => b.make(builder),
            ModuleItemMake::ImplBlockMake(b) => b.make(builder),
            ModuleItemMake::UseItemMake(b) => b.make(builder),
            ModuleItemMake::ExternCrateItemMake(b) => b.make(builder),
            ModuleItemMake::ConstDefMake(b) => b.make(builder),
            ModuleItemMake::StaticDefMake(b) => b.make(builder),
            ModuleItemMake::ModuleMake(b) => b.make(builder),
        }
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
impl TraitDef {
    pub fn new() -> TraitDefMake {
        TraitDefMake::default()
    }
}
#[derive(Default)]
pub struct TraitDefMake {
    item_list: Option<Box<ItemListMake>>,
    visibility: Option<Box<VisibilityMake>>,
    name: Option<Box<NameMake>>,
    attrs: Vec<Box<AttrMake>>,
    type_param_list: Option<Box<TypeParamListMake>>,
    where_clause: Option<Box<WhereClauseMake>>,
    type_bound_list: Option<Box<TypeBoundListMake>>,
}
impl TraitDefMake {
    pub fn item_list(mut self, f: ItemListMake) -> Self {
        self.item_list = Some(Box::new(f));
        self
    }
    pub fn visibility(mut self, f: VisibilityMake) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameMake) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
    pub fn type_param_list(mut self, f: TypeParamListMake) -> Self {
        self.type_param_list = Some(Box::new(f));
        self
    }
    pub fn where_clause(mut self, f: WhereClauseMake) -> Self {
        self.where_clause = Some(Box::new(f));
        self
    }
    pub fn type_bound_list(mut self, f: TypeBoundListMake) -> Self {
        self.type_bound_list = Some(Box::new(f));
        self
    }
}
impl AstMake for TraitDefMake {
    type I = TraitDef;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TRAIT_DEF);
        if let Some(b) = self.item_list {
            b.make(builder);
        }
        if let Some(b) = self.visibility {
            b.make(builder);
        }
        if let Some(b) = self.name {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        if let Some(b) = self.type_param_list {
            b.make(builder);
        }
        if let Some(b) = self.where_clause {
            b.make(builder);
        }
        if let Some(b) = self.type_bound_list {
            b.make(builder);
        }
        builder.finish_node();
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
impl ImplBlock {
    pub fn new() -> ImplBlockMake {
        ImplBlockMake::default()
    }
}
#[derive(Default)]
pub struct ImplBlockMake {
    item_list: Option<Box<ItemListMake>>,
    type_param_list: Option<Box<TypeParamListMake>>,
    where_clause: Option<Box<WhereClauseMake>>,
    attrs: Vec<Box<AttrMake>>,
}
impl ImplBlockMake {
    pub fn item_list(mut self, f: ItemListMake) -> Self {
        self.item_list = Some(Box::new(f));
        self
    }
    pub fn type_param_list(mut self, f: TypeParamListMake) -> Self {
        self.type_param_list = Some(Box::new(f));
        self
    }
    pub fn where_clause(mut self, f: WhereClauseMake) -> Self {
        self.where_clause = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstMake for ImplBlockMake {
    type I = ImplBlock;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::IMPL_BLOCK);
        if let Some(b) = self.item_list {
            b.make(builder);
        }
        if let Some(b) = self.type_param_list {
            b.make(builder);
        }
        if let Some(b) = self.where_clause {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        builder.finish_node();
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
impl UseItem {
    pub fn new() -> UseItemMake {
        UseItemMake::default()
    }
}
#[derive(Default)]
pub struct UseItemMake {
    use_tree: Option<Box<UseTreeMake>>,
    attrs: Vec<Box<AttrMake>>,
}
impl UseItemMake {
    pub fn use_tree(mut self, f: UseTreeMake) -> Self {
        self.use_tree = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstMake for UseItemMake {
    type I = UseItem;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::USE_ITEM);
        if let Some(b) = self.use_tree {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        builder.finish_node();
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
impl ExternCrateItem {
    pub fn new() -> ExternCrateItemMake {
        ExternCrateItemMake::default()
    }
}
#[derive(Default)]
pub struct ExternCrateItemMake {
    name_ref: Option<Box<NameRefMake>>,
    alias: Option<Box<AliasMake>>,
    attrs: Vec<Box<AttrMake>>,
}
impl ExternCrateItemMake {
    pub fn name_ref(mut self, f: NameRefMake) -> Self {
        self.name_ref = Some(Box::new(f));
        self
    }
    pub fn alia(mut self, f: AliasMake) -> Self {
        self.alias = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstMake for ExternCrateItemMake {
    type I = ExternCrateItem;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::EXTERN_CRATE_ITEM);
        if let Some(b) = self.name_ref {
            b.make(builder);
        }
        if let Some(b) = self.alias {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        builder.finish_node();
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
impl StaticDef {
    pub fn new() -> StaticDefMake {
        StaticDefMake::default()
    }
}
#[derive(Default)]
pub struct StaticDefMake {
    body: Option<Box<ExprMake>>,
    visibility: Option<Box<VisibilityMake>>,
    name: Option<Box<NameMake>>,
    type_param_list: Option<Box<TypeParamListMake>>,
    where_clause: Option<Box<WhereClauseMake>>,
    attrs: Vec<Box<AttrMake>>,
    ascribed_type: Option<Box<TypeRefMake>>,
}
impl StaticDefMake {
    pub fn body(mut self, f: ExprMake) -> Self {
        self.body = Some(Box::new(f));
        self
    }
    pub fn visibility(mut self, f: VisibilityMake) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameMake) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn type_param_list(mut self, f: TypeParamListMake) -> Self {
        self.type_param_list = Some(Box::new(f));
        self
    }
    pub fn where_clause(mut self, f: WhereClauseMake) -> Self {
        self.where_clause = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
    pub fn ascribed_type(mut self, f: TypeRefMake) -> Self {
        self.ascribed_type = Some(Box::new(f));
        self
    }
}
impl AstMake for StaticDefMake {
    type I = StaticDef;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::STATIC_DEF);
        if let Some(b) = self.body {
            b.make(builder);
        }
        if let Some(b) = self.visibility {
            b.make(builder);
        }
        if let Some(b) = self.name {
            b.make(builder);
        }
        if let Some(b) = self.type_param_list {
            b.make(builder);
        }
        if let Some(b) = self.where_clause {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        if let Some(b) = self.ascribed_type {
            b.make(builder);
        }
        builder.finish_node();
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
impl Module {
    pub fn new() -> ModuleMake {
        ModuleMake::default()
    }
}
#[derive(Default)]
pub struct ModuleMake {
    item_list: Option<Box<ItemListMake>>,
    visibility: Option<Box<VisibilityMake>>,
    name: Option<Box<NameMake>>,
    attrs: Vec<Box<AttrMake>>,
}
impl ModuleMake {
    pub fn item_list(mut self, f: ItemListMake) -> Self {
        self.item_list = Some(Box::new(f));
        self
    }
    pub fn visibility(mut self, f: VisibilityMake) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameMake) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstMake for ModuleMake {
    type I = Module;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MODULE);
        if let Some(b) = self.item_list {
            b.make(builder);
        }
        if let Some(b) = self.visibility {
            b.make(builder);
        }
        if let Some(b) = self.name {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        builder.finish_node();
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
pub enum NominalDefMake {
    StructDefMake(Box<StructDefMake>),
    UnionDefMake(Box<UnionDefMake>),
    EnumDefMake(Box<EnumDefMake>),
}
impl From<StructDefMake> for NominalDefMake {
    fn from(builder: StructDefMake) -> NominalDefMake {
        NominalDefMake::StructDefMake(Box::new(builder))
    }
}
impl From<UnionDefMake> for NominalDefMake {
    fn from(builder: UnionDefMake) -> NominalDefMake {
        NominalDefMake::UnionDefMake(Box::new(builder))
    }
}
impl From<EnumDefMake> for NominalDefMake {
    fn from(builder: EnumDefMake) -> NominalDefMake {
        NominalDefMake::EnumDefMake(Box::new(builder))
    }
}
impl AstMake for NominalDefMake {
    type I = NominalDef;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        match self {
            NominalDefMake::StructDefMake(b) => b.make(builder),
            NominalDefMake::UnionDefMake(b) => b.make(builder),
            NominalDefMake::EnumDefMake(b) => b.make(builder),
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
impl BlockExpr {
    pub fn new() -> BlockExprMake {
        BlockExprMake::default()
    }
}
#[derive(Default)]
pub struct BlockExprMake {
    block: Option<Box<BlockMake>>,
}
impl BlockExprMake {
    pub fn block(mut self, f: BlockMake) -> Self {
        self.block = Some(Box::new(f));
        self
    }
}
impl AstMake for BlockExprMake {
    type I = BlockExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::BLOCK_EXPR);
        if let Some(b) = self.block {
            b.make(builder);
        }
        builder.finish_node();
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
impl StructDef {
    pub fn new() -> StructDefMake {
        StructDefMake::default()
    }
}
#[derive(Default)]
pub struct StructDefMake {
    visibility: Option<Box<VisibilityMake>>,
    name: Option<Box<NameMake>>,
    type_param_list: Option<Box<TypeParamListMake>>,
    where_clause: Option<Box<WhereClauseMake>>,
    attrs: Vec<Box<AttrMake>>,
}
impl StructDefMake {
    pub fn visibility(mut self, f: VisibilityMake) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameMake) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn type_param_list(mut self, f: TypeParamListMake) -> Self {
        self.type_param_list = Some(Box::new(f));
        self
    }
    pub fn where_clause(mut self, f: WhereClauseMake) -> Self {
        self.where_clause = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstMake for StructDefMake {
    type I = StructDef;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::STRUCT_DEF);
        if let Some(b) = self.visibility {
            b.make(builder);
        }
        if let Some(b) = self.name {
            b.make(builder);
        }
        if let Some(b) = self.type_param_list {
            b.make(builder);
        }
        if let Some(b) = self.where_clause {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        builder.finish_node();
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
impl UnionDef {
    pub fn new() -> UnionDefMake {
        UnionDefMake::default()
    }
}
#[derive(Default)]
pub struct UnionDefMake {
    visibility: Option<Box<VisibilityMake>>,
    name: Option<Box<NameMake>>,
    type_param_list: Option<Box<TypeParamListMake>>,
    where_clause: Option<Box<WhereClauseMake>>,
    attrs: Vec<Box<AttrMake>>,
    record_field_def_list: Option<Box<RecordFieldDefListMake>>,
}
impl UnionDefMake {
    pub fn visibility(mut self, f: VisibilityMake) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameMake) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn type_param_list(mut self, f: TypeParamListMake) -> Self {
        self.type_param_list = Some(Box::new(f));
        self
    }
    pub fn where_clause(mut self, f: WhereClauseMake) -> Self {
        self.where_clause = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
    pub fn record_field_def_list(mut self, f: RecordFieldDefListMake) -> Self {
        self.record_field_def_list = Some(Box::new(f));
        self
    }
}
impl AstMake for UnionDefMake {
    type I = UnionDef;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::UNION_DEF);
        if let Some(b) = self.visibility {
            b.make(builder);
        }
        if let Some(b) = self.name {
            b.make(builder);
        }
        if let Some(b) = self.type_param_list {
            b.make(builder);
        }
        if let Some(b) = self.where_clause {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        if let Some(b) = self.record_field_def_list {
            b.make(builder);
        }
        builder.finish_node();
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
impl EnumDef {
    pub fn new() -> EnumDefMake {
        EnumDefMake::default()
    }
}
#[derive(Default)]
pub struct EnumDefMake {
    variant_list: Option<Box<EnumVariantListMake>>,
    visibility: Option<Box<VisibilityMake>>,
    name: Option<Box<NameMake>>,
    type_param_list: Option<Box<TypeParamListMake>>,
    where_clause: Option<Box<WhereClauseMake>>,
    attrs: Vec<Box<AttrMake>>,
}
impl EnumDefMake {
    pub fn variant_list(mut self, f: EnumVariantListMake) -> Self {
        self.variant_list = Some(Box::new(f));
        self
    }
    pub fn visibility(mut self, f: VisibilityMake) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameMake) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn type_param_list(mut self, f: TypeParamListMake) -> Self {
        self.type_param_list = Some(Box::new(f));
        self
    }
    pub fn where_clause(mut self, f: WhereClauseMake) -> Self {
        self.where_clause = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstMake for EnumDefMake {
    type I = EnumDef;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ENUM_DEF);
        if let Some(b) = self.variant_list {
            b.make(builder);
        }
        if let Some(b) = self.visibility {
            b.make(builder);
        }
        if let Some(b) = self.name {
            b.make(builder);
        }
        if let Some(b) = self.type_param_list {
            b.make(builder);
        }
        if let Some(b) = self.where_clause {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        builder.finish_node();
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
impl FnDef {
    pub fn new() -> FnDefMake {
        FnDefMake::default()
    }
}
#[derive(Default)]
pub struct FnDefMake {
    param_list: Option<Box<ParamListMake>>,
    body: Option<Box<BlockExprMake>>,
    ret_type: Option<Box<RetTypeMake>>,
    visibility: Option<Box<VisibilityMake>>,
    name: Option<Box<NameMake>>,
    type_param_list: Option<Box<TypeParamListMake>>,
    where_clause: Option<Box<WhereClauseMake>>,
    attrs: Vec<Box<AttrMake>>,
}
impl FnDefMake {
    pub fn param_list(mut self, f: ParamListMake) -> Self {
        self.param_list = Some(Box::new(f));
        self
    }
    pub fn body(mut self, f: BlockExprMake) -> Self {
        self.body = Some(Box::new(f));
        self
    }
    pub fn ret_type(mut self, f: RetTypeMake) -> Self {
        self.ret_type = Some(Box::new(f));
        self
    }
    pub fn visibility(mut self, f: VisibilityMake) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameMake) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn type_param_list(mut self, f: TypeParamListMake) -> Self {
        self.type_param_list = Some(Box::new(f));
        self
    }
    pub fn where_clause(mut self, f: WhereClauseMake) -> Self {
        self.where_clause = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstMake for FnDefMake {
    type I = FnDef;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::FN_DEF);
        if let Some(b) = self.param_list {
            b.make(builder);
        }
        if let Some(b) = self.body {
            b.make(builder);
        }
        if let Some(b) = self.ret_type {
            b.make(builder);
        }
        if let Some(b) = self.visibility {
            b.make(builder);
        }
        if let Some(b) = self.name {
            b.make(builder);
        }
        if let Some(b) = self.type_param_list {
            b.make(builder);
        }
        if let Some(b) = self.where_clause {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        builder.finish_node();
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
impl TypeAliasDef {
    pub fn new() -> TypeAliasDefMake {
        TypeAliasDefMake::default()
    }
}
#[derive(Default)]
pub struct TypeAliasDefMake {
    type_ref: Option<Box<TypeRefMake>>,
    visibility: Option<Box<VisibilityMake>>,
    name: Option<Box<NameMake>>,
    type_param_list: Option<Box<TypeParamListMake>>,
    where_clause: Option<Box<WhereClauseMake>>,
    attrs: Vec<Box<AttrMake>>,
    type_bound_list: Option<Box<TypeBoundListMake>>,
}
impl TypeAliasDefMake {
    pub fn type_ref(mut self, f: TypeRefMake) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
    pub fn visibility(mut self, f: VisibilityMake) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameMake) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn type_param_list(mut self, f: TypeParamListMake) -> Self {
        self.type_param_list = Some(Box::new(f));
        self
    }
    pub fn where_clause(mut self, f: WhereClauseMake) -> Self {
        self.where_clause = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
    pub fn type_bound_list(mut self, f: TypeBoundListMake) -> Self {
        self.type_bound_list = Some(Box::new(f));
        self
    }
}
impl AstMake for TypeAliasDefMake {
    type I = TypeAliasDef;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TYPE_ALIAS_DEF);
        if let Some(b) = self.type_ref {
            b.make(builder);
        }
        if let Some(b) = self.visibility {
            b.make(builder);
        }
        if let Some(b) = self.name {
            b.make(builder);
        }
        if let Some(b) = self.type_param_list {
            b.make(builder);
        }
        if let Some(b) = self.where_clause {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        if let Some(b) = self.type_bound_list {
            b.make(builder);
        }
        builder.finish_node();
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
impl ConstDef {
    pub fn new() -> ConstDefMake {
        ConstDefMake::default()
    }
}
#[derive(Default)]
pub struct ConstDefMake {
    body: Option<Box<ExprMake>>,
    visibility: Option<Box<VisibilityMake>>,
    name: Option<Box<NameMake>>,
    type_param_list: Option<Box<TypeParamListMake>>,
    where_clause: Option<Box<WhereClauseMake>>,
    attrs: Vec<Box<AttrMake>>,
    ascribed_type: Option<Box<TypeRefMake>>,
}
impl ConstDefMake {
    pub fn body(mut self, f: ExprMake) -> Self {
        self.body = Some(Box::new(f));
        self
    }
    pub fn visibility(mut self, f: VisibilityMake) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameMake) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn type_param_list(mut self, f: TypeParamListMake) -> Self {
        self.type_param_list = Some(Box::new(f));
        self
    }
    pub fn where_clause(mut self, f: WhereClauseMake) -> Self {
        self.where_clause = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
    pub fn ascribed_type(mut self, f: TypeRefMake) -> Self {
        self.ascribed_type = Some(Box::new(f));
        self
    }
}
impl AstMake for ConstDefMake {
    type I = ConstDef;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::CONST_DEF);
        if let Some(b) = self.body {
            b.make(builder);
        }
        if let Some(b) = self.visibility {
            b.make(builder);
        }
        if let Some(b) = self.name {
            b.make(builder);
        }
        if let Some(b) = self.type_param_list {
            b.make(builder);
        }
        if let Some(b) = self.where_clause {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        if let Some(b) = self.ascribed_type {
            b.make(builder);
        }
        builder.finish_node();
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
pub enum PatMake {
    RefPatMake(Box<RefPatMake>),
    BoxPatMake(Box<BoxPatMake>),
    BindPatMake(Box<BindPatMake>),
    PlaceholderPatMake(Box<PlaceholderPatMake>),
    DotDotPatMake(Box<DotDotPatMake>),
    PathPatMake(Box<PathPatMake>),
    RecordPatMake(Box<RecordPatMake>),
    TupleStructPatMake(Box<TupleStructPatMake>),
    TuplePatMake(Box<TuplePatMake>),
    SlicePatMake(Box<SlicePatMake>),
    RangePatMake(Box<RangePatMake>),
    LiteralPatMake(Box<LiteralPatMake>),
}
impl From<RefPatMake> for PatMake {
    fn from(builder: RefPatMake) -> PatMake {
        PatMake::RefPatMake(Box::new(builder))
    }
}
impl From<BoxPatMake> for PatMake {
    fn from(builder: BoxPatMake) -> PatMake {
        PatMake::BoxPatMake(Box::new(builder))
    }
}
impl From<BindPatMake> for PatMake {
    fn from(builder: BindPatMake) -> PatMake {
        PatMake::BindPatMake(Box::new(builder))
    }
}
impl From<PlaceholderPatMake> for PatMake {
    fn from(builder: PlaceholderPatMake) -> PatMake {
        PatMake::PlaceholderPatMake(Box::new(builder))
    }
}
impl From<DotDotPatMake> for PatMake {
    fn from(builder: DotDotPatMake) -> PatMake {
        PatMake::DotDotPatMake(Box::new(builder))
    }
}
impl From<PathPatMake> for PatMake {
    fn from(builder: PathPatMake) -> PatMake {
        PatMake::PathPatMake(Box::new(builder))
    }
}
impl From<RecordPatMake> for PatMake {
    fn from(builder: RecordPatMake) -> PatMake {
        PatMake::RecordPatMake(Box::new(builder))
    }
}
impl From<TupleStructPatMake> for PatMake {
    fn from(builder: TupleStructPatMake) -> PatMake {
        PatMake::TupleStructPatMake(Box::new(builder))
    }
}
impl From<TuplePatMake> for PatMake {
    fn from(builder: TuplePatMake) -> PatMake {
        PatMake::TuplePatMake(Box::new(builder))
    }
}
impl From<SlicePatMake> for PatMake {
    fn from(builder: SlicePatMake) -> PatMake {
        PatMake::SlicePatMake(Box::new(builder))
    }
}
impl From<RangePatMake> for PatMake {
    fn from(builder: RangePatMake) -> PatMake {
        PatMake::RangePatMake(Box::new(builder))
    }
}
impl From<LiteralPatMake> for PatMake {
    fn from(builder: LiteralPatMake) -> PatMake {
        PatMake::LiteralPatMake(Box::new(builder))
    }
}
impl AstMake for PatMake {
    type I = Pat;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        match self {
            PatMake::RefPatMake(b) => b.make(builder),
            PatMake::BoxPatMake(b) => b.make(builder),
            PatMake::BindPatMake(b) => b.make(builder),
            PatMake::PlaceholderPatMake(b) => b.make(builder),
            PatMake::DotDotPatMake(b) => b.make(builder),
            PatMake::PathPatMake(b) => b.make(builder),
            PatMake::RecordPatMake(b) => b.make(builder),
            PatMake::TupleStructPatMake(b) => b.make(builder),
            PatMake::TuplePatMake(b) => b.make(builder),
            PatMake::SlicePatMake(b) => b.make(builder),
            PatMake::RangePatMake(b) => b.make(builder),
            PatMake::LiteralPatMake(b) => b.make(builder),
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
impl RefPat {
    pub fn new() -> RefPatMake {
        RefPatMake::default()
    }
}
#[derive(Default)]
pub struct RefPatMake {
    pat: Option<Box<PatMake>>,
}
impl RefPatMake {
    pub fn pat(mut self, f: PatMake) -> Self {
        self.pat = Some(Box::new(f));
        self
    }
}
impl AstMake for RefPatMake {
    type I = RefPat;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::REF_PAT);
        if let Some(b) = self.pat {
            b.make(builder);
        }
        builder.finish_node();
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
impl BoxPat {
    pub fn new() -> BoxPatMake {
        BoxPatMake::default()
    }
}
#[derive(Default)]
pub struct BoxPatMake {
    pat: Option<Box<PatMake>>,
}
impl BoxPatMake {
    pub fn pat(mut self, f: PatMake) -> Self {
        self.pat = Some(Box::new(f));
        self
    }
}
impl AstMake for BoxPatMake {
    type I = BoxPat;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::BOX_PAT);
        if let Some(b) = self.pat {
            b.make(builder);
        }
        builder.finish_node();
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
impl PathPat {
    pub fn new() -> PathPatMake {
        PathPatMake::default()
    }
}
#[derive(Default)]
pub struct PathPatMake {
    path: Option<Box<PathMake>>,
}
impl PathPatMake {
    pub fn path(mut self, f: PathMake) -> Self {
        self.path = Some(Box::new(f));
        self
    }
}
impl AstMake for PathPatMake {
    type I = PathPat;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PATH_PAT);
        if let Some(b) = self.path {
            b.make(builder);
        }
        builder.finish_node();
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
impl RecordPat {
    pub fn new() -> RecordPatMake {
        RecordPatMake::default()
    }
}
#[derive(Default)]
pub struct RecordPatMake {
    record_field_pat_list: Option<Box<RecordFieldPatListMake>>,
    path: Option<Box<PathMake>>,
}
impl RecordPatMake {
    pub fn record_field_pat_list(mut self, f: RecordFieldPatListMake) -> Self {
        self.record_field_pat_list = Some(Box::new(f));
        self
    }
    pub fn path(mut self, f: PathMake) -> Self {
        self.path = Some(Box::new(f));
        self
    }
}
impl AstMake for RecordPatMake {
    type I = RecordPat;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_PAT);
        if let Some(b) = self.record_field_pat_list {
            b.make(builder);
        }
        if let Some(b) = self.path {
            b.make(builder);
        }
        builder.finish_node();
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
impl TupleStructPat {
    pub fn new() -> TupleStructPatMake {
        TupleStructPatMake::default()
    }
}
#[derive(Default)]
pub struct TupleStructPatMake {
    args: Vec<Box<PatMake>>,
    path: Option<Box<PathMake>>,
}
impl TupleStructPatMake {
    pub fn arg(mut self, f: PatMake) -> Self {
        self.args.push(Box::new(f));
        self
    }
    pub fn path(mut self, f: PathMake) -> Self {
        self.path = Some(Box::new(f));
        self
    }
}
impl AstMake for TupleStructPatMake {
    type I = TupleStructPat;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TUPLE_STRUCT_PAT);
        for b in self.args {
            b.make(builder);
        }
        if let Some(b) = self.path {
            b.make(builder);
        }
        builder.finish_node();
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
impl TuplePat {
    pub fn new() -> TuplePatMake {
        TuplePatMake::default()
    }
}
#[derive(Default)]
pub struct TuplePatMake {
    args: Vec<Box<PatMake>>,
}
impl TuplePatMake {
    pub fn arg(mut self, f: PatMake) -> Self {
        self.args.push(Box::new(f));
        self
    }
}
impl AstMake for TuplePatMake {
    type I = TuplePat;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TUPLE_PAT);
        for b in self.args {
            b.make(builder);
        }
        builder.finish_node();
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
impl LiteralPat {
    pub fn new() -> LiteralPatMake {
        LiteralPatMake::default()
    }
}
#[derive(Default)]
pub struct LiteralPatMake {
    literal: Option<Box<LiteralMake>>,
}
impl LiteralPatMake {
    pub fn literal(mut self, f: LiteralMake) -> Self {
        self.literal = Some(Box::new(f));
        self
    }
}
impl AstMake for LiteralPatMake {
    type I = LiteralPat;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::LITERAL_PAT);
        if let Some(b) = self.literal {
            b.make(builder);
        }
        builder.finish_node();
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
pub enum StmtMake {
    ExprStmtMake(Box<ExprStmtMake>),
    LetStmtMake(Box<LetStmtMake>),
}
impl From<ExprStmtMake> for StmtMake {
    fn from(builder: ExprStmtMake) -> StmtMake {
        StmtMake::ExprStmtMake(Box::new(builder))
    }
}
impl From<LetStmtMake> for StmtMake {
    fn from(builder: LetStmtMake) -> StmtMake {
        StmtMake::LetStmtMake(Box::new(builder))
    }
}
impl AstMake for StmtMake {
    type I = Stmt;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        match self {
            StmtMake::ExprStmtMake(b) => b.make(builder),
            StmtMake::LetStmtMake(b) => b.make(builder),
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
impl ExprStmt {
    pub fn new() -> ExprStmtMake {
        ExprStmtMake::default()
    }
}
#[derive(Default)]
pub struct ExprStmtMake {
    expr: Option<Box<ExprMake>>,
}
impl ExprStmtMake {
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstMake for ExprStmtMake {
    type I = ExprStmt;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::EXPR_STMT);
        if let Some(b) = self.expr {
            b.make(builder);
        }
        builder.finish_node();
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
impl LetStmt {
    pub fn new() -> LetStmtMake {
        LetStmtMake::default()
    }
}
#[derive(Default)]
pub struct LetStmtMake {
    pat: Option<Box<PatMake>>,
    initializer: Option<Box<ExprMake>>,
    ascribed_type: Option<Box<TypeRefMake>>,
}
impl LetStmtMake {
    pub fn pat(mut self, f: PatMake) -> Self {
        self.pat = Some(Box::new(f));
        self
    }
    pub fn initializer(mut self, f: ExprMake) -> Self {
        self.initializer = Some(Box::new(f));
        self
    }
    pub fn ascribed_type(mut self, f: TypeRefMake) -> Self {
        self.ascribed_type = Some(Box::new(f));
        self
    }
}
impl AstMake for LetStmtMake {
    type I = LetStmt;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::LET_STMT);
        if let Some(b) = self.pat {
            b.make(builder);
        }
        if let Some(b) = self.initializer {
            b.make(builder);
        }
        if let Some(b) = self.ascribed_type {
            b.make(builder);
        }
        builder.finish_node();
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
pub enum TypeRefMake {
    ParenTypeMake(Box<ParenTypeMake>),
    TupleTypeMake(Box<TupleTypeMake>),
    NeverTypeMake(Box<NeverTypeMake>),
    PathTypeMake(Box<PathTypeMake>),
    PointerTypeMake(Box<PointerTypeMake>),
    ArrayTypeMake(Box<ArrayTypeMake>),
    SliceTypeMake(Box<SliceTypeMake>),
    ReferenceTypeMake(Box<ReferenceTypeMake>),
    PlaceholderTypeMake(Box<PlaceholderTypeMake>),
    FnPointerTypeMake(Box<FnPointerTypeMake>),
    ForTypeMake(Box<ForTypeMake>),
    ImplTraitTypeMake(Box<ImplTraitTypeMake>),
    DynTraitTypeMake(Box<DynTraitTypeMake>),
}
impl From<ParenTypeMake> for TypeRefMake {
    fn from(builder: ParenTypeMake) -> TypeRefMake {
        TypeRefMake::ParenTypeMake(Box::new(builder))
    }
}
impl From<TupleTypeMake> for TypeRefMake {
    fn from(builder: TupleTypeMake) -> TypeRefMake {
        TypeRefMake::TupleTypeMake(Box::new(builder))
    }
}
impl From<NeverTypeMake> for TypeRefMake {
    fn from(builder: NeverTypeMake) -> TypeRefMake {
        TypeRefMake::NeverTypeMake(Box::new(builder))
    }
}
impl From<PathTypeMake> for TypeRefMake {
    fn from(builder: PathTypeMake) -> TypeRefMake {
        TypeRefMake::PathTypeMake(Box::new(builder))
    }
}
impl From<PointerTypeMake> for TypeRefMake {
    fn from(builder: PointerTypeMake) -> TypeRefMake {
        TypeRefMake::PointerTypeMake(Box::new(builder))
    }
}
impl From<ArrayTypeMake> for TypeRefMake {
    fn from(builder: ArrayTypeMake) -> TypeRefMake {
        TypeRefMake::ArrayTypeMake(Box::new(builder))
    }
}
impl From<SliceTypeMake> for TypeRefMake {
    fn from(builder: SliceTypeMake) -> TypeRefMake {
        TypeRefMake::SliceTypeMake(Box::new(builder))
    }
}
impl From<ReferenceTypeMake> for TypeRefMake {
    fn from(builder: ReferenceTypeMake) -> TypeRefMake {
        TypeRefMake::ReferenceTypeMake(Box::new(builder))
    }
}
impl From<PlaceholderTypeMake> for TypeRefMake {
    fn from(builder: PlaceholderTypeMake) -> TypeRefMake {
        TypeRefMake::PlaceholderTypeMake(Box::new(builder))
    }
}
impl From<FnPointerTypeMake> for TypeRefMake {
    fn from(builder: FnPointerTypeMake) -> TypeRefMake {
        TypeRefMake::FnPointerTypeMake(Box::new(builder))
    }
}
impl From<ForTypeMake> for TypeRefMake {
    fn from(builder: ForTypeMake) -> TypeRefMake {
        TypeRefMake::ForTypeMake(Box::new(builder))
    }
}
impl From<ImplTraitTypeMake> for TypeRefMake {
    fn from(builder: ImplTraitTypeMake) -> TypeRefMake {
        TypeRefMake::ImplTraitTypeMake(Box::new(builder))
    }
}
impl From<DynTraitTypeMake> for TypeRefMake {
    fn from(builder: DynTraitTypeMake) -> TypeRefMake {
        TypeRefMake::DynTraitTypeMake(Box::new(builder))
    }
}
impl AstMake for TypeRefMake {
    type I = TypeRef;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        match self {
            TypeRefMake::ParenTypeMake(b) => b.make(builder),
            TypeRefMake::TupleTypeMake(b) => b.make(builder),
            TypeRefMake::NeverTypeMake(b) => b.make(builder),
            TypeRefMake::PathTypeMake(b) => b.make(builder),
            TypeRefMake::PointerTypeMake(b) => b.make(builder),
            TypeRefMake::ArrayTypeMake(b) => b.make(builder),
            TypeRefMake::SliceTypeMake(b) => b.make(builder),
            TypeRefMake::ReferenceTypeMake(b) => b.make(builder),
            TypeRefMake::PlaceholderTypeMake(b) => b.make(builder),
            TypeRefMake::FnPointerTypeMake(b) => b.make(builder),
            TypeRefMake::ForTypeMake(b) => b.make(builder),
            TypeRefMake::ImplTraitTypeMake(b) => b.make(builder),
            TypeRefMake::DynTraitTypeMake(b) => b.make(builder),
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
impl ParenType {
    pub fn new() -> ParenTypeMake {
        ParenTypeMake::default()
    }
}
#[derive(Default)]
pub struct ParenTypeMake {
    type_ref: Option<Box<TypeRefMake>>,
}
impl ParenTypeMake {
    pub fn type_ref(mut self, f: TypeRefMake) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstMake for ParenTypeMake {
    type I = ParenType;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PAREN_TYPE);
        if let Some(b) = self.type_ref {
            b.make(builder);
        }
        builder.finish_node();
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
impl TupleType {
    pub fn new() -> TupleTypeMake {
        TupleTypeMake::default()
    }
}
#[derive(Default)]
pub struct TupleTypeMake {
    fields: Vec<Box<TypeRefMake>>,
}
impl TupleTypeMake {
    pub fn field(mut self, f: TypeRefMake) -> Self {
        self.fields.push(Box::new(f));
        self
    }
}
impl AstMake for TupleTypeMake {
    type I = TupleType;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TUPLE_TYPE);
        builder.token(SyntaxKind::L_PAREN, SmolStr::new(T_STR!(L_PAREN)));
        for b in self.fields {
            b.make(builder);
        }
        builder.token(SyntaxKind::R_PAREN, SmolStr::new(T_STR!(R_PAREN)));
        builder.finish_node();
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
impl PointerType {
    pub fn new() -> PointerTypeMake {
        PointerTypeMake::default()
    }
}
#[derive(Default)]
pub struct PointerTypeMake {
    type_ref: Option<Box<TypeRefMake>>,
}
impl PointerTypeMake {
    pub fn type_ref(mut self, f: TypeRefMake) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstMake for PointerTypeMake {
    type I = PointerType;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::POINTER_TYPE);
        if let Some(b) = self.type_ref {
            b.make(builder);
        }
        builder.finish_node();
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
impl ArrayType {
    pub fn new() -> ArrayTypeMake {
        ArrayTypeMake::default()
    }
}
#[derive(Default)]
pub struct ArrayTypeMake {
    type_ref: Option<Box<TypeRefMake>>,
    expr: Option<Box<ExprMake>>,
}
impl ArrayTypeMake {
    pub fn type_ref(mut self, f: TypeRefMake) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstMake for ArrayTypeMake {
    type I = ArrayType;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ARRAY_TYPE);
        if let Some(b) = self.type_ref {
            b.make(builder);
        }
        if let Some(b) = self.expr {
            b.make(builder);
        }
        builder.finish_node();
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
impl SliceType {
    pub fn new() -> SliceTypeMake {
        SliceTypeMake::default()
    }
}
#[derive(Default)]
pub struct SliceTypeMake {
    type_ref: Option<Box<TypeRefMake>>,
}
impl SliceTypeMake {
    pub fn type_ref(mut self, f: TypeRefMake) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstMake for SliceTypeMake {
    type I = SliceType;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::SLICE_TYPE);
        if let Some(b) = self.type_ref {
            b.make(builder);
        }
        builder.finish_node();
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
impl ReferenceType {
    pub fn new() -> ReferenceTypeMake {
        ReferenceTypeMake::default()
    }
}
#[derive(Default)]
pub struct ReferenceTypeMake {
    type_ref: Option<Box<TypeRefMake>>,
}
impl ReferenceTypeMake {
    pub fn type_ref(mut self, f: TypeRefMake) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstMake for ReferenceTypeMake {
    type I = ReferenceType;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::REFERENCE_TYPE);
        if let Some(b) = self.type_ref {
            b.make(builder);
        }
        builder.finish_node();
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
impl FnPointerType {
    pub fn new() -> FnPointerTypeMake {
        FnPointerTypeMake::default()
    }
}
#[derive(Default)]
pub struct FnPointerTypeMake {
    param_list: Option<Box<ParamListMake>>,
    ret_type: Option<Box<RetTypeMake>>,
}
impl FnPointerTypeMake {
    pub fn param_list(mut self, f: ParamListMake) -> Self {
        self.param_list = Some(Box::new(f));
        self
    }
    pub fn ret_type(mut self, f: RetTypeMake) -> Self {
        self.ret_type = Some(Box::new(f));
        self
    }
}
impl AstMake for FnPointerTypeMake {
    type I = FnPointerType;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::FN_POINTER_TYPE);
        if let Some(b) = self.param_list {
            b.make(builder);
        }
        if let Some(b) = self.ret_type {
            b.make(builder);
        }
        builder.finish_node();
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
impl ForType {
    pub fn new() -> ForTypeMake {
        ForTypeMake::default()
    }
}
#[derive(Default)]
pub struct ForTypeMake {
    type_ref: Option<Box<TypeRefMake>>,
}
impl ForTypeMake {
    pub fn type_ref(mut self, f: TypeRefMake) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstMake for ForTypeMake {
    type I = ForType;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::FOR_TYPE);
        if let Some(b) = self.type_ref {
            b.make(builder);
        }
        builder.finish_node();
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
impl ImplTraitType {
    pub fn new() -> ImplTraitTypeMake {
        ImplTraitTypeMake::default()
    }
}
#[derive(Default)]
pub struct ImplTraitTypeMake {
    type_bound_list: Option<Box<TypeBoundListMake>>,
}
impl ImplTraitTypeMake {
    pub fn type_bound_list(mut self, f: TypeBoundListMake) -> Self {
        self.type_bound_list = Some(Box::new(f));
        self
    }
}
impl AstMake for ImplTraitTypeMake {
    type I = ImplTraitType;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::IMPL_TRAIT_TYPE);
        if let Some(b) = self.type_bound_list {
            b.make(builder);
        }
        builder.finish_node();
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
impl DynTraitType {
    pub fn new() -> DynTraitTypeMake {
        DynTraitTypeMake::default()
    }
}
#[derive(Default)]
pub struct DynTraitTypeMake {
    type_bound_list: Option<Box<TypeBoundListMake>>,
}
impl DynTraitTypeMake {
    pub fn type_bound_list(mut self, f: TypeBoundListMake) -> Self {
        self.type_bound_list = Some(Box::new(f));
        self
    }
}
impl AstMake for DynTraitTypeMake {
    type I = DynTraitType;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::DYN_TRAIT_TYPE);
        if let Some(b) = self.type_bound_list {
            b.make(builder);
        }
        builder.finish_node();
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
pub enum AttrInputMake {
    LiteralMake(Box<LiteralMake>),
    TokenTreeMake(Box<TokenTreeMake>),
}
impl From<LiteralMake> for AttrInputMake {
    fn from(builder: LiteralMake) -> AttrInputMake {
        AttrInputMake::LiteralMake(Box::new(builder))
    }
}
impl From<TokenTreeMake> for AttrInputMake {
    fn from(builder: TokenTreeMake) -> AttrInputMake {
        AttrInputMake::TokenTreeMake(Box::new(builder))
    }
}
impl AstMake for AttrInputMake {
    type I = AttrInput;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        match self {
            AttrInputMake::LiteralMake(b) => b.make(builder),
            AttrInputMake::TokenTreeMake(b) => b.make(builder),
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
impl Alias {
    pub fn new() -> AliasMake {
        AliasMake::default()
    }
}
#[derive(Default)]
pub struct AliasMake {
    name: Option<Box<NameMake>>,
}
impl AliasMake {
    pub fn name(mut self, f: NameMake) -> Self {
        self.name = Some(Box::new(f));
        self
    }
}
impl AstMake for AliasMake {
    type I = Alias;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ALIAS);
        if let Some(b) = self.name {
            b.make(builder);
        }
        builder.finish_node();
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
impl ArgList {
    pub fn new() -> ArgListMake {
        ArgListMake::default()
    }
}
#[derive(Default)]
pub struct ArgListMake {
    args: Vec<Box<ExprMake>>,
}
impl ArgListMake {
    pub fn arg(mut self, f: ExprMake) -> Self {
        self.args.push(Box::new(f));
        self
    }
}
impl AstMake for ArgListMake {
    type I = ArgList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ARG_LIST);
        for b in self.args {
            b.make(builder);
        }
        builder.finish_node();
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
impl BindPat {
    pub fn new() -> BindPatMake {
        BindPatMake::default()
    }
}
#[derive(Default)]
pub struct BindPatMake {
    pat: Option<Box<PatMake>>,
    name: Option<Box<NameMake>>,
}
impl BindPatMake {
    pub fn pat(mut self, f: PatMake) -> Self {
        self.pat = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameMake) -> Self {
        self.name = Some(Box::new(f));
        self
    }
}
impl AstMake for BindPatMake {
    type I = BindPat;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::BIND_PAT);
        if let Some(b) = self.pat {
            b.make(builder);
        }
        if let Some(b) = self.name {
            b.make(builder);
        }
        builder.finish_node();
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
impl AssocTypeArg {
    pub fn new() -> AssocTypeArgMake {
        AssocTypeArgMake::default()
    }
}
#[derive(Default)]
pub struct AssocTypeArgMake {
    name_ref: Option<Box<NameRefMake>>,
    type_ref: Option<Box<TypeRefMake>>,
}
impl AssocTypeArgMake {
    pub fn name_ref(mut self, f: NameRefMake) -> Self {
        self.name_ref = Some(Box::new(f));
        self
    }
    pub fn type_ref(mut self, f: TypeRefMake) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstMake for AssocTypeArgMake {
    type I = AssocTypeArg;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ASSOC_TYPE_ARG);
        if let Some(b) = self.name_ref {
            b.make(builder);
        }
        if let Some(b) = self.type_ref {
            b.make(builder);
        }
        builder.finish_node();
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
impl Attr {
    pub fn new() -> AttrMake {
        AttrMake::default()
    }
}
#[derive(Default)]
pub struct AttrMake {
    path: Option<Box<PathMake>>,
    input: Option<Box<AttrInputMake>>,
}
impl AttrMake {
    pub fn path(mut self, f: PathMake) -> Self {
        self.path = Some(Box::new(f));
        self
    }
    pub fn input(mut self, f: AttrInputMake) -> Self {
        self.input = Some(Box::new(f));
        self
    }
}
impl AstMake for AttrMake {
    type I = Attr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ATTR);
        if let Some(b) = self.path {
            b.make(builder);
        }
        if let Some(b) = self.input {
            b.make(builder);
        }
        builder.finish_node();
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
impl Block {
    pub fn new() -> BlockMake {
        BlockMake::default()
    }
}
#[derive(Default)]
pub struct BlockMake {
    statements: Vec<Box<StmtMake>>,
    expr: Option<Box<ExprMake>>,
    attrs: Vec<Box<AttrMake>>,
}
impl BlockMake {
    pub fn statement(mut self, f: StmtMake) -> Self {
        self.statements.push(Box::new(f));
        self
    }
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstMake for BlockMake {
    type I = Block;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::BLOCK);
        for b in self.statements {
            b.make(builder);
        }
        if let Some(b) = self.expr {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        builder.finish_node();
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
impl Condition {
    pub fn new() -> ConditionMake {
        ConditionMake::default()
    }
}
#[derive(Default)]
pub struct ConditionMake {
    pat: Option<Box<PatMake>>,
    expr: Option<Box<ExprMake>>,
}
impl ConditionMake {
    pub fn pat(mut self, f: PatMake) -> Self {
        self.pat = Some(Box::new(f));
        self
    }
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstMake for ConditionMake {
    type I = Condition;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::CONDITION);
        if let Some(b) = self.pat {
            b.make(builder);
        }
        if let Some(b) = self.expr {
            b.make(builder);
        }
        builder.finish_node();
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
impl EnumVariant {
    pub fn new() -> EnumVariantMake {
        EnumVariantMake::default()
    }
}
#[derive(Default)]
pub struct EnumVariantMake {
    expr: Option<Box<ExprMake>>,
    name: Option<Box<NameMake>>,
    attrs: Vec<Box<AttrMake>>,
}
impl EnumVariantMake {
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameMake) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstMake for EnumVariantMake {
    type I = EnumVariant;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ENUM_VARIANT);
        if let Some(b) = self.expr {
            b.make(builder);
        }
        if let Some(b) = self.name {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        builder.finish_node();
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
impl EnumVariantList {
    pub fn new() -> EnumVariantListMake {
        EnumVariantListMake::default()
    }
}
#[derive(Default)]
pub struct EnumVariantListMake {
    variants: Vec<Box<EnumVariantMake>>,
}
impl EnumVariantListMake {
    pub fn variant(mut self, f: EnumVariantMake) -> Self {
        self.variants.push(Box::new(f));
        self
    }
}
impl AstMake for EnumVariantListMake {
    type I = EnumVariantList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ENUM_VARIANT_LIST);
        for b in self.variants {
            b.make(builder);
        }
        builder.finish_node();
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
impl RecordFieldPat {
    pub fn new() -> RecordFieldPatMake {
        RecordFieldPatMake::default()
    }
}
#[derive(Default)]
pub struct RecordFieldPatMake {
    pat: Option<Box<PatMake>>,
    name: Option<Box<NameMake>>,
}
impl RecordFieldPatMake {
    pub fn pat(mut self, f: PatMake) -> Self {
        self.pat = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameMake) -> Self {
        self.name = Some(Box::new(f));
        self
    }
}
impl AstMake for RecordFieldPatMake {
    type I = RecordFieldPat;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_FIELD_PAT);
        if let Some(b) = self.pat {
            b.make(builder);
        }
        if let Some(b) = self.name {
            b.make(builder);
        }
        builder.finish_node();
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
impl RecordFieldPatList {
    pub fn new() -> RecordFieldPatListMake {
        RecordFieldPatListMake::default()
    }
}
#[derive(Default)]
pub struct RecordFieldPatListMake {
    record_field_pats: Vec<Box<RecordFieldPatMake>>,
    bind_pats: Vec<Box<BindPatMake>>,
}
impl RecordFieldPatListMake {
    pub fn record_field_pat(mut self, f: RecordFieldPatMake) -> Self {
        self.record_field_pats.push(Box::new(f));
        self
    }
    pub fn bind_pat(mut self, f: BindPatMake) -> Self {
        self.bind_pats.push(Box::new(f));
        self
    }
}
impl AstMake for RecordFieldPatListMake {
    type I = RecordFieldPatList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_FIELD_PAT_LIST);
        for b in self.record_field_pats {
            b.make(builder);
        }
        for b in self.bind_pats {
            b.make(builder);
        }
        builder.finish_node();
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
impl ItemList {
    pub fn new() -> ItemListMake {
        ItemListMake::default()
    }
}
#[derive(Default)]
pub struct ItemListMake {
    impl_items: Vec<Box<ImplItemMake>>,
    functions: Vec<Box<FnDefMake>>,
    items: Vec<Box<ModuleItemMake>>,
}
impl ItemListMake {
    pub fn impl_item(mut self, f: ImplItemMake) -> Self {
        self.impl_items.push(Box::new(f));
        self
    }
    pub fn function(mut self, f: FnDefMake) -> Self {
        self.functions.push(Box::new(f));
        self
    }
    pub fn item(mut self, f: ModuleItemMake) -> Self {
        self.items.push(Box::new(f));
        self
    }
}
impl AstMake for ItemListMake {
    type I = ItemList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ITEM_LIST);
        for b in self.impl_items {
            b.make(builder);
        }
        for b in self.functions {
            b.make(builder);
        }
        for b in self.items {
            b.make(builder);
        }
        builder.finish_node();
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
impl LifetimeParam {
    pub fn new() -> LifetimeParamMake {
        LifetimeParamMake::default()
    }
}
#[derive(Default)]
pub struct LifetimeParamMake {
    attrs: Vec<Box<AttrMake>>,
}
impl LifetimeParamMake {
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstMake for LifetimeParamMake {
    type I = LifetimeParam;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::LIFETIME_PARAM);
        for b in self.attrs {
            b.make(builder);
        }
        builder.finish_node();
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
impl MacroItems {
    pub fn new() -> MacroItemsMake {
        MacroItemsMake::default()
    }
}
#[derive(Default)]
pub struct MacroItemsMake {
    items: Vec<Box<ModuleItemMake>>,
    functions: Vec<Box<FnDefMake>>,
}
impl MacroItemsMake {
    pub fn item(mut self, f: ModuleItemMake) -> Self {
        self.items.push(Box::new(f));
        self
    }
    pub fn function(mut self, f: FnDefMake) -> Self {
        self.functions.push(Box::new(f));
        self
    }
}
impl AstMake for MacroItemsMake {
    type I = MacroItems;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MACRO_ITEMS);
        for b in self.items {
            b.make(builder);
        }
        for b in self.functions {
            b.make(builder);
        }
        builder.finish_node();
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
impl MacroStmts {
    pub fn new() -> MacroStmtsMake {
        MacroStmtsMake::default()
    }
}
#[derive(Default)]
pub struct MacroStmtsMake {
    statements: Vec<Box<StmtMake>>,
    expr: Option<Box<ExprMake>>,
}
impl MacroStmtsMake {
    pub fn statement(mut self, f: StmtMake) -> Self {
        self.statements.push(Box::new(f));
        self
    }
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstMake for MacroStmtsMake {
    type I = MacroStmts;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MACRO_STMTS);
        for b in self.statements {
            b.make(builder);
        }
        if let Some(b) = self.expr {
            b.make(builder);
        }
        builder.finish_node();
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
impl MatchArm {
    pub fn new() -> MatchArmMake {
        MatchArmMake::default()
    }
}
#[derive(Default)]
pub struct MatchArmMake {
    pats: Vec<Box<PatMake>>,
    guard: Option<Box<MatchGuardMake>>,
    expr: Option<Box<ExprMake>>,
    attrs: Vec<Box<AttrMake>>,
}
impl MatchArmMake {
    pub fn pat(mut self, f: PatMake) -> Self {
        self.pats.push(Box::new(f));
        self
    }
    pub fn guard(mut self, f: MatchGuardMake) -> Self {
        self.guard = Some(Box::new(f));
        self
    }
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstMake for MatchArmMake {
    type I = MatchArm;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MATCH_ARM);
        for b in self.pats {
            b.make(builder);
        }
        if let Some(b) = self.guard {
            b.make(builder);
        }
        if let Some(b) = self.expr {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        builder.finish_node();
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
impl MatchArmList {
    pub fn new() -> MatchArmListMake {
        MatchArmListMake::default()
    }
}
#[derive(Default)]
pub struct MatchArmListMake {
    arms: Vec<Box<MatchArmMake>>,
    attrs: Vec<Box<AttrMake>>,
}
impl MatchArmListMake {
    pub fn arm(mut self, f: MatchArmMake) -> Self {
        self.arms.push(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstMake for MatchArmListMake {
    type I = MatchArmList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MATCH_ARM_LIST);
        for b in self.arms {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        builder.finish_node();
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
impl MatchGuard {
    pub fn new() -> MatchGuardMake {
        MatchGuardMake::default()
    }
}
#[derive(Default)]
pub struct MatchGuardMake {
    expr: Option<Box<ExprMake>>,
}
impl MatchGuardMake {
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstMake for MatchGuardMake {
    type I = MatchGuard;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MATCH_GUARD);
        if let Some(b) = self.expr {
            b.make(builder);
        }
        builder.finish_node();
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
impl RecordField {
    pub fn new() -> RecordFieldMake {
        RecordFieldMake::default()
    }
}
#[derive(Default)]
pub struct RecordFieldMake {
    name_ref: Option<Box<NameRefMake>>,
    expr: Option<Box<ExprMake>>,
}
impl RecordFieldMake {
    pub fn name_ref(mut self, f: NameRefMake) -> Self {
        self.name_ref = Some(Box::new(f));
        self
    }
    pub fn expr(mut self, f: ExprMake) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstMake for RecordFieldMake {
    type I = RecordField;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_FIELD);
        if let Some(b) = self.name_ref {
            b.make(builder);
        }
        if let Some(b) = self.expr {
            b.make(builder);
        }
        builder.finish_node();
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
impl RecordFieldDef {
    pub fn new() -> RecordFieldDefMake {
        RecordFieldDefMake::default()
    }
}
#[derive(Default)]
pub struct RecordFieldDefMake {
    visibility: Option<Box<VisibilityMake>>,
    name: Option<Box<NameMake>>,
    attrs: Vec<Box<AttrMake>>,
    ascribed_type: Option<Box<TypeRefMake>>,
}
impl RecordFieldDefMake {
    pub fn visibility(mut self, f: VisibilityMake) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameMake) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
    pub fn ascribed_type(mut self, f: TypeRefMake) -> Self {
        self.ascribed_type = Some(Box::new(f));
        self
    }
}
impl AstMake for RecordFieldDefMake {
    type I = RecordFieldDef;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_FIELD_DEF);
        if let Some(b) = self.visibility {
            b.make(builder);
        }
        if let Some(b) = self.name {
            b.make(builder);
        }
        builder.token(SyntaxKind::COLON, SmolStr::new(T_STR!(COLON)));
        for b in self.attrs {
            b.make(builder);
        }
        if let Some(b) = self.ascribed_type {
            b.make(builder);
        }
        builder.finish_node();
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
impl RecordFieldDefList {
    pub fn new() -> RecordFieldDefListMake {
        RecordFieldDefListMake::default()
    }
}
#[derive(Default)]
pub struct RecordFieldDefListMake {
    fields: Vec<Box<RecordFieldDefMake>>,
}
impl RecordFieldDefListMake {
    pub fn field(mut self, f: RecordFieldDefMake) -> Self {
        self.fields.push(Box::new(f));
        self
    }
}
impl AstMake for RecordFieldDefListMake {
    type I = RecordFieldDefList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_FIELD_DEF_LIST);
        builder.token(SyntaxKind::L_CURLY, SmolStr::new(T_STR!(L_CURLY)));
        for b in self.fields {
            b.make(builder);
            builder.token(SyntaxKind::COMMA, SmolStr::new(T_STR!(COMMA)));
        }
        builder.token(SyntaxKind::R_CURLY, SmolStr::new(T_STR!(R_CURLY)));
        builder.finish_node();
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
impl RecordFieldList {
    pub fn new() -> RecordFieldListMake {
        RecordFieldListMake::default()
    }
}
#[derive(Default)]
pub struct RecordFieldListMake {
    fields: Vec<Box<RecordFieldMake>>,
    spread: Option<Box<ExprMake>>,
}
impl RecordFieldListMake {
    pub fn field(mut self, f: RecordFieldMake) -> Self {
        self.fields.push(Box::new(f));
        self
    }
    pub fn spread(mut self, f: ExprMake) -> Self {
        self.spread = Some(Box::new(f));
        self
    }
}
impl AstMake for RecordFieldListMake {
    type I = RecordFieldList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_FIELD_LIST);
        builder.token(SyntaxKind::L_CURLY, SmolStr::new(T_STR!(L_CURLY)));
        for b in self.fields {
            b.make(builder);
        }
        if let Some(b) = self.spread {
            b.make(builder);
        }
        builder.token(SyntaxKind::R_CURLY, SmolStr::new(T_STR!(R_CURLY)));
        builder.finish_node();
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
impl Param {
    pub fn new() -> ParamMake {
        ParamMake::default()
    }
}
#[derive(Default)]
pub struct ParamMake {
    pat: Option<Box<PatMake>>,
    ascribed_type: Option<Box<TypeRefMake>>,
    attrs: Vec<Box<AttrMake>>,
}
impl ParamMake {
    pub fn pat(mut self, f: PatMake) -> Self {
        self.pat = Some(Box::new(f));
        self
    }
    pub fn ascribed_type(mut self, f: TypeRefMake) -> Self {
        self.ascribed_type = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstMake for ParamMake {
    type I = Param;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PARAM);
        if let Some(b) = self.pat {
            b.make(builder);
        }
        if let Some(b) = self.ascribed_type {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        builder.finish_node();
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
impl ParamList {
    pub fn new() -> ParamListMake {
        ParamListMake::default()
    }
}
#[derive(Default)]
pub struct ParamListMake {
    params: Vec<Box<ParamMake>>,
    self_param: Option<Box<SelfParamMake>>,
}
impl ParamListMake {
    pub fn param(mut self, f: ParamMake) -> Self {
        self.params.push(Box::new(f));
        self
    }
    pub fn self_param(mut self, f: SelfParamMake) -> Self {
        self.self_param = Some(Box::new(f));
        self
    }
}
impl AstMake for ParamListMake {
    type I = ParamList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PARAM_LIST);
        for b in self.params {
            b.make(builder);
        }
        if let Some(b) = self.self_param {
            b.make(builder);
        }
        builder.finish_node();
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
impl PathType {
    pub fn new() -> PathTypeMake {
        PathTypeMake::default()
    }
}
#[derive(Default)]
pub struct PathTypeMake {
    path: Option<Box<PathMake>>,
}
impl PathTypeMake {
    pub fn path(mut self, f: PathMake) -> Self {
        self.path = Some(Box::new(f));
        self
    }
}
impl AstMake for PathTypeMake {
    type I = PathType;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PATH_TYPE);
        if let Some(b) = self.path {
            b.make(builder);
        }
        builder.finish_node();
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
impl Path {
    pub fn new() -> PathMake {
        PathMake::default()
    }
}
#[derive(Default)]
pub struct PathMake {
    segment: Option<Box<PathSegmentMake>>,
    qualifier: Option<Box<PathMake>>,
}
impl PathMake {
    pub fn segment(mut self, f: PathSegmentMake) -> Self {
        self.segment = Some(Box::new(f));
        self
    }
    pub fn qualifier(mut self, f: PathMake) -> Self {
        self.qualifier = Some(Box::new(f));
        self
    }
}
impl AstMake for PathMake {
    type I = Path;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PATH);
        if let Some(b) = self.segment {
            b.make(builder);
        }
        if let Some(b) = self.qualifier {
            b.make(builder);
        }
        builder.finish_node();
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
impl PathSegment {
    pub fn new() -> PathSegmentMake {
        PathSegmentMake::default()
    }
}
#[derive(Default)]
pub struct PathSegmentMake {
    name_ref: Option<Box<NameRefMake>>,
    type_arg_list: Option<Box<TypeArgListMake>>,
    param_list: Option<Box<ParamListMake>>,
    ret_type: Option<Box<RetTypeMake>>,
    path_type: Option<Box<PathTypeMake>>,
}
impl PathSegmentMake {
    pub fn name_ref(mut self, f: NameRefMake) -> Self {
        self.name_ref = Some(Box::new(f));
        self
    }
    pub fn type_arg_list(mut self, f: TypeArgListMake) -> Self {
        self.type_arg_list = Some(Box::new(f));
        self
    }
    pub fn param_list(mut self, f: ParamListMake) -> Self {
        self.param_list = Some(Box::new(f));
        self
    }
    pub fn ret_type(mut self, f: RetTypeMake) -> Self {
        self.ret_type = Some(Box::new(f));
        self
    }
    pub fn path_type(mut self, f: PathTypeMake) -> Self {
        self.path_type = Some(Box::new(f));
        self
    }
}
impl AstMake for PathSegmentMake {
    type I = PathSegment;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PATH_SEGMENT);
        if let Some(b) = self.name_ref {
            b.make(builder);
        }
        if let Some(b) = self.type_arg_list {
            b.make(builder);
        }
        if let Some(b) = self.param_list {
            b.make(builder);
        }
        if let Some(b) = self.ret_type {
            b.make(builder);
        }
        if let Some(b) = self.path_type {
            b.make(builder);
        }
        builder.finish_node();
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
impl TupleFieldDef {
    pub fn new() -> TupleFieldDefMake {
        TupleFieldDefMake::default()
    }
}
#[derive(Default)]
pub struct TupleFieldDefMake {
    type_ref: Option<Box<TypeRefMake>>,
    visibility: Option<Box<VisibilityMake>>,
    attrs: Vec<Box<AttrMake>>,
}
impl TupleFieldDefMake {
    pub fn type_ref(mut self, f: TypeRefMake) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
    pub fn visibility(mut self, f: VisibilityMake) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstMake for TupleFieldDefMake {
    type I = TupleFieldDef;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TUPLE_FIELD_DEF);
        if let Some(b) = self.type_ref {
            b.make(builder);
        }
        if let Some(b) = self.visibility {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        builder.finish_node();
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
impl TupleFieldDefList {
    pub fn new() -> TupleFieldDefListMake {
        TupleFieldDefListMake::default()
    }
}
#[derive(Default)]
pub struct TupleFieldDefListMake {
    fields: Vec<Box<TupleFieldDefMake>>,
}
impl TupleFieldDefListMake {
    pub fn field(mut self, f: TupleFieldDefMake) -> Self {
        self.fields.push(Box::new(f));
        self
    }
}
impl AstMake for TupleFieldDefListMake {
    type I = TupleFieldDefList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TUPLE_FIELD_DEF_LIST);
        for b in self.fields {
            b.make(builder);
        }
        builder.finish_node();
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
impl RetType {
    pub fn new() -> RetTypeMake {
        RetTypeMake::default()
    }
}
#[derive(Default)]
pub struct RetTypeMake {
    type_ref: Option<Box<TypeRefMake>>,
}
impl RetTypeMake {
    pub fn type_ref(mut self, f: TypeRefMake) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstMake for RetTypeMake {
    type I = RetType;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RET_TYPE);
        if let Some(b) = self.type_ref {
            b.make(builder);
        }
        builder.finish_node();
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
impl SelfParam {
    pub fn new() -> SelfParamMake {
        SelfParamMake::default()
    }
}
#[derive(Default)]
pub struct SelfParamMake {
    ascribed_type: Option<Box<TypeRefMake>>,
    attrs: Vec<Box<AttrMake>>,
}
impl SelfParamMake {
    pub fn ascribed_type(mut self, f: TypeRefMake) -> Self {
        self.ascribed_type = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstMake for SelfParamMake {
    type I = SelfParam;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::SELF_PARAM);
        if let Some(b) = self.ascribed_type {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        builder.finish_node();
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
impl SourceFile {
    pub fn new() -> SourceFileMake {
        SourceFileMake::default()
    }
}
#[derive(Default)]
pub struct SourceFileMake {
    items: Vec<Box<ModuleItemMake>>,
    functions: Vec<Box<FnDefMake>>,
}
impl SourceFileMake {
    pub fn item(mut self, f: ModuleItemMake) -> Self {
        self.items.push(Box::new(f));
        self
    }
    pub fn function(mut self, f: FnDefMake) -> Self {
        self.functions.push(Box::new(f));
        self
    }
}
impl AstMake for SourceFileMake {
    type I = SourceFile;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::SOURCE_FILE);
        for b in self.items {
            b.make(builder);
        }
        for b in self.functions {
            b.make(builder);
        }
        builder.finish_node();
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
impl TypeArg {
    pub fn new() -> TypeArgMake {
        TypeArgMake::default()
    }
}
#[derive(Default)]
pub struct TypeArgMake {
    type_ref: Option<Box<TypeRefMake>>,
}
impl TypeArgMake {
    pub fn type_ref(mut self, f: TypeRefMake) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstMake for TypeArgMake {
    type I = TypeArg;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TYPE_ARG);
        if let Some(b) = self.type_ref {
            b.make(builder);
        }
        builder.finish_node();
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
impl TypeArgList {
    pub fn new() -> TypeArgListMake {
        TypeArgListMake::default()
    }
}
#[derive(Default)]
pub struct TypeArgListMake {
    type_args: Vec<Box<TypeArgMake>>,
    lifetime_args: Vec<Box<LifetimeArgMake>>,
    assoc_type_args: Vec<Box<AssocTypeArgMake>>,
}
impl TypeArgListMake {
    pub fn type_arg(mut self, f: TypeArgMake) -> Self {
        self.type_args.push(Box::new(f));
        self
    }
    pub fn lifetime_arg(mut self, f: LifetimeArgMake) -> Self {
        self.lifetime_args.push(Box::new(f));
        self
    }
    pub fn assoc_type_arg(mut self, f: AssocTypeArgMake) -> Self {
        self.assoc_type_args.push(Box::new(f));
        self
    }
}
impl AstMake for TypeArgListMake {
    type I = TypeArgList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TYPE_ARG_LIST);
        for b in self.type_args {
            b.make(builder);
        }
        for b in self.lifetime_args {
            b.make(builder);
        }
        for b in self.assoc_type_args {
            b.make(builder);
        }
        builder.finish_node();
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
impl TypeBound {
    pub fn new() -> TypeBoundMake {
        TypeBoundMake::default()
    }
}
#[derive(Default)]
pub struct TypeBoundMake {
    type_ref: Option<Box<TypeRefMake>>,
}
impl TypeBoundMake {
    pub fn type_ref(mut self, f: TypeRefMake) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstMake for TypeBoundMake {
    type I = TypeBound;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TYPE_BOUND);
        if let Some(b) = self.type_ref {
            b.make(builder);
        }
        builder.finish_node();
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
impl TypeBoundList {
    pub fn new() -> TypeBoundListMake {
        TypeBoundListMake::default()
    }
}
#[derive(Default)]
pub struct TypeBoundListMake {
    bounds: Vec<Box<TypeBoundMake>>,
}
impl TypeBoundListMake {
    pub fn bound(mut self, f: TypeBoundMake) -> Self {
        self.bounds.push(Box::new(f));
        self
    }
}
impl AstMake for TypeBoundListMake {
    type I = TypeBoundList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TYPE_BOUND_LIST);
        for b in self.bounds {
            b.make(builder);
        }
        builder.finish_node();
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
impl TypeParam {
    pub fn new() -> TypeParamMake {
        TypeParamMake::default()
    }
}
#[derive(Default)]
pub struct TypeParamMake {
    name: Option<Box<NameMake>>,
    attrs: Vec<Box<AttrMake>>,
    type_bound_list: Option<Box<TypeBoundListMake>>,
    default_type: Option<Box<TypeRefMake>>,
}
impl TypeParamMake {
    pub fn name(mut self, f: NameMake) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrMake) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
    pub fn type_bound_list(mut self, f: TypeBoundListMake) -> Self {
        self.type_bound_list = Some(Box::new(f));
        self
    }
    pub fn default_type(mut self, f: TypeRefMake) -> Self {
        self.default_type = Some(Box::new(f));
        self
    }
}
impl AstMake for TypeParamMake {
    type I = TypeParam;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TYPE_PARAM);
        if let Some(b) = self.name {
            b.make(builder);
        }
        for b in self.attrs {
            b.make(builder);
        }
        if let Some(b) = self.type_bound_list {
            b.make(builder);
        }
        if let Some(b) = self.default_type {
            b.make(builder);
        }
        builder.finish_node();
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
impl TypeParamList {
    pub fn new() -> TypeParamListMake {
        TypeParamListMake::default()
    }
}
#[derive(Default)]
pub struct TypeParamListMake {
    type_params: Vec<Box<TypeParamMake>>,
    lifetime_params: Vec<Box<LifetimeParamMake>>,
}
impl TypeParamListMake {
    pub fn type_param(mut self, f: TypeParamMake) -> Self {
        self.type_params.push(Box::new(f));
        self
    }
    pub fn lifetime_param(mut self, f: LifetimeParamMake) -> Self {
        self.lifetime_params.push(Box::new(f));
        self
    }
}
impl AstMake for TypeParamListMake {
    type I = TypeParamList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TYPE_PARAM_LIST);
        for b in self.type_params {
            b.make(builder);
        }
        for b in self.lifetime_params {
            b.make(builder);
        }
        builder.finish_node();
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
impl UseTree {
    pub fn new() -> UseTreeMake {
        UseTreeMake::default()
    }
}
#[derive(Default)]
pub struct UseTreeMake {
    path: Option<Box<PathMake>>,
    use_tree_list: Option<Box<UseTreeListMake>>,
    alias: Option<Box<AliasMake>>,
}
impl UseTreeMake {
    pub fn path(mut self, f: PathMake) -> Self {
        self.path = Some(Box::new(f));
        self
    }
    pub fn use_tree_list(mut self, f: UseTreeListMake) -> Self {
        self.use_tree_list = Some(Box::new(f));
        self
    }
    pub fn alia(mut self, f: AliasMake) -> Self {
        self.alias = Some(Box::new(f));
        self
    }
}
impl AstMake for UseTreeMake {
    type I = UseTree;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::USE_TREE);
        if let Some(b) = self.path {
            b.make(builder);
        }
        if let Some(b) = self.use_tree_list {
            b.make(builder);
        }
        if let Some(b) = self.alias {
            b.make(builder);
        }
        builder.finish_node();
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
impl UseTreeList {
    pub fn new() -> UseTreeListMake {
        UseTreeListMake::default()
    }
}
#[derive(Default)]
pub struct UseTreeListMake {
    use_trees: Vec<Box<UseTreeMake>>,
}
impl UseTreeListMake {
    pub fn use_tree(mut self, f: UseTreeMake) -> Self {
        self.use_trees.push(Box::new(f));
        self
    }
}
impl AstMake for UseTreeListMake {
    type I = UseTreeList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::USE_TREE_LIST);
        for b in self.use_trees {
            b.make(builder);
        }
        builder.finish_node();
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
impl WhereClause {
    pub fn new() -> WhereClauseMake {
        WhereClauseMake::default()
    }
}
#[derive(Default)]
pub struct WhereClauseMake {
    predicates: Vec<Box<WherePredMake>>,
}
impl WhereClauseMake {
    pub fn predicate(mut self, f: WherePredMake) -> Self {
        self.predicates.push(Box::new(f));
        self
    }
}
impl AstMake for WhereClauseMake {
    type I = WhereClause;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::WHERE_CLAUSE);
        for b in self.predicates {
            b.make(builder);
        }
        builder.finish_node();
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
impl WherePred {
    pub fn new() -> WherePredMake {
        WherePredMake::default()
    }
}
#[derive(Default)]
pub struct WherePredMake {
    type_ref: Option<Box<TypeRefMake>>,
    type_bound_list: Option<Box<TypeBoundListMake>>,
}
impl WherePredMake {
    pub fn type_ref(mut self, f: TypeRefMake) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
    pub fn type_bound_list(mut self, f: TypeBoundListMake) -> Self {
        self.type_bound_list = Some(Box::new(f));
        self
    }
}
impl AstMake for WherePredMake {
    type I = WherePred;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::WHERE_PRED);
        if let Some(b) = self.type_ref {
            b.make(builder);
        }
        if let Some(b) = self.type_bound_list {
            b.make(builder);
        }
        builder.finish_node();
    }
}
