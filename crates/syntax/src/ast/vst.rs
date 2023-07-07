// defines VST's Expr

use crate::{
    ast::{
        self,
        operators::{ArithOp, BinaryOp, CmpOp, LogicOp, Ordering, RangeOp, UnaryOp},
        generated::vst_nodes::*,
        support, AstChildren, AstNode,
    },
    AstToken,
    SyntaxKind::*,
    SyntaxNode, SyntaxToken, T,
};

use super::{generated, HasAttrs};


// pub struct BinExpr {
//     pub attrs: Vec<Attr>,
//     pub lhs: Option<Box<Expr>>,
//     pub op: Option<BinaryOp>,
//     pub rhs: Option<Box<Expr>>,
// }

// impl From<generated::nodes::BinExpr> for BinExpr {
//     fn from(item: generated::nodes::BinExpr) -> Self {
//         Self { 
//             attrs: item.attrs().into_iter().map(Attr::from).collect(),
//             lhs: item.lhs().map(Expr::from).map(Box::new),
//             op: item.op_details().map(|it| it.1),
//             rhs:item.rhs().map(Expr::from).map(Box::new),
//         }
//     }
// }