//! Generated file, do not edit by hand, see `crate/ra_tools/src/codegen`

use crate::{
    ast::{self, builders::*, traits::CommentIter, AstChildren, AstNode},
    SmolStr,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxTreeBuilder, T_STR,
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
pub enum ExprBuilder {
    TupleExprBuilder(Box<TupleExprBuilder>),
    ArrayExprBuilder(Box<ArrayExprBuilder>),
    ParenExprBuilder(Box<ParenExprBuilder>),
    PathExprBuilder(Box<PathExprBuilder>),
    LambdaExprBuilder(Box<LambdaExprBuilder>),
    IfExprBuilder(Box<IfExprBuilder>),
    LoopExprBuilder(Box<LoopExprBuilder>),
    ForExprBuilder(Box<ForExprBuilder>),
    WhileExprBuilder(Box<WhileExprBuilder>),
    ContinueExprBuilder(Box<ContinueExprBuilder>),
    BreakExprBuilder(Box<BreakExprBuilder>),
    LabelBuilder(Box<LabelBuilder>),
    BlockExprBuilder(Box<BlockExprBuilder>),
    ReturnExprBuilder(Box<ReturnExprBuilder>),
    MatchExprBuilder(Box<MatchExprBuilder>),
    RecordLitBuilder(Box<RecordLitBuilder>),
    CallExprBuilder(Box<CallExprBuilder>),
    IndexExprBuilder(Box<IndexExprBuilder>),
    MethodCallExprBuilder(Box<MethodCallExprBuilder>),
    FieldExprBuilder(Box<FieldExprBuilder>),
    AwaitExprBuilder(Box<AwaitExprBuilder>),
    TryExprBuilder(Box<TryExprBuilder>),
    TryBlockExprBuilder(Box<TryBlockExprBuilder>),
    CastExprBuilder(Box<CastExprBuilder>),
    RefExprBuilder(Box<RefExprBuilder>),
    PrefixExprBuilder(Box<PrefixExprBuilder>),
    BoxExprBuilder(Box<BoxExprBuilder>),
    RangeExprBuilder(Box<RangeExprBuilder>),
    BinExprBuilder(Box<BinExprBuilder>),
    LiteralBuilder(Box<LiteralBuilder>),
    MacroCallBuilder(Box<MacroCallBuilder>),
}
impl From<TupleExprBuilder> for ExprBuilder {
    fn from(builder: TupleExprBuilder) -> ExprBuilder {
        ExprBuilder::TupleExprBuilder(Box::new(builder))
    }
}
impl From<ArrayExprBuilder> for ExprBuilder {
    fn from(builder: ArrayExprBuilder) -> ExprBuilder {
        ExprBuilder::ArrayExprBuilder(Box::new(builder))
    }
}
impl From<ParenExprBuilder> for ExprBuilder {
    fn from(builder: ParenExprBuilder) -> ExprBuilder {
        ExprBuilder::ParenExprBuilder(Box::new(builder))
    }
}
impl From<PathExprBuilder> for ExprBuilder {
    fn from(builder: PathExprBuilder) -> ExprBuilder {
        ExprBuilder::PathExprBuilder(Box::new(builder))
    }
}
impl From<LambdaExprBuilder> for ExprBuilder {
    fn from(builder: LambdaExprBuilder) -> ExprBuilder {
        ExprBuilder::LambdaExprBuilder(Box::new(builder))
    }
}
impl From<IfExprBuilder> for ExprBuilder {
    fn from(builder: IfExprBuilder) -> ExprBuilder {
        ExprBuilder::IfExprBuilder(Box::new(builder))
    }
}
impl From<LoopExprBuilder> for ExprBuilder {
    fn from(builder: LoopExprBuilder) -> ExprBuilder {
        ExprBuilder::LoopExprBuilder(Box::new(builder))
    }
}
impl From<ForExprBuilder> for ExprBuilder {
    fn from(builder: ForExprBuilder) -> ExprBuilder {
        ExprBuilder::ForExprBuilder(Box::new(builder))
    }
}
impl From<WhileExprBuilder> for ExprBuilder {
    fn from(builder: WhileExprBuilder) -> ExprBuilder {
        ExprBuilder::WhileExprBuilder(Box::new(builder))
    }
}
impl From<ContinueExprBuilder> for ExprBuilder {
    fn from(builder: ContinueExprBuilder) -> ExprBuilder {
        ExprBuilder::ContinueExprBuilder(Box::new(builder))
    }
}
impl From<BreakExprBuilder> for ExprBuilder {
    fn from(builder: BreakExprBuilder) -> ExprBuilder {
        ExprBuilder::BreakExprBuilder(Box::new(builder))
    }
}
impl From<LabelBuilder> for ExprBuilder {
    fn from(builder: LabelBuilder) -> ExprBuilder {
        ExprBuilder::LabelBuilder(Box::new(builder))
    }
}
impl From<BlockExprBuilder> for ExprBuilder {
    fn from(builder: BlockExprBuilder) -> ExprBuilder {
        ExprBuilder::BlockExprBuilder(Box::new(builder))
    }
}
impl From<ReturnExprBuilder> for ExprBuilder {
    fn from(builder: ReturnExprBuilder) -> ExprBuilder {
        ExprBuilder::ReturnExprBuilder(Box::new(builder))
    }
}
impl From<MatchExprBuilder> for ExprBuilder {
    fn from(builder: MatchExprBuilder) -> ExprBuilder {
        ExprBuilder::MatchExprBuilder(Box::new(builder))
    }
}
impl From<RecordLitBuilder> for ExprBuilder {
    fn from(builder: RecordLitBuilder) -> ExprBuilder {
        ExprBuilder::RecordLitBuilder(Box::new(builder))
    }
}
impl From<CallExprBuilder> for ExprBuilder {
    fn from(builder: CallExprBuilder) -> ExprBuilder {
        ExprBuilder::CallExprBuilder(Box::new(builder))
    }
}
impl From<IndexExprBuilder> for ExprBuilder {
    fn from(builder: IndexExprBuilder) -> ExprBuilder {
        ExprBuilder::IndexExprBuilder(Box::new(builder))
    }
}
impl From<MethodCallExprBuilder> for ExprBuilder {
    fn from(builder: MethodCallExprBuilder) -> ExprBuilder {
        ExprBuilder::MethodCallExprBuilder(Box::new(builder))
    }
}
impl From<FieldExprBuilder> for ExprBuilder {
    fn from(builder: FieldExprBuilder) -> ExprBuilder {
        ExprBuilder::FieldExprBuilder(Box::new(builder))
    }
}
impl From<AwaitExprBuilder> for ExprBuilder {
    fn from(builder: AwaitExprBuilder) -> ExprBuilder {
        ExprBuilder::AwaitExprBuilder(Box::new(builder))
    }
}
impl From<TryExprBuilder> for ExprBuilder {
    fn from(builder: TryExprBuilder) -> ExprBuilder {
        ExprBuilder::TryExprBuilder(Box::new(builder))
    }
}
impl From<TryBlockExprBuilder> for ExprBuilder {
    fn from(builder: TryBlockExprBuilder) -> ExprBuilder {
        ExprBuilder::TryBlockExprBuilder(Box::new(builder))
    }
}
impl From<CastExprBuilder> for ExprBuilder {
    fn from(builder: CastExprBuilder) -> ExprBuilder {
        ExprBuilder::CastExprBuilder(Box::new(builder))
    }
}
impl From<RefExprBuilder> for ExprBuilder {
    fn from(builder: RefExprBuilder) -> ExprBuilder {
        ExprBuilder::RefExprBuilder(Box::new(builder))
    }
}
impl From<PrefixExprBuilder> for ExprBuilder {
    fn from(builder: PrefixExprBuilder) -> ExprBuilder {
        ExprBuilder::PrefixExprBuilder(Box::new(builder))
    }
}
impl From<BoxExprBuilder> for ExprBuilder {
    fn from(builder: BoxExprBuilder) -> ExprBuilder {
        ExprBuilder::BoxExprBuilder(Box::new(builder))
    }
}
impl From<RangeExprBuilder> for ExprBuilder {
    fn from(builder: RangeExprBuilder) -> ExprBuilder {
        ExprBuilder::RangeExprBuilder(Box::new(builder))
    }
}
impl From<BinExprBuilder> for ExprBuilder {
    fn from(builder: BinExprBuilder) -> ExprBuilder {
        ExprBuilder::BinExprBuilder(Box::new(builder))
    }
}
impl From<LiteralBuilder> for ExprBuilder {
    fn from(builder: LiteralBuilder) -> ExprBuilder {
        ExprBuilder::LiteralBuilder(Box::new(builder))
    }
}
impl From<MacroCallBuilder> for ExprBuilder {
    fn from(builder: MacroCallBuilder) -> ExprBuilder {
        ExprBuilder::MacroCallBuilder(Box::new(builder))
    }
}
impl AstNodeBuilder for ExprBuilder {
    type Node = Expr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        match self {
            ExprBuilder::TupleExprBuilder(b) => b.make(builder),
            ExprBuilder::ArrayExprBuilder(b) => b.make(builder),
            ExprBuilder::ParenExprBuilder(b) => b.make(builder),
            ExprBuilder::PathExprBuilder(b) => b.make(builder),
            ExprBuilder::LambdaExprBuilder(b) => b.make(builder),
            ExprBuilder::IfExprBuilder(b) => b.make(builder),
            ExprBuilder::LoopExprBuilder(b) => b.make(builder),
            ExprBuilder::ForExprBuilder(b) => b.make(builder),
            ExprBuilder::WhileExprBuilder(b) => b.make(builder),
            ExprBuilder::ContinueExprBuilder(b) => b.make(builder),
            ExprBuilder::BreakExprBuilder(b) => b.make(builder),
            ExprBuilder::LabelBuilder(b) => b.make(builder),
            ExprBuilder::BlockExprBuilder(b) => b.make(builder),
            ExprBuilder::ReturnExprBuilder(b) => b.make(builder),
            ExprBuilder::MatchExprBuilder(b) => b.make(builder),
            ExprBuilder::RecordLitBuilder(b) => b.make(builder),
            ExprBuilder::CallExprBuilder(b) => b.make(builder),
            ExprBuilder::IndexExprBuilder(b) => b.make(builder),
            ExprBuilder::MethodCallExprBuilder(b) => b.make(builder),
            ExprBuilder::FieldExprBuilder(b) => b.make(builder),
            ExprBuilder::AwaitExprBuilder(b) => b.make(builder),
            ExprBuilder::TryExprBuilder(b) => b.make(builder),
            ExprBuilder::TryBlockExprBuilder(b) => b.make(builder),
            ExprBuilder::CastExprBuilder(b) => b.make(builder),
            ExprBuilder::RefExprBuilder(b) => b.make(builder),
            ExprBuilder::PrefixExprBuilder(b) => b.make(builder),
            ExprBuilder::BoxExprBuilder(b) => b.make(builder),
            ExprBuilder::RangeExprBuilder(b) => b.make(builder),
            ExprBuilder::BinExprBuilder(b) => b.make(builder),
            ExprBuilder::LiteralBuilder(b) => b.make(builder),
            ExprBuilder::MacroCallBuilder(b) => b.make(builder),
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
    pub fn new() -> TupleExprBuilder {
        TupleExprBuilder::default()
    }
}
#[derive(Default)]
pub struct TupleExprBuilder {
    exprs: Vec<Box<ExprBuilder>>,
}
impl TupleExprBuilder {
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.exprs.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for TupleExprBuilder {
    type Node = TupleExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TUPLE_EXPR);
        self.exprs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> ArrayExprBuilder {
        ArrayExprBuilder::default()
    }
}
#[derive(Default)]
pub struct ArrayExprBuilder {
    exprs: Vec<Box<ExprBuilder>>,
}
impl ArrayExprBuilder {
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.exprs.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ArrayExprBuilder {
    type Node = ArrayExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ARRAY_EXPR);
        self.exprs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> ParenExprBuilder {
        ParenExprBuilder::default()
    }
}
#[derive(Default)]
pub struct ParenExprBuilder {
    expr: Option<Box<ExprBuilder>>,
}
impl ParenExprBuilder {
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ParenExprBuilder {
    type Node = ParenExpr;
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
    pub fn new() -> PathExprBuilder {
        PathExprBuilder::default()
    }
}
#[derive(Default)]
pub struct PathExprBuilder {
    path: Option<Box<PathBuilder>>,
}
impl PathExprBuilder {
    pub fn path(mut self, f: PathBuilder) -> Self {
        self.path = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for PathExprBuilder {
    type Node = PathExpr;
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
    pub fn new() -> LambdaExprBuilder {
        LambdaExprBuilder::default()
    }
}
#[derive(Default)]
pub struct LambdaExprBuilder {
    param_list: Option<Box<ParamListBuilder>>,
    body: Option<Box<ExprBuilder>>,
}
impl LambdaExprBuilder {
    pub fn param_list(mut self, f: ParamListBuilder) -> Self {
        self.param_list = Some(Box::new(f));
        self
    }
    pub fn body(mut self, f: ExprBuilder) -> Self {
        self.body = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for LambdaExprBuilder {
    type Node = LambdaExpr;
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
    pub fn new() -> IfExprBuilder {
        IfExprBuilder::default()
    }
}
#[derive(Default)]
pub struct IfExprBuilder {
    condition: Option<Box<ConditionBuilder>>,
}
impl IfExprBuilder {
    pub fn condition(mut self, f: ConditionBuilder) -> Self {
        self.condition = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for IfExprBuilder {
    type Node = IfExpr;
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
    pub fn new() -> LoopExprBuilder {
        LoopExprBuilder::default()
    }
}
#[derive(Default)]
pub struct LoopExprBuilder {
    loop_body: Option<Box<BlockExprBuilder>>,
}
impl LoopExprBuilder {
    pub fn loop_body(mut self, f: BlockExprBuilder) -> Self {
        self.loop_body = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for LoopExprBuilder {
    type Node = LoopExpr;
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
    pub fn new() -> ForExprBuilder {
        ForExprBuilder::default()
    }
}
#[derive(Default)]
pub struct ForExprBuilder {
    pat: Option<Box<PatBuilder>>,
    iterable: Option<Box<ExprBuilder>>,
    loop_body: Option<Box<BlockExprBuilder>>,
}
impl ForExprBuilder {
    pub fn pat(mut self, f: PatBuilder) -> Self {
        self.pat = Some(Box::new(f));
        self
    }
    pub fn iterable(mut self, f: ExprBuilder) -> Self {
        self.iterable = Some(Box::new(f));
        self
    }
    pub fn loop_body(mut self, f: BlockExprBuilder) -> Self {
        self.loop_body = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ForExprBuilder {
    type Node = ForExpr;
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
    pub fn new() -> WhileExprBuilder {
        WhileExprBuilder::default()
    }
}
#[derive(Default)]
pub struct WhileExprBuilder {
    condition: Option<Box<ConditionBuilder>>,
    loop_body: Option<Box<BlockExprBuilder>>,
}
impl WhileExprBuilder {
    pub fn condition(mut self, f: ConditionBuilder) -> Self {
        self.condition = Some(Box::new(f));
        self
    }
    pub fn loop_body(mut self, f: BlockExprBuilder) -> Self {
        self.loop_body = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for WhileExprBuilder {
    type Node = WhileExpr;
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
impl ContinueExpr {}
impl ContinueExpr {
    pub fn new() -> ContinueExprBuilder {
        ContinueExprBuilder::default()
    }
}
#[derive(Default)]
pub struct ContinueExprBuilder {}
impl ContinueExprBuilder {}
impl AstNodeBuilder for ContinueExprBuilder {
    type Node = ContinueExpr;
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
    pub fn new() -> BreakExprBuilder {
        BreakExprBuilder::default()
    }
}
#[derive(Default)]
pub struct BreakExprBuilder {
    expr: Option<Box<ExprBuilder>>,
}
impl BreakExprBuilder {
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for BreakExprBuilder {
    type Node = BreakExpr;
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
    pub fn new() -> ReturnExprBuilder {
        ReturnExprBuilder::default()
    }
}
#[derive(Default)]
pub struct ReturnExprBuilder {
    expr: Option<Box<ExprBuilder>>,
}
impl ReturnExprBuilder {
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ReturnExprBuilder {
    type Node = ReturnExpr;
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
    pub fn new() -> MatchExprBuilder {
        MatchExprBuilder::default()
    }
}
#[derive(Default)]
pub struct MatchExprBuilder {
    expr: Option<Box<ExprBuilder>>,
    match_arm_list: Option<Box<MatchArmListBuilder>>,
}
impl MatchExprBuilder {
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
    pub fn match_arm_list(mut self, f: MatchArmListBuilder) -> Self {
        self.match_arm_list = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for MatchExprBuilder {
    type Node = MatchExpr;
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
    pub fn new() -> RecordLitBuilder {
        RecordLitBuilder::default()
    }
}
#[derive(Default)]
pub struct RecordLitBuilder {
    path: Option<Box<PathBuilder>>,
    record_field_list: Option<Box<RecordFieldListBuilder>>,
}
impl RecordLitBuilder {
    pub fn path(mut self, f: PathBuilder) -> Self {
        self.path = Some(Box::new(f));
        self
    }
    pub fn record_field_list(mut self, f: RecordFieldListBuilder) -> Self {
        self.record_field_list = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for RecordLitBuilder {
    type Node = RecordLit;
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
    pub fn new() -> CallExprBuilder {
        CallExprBuilder::default()
    }
}
#[derive(Default)]
pub struct CallExprBuilder {
    expr: Option<Box<ExprBuilder>>,
    arg_list: Option<Box<ArgListBuilder>>,
}
impl CallExprBuilder {
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
    pub fn arg_list(mut self, f: ArgListBuilder) -> Self {
        self.arg_list = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for CallExprBuilder {
    type Node = CallExpr;
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
    pub fn new() -> MethodCallExprBuilder {
        MethodCallExprBuilder::default()
    }
}
#[derive(Default)]
pub struct MethodCallExprBuilder {
    expr: Option<Box<ExprBuilder>>,
    name_ref: Option<Box<NameRefBuilder>>,
    type_arg_list: Option<Box<TypeArgListBuilder>>,
    arg_list: Option<Box<ArgListBuilder>>,
}
impl MethodCallExprBuilder {
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
    pub fn name_ref(mut self, f: NameRefBuilder) -> Self {
        self.name_ref = Some(Box::new(f));
        self
    }
    pub fn type_arg_list(mut self, f: TypeArgListBuilder) -> Self {
        self.type_arg_list = Some(Box::new(f));
        self
    }
    pub fn arg_list(mut self, f: ArgListBuilder) -> Self {
        self.arg_list = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for MethodCallExprBuilder {
    type Node = MethodCallExpr;
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
    pub fn new() -> FieldExprBuilder {
        FieldExprBuilder::default()
    }
}
#[derive(Default)]
pub struct FieldExprBuilder {
    expr: Option<Box<ExprBuilder>>,
    name_ref: Option<Box<NameRefBuilder>>,
}
impl FieldExprBuilder {
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
    pub fn name_ref(mut self, f: NameRefBuilder) -> Self {
        self.name_ref = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for FieldExprBuilder {
    type Node = FieldExpr;
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
    pub fn new() -> AwaitExprBuilder {
        AwaitExprBuilder::default()
    }
}
#[derive(Default)]
pub struct AwaitExprBuilder {
    expr: Option<Box<ExprBuilder>>,
}
impl AwaitExprBuilder {
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for AwaitExprBuilder {
    type Node = AwaitExpr;
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
    pub fn new() -> TryExprBuilder {
        TryExprBuilder::default()
    }
}
#[derive(Default)]
pub struct TryExprBuilder {
    expr: Option<Box<ExprBuilder>>,
}
impl TryExprBuilder {
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for TryExprBuilder {
    type Node = TryExpr;
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
    pub fn new() -> TryBlockExprBuilder {
        TryBlockExprBuilder::default()
    }
}
#[derive(Default)]
pub struct TryBlockExprBuilder {
    body: Option<Box<BlockExprBuilder>>,
}
impl TryBlockExprBuilder {
    pub fn body(mut self, f: BlockExprBuilder) -> Self {
        self.body = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for TryBlockExprBuilder {
    type Node = TryBlockExpr;
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
    pub fn new() -> CastExprBuilder {
        CastExprBuilder::default()
    }
}
#[derive(Default)]
pub struct CastExprBuilder {
    expr: Option<Box<ExprBuilder>>,
    type_ref: Option<Box<TypeRefBuilder>>,
}
impl CastExprBuilder {
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
    pub fn type_ref(mut self, f: TypeRefBuilder) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for CastExprBuilder {
    type Node = CastExpr;
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
    pub fn new() -> RefExprBuilder {
        RefExprBuilder::default()
    }
}
#[derive(Default)]
pub struct RefExprBuilder {
    expr: Option<Box<ExprBuilder>>,
}
impl RefExprBuilder {
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for RefExprBuilder {
    type Node = RefExpr;
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
    pub fn expr(&self) -> Option<Expr> {
        super::child_opt(self)
    }
}
impl PrefixExpr {
    pub fn new() -> PrefixExprBuilder {
        PrefixExprBuilder::default()
    }
}
#[derive(Default)]
pub struct PrefixExprBuilder {
    expr: Option<Box<ExprBuilder>>,
}
impl PrefixExprBuilder {
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for PrefixExprBuilder {
    type Node = PrefixExpr;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PREFIX_EXPR);
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
    pub fn new() -> BoxExprBuilder {
        BoxExprBuilder::default()
    }
}
#[derive(Default)]
pub struct BoxExprBuilder {
    expr: Option<Box<ExprBuilder>>,
}
impl BoxExprBuilder {
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for BoxExprBuilder {
    type Node = BoxExpr;
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
    pub fn new() -> MacroCallBuilder {
        MacroCallBuilder::default()
    }
}
#[derive(Default)]
pub struct MacroCallBuilder {
    token_tree: Option<Box<TokenTreeBuilder>>,
    path: Option<Box<PathBuilder>>,
    name: Option<Box<NameBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
}
impl MacroCallBuilder {
    pub fn token_tree(mut self, f: TokenTreeBuilder) -> Self {
        self.token_tree = Some(Box::new(f));
        self
    }
    pub fn path(mut self, f: PathBuilder) -> Self {
        self.path = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameBuilder) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for MacroCallBuilder {
    type Node = MacroCall;
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
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
pub enum ImplItemBuilder {
    FnDefBuilder(Box<FnDefBuilder>),
    TypeAliasDefBuilder(Box<TypeAliasDefBuilder>),
    ConstDefBuilder(Box<ConstDefBuilder>),
}
impl From<FnDefBuilder> for ImplItemBuilder {
    fn from(builder: FnDefBuilder) -> ImplItemBuilder {
        ImplItemBuilder::FnDefBuilder(Box::new(builder))
    }
}
impl From<TypeAliasDefBuilder> for ImplItemBuilder {
    fn from(builder: TypeAliasDefBuilder) -> ImplItemBuilder {
        ImplItemBuilder::TypeAliasDefBuilder(Box::new(builder))
    }
}
impl From<ConstDefBuilder> for ImplItemBuilder {
    fn from(builder: ConstDefBuilder) -> ImplItemBuilder {
        ImplItemBuilder::ConstDefBuilder(Box::new(builder))
    }
}
impl AstNodeBuilder for ImplItemBuilder {
    type Node = ImplItem;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        match self {
            ImplItemBuilder::FnDefBuilder(b) => b.make(builder),
            ImplItemBuilder::TypeAliasDefBuilder(b) => b.make(builder),
            ImplItemBuilder::ConstDefBuilder(b) => b.make(builder),
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
pub enum ModuleItemBuilder {
    StructDefBuilder(Box<StructDefBuilder>),
    EnumDefBuilder(Box<EnumDefBuilder>),
    FnDefBuilder(Box<FnDefBuilder>),
    TraitDefBuilder(Box<TraitDefBuilder>),
    TypeAliasDefBuilder(Box<TypeAliasDefBuilder>),
    ImplBlockBuilder(Box<ImplBlockBuilder>),
    UseItemBuilder(Box<UseItemBuilder>),
    ExternCrateItemBuilder(Box<ExternCrateItemBuilder>),
    ConstDefBuilder(Box<ConstDefBuilder>),
    StaticDefBuilder(Box<StaticDefBuilder>),
    ModuleBuilder(Box<ModuleBuilder>),
}
impl From<StructDefBuilder> for ModuleItemBuilder {
    fn from(builder: StructDefBuilder) -> ModuleItemBuilder {
        ModuleItemBuilder::StructDefBuilder(Box::new(builder))
    }
}
impl From<EnumDefBuilder> for ModuleItemBuilder {
    fn from(builder: EnumDefBuilder) -> ModuleItemBuilder {
        ModuleItemBuilder::EnumDefBuilder(Box::new(builder))
    }
}
impl From<FnDefBuilder> for ModuleItemBuilder {
    fn from(builder: FnDefBuilder) -> ModuleItemBuilder {
        ModuleItemBuilder::FnDefBuilder(Box::new(builder))
    }
}
impl From<TraitDefBuilder> for ModuleItemBuilder {
    fn from(builder: TraitDefBuilder) -> ModuleItemBuilder {
        ModuleItemBuilder::TraitDefBuilder(Box::new(builder))
    }
}
impl From<TypeAliasDefBuilder> for ModuleItemBuilder {
    fn from(builder: TypeAliasDefBuilder) -> ModuleItemBuilder {
        ModuleItemBuilder::TypeAliasDefBuilder(Box::new(builder))
    }
}
impl From<ImplBlockBuilder> for ModuleItemBuilder {
    fn from(builder: ImplBlockBuilder) -> ModuleItemBuilder {
        ModuleItemBuilder::ImplBlockBuilder(Box::new(builder))
    }
}
impl From<UseItemBuilder> for ModuleItemBuilder {
    fn from(builder: UseItemBuilder) -> ModuleItemBuilder {
        ModuleItemBuilder::UseItemBuilder(Box::new(builder))
    }
}
impl From<ExternCrateItemBuilder> for ModuleItemBuilder {
    fn from(builder: ExternCrateItemBuilder) -> ModuleItemBuilder {
        ModuleItemBuilder::ExternCrateItemBuilder(Box::new(builder))
    }
}
impl From<ConstDefBuilder> for ModuleItemBuilder {
    fn from(builder: ConstDefBuilder) -> ModuleItemBuilder {
        ModuleItemBuilder::ConstDefBuilder(Box::new(builder))
    }
}
impl From<StaticDefBuilder> for ModuleItemBuilder {
    fn from(builder: StaticDefBuilder) -> ModuleItemBuilder {
        ModuleItemBuilder::StaticDefBuilder(Box::new(builder))
    }
}
impl From<ModuleBuilder> for ModuleItemBuilder {
    fn from(builder: ModuleBuilder) -> ModuleItemBuilder {
        ModuleItemBuilder::ModuleBuilder(Box::new(builder))
    }
}
impl AstNodeBuilder for ModuleItemBuilder {
    type Node = ModuleItem;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        match self {
            ModuleItemBuilder::StructDefBuilder(b) => b.make(builder),
            ModuleItemBuilder::EnumDefBuilder(b) => b.make(builder),
            ModuleItemBuilder::FnDefBuilder(b) => b.make(builder),
            ModuleItemBuilder::TraitDefBuilder(b) => b.make(builder),
            ModuleItemBuilder::TypeAliasDefBuilder(b) => b.make(builder),
            ModuleItemBuilder::ImplBlockBuilder(b) => b.make(builder),
            ModuleItemBuilder::UseItemBuilder(b) => b.make(builder),
            ModuleItemBuilder::ExternCrateItemBuilder(b) => b.make(builder),
            ModuleItemBuilder::ConstDefBuilder(b) => b.make(builder),
            ModuleItemBuilder::StaticDefBuilder(b) => b.make(builder),
            ModuleItemBuilder::ModuleBuilder(b) => b.make(builder),
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
    pub fn new() -> TraitDefBuilder {
        TraitDefBuilder::default()
    }
}
#[derive(Default)]
pub struct TraitDefBuilder {
    item_list: Option<Box<ItemListBuilder>>,
    visibility: Option<Box<VisibilityBuilder>>,
    name: Option<Box<NameBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
    type_param_list: Option<Box<TypeParamListBuilder>>,
    where_clause: Option<Box<WhereClauseBuilder>>,
    type_bound_list: Option<Box<TypeBoundListBuilder>>,
}
impl TraitDefBuilder {
    pub fn item_list(mut self, f: ItemListBuilder) -> Self {
        self.item_list = Some(Box::new(f));
        self
    }
    pub fn visibility(mut self, f: VisibilityBuilder) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameBuilder) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
    pub fn type_param_list(mut self, f: TypeParamListBuilder) -> Self {
        self.type_param_list = Some(Box::new(f));
        self
    }
    pub fn where_clause(mut self, f: WhereClauseBuilder) -> Self {
        self.where_clause = Some(Box::new(f));
        self
    }
    pub fn type_bound_list(mut self, f: TypeBoundListBuilder) -> Self {
        self.type_bound_list = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for TraitDefBuilder {
    type Node = TraitDef;
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
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> ImplBlockBuilder {
        ImplBlockBuilder::default()
    }
}
#[derive(Default)]
pub struct ImplBlockBuilder {
    item_list: Option<Box<ItemListBuilder>>,
    type_param_list: Option<Box<TypeParamListBuilder>>,
    where_clause: Option<Box<WhereClauseBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
}
impl ImplBlockBuilder {
    pub fn item_list(mut self, f: ItemListBuilder) -> Self {
        self.item_list = Some(Box::new(f));
        self
    }
    pub fn type_param_list(mut self, f: TypeParamListBuilder) -> Self {
        self.type_param_list = Some(Box::new(f));
        self
    }
    pub fn where_clause(mut self, f: WhereClauseBuilder) -> Self {
        self.where_clause = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ImplBlockBuilder {
    type Node = ImplBlock;
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
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> UseItemBuilder {
        UseItemBuilder::default()
    }
}
#[derive(Default)]
pub struct UseItemBuilder {
    use_tree: Option<Box<UseTreeBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
}
impl UseItemBuilder {
    pub fn use_tree(mut self, f: UseTreeBuilder) -> Self {
        self.use_tree = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for UseItemBuilder {
    type Node = UseItem;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::USE_ITEM);
        if let Some(b) = self.use_tree {
            b.make(builder);
        }
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> ExternCrateItemBuilder {
        ExternCrateItemBuilder::default()
    }
}
#[derive(Default)]
pub struct ExternCrateItemBuilder {
    name_ref: Option<Box<NameRefBuilder>>,
    alias: Option<Box<AliasBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
}
impl ExternCrateItemBuilder {
    pub fn name_ref(mut self, f: NameRefBuilder) -> Self {
        self.name_ref = Some(Box::new(f));
        self
    }
    pub fn alia(mut self, f: AliasBuilder) -> Self {
        self.alias = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ExternCrateItemBuilder {
    type Node = ExternCrateItem;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::EXTERN_CRATE_ITEM);
        if let Some(b) = self.name_ref {
            b.make(builder);
        }
        if let Some(b) = self.alias {
            b.make(builder);
        }
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> StaticDefBuilder {
        StaticDefBuilder::default()
    }
}
#[derive(Default)]
pub struct StaticDefBuilder {
    body: Option<Box<ExprBuilder>>,
    visibility: Option<Box<VisibilityBuilder>>,
    name: Option<Box<NameBuilder>>,
    type_param_list: Option<Box<TypeParamListBuilder>>,
    where_clause: Option<Box<WhereClauseBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
    ascribed_type: Option<Box<TypeRefBuilder>>,
}
impl StaticDefBuilder {
    pub fn body(mut self, f: ExprBuilder) -> Self {
        self.body = Some(Box::new(f));
        self
    }
    pub fn visibility(mut self, f: VisibilityBuilder) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameBuilder) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn type_param_list(mut self, f: TypeParamListBuilder) -> Self {
        self.type_param_list = Some(Box::new(f));
        self
    }
    pub fn where_clause(mut self, f: WhereClauseBuilder) -> Self {
        self.where_clause = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
    pub fn ascribed_type(mut self, f: TypeRefBuilder) -> Self {
        self.ascribed_type = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for StaticDefBuilder {
    type Node = StaticDef;
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
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> ModuleBuilder {
        ModuleBuilder::default()
    }
}
#[derive(Default)]
pub struct ModuleBuilder {
    item_list: Option<Box<ItemListBuilder>>,
    visibility: Option<Box<VisibilityBuilder>>,
    name: Option<Box<NameBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
}
impl ModuleBuilder {
    pub fn item_list(mut self, f: ItemListBuilder) -> Self {
        self.item_list = Some(Box::new(f));
        self
    }
    pub fn visibility(mut self, f: VisibilityBuilder) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameBuilder) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ModuleBuilder {
    type Node = Module;
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
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
pub enum NominalDefBuilder {
    StructDefBuilder(Box<StructDefBuilder>),
    EnumDefBuilder(Box<EnumDefBuilder>),
}
impl From<StructDefBuilder> for NominalDefBuilder {
    fn from(builder: StructDefBuilder) -> NominalDefBuilder {
        NominalDefBuilder::StructDefBuilder(Box::new(builder))
    }
}
impl From<EnumDefBuilder> for NominalDefBuilder {
    fn from(builder: EnumDefBuilder) -> NominalDefBuilder {
        NominalDefBuilder::EnumDefBuilder(Box::new(builder))
    }
}
impl AstNodeBuilder for NominalDefBuilder {
    type Node = NominalDef;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        match self {
            NominalDefBuilder::StructDefBuilder(b) => b.make(builder),
            NominalDefBuilder::EnumDefBuilder(b) => b.make(builder),
        }
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
pub enum PatBuilder {
    RefPatBuilder(Box<RefPatBuilder>),
    BoxPatBuilder(Box<BoxPatBuilder>),
    BindPatBuilder(Box<BindPatBuilder>),
    PlaceholderPatBuilder(Box<PlaceholderPatBuilder>),
    DotDotPatBuilder(Box<DotDotPatBuilder>),
    PathPatBuilder(Box<PathPatBuilder>),
    RecordPatBuilder(Box<RecordPatBuilder>),
    TupleStructPatBuilder(Box<TupleStructPatBuilder>),
    TuplePatBuilder(Box<TuplePatBuilder>),
    SlicePatBuilder(Box<SlicePatBuilder>),
    RangePatBuilder(Box<RangePatBuilder>),
    LiteralPatBuilder(Box<LiteralPatBuilder>),
}
impl From<RefPatBuilder> for PatBuilder {
    fn from(builder: RefPatBuilder) -> PatBuilder {
        PatBuilder::RefPatBuilder(Box::new(builder))
    }
}
impl From<BoxPatBuilder> for PatBuilder {
    fn from(builder: BoxPatBuilder) -> PatBuilder {
        PatBuilder::BoxPatBuilder(Box::new(builder))
    }
}
impl From<BindPatBuilder> for PatBuilder {
    fn from(builder: BindPatBuilder) -> PatBuilder {
        PatBuilder::BindPatBuilder(Box::new(builder))
    }
}
impl From<PlaceholderPatBuilder> for PatBuilder {
    fn from(builder: PlaceholderPatBuilder) -> PatBuilder {
        PatBuilder::PlaceholderPatBuilder(Box::new(builder))
    }
}
impl From<DotDotPatBuilder> for PatBuilder {
    fn from(builder: DotDotPatBuilder) -> PatBuilder {
        PatBuilder::DotDotPatBuilder(Box::new(builder))
    }
}
impl From<PathPatBuilder> for PatBuilder {
    fn from(builder: PathPatBuilder) -> PatBuilder {
        PatBuilder::PathPatBuilder(Box::new(builder))
    }
}
impl From<RecordPatBuilder> for PatBuilder {
    fn from(builder: RecordPatBuilder) -> PatBuilder {
        PatBuilder::RecordPatBuilder(Box::new(builder))
    }
}
impl From<TupleStructPatBuilder> for PatBuilder {
    fn from(builder: TupleStructPatBuilder) -> PatBuilder {
        PatBuilder::TupleStructPatBuilder(Box::new(builder))
    }
}
impl From<TuplePatBuilder> for PatBuilder {
    fn from(builder: TuplePatBuilder) -> PatBuilder {
        PatBuilder::TuplePatBuilder(Box::new(builder))
    }
}
impl From<SlicePatBuilder> for PatBuilder {
    fn from(builder: SlicePatBuilder) -> PatBuilder {
        PatBuilder::SlicePatBuilder(Box::new(builder))
    }
}
impl From<RangePatBuilder> for PatBuilder {
    fn from(builder: RangePatBuilder) -> PatBuilder {
        PatBuilder::RangePatBuilder(Box::new(builder))
    }
}
impl From<LiteralPatBuilder> for PatBuilder {
    fn from(builder: LiteralPatBuilder) -> PatBuilder {
        PatBuilder::LiteralPatBuilder(Box::new(builder))
    }
}
impl AstNodeBuilder for PatBuilder {
    type Node = Pat;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        match self {
            PatBuilder::RefPatBuilder(b) => b.make(builder),
            PatBuilder::BoxPatBuilder(b) => b.make(builder),
            PatBuilder::BindPatBuilder(b) => b.make(builder),
            PatBuilder::PlaceholderPatBuilder(b) => b.make(builder),
            PatBuilder::DotDotPatBuilder(b) => b.make(builder),
            PatBuilder::PathPatBuilder(b) => b.make(builder),
            PatBuilder::RecordPatBuilder(b) => b.make(builder),
            PatBuilder::TupleStructPatBuilder(b) => b.make(builder),
            PatBuilder::TuplePatBuilder(b) => b.make(builder),
            PatBuilder::SlicePatBuilder(b) => b.make(builder),
            PatBuilder::RangePatBuilder(b) => b.make(builder),
            PatBuilder::LiteralPatBuilder(b) => b.make(builder),
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
    pub fn new() -> RefPatBuilder {
        RefPatBuilder::default()
    }
}
#[derive(Default)]
pub struct RefPatBuilder {
    pat: Option<Box<PatBuilder>>,
}
impl RefPatBuilder {
    pub fn pat(mut self, f: PatBuilder) -> Self {
        self.pat = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for RefPatBuilder {
    type Node = RefPat;
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
    pub fn new() -> BoxPatBuilder {
        BoxPatBuilder::default()
    }
}
#[derive(Default)]
pub struct BoxPatBuilder {
    pat: Option<Box<PatBuilder>>,
}
impl BoxPatBuilder {
    pub fn pat(mut self, f: PatBuilder) -> Self {
        self.pat = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for BoxPatBuilder {
    type Node = BoxPat;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::BOX_PAT);
        if let Some(b) = self.pat {
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
    pub fn new() -> BindPatBuilder {
        BindPatBuilder::default()
    }
}
#[derive(Default)]
pub struct BindPatBuilder {
    pat: Option<Box<PatBuilder>>,
    name: Option<Box<NameBuilder>>,
}
impl BindPatBuilder {
    pub fn pat(mut self, f: PatBuilder) -> Self {
        self.pat = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameBuilder) -> Self {
        self.name = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for BindPatBuilder {
    type Node = BindPat;
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
    pub fn new() -> PathPatBuilder {
        PathPatBuilder::default()
    }
}
#[derive(Default)]
pub struct PathPatBuilder {
    path: Option<Box<PathBuilder>>,
}
impl PathPatBuilder {
    pub fn path(mut self, f: PathBuilder) -> Self {
        self.path = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for PathPatBuilder {
    type Node = PathPat;
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
    pub fn new() -> RecordPatBuilder {
        RecordPatBuilder::default()
    }
}
#[derive(Default)]
pub struct RecordPatBuilder {
    record_field_pat_list: Option<Box<RecordFieldPatListBuilder>>,
    path: Option<Box<PathBuilder>>,
}
impl RecordPatBuilder {
    pub fn record_field_pat_list(mut self, f: RecordFieldPatListBuilder) -> Self {
        self.record_field_pat_list = Some(Box::new(f));
        self
    }
    pub fn path(mut self, f: PathBuilder) -> Self {
        self.path = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for RecordPatBuilder {
    type Node = RecordPat;
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
    pub fn new() -> TupleStructPatBuilder {
        TupleStructPatBuilder::default()
    }
}
#[derive(Default)]
pub struct TupleStructPatBuilder {
    args: Vec<Box<PatBuilder>>,
    path: Option<Box<PathBuilder>>,
}
impl TupleStructPatBuilder {
    pub fn arg(mut self, f: PatBuilder) -> Self {
        self.args.push(Box::new(f));
        self
    }
    pub fn path(mut self, f: PathBuilder) -> Self {
        self.path = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for TupleStructPatBuilder {
    type Node = TupleStructPat;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TUPLE_STRUCT_PAT);
        self.args.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> TuplePatBuilder {
        TuplePatBuilder::default()
    }
}
#[derive(Default)]
pub struct TuplePatBuilder {
    args: Vec<Box<PatBuilder>>,
}
impl TuplePatBuilder {
    pub fn arg(mut self, f: PatBuilder) -> Self {
        self.args.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for TuplePatBuilder {
    type Node = TuplePat;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TUPLE_PAT);
        self.args.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> LiteralPatBuilder {
        LiteralPatBuilder::default()
    }
}
#[derive(Default)]
pub struct LiteralPatBuilder {
    literal: Option<Box<LiteralBuilder>>,
}
impl LiteralPatBuilder {
    pub fn literal(mut self, f: LiteralBuilder) -> Self {
        self.literal = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for LiteralPatBuilder {
    type Node = LiteralPat;
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
pub enum StmtBuilder {
    ExprStmtBuilder(Box<ExprStmtBuilder>),
    LetStmtBuilder(Box<LetStmtBuilder>),
}
impl From<ExprStmtBuilder> for StmtBuilder {
    fn from(builder: ExprStmtBuilder) -> StmtBuilder {
        StmtBuilder::ExprStmtBuilder(Box::new(builder))
    }
}
impl From<LetStmtBuilder> for StmtBuilder {
    fn from(builder: LetStmtBuilder) -> StmtBuilder {
        StmtBuilder::LetStmtBuilder(Box::new(builder))
    }
}
impl AstNodeBuilder for StmtBuilder {
    type Node = Stmt;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        match self {
            StmtBuilder::ExprStmtBuilder(b) => b.make(builder),
            StmtBuilder::LetStmtBuilder(b) => b.make(builder),
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
    pub fn new() -> ExprStmtBuilder {
        ExprStmtBuilder::default()
    }
}
#[derive(Default)]
pub struct ExprStmtBuilder {
    expr: Option<Box<ExprBuilder>>,
}
impl ExprStmtBuilder {
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ExprStmtBuilder {
    type Node = ExprStmt;
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
    pub fn new() -> LetStmtBuilder {
        LetStmtBuilder::default()
    }
}
#[derive(Default)]
pub struct LetStmtBuilder {
    pat: Option<Box<PatBuilder>>,
    initializer: Option<Box<ExprBuilder>>,
    ascribed_type: Option<Box<TypeRefBuilder>>,
}
impl LetStmtBuilder {
    pub fn pat(mut self, f: PatBuilder) -> Self {
        self.pat = Some(Box::new(f));
        self
    }
    pub fn initializer(mut self, f: ExprBuilder) -> Self {
        self.initializer = Some(Box::new(f));
        self
    }
    pub fn ascribed_type(mut self, f: TypeRefBuilder) -> Self {
        self.ascribed_type = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for LetStmtBuilder {
    type Node = LetStmt;
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
pub enum TypeRefBuilder {
    ParenTypeBuilder(Box<ParenTypeBuilder>),
    TupleTypeBuilder(Box<TupleTypeBuilder>),
    NeverTypeBuilder(Box<NeverTypeBuilder>),
    PathTypeBuilder(Box<PathTypeBuilder>),
    PointerTypeBuilder(Box<PointerTypeBuilder>),
    ArrayTypeBuilder(Box<ArrayTypeBuilder>),
    SliceTypeBuilder(Box<SliceTypeBuilder>),
    ReferenceTypeBuilder(Box<ReferenceTypeBuilder>),
    PlaceholderTypeBuilder(Box<PlaceholderTypeBuilder>),
    FnPointerTypeBuilder(Box<FnPointerTypeBuilder>),
    ForTypeBuilder(Box<ForTypeBuilder>),
    ImplTraitTypeBuilder(Box<ImplTraitTypeBuilder>),
    DynTraitTypeBuilder(Box<DynTraitTypeBuilder>),
}
impl From<ParenTypeBuilder> for TypeRefBuilder {
    fn from(builder: ParenTypeBuilder) -> TypeRefBuilder {
        TypeRefBuilder::ParenTypeBuilder(Box::new(builder))
    }
}
impl From<TupleTypeBuilder> for TypeRefBuilder {
    fn from(builder: TupleTypeBuilder) -> TypeRefBuilder {
        TypeRefBuilder::TupleTypeBuilder(Box::new(builder))
    }
}
impl From<NeverTypeBuilder> for TypeRefBuilder {
    fn from(builder: NeverTypeBuilder) -> TypeRefBuilder {
        TypeRefBuilder::NeverTypeBuilder(Box::new(builder))
    }
}
impl From<PathTypeBuilder> for TypeRefBuilder {
    fn from(builder: PathTypeBuilder) -> TypeRefBuilder {
        TypeRefBuilder::PathTypeBuilder(Box::new(builder))
    }
}
impl From<PointerTypeBuilder> for TypeRefBuilder {
    fn from(builder: PointerTypeBuilder) -> TypeRefBuilder {
        TypeRefBuilder::PointerTypeBuilder(Box::new(builder))
    }
}
impl From<ArrayTypeBuilder> for TypeRefBuilder {
    fn from(builder: ArrayTypeBuilder) -> TypeRefBuilder {
        TypeRefBuilder::ArrayTypeBuilder(Box::new(builder))
    }
}
impl From<SliceTypeBuilder> for TypeRefBuilder {
    fn from(builder: SliceTypeBuilder) -> TypeRefBuilder {
        TypeRefBuilder::SliceTypeBuilder(Box::new(builder))
    }
}
impl From<ReferenceTypeBuilder> for TypeRefBuilder {
    fn from(builder: ReferenceTypeBuilder) -> TypeRefBuilder {
        TypeRefBuilder::ReferenceTypeBuilder(Box::new(builder))
    }
}
impl From<PlaceholderTypeBuilder> for TypeRefBuilder {
    fn from(builder: PlaceholderTypeBuilder) -> TypeRefBuilder {
        TypeRefBuilder::PlaceholderTypeBuilder(Box::new(builder))
    }
}
impl From<FnPointerTypeBuilder> for TypeRefBuilder {
    fn from(builder: FnPointerTypeBuilder) -> TypeRefBuilder {
        TypeRefBuilder::FnPointerTypeBuilder(Box::new(builder))
    }
}
impl From<ForTypeBuilder> for TypeRefBuilder {
    fn from(builder: ForTypeBuilder) -> TypeRefBuilder {
        TypeRefBuilder::ForTypeBuilder(Box::new(builder))
    }
}
impl From<ImplTraitTypeBuilder> for TypeRefBuilder {
    fn from(builder: ImplTraitTypeBuilder) -> TypeRefBuilder {
        TypeRefBuilder::ImplTraitTypeBuilder(Box::new(builder))
    }
}
impl From<DynTraitTypeBuilder> for TypeRefBuilder {
    fn from(builder: DynTraitTypeBuilder) -> TypeRefBuilder {
        TypeRefBuilder::DynTraitTypeBuilder(Box::new(builder))
    }
}
impl AstNodeBuilder for TypeRefBuilder {
    type Node = TypeRef;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        match self {
            TypeRefBuilder::ParenTypeBuilder(b) => b.make(builder),
            TypeRefBuilder::TupleTypeBuilder(b) => b.make(builder),
            TypeRefBuilder::NeverTypeBuilder(b) => b.make(builder),
            TypeRefBuilder::PathTypeBuilder(b) => b.make(builder),
            TypeRefBuilder::PointerTypeBuilder(b) => b.make(builder),
            TypeRefBuilder::ArrayTypeBuilder(b) => b.make(builder),
            TypeRefBuilder::SliceTypeBuilder(b) => b.make(builder),
            TypeRefBuilder::ReferenceTypeBuilder(b) => b.make(builder),
            TypeRefBuilder::PlaceholderTypeBuilder(b) => b.make(builder),
            TypeRefBuilder::FnPointerTypeBuilder(b) => b.make(builder),
            TypeRefBuilder::ForTypeBuilder(b) => b.make(builder),
            TypeRefBuilder::ImplTraitTypeBuilder(b) => b.make(builder),
            TypeRefBuilder::DynTraitTypeBuilder(b) => b.make(builder),
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
    pub fn new() -> ParenTypeBuilder {
        ParenTypeBuilder::default()
    }
}
#[derive(Default)]
pub struct ParenTypeBuilder {
    type_ref: Option<Box<TypeRefBuilder>>,
}
impl ParenTypeBuilder {
    pub fn type_ref(mut self, f: TypeRefBuilder) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ParenTypeBuilder {
    type Node = ParenType;
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
    pub fn new() -> TupleTypeBuilder {
        TupleTypeBuilder::default()
    }
}
#[derive(Default)]
pub struct TupleTypeBuilder {
    fields: Vec<Box<TypeRefBuilder>>,
}
impl TupleTypeBuilder {
    pub fn field(mut self, f: TypeRefBuilder) -> Self {
        self.fields.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for TupleTypeBuilder {
    type Node = TupleType;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TUPLE_TYPE);
        builder.token(SyntaxKind::L_PAREN, SmolStr::new(T_STR!(L_PAREN)));
        self.fields.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> PointerTypeBuilder {
        PointerTypeBuilder::default()
    }
}
#[derive(Default)]
pub struct PointerTypeBuilder {
    type_ref: Option<Box<TypeRefBuilder>>,
}
impl PointerTypeBuilder {
    pub fn type_ref(mut self, f: TypeRefBuilder) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for PointerTypeBuilder {
    type Node = PointerType;
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
    pub fn new() -> ArrayTypeBuilder {
        ArrayTypeBuilder::default()
    }
}
#[derive(Default)]
pub struct ArrayTypeBuilder {
    type_ref: Option<Box<TypeRefBuilder>>,
    expr: Option<Box<ExprBuilder>>,
}
impl ArrayTypeBuilder {
    pub fn type_ref(mut self, f: TypeRefBuilder) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ArrayTypeBuilder {
    type Node = ArrayType;
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
    pub fn new() -> SliceTypeBuilder {
        SliceTypeBuilder::default()
    }
}
#[derive(Default)]
pub struct SliceTypeBuilder {
    type_ref: Option<Box<TypeRefBuilder>>,
}
impl SliceTypeBuilder {
    pub fn type_ref(mut self, f: TypeRefBuilder) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for SliceTypeBuilder {
    type Node = SliceType;
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
    pub fn new() -> ReferenceTypeBuilder {
        ReferenceTypeBuilder::default()
    }
}
#[derive(Default)]
pub struct ReferenceTypeBuilder {
    type_ref: Option<Box<TypeRefBuilder>>,
}
impl ReferenceTypeBuilder {
    pub fn type_ref(mut self, f: TypeRefBuilder) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ReferenceTypeBuilder {
    type Node = ReferenceType;
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
    pub fn new() -> FnPointerTypeBuilder {
        FnPointerTypeBuilder::default()
    }
}
#[derive(Default)]
pub struct FnPointerTypeBuilder {
    param_list: Option<Box<ParamListBuilder>>,
    ret_type: Option<Box<RetTypeBuilder>>,
}
impl FnPointerTypeBuilder {
    pub fn param_list(mut self, f: ParamListBuilder) -> Self {
        self.param_list = Some(Box::new(f));
        self
    }
    pub fn ret_type(mut self, f: RetTypeBuilder) -> Self {
        self.ret_type = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for FnPointerTypeBuilder {
    type Node = FnPointerType;
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
    pub fn new() -> ForTypeBuilder {
        ForTypeBuilder::default()
    }
}
#[derive(Default)]
pub struct ForTypeBuilder {
    type_ref: Option<Box<TypeRefBuilder>>,
}
impl ForTypeBuilder {
    pub fn type_ref(mut self, f: TypeRefBuilder) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ForTypeBuilder {
    type Node = ForType;
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
    pub fn new() -> ImplTraitTypeBuilder {
        ImplTraitTypeBuilder::default()
    }
}
#[derive(Default)]
pub struct ImplTraitTypeBuilder {
    type_bound_list: Option<Box<TypeBoundListBuilder>>,
}
impl ImplTraitTypeBuilder {
    pub fn type_bound_list(mut self, f: TypeBoundListBuilder) -> Self {
        self.type_bound_list = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ImplTraitTypeBuilder {
    type Node = ImplTraitType;
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
    pub fn new() -> DynTraitTypeBuilder {
        DynTraitTypeBuilder::default()
    }
}
#[derive(Default)]
pub struct DynTraitTypeBuilder {
    type_bound_list: Option<Box<TypeBoundListBuilder>>,
}
impl DynTraitTypeBuilder {
    pub fn type_bound_list(mut self, f: TypeBoundListBuilder) -> Self {
        self.type_bound_list = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for DynTraitTypeBuilder {
    type Node = DynTraitType;
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
pub enum AttrInputBuilder {
    LiteralBuilder(Box<LiteralBuilder>),
    TokenTreeBuilder(Box<TokenTreeBuilder>),
}
impl From<LiteralBuilder> for AttrInputBuilder {
    fn from(builder: LiteralBuilder) -> AttrInputBuilder {
        AttrInputBuilder::LiteralBuilder(Box::new(builder))
    }
}
impl From<TokenTreeBuilder> for AttrInputBuilder {
    fn from(builder: TokenTreeBuilder) -> AttrInputBuilder {
        AttrInputBuilder::TokenTreeBuilder(Box::new(builder))
    }
}
impl AstNodeBuilder for AttrInputBuilder {
    type Node = AttrInput;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        match self {
            AttrInputBuilder::LiteralBuilder(b) => b.make(builder),
            AttrInputBuilder::TokenTreeBuilder(b) => b.make(builder),
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
    pub fn new() -> BlockExprBuilder {
        BlockExprBuilder::default()
    }
}
#[derive(Default)]
pub struct BlockExprBuilder {
    block: Option<Box<BlockBuilder>>,
}
impl BlockExprBuilder {
    pub fn block(mut self, f: BlockBuilder) -> Self {
        self.block = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for BlockExprBuilder {
    type Node = BlockExpr;
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
impl StructDef {
    pub fn new() -> StructDefBuilder {
        StructDefBuilder::default()
    }
}
#[derive(Default)]
pub struct StructDefBuilder {
    visibility: Option<Box<VisibilityBuilder>>,
    name: Option<Box<NameBuilder>>,
    type_param_list: Option<Box<TypeParamListBuilder>>,
    where_clause: Option<Box<WhereClauseBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
}
impl StructDefBuilder {
    pub fn visibility(mut self, f: VisibilityBuilder) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameBuilder) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn type_param_list(mut self, f: TypeParamListBuilder) -> Self {
        self.type_param_list = Some(Box::new(f));
        self
    }
    pub fn where_clause(mut self, f: WhereClauseBuilder) -> Self {
        self.where_clause = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for StructDefBuilder {
    type Node = StructDef;
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
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> EnumDefBuilder {
        EnumDefBuilder::default()
    }
}
#[derive(Default)]
pub struct EnumDefBuilder {
    variant_list: Option<Box<EnumVariantListBuilder>>,
    visibility: Option<Box<VisibilityBuilder>>,
    name: Option<Box<NameBuilder>>,
    type_param_list: Option<Box<TypeParamListBuilder>>,
    where_clause: Option<Box<WhereClauseBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
}
impl EnumDefBuilder {
    pub fn variant_list(mut self, f: EnumVariantListBuilder) -> Self {
        self.variant_list = Some(Box::new(f));
        self
    }
    pub fn visibility(mut self, f: VisibilityBuilder) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameBuilder) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn type_param_list(mut self, f: TypeParamListBuilder) -> Self {
        self.type_param_list = Some(Box::new(f));
        self
    }
    pub fn where_clause(mut self, f: WhereClauseBuilder) -> Self {
        self.where_clause = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for EnumDefBuilder {
    type Node = EnumDef;
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
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> FnDefBuilder {
        FnDefBuilder::default()
    }
}
#[derive(Default)]
pub struct FnDefBuilder {
    param_list: Option<Box<ParamListBuilder>>,
    body: Option<Box<BlockExprBuilder>>,
    ret_type: Option<Box<RetTypeBuilder>>,
    visibility: Option<Box<VisibilityBuilder>>,
    name: Option<Box<NameBuilder>>,
    type_param_list: Option<Box<TypeParamListBuilder>>,
    where_clause: Option<Box<WhereClauseBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
}
impl FnDefBuilder {
    pub fn param_list(mut self, f: ParamListBuilder) -> Self {
        self.param_list = Some(Box::new(f));
        self
    }
    pub fn body(mut self, f: BlockExprBuilder) -> Self {
        self.body = Some(Box::new(f));
        self
    }
    pub fn ret_type(mut self, f: RetTypeBuilder) -> Self {
        self.ret_type = Some(Box::new(f));
        self
    }
    pub fn visibility(mut self, f: VisibilityBuilder) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameBuilder) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn type_param_list(mut self, f: TypeParamListBuilder) -> Self {
        self.type_param_list = Some(Box::new(f));
        self
    }
    pub fn where_clause(mut self, f: WhereClauseBuilder) -> Self {
        self.where_clause = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for FnDefBuilder {
    type Node = FnDef;
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
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> TypeAliasDefBuilder {
        TypeAliasDefBuilder::default()
    }
}
#[derive(Default)]
pub struct TypeAliasDefBuilder {
    type_ref: Option<Box<TypeRefBuilder>>,
    visibility: Option<Box<VisibilityBuilder>>,
    name: Option<Box<NameBuilder>>,
    type_param_list: Option<Box<TypeParamListBuilder>>,
    where_clause: Option<Box<WhereClauseBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
    type_bound_list: Option<Box<TypeBoundListBuilder>>,
}
impl TypeAliasDefBuilder {
    pub fn type_ref(mut self, f: TypeRefBuilder) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
    pub fn visibility(mut self, f: VisibilityBuilder) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameBuilder) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn type_param_list(mut self, f: TypeParamListBuilder) -> Self {
        self.type_param_list = Some(Box::new(f));
        self
    }
    pub fn where_clause(mut self, f: WhereClauseBuilder) -> Self {
        self.where_clause = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
    pub fn type_bound_list(mut self, f: TypeBoundListBuilder) -> Self {
        self.type_bound_list = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for TypeAliasDefBuilder {
    type Node = TypeAliasDef;
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
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> ConstDefBuilder {
        ConstDefBuilder::default()
    }
}
#[derive(Default)]
pub struct ConstDefBuilder {
    body: Option<Box<ExprBuilder>>,
    visibility: Option<Box<VisibilityBuilder>>,
    name: Option<Box<NameBuilder>>,
    type_param_list: Option<Box<TypeParamListBuilder>>,
    where_clause: Option<Box<WhereClauseBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
    ascribed_type: Option<Box<TypeRefBuilder>>,
}
impl ConstDefBuilder {
    pub fn body(mut self, f: ExprBuilder) -> Self {
        self.body = Some(Box::new(f));
        self
    }
    pub fn visibility(mut self, f: VisibilityBuilder) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameBuilder) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn type_param_list(mut self, f: TypeParamListBuilder) -> Self {
        self.type_param_list = Some(Box::new(f));
        self
    }
    pub fn where_clause(mut self, f: WhereClauseBuilder) -> Self {
        self.where_clause = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
    pub fn ascribed_type(mut self, f: TypeRefBuilder) -> Self {
        self.ascribed_type = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ConstDefBuilder {
    type Node = ConstDef;
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
        self.attrs.into_iter().for_each(|b| b.make(builder));
        if let Some(b) = self.ascribed_type {
            b.make(builder);
        }
        builder.finish_node();
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
    pub fn new() -> AliasBuilder {
        AliasBuilder::default()
    }
}
#[derive(Default)]
pub struct AliasBuilder {
    name: Option<Box<NameBuilder>>,
}
impl AliasBuilder {
    pub fn name(mut self, f: NameBuilder) -> Self {
        self.name = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for AliasBuilder {
    type Node = Alias;
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
    pub fn new() -> ArgListBuilder {
        ArgListBuilder::default()
    }
}
#[derive(Default)]
pub struct ArgListBuilder {
    args: Vec<Box<ExprBuilder>>,
}
impl ArgListBuilder {
    pub fn arg(mut self, f: ExprBuilder) -> Self {
        self.args.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ArgListBuilder {
    type Node = ArgList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ARG_LIST);
        self.args.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> AssocTypeArgBuilder {
        AssocTypeArgBuilder::default()
    }
}
#[derive(Default)]
pub struct AssocTypeArgBuilder {
    name_ref: Option<Box<NameRefBuilder>>,
    type_ref: Option<Box<TypeRefBuilder>>,
}
impl AssocTypeArgBuilder {
    pub fn name_ref(mut self, f: NameRefBuilder) -> Self {
        self.name_ref = Some(Box::new(f));
        self
    }
    pub fn type_ref(mut self, f: TypeRefBuilder) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for AssocTypeArgBuilder {
    type Node = AssocTypeArg;
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
    pub fn new() -> AttrBuilder {
        AttrBuilder::default()
    }
}
#[derive(Default)]
pub struct AttrBuilder {
    path: Option<Box<PathBuilder>>,
    input: Option<Box<AttrInputBuilder>>,
}
impl AttrBuilder {
    pub fn path(mut self, f: PathBuilder) -> Self {
        self.path = Some(Box::new(f));
        self
    }
    pub fn input(mut self, f: AttrInputBuilder) -> Self {
        self.input = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for AttrBuilder {
    type Node = Attr;
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
    pub fn new() -> BlockBuilder {
        BlockBuilder::default()
    }
}
#[derive(Default)]
pub struct BlockBuilder {
    statements: Vec<Box<StmtBuilder>>,
    expr: Option<Box<ExprBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
}
impl BlockBuilder {
    pub fn statement(mut self, f: StmtBuilder) -> Self {
        self.statements.push(Box::new(f));
        self
    }
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for BlockBuilder {
    type Node = Block;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::BLOCK);
        self.statements.into_iter().for_each(|b| b.make(builder));
        if let Some(b) = self.expr {
            b.make(builder);
        }
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> ConditionBuilder {
        ConditionBuilder::default()
    }
}
#[derive(Default)]
pub struct ConditionBuilder {
    pat: Option<Box<PatBuilder>>,
    expr: Option<Box<ExprBuilder>>,
}
impl ConditionBuilder {
    pub fn pat(mut self, f: PatBuilder) -> Self {
        self.pat = Some(Box::new(f));
        self
    }
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ConditionBuilder {
    type Node = Condition;
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
    pub fn new() -> EnumVariantBuilder {
        EnumVariantBuilder::default()
    }
}
#[derive(Default)]
pub struct EnumVariantBuilder {
    expr: Option<Box<ExprBuilder>>,
    name: Option<Box<NameBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
}
impl EnumVariantBuilder {
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameBuilder) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for EnumVariantBuilder {
    type Node = EnumVariant;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ENUM_VARIANT);
        if let Some(b) = self.expr {
            b.make(builder);
        }
        if let Some(b) = self.name {
            b.make(builder);
        }
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> EnumVariantListBuilder {
        EnumVariantListBuilder::default()
    }
}
#[derive(Default)]
pub struct EnumVariantListBuilder {
    variants: Vec<Box<EnumVariantBuilder>>,
}
impl EnumVariantListBuilder {
    pub fn variant(mut self, f: EnumVariantBuilder) -> Self {
        self.variants.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for EnumVariantListBuilder {
    type Node = EnumVariantList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ENUM_VARIANT_LIST);
        self.variants.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> RecordFieldPatBuilder {
        RecordFieldPatBuilder::default()
    }
}
#[derive(Default)]
pub struct RecordFieldPatBuilder {
    pat: Option<Box<PatBuilder>>,
    name: Option<Box<NameBuilder>>,
}
impl RecordFieldPatBuilder {
    pub fn pat(mut self, f: PatBuilder) -> Self {
        self.pat = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameBuilder) -> Self {
        self.name = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for RecordFieldPatBuilder {
    type Node = RecordFieldPat;
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
    pub fn new() -> RecordFieldPatListBuilder {
        RecordFieldPatListBuilder::default()
    }
}
#[derive(Default)]
pub struct RecordFieldPatListBuilder {
    record_field_pats: Vec<Box<RecordFieldPatBuilder>>,
    bind_pats: Vec<Box<BindPatBuilder>>,
}
impl RecordFieldPatListBuilder {
    pub fn record_field_pat(mut self, f: RecordFieldPatBuilder) -> Self {
        self.record_field_pats.push(Box::new(f));
        self
    }
    pub fn bind_pat(mut self, f: BindPatBuilder) -> Self {
        self.bind_pats.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for RecordFieldPatListBuilder {
    type Node = RecordFieldPatList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_FIELD_PAT_LIST);
        self.record_field_pats.into_iter().for_each(|b| b.make(builder));
        self.bind_pats.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> ItemListBuilder {
        ItemListBuilder::default()
    }
}
#[derive(Default)]
pub struct ItemListBuilder {
    impl_items: Vec<Box<ImplItemBuilder>>,
    functions: Vec<Box<FnDefBuilder>>,
    items: Vec<Box<ModuleItemBuilder>>,
}
impl ItemListBuilder {
    pub fn impl_item(mut self, f: ImplItemBuilder) -> Self {
        self.impl_items.push(Box::new(f));
        self
    }
    pub fn function(mut self, f: FnDefBuilder) -> Self {
        self.functions.push(Box::new(f));
        self
    }
    pub fn item(mut self, f: ModuleItemBuilder) -> Self {
        self.items.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ItemListBuilder {
    type Node = ItemList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::ITEM_LIST);
        self.impl_items.into_iter().for_each(|b| b.make(builder));
        self.functions.into_iter().for_each(|b| b.make(builder));
        self.items.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> LifetimeParamBuilder {
        LifetimeParamBuilder::default()
    }
}
#[derive(Default)]
pub struct LifetimeParamBuilder {
    attrs: Vec<Box<AttrBuilder>>,
}
impl LifetimeParamBuilder {
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for LifetimeParamBuilder {
    type Node = LifetimeParam;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::LIFETIME_PARAM);
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> MacroItemsBuilder {
        MacroItemsBuilder::default()
    }
}
#[derive(Default)]
pub struct MacroItemsBuilder {
    items: Vec<Box<ModuleItemBuilder>>,
    functions: Vec<Box<FnDefBuilder>>,
}
impl MacroItemsBuilder {
    pub fn item(mut self, f: ModuleItemBuilder) -> Self {
        self.items.push(Box::new(f));
        self
    }
    pub fn function(mut self, f: FnDefBuilder) -> Self {
        self.functions.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for MacroItemsBuilder {
    type Node = MacroItems;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MACRO_ITEMS);
        self.items.into_iter().for_each(|b| b.make(builder));
        self.functions.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> MacroStmtsBuilder {
        MacroStmtsBuilder::default()
    }
}
#[derive(Default)]
pub struct MacroStmtsBuilder {
    statements: Vec<Box<StmtBuilder>>,
    expr: Option<Box<ExprBuilder>>,
}
impl MacroStmtsBuilder {
    pub fn statement(mut self, f: StmtBuilder) -> Self {
        self.statements.push(Box::new(f));
        self
    }
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for MacroStmtsBuilder {
    type Node = MacroStmts;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MACRO_STMTS);
        self.statements.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> MatchArmBuilder {
        MatchArmBuilder::default()
    }
}
#[derive(Default)]
pub struct MatchArmBuilder {
    pats: Vec<Box<PatBuilder>>,
    guard: Option<Box<MatchGuardBuilder>>,
    expr: Option<Box<ExprBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
}
impl MatchArmBuilder {
    pub fn pat(mut self, f: PatBuilder) -> Self {
        self.pats.push(Box::new(f));
        self
    }
    pub fn guard(mut self, f: MatchGuardBuilder) -> Self {
        self.guard = Some(Box::new(f));
        self
    }
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for MatchArmBuilder {
    type Node = MatchArm;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MATCH_ARM);
        self.pats.into_iter().for_each(|b| b.make(builder));
        if let Some(b) = self.guard {
            b.make(builder);
        }
        if let Some(b) = self.expr {
            b.make(builder);
        }
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> MatchArmListBuilder {
        MatchArmListBuilder::default()
    }
}
#[derive(Default)]
pub struct MatchArmListBuilder {
    arms: Vec<Box<MatchArmBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
}
impl MatchArmListBuilder {
    pub fn arm(mut self, f: MatchArmBuilder) -> Self {
        self.arms.push(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for MatchArmListBuilder {
    type Node = MatchArmList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::MATCH_ARM_LIST);
        self.arms.into_iter().for_each(|b| b.make(builder));
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> MatchGuardBuilder {
        MatchGuardBuilder::default()
    }
}
#[derive(Default)]
pub struct MatchGuardBuilder {
    expr: Option<Box<ExprBuilder>>,
}
impl MatchGuardBuilder {
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for MatchGuardBuilder {
    type Node = MatchGuard;
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
    pub fn new() -> RecordFieldBuilder {
        RecordFieldBuilder::default()
    }
}
#[derive(Default)]
pub struct RecordFieldBuilder {
    name_ref: Option<Box<NameRefBuilder>>,
    expr: Option<Box<ExprBuilder>>,
}
impl RecordFieldBuilder {
    pub fn name_ref(mut self, f: NameRefBuilder) -> Self {
        self.name_ref = Some(Box::new(f));
        self
    }
    pub fn expr(mut self, f: ExprBuilder) -> Self {
        self.expr = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for RecordFieldBuilder {
    type Node = RecordField;
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
    pub fn new() -> RecordFieldDefBuilder {
        RecordFieldDefBuilder::default()
    }
}
#[derive(Default)]
pub struct RecordFieldDefBuilder {
    visibility: Option<Box<VisibilityBuilder>>,
    name: Option<Box<NameBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
    ascribed_type: Option<Box<TypeRefBuilder>>,
}
impl RecordFieldDefBuilder {
    pub fn visibility(mut self, f: VisibilityBuilder) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn name(mut self, f: NameBuilder) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
    pub fn ascribed_type(mut self, f: TypeRefBuilder) -> Self {
        self.ascribed_type = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for RecordFieldDefBuilder {
    type Node = RecordFieldDef;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_FIELD_DEF);
        if let Some(b) = self.visibility {
            b.make(builder);
        }
        if let Some(b) = self.name {
            b.make(builder);
        }
        builder.token(SyntaxKind::COLON, SmolStr::new(T_STR!(COLON)));
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> RecordFieldDefListBuilder {
        RecordFieldDefListBuilder::default()
    }
}
#[derive(Default)]
pub struct RecordFieldDefListBuilder {
    fields: Vec<Box<RecordFieldDefBuilder>>,
}
impl RecordFieldDefListBuilder {
    pub fn field(mut self, f: RecordFieldDefBuilder) -> Self {
        self.fields.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for RecordFieldDefListBuilder {
    type Node = RecordFieldDefList;
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
    pub fn new() -> RecordFieldListBuilder {
        RecordFieldListBuilder::default()
    }
}
#[derive(Default)]
pub struct RecordFieldListBuilder {
    fields: Vec<Box<RecordFieldBuilder>>,
    spread: Option<Box<ExprBuilder>>,
}
impl RecordFieldListBuilder {
    pub fn field(mut self, f: RecordFieldBuilder) -> Self {
        self.fields.push(Box::new(f));
        self
    }
    pub fn spread(mut self, f: ExprBuilder) -> Self {
        self.spread = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for RecordFieldListBuilder {
    type Node = RecordFieldList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::RECORD_FIELD_LIST);
        builder.token(SyntaxKind::L_CURLY, SmolStr::new(T_STR!(L_CURLY)));
        self.fields.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> ParamBuilder {
        ParamBuilder::default()
    }
}
#[derive(Default)]
pub struct ParamBuilder {
    pat: Option<Box<PatBuilder>>,
    ascribed_type: Option<Box<TypeRefBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
}
impl ParamBuilder {
    pub fn pat(mut self, f: PatBuilder) -> Self {
        self.pat = Some(Box::new(f));
        self
    }
    pub fn ascribed_type(mut self, f: TypeRefBuilder) -> Self {
        self.ascribed_type = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ParamBuilder {
    type Node = Param;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PARAM);
        if let Some(b) = self.pat {
            b.make(builder);
        }
        if let Some(b) = self.ascribed_type {
            b.make(builder);
        }
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> ParamListBuilder {
        ParamListBuilder::default()
    }
}
#[derive(Default)]
pub struct ParamListBuilder {
    params: Vec<Box<ParamBuilder>>,
    self_param: Option<Box<SelfParamBuilder>>,
}
impl ParamListBuilder {
    pub fn param(mut self, f: ParamBuilder) -> Self {
        self.params.push(Box::new(f));
        self
    }
    pub fn self_param(mut self, f: SelfParamBuilder) -> Self {
        self.self_param = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for ParamListBuilder {
    type Node = ParamList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::PARAM_LIST);
        self.params.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> PathTypeBuilder {
        PathTypeBuilder::default()
    }
}
#[derive(Default)]
pub struct PathTypeBuilder {
    path: Option<Box<PathBuilder>>,
}
impl PathTypeBuilder {
    pub fn path(mut self, f: PathBuilder) -> Self {
        self.path = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for PathTypeBuilder {
    type Node = PathType;
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
    pub fn new() -> PathBuilder {
        PathBuilder::default()
    }
}
#[derive(Default)]
pub struct PathBuilder {
    segment: Option<Box<PathSegmentBuilder>>,
    qualifier: Option<Box<PathBuilder>>,
}
impl PathBuilder {
    pub fn segment(mut self, f: PathSegmentBuilder) -> Self {
        self.segment = Some(Box::new(f));
        self
    }
    pub fn qualifier(mut self, f: PathBuilder) -> Self {
        self.qualifier = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for PathBuilder {
    type Node = Path;
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
    pub fn new() -> PathSegmentBuilder {
        PathSegmentBuilder::default()
    }
}
#[derive(Default)]
pub struct PathSegmentBuilder {
    name_ref: Option<Box<NameRefBuilder>>,
    type_arg_list: Option<Box<TypeArgListBuilder>>,
    param_list: Option<Box<ParamListBuilder>>,
    ret_type: Option<Box<RetTypeBuilder>>,
    path_type: Option<Box<PathTypeBuilder>>,
}
impl PathSegmentBuilder {
    pub fn name_ref(mut self, f: NameRefBuilder) -> Self {
        self.name_ref = Some(Box::new(f));
        self
    }
    pub fn type_arg_list(mut self, f: TypeArgListBuilder) -> Self {
        self.type_arg_list = Some(Box::new(f));
        self
    }
    pub fn param_list(mut self, f: ParamListBuilder) -> Self {
        self.param_list = Some(Box::new(f));
        self
    }
    pub fn ret_type(mut self, f: RetTypeBuilder) -> Self {
        self.ret_type = Some(Box::new(f));
        self
    }
    pub fn path_type(mut self, f: PathTypeBuilder) -> Self {
        self.path_type = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for PathSegmentBuilder {
    type Node = PathSegment;
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
    pub fn new() -> TupleFieldDefBuilder {
        TupleFieldDefBuilder::default()
    }
}
#[derive(Default)]
pub struct TupleFieldDefBuilder {
    type_ref: Option<Box<TypeRefBuilder>>,
    visibility: Option<Box<VisibilityBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
}
impl TupleFieldDefBuilder {
    pub fn type_ref(mut self, f: TypeRefBuilder) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
    pub fn visibility(mut self, f: VisibilityBuilder) -> Self {
        self.visibility = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for TupleFieldDefBuilder {
    type Node = TupleFieldDef;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TUPLE_FIELD_DEF);
        if let Some(b) = self.type_ref {
            b.make(builder);
        }
        if let Some(b) = self.visibility {
            b.make(builder);
        }
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> TupleFieldDefListBuilder {
        TupleFieldDefListBuilder::default()
    }
}
#[derive(Default)]
pub struct TupleFieldDefListBuilder {
    fields: Vec<Box<TupleFieldDefBuilder>>,
}
impl TupleFieldDefListBuilder {
    pub fn field(mut self, f: TupleFieldDefBuilder) -> Self {
        self.fields.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for TupleFieldDefListBuilder {
    type Node = TupleFieldDefList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TUPLE_FIELD_DEF_LIST);
        self.fields.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> RetTypeBuilder {
        RetTypeBuilder::default()
    }
}
#[derive(Default)]
pub struct RetTypeBuilder {
    type_ref: Option<Box<TypeRefBuilder>>,
}
impl RetTypeBuilder {
    pub fn type_ref(mut self, f: TypeRefBuilder) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for RetTypeBuilder {
    type Node = RetType;
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
    pub fn new() -> SelfParamBuilder {
        SelfParamBuilder::default()
    }
}
#[derive(Default)]
pub struct SelfParamBuilder {
    ascribed_type: Option<Box<TypeRefBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
}
impl SelfParamBuilder {
    pub fn ascribed_type(mut self, f: TypeRefBuilder) -> Self {
        self.ascribed_type = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for SelfParamBuilder {
    type Node = SelfParam;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::SELF_PARAM);
        if let Some(b) = self.ascribed_type {
            b.make(builder);
        }
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
impl SourceFile {
    pub fn new() -> SourceFileBuilder {
        SourceFileBuilder::default()
    }
}
#[derive(Default)]
pub struct SourceFileBuilder {
    modules: Vec<Box<ModuleBuilder>>,
    items: Vec<Box<ModuleItemBuilder>>,
    functions: Vec<Box<FnDefBuilder>>,
}
impl SourceFileBuilder {
    pub fn module(mut self, f: ModuleBuilder) -> Self {
        self.modules.push(Box::new(f));
        self
    }
    pub fn item(mut self, f: ModuleItemBuilder) -> Self {
        self.items.push(Box::new(f));
        self
    }
    pub fn function(mut self, f: FnDefBuilder) -> Self {
        self.functions.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for SourceFileBuilder {
    type Node = SourceFile;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::SOURCE_FILE);
        self.modules.into_iter().for_each(|b| b.make(builder));
        self.items.into_iter().for_each(|b| b.make(builder));
        self.functions.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> TypeArgBuilder {
        TypeArgBuilder::default()
    }
}
#[derive(Default)]
pub struct TypeArgBuilder {
    type_ref: Option<Box<TypeRefBuilder>>,
}
impl TypeArgBuilder {
    pub fn type_ref(mut self, f: TypeRefBuilder) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for TypeArgBuilder {
    type Node = TypeArg;
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
    pub fn new() -> TypeArgListBuilder {
        TypeArgListBuilder::default()
    }
}
#[derive(Default)]
pub struct TypeArgListBuilder {
    type_args: Vec<Box<TypeArgBuilder>>,
    lifetime_args: Vec<Box<LifetimeArgBuilder>>,
    assoc_type_args: Vec<Box<AssocTypeArgBuilder>>,
}
impl TypeArgListBuilder {
    pub fn type_arg(mut self, f: TypeArgBuilder) -> Self {
        self.type_args.push(Box::new(f));
        self
    }
    pub fn lifetime_arg(mut self, f: LifetimeArgBuilder) -> Self {
        self.lifetime_args.push(Box::new(f));
        self
    }
    pub fn assoc_type_arg(mut self, f: AssocTypeArgBuilder) -> Self {
        self.assoc_type_args.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for TypeArgListBuilder {
    type Node = TypeArgList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TYPE_ARG_LIST);
        self.type_args.into_iter().for_each(|b| b.make(builder));
        self.lifetime_args.into_iter().for_each(|b| b.make(builder));
        self.assoc_type_args.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> TypeBoundBuilder {
        TypeBoundBuilder::default()
    }
}
#[derive(Default)]
pub struct TypeBoundBuilder {
    type_ref: Option<Box<TypeRefBuilder>>,
}
impl TypeBoundBuilder {
    pub fn type_ref(mut self, f: TypeRefBuilder) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for TypeBoundBuilder {
    type Node = TypeBound;
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
    pub fn new() -> TypeBoundListBuilder {
        TypeBoundListBuilder::default()
    }
}
#[derive(Default)]
pub struct TypeBoundListBuilder {
    bounds: Vec<Box<TypeBoundBuilder>>,
}
impl TypeBoundListBuilder {
    pub fn bound(mut self, f: TypeBoundBuilder) -> Self {
        self.bounds.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for TypeBoundListBuilder {
    type Node = TypeBoundList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TYPE_BOUND_LIST);
        self.bounds.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> TypeParamBuilder {
        TypeParamBuilder::default()
    }
}
#[derive(Default)]
pub struct TypeParamBuilder {
    name: Option<Box<NameBuilder>>,
    attrs: Vec<Box<AttrBuilder>>,
    type_bound_list: Option<Box<TypeBoundListBuilder>>,
    default_type: Option<Box<TypeRefBuilder>>,
}
impl TypeParamBuilder {
    pub fn name(mut self, f: NameBuilder) -> Self {
        self.name = Some(Box::new(f));
        self
    }
    pub fn attr(mut self, f: AttrBuilder) -> Self {
        self.attrs.push(Box::new(f));
        self
    }
    pub fn type_bound_list(mut self, f: TypeBoundListBuilder) -> Self {
        self.type_bound_list = Some(Box::new(f));
        self
    }
    pub fn default_type(mut self, f: TypeRefBuilder) -> Self {
        self.default_type = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for TypeParamBuilder {
    type Node = TypeParam;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TYPE_PARAM);
        if let Some(b) = self.name {
            b.make(builder);
        }
        self.attrs.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> TypeParamListBuilder {
        TypeParamListBuilder::default()
    }
}
#[derive(Default)]
pub struct TypeParamListBuilder {
    type_params: Vec<Box<TypeParamBuilder>>,
    lifetime_params: Vec<Box<LifetimeParamBuilder>>,
}
impl TypeParamListBuilder {
    pub fn type_param(mut self, f: TypeParamBuilder) -> Self {
        self.type_params.push(Box::new(f));
        self
    }
    pub fn lifetime_param(mut self, f: LifetimeParamBuilder) -> Self {
        self.lifetime_params.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for TypeParamListBuilder {
    type Node = TypeParamList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::TYPE_PARAM_LIST);
        self.type_params.into_iter().for_each(|b| b.make(builder));
        self.lifetime_params.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> UseTreeBuilder {
        UseTreeBuilder::default()
    }
}
#[derive(Default)]
pub struct UseTreeBuilder {
    path: Option<Box<PathBuilder>>,
    use_tree_list: Option<Box<UseTreeListBuilder>>,
    alias: Option<Box<AliasBuilder>>,
}
impl UseTreeBuilder {
    pub fn path(mut self, f: PathBuilder) -> Self {
        self.path = Some(Box::new(f));
        self
    }
    pub fn use_tree_list(mut self, f: UseTreeListBuilder) -> Self {
        self.use_tree_list = Some(Box::new(f));
        self
    }
    pub fn alia(mut self, f: AliasBuilder) -> Self {
        self.alias = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for UseTreeBuilder {
    type Node = UseTree;
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
    pub fn new() -> UseTreeListBuilder {
        UseTreeListBuilder::default()
    }
}
#[derive(Default)]
pub struct UseTreeListBuilder {
    use_trees: Vec<Box<UseTreeBuilder>>,
}
impl UseTreeListBuilder {
    pub fn use_tree(mut self, f: UseTreeBuilder) -> Self {
        self.use_trees.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for UseTreeListBuilder {
    type Node = UseTreeList;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::USE_TREE_LIST);
        self.use_trees.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> WhereClauseBuilder {
        WhereClauseBuilder::default()
    }
}
#[derive(Default)]
pub struct WhereClauseBuilder {
    predicates: Vec<Box<WherePredBuilder>>,
}
impl WhereClauseBuilder {
    pub fn predicate(mut self, f: WherePredBuilder) -> Self {
        self.predicates.push(Box::new(f));
        self
    }
}
impl AstNodeBuilder for WhereClauseBuilder {
    type Node = WhereClause;
    fn make(self, builder: &mut SyntaxTreeBuilder) {
        builder.start_node(SyntaxKind::WHERE_CLAUSE);
        self.predicates.into_iter().for_each(|b| b.make(builder));
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
    pub fn new() -> WherePredBuilder {
        WherePredBuilder::default()
    }
}
#[derive(Default)]
pub struct WherePredBuilder {
    type_ref: Option<Box<TypeRefBuilder>>,
    type_bound_list: Option<Box<TypeBoundListBuilder>>,
}
impl WherePredBuilder {
    pub fn type_ref(mut self, f: TypeRefBuilder) -> Self {
        self.type_ref = Some(Box::new(f));
        self
    }
    pub fn type_bound_list(mut self, f: TypeBoundListBuilder) -> Self {
        self.type_bound_list = Some(Box::new(f));
        self
    }
}
impl AstNodeBuilder for WherePredBuilder {
    type Node = WherePred;
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
