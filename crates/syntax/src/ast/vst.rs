// defines VST's Expr

use rustc_lexer::LiteralKind;

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinExpr {
    pub attrs: Vec<Attr>,
    pub lhs: Box<Expr>,
    pub op: BinaryOp,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IfExpr {
    pub attrs: Vec<Attr>,
    if_token: bool,
    pub condition: Box<Expr>,
    pub then_branch: Box<BlockExpr>,
    else_token: bool,
    pub else_branch: Option<Box<ElseBranch>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElseBranch {
    Block(Box<BlockExpr>),
    IfExpr(Box<IfExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Literal {
    pub attrs: Vec<Attr>,
    pub literal: String
}

impl TryFrom<generated::nodes::Literal> for Literal {
    type Error = String;
    fn try_from(item: generated::nodes::Literal) -> Result<Self, Self::Error> {
        use ast::expr_ext::LiteralKind;
        Ok(Self { 
            attrs:item
            .attrs()
            .into_iter()
            .map(Attr::try_from)
            .collect::<Result<Vec<Attr>, String>>()?,
            literal: match item.kind() {
                LiteralKind::String(it) => it.to_string(),
                LiteralKind::ByteString(it) => it.to_string(),
                LiteralKind::CString(it) => it.to_string(),
                LiteralKind::IntNumber(it) => it.to_string(),
                LiteralKind::FloatNumber(it) => it.to_string(),
                LiteralKind::Char(it) => it.to_string(),
                LiteralKind::Byte(it) => it.to_string(),
                LiteralKind::Bool(it) => it.to_string(),
            },
        })
    }
}

impl TryFrom<generated::nodes::BinExpr> for BinExpr {
    type Error = String;
    fn try_from(item: generated::nodes::BinExpr) -> Result<Self, Self::Error> {
        Ok(Self { 
            attrs:item
            .attrs()
            .into_iter()
            .map(Attr::try_from)
            .collect::<Result<Vec<Attr>, String>>()?,
            lhs: Box::new(
                item.lhs()
                    .ok_or(format!("{}", stringify!(lhs)))
                    .map(|it| Expr::try_from(it))??),
            op: item.op_details().ok_or(format!("{}", stringify!(op_details))).map(|it| it.1)?,
            rhs: Box::new(
                item.rhs()
                    .ok_or(format!("{}", stringify!(rhs)))
                    .map(|it| Expr::try_from(it))??),        })
    }
}

impl TryFrom<super::expr_ext::ElseBranch> for ElseBranch {
    type Error = String;
    fn try_from(item: super::expr_ext::ElseBranch) -> Result<Self, Self::Error> {
        match item {
            super::expr_ext::ElseBranch::Block(it) => {
                Ok(Self::Block(Box::new(it.try_into()?)))
            }
            super::expr_ext::ElseBranch::IfExpr(it) => {
                Ok(Self::IfExpr(Box::new(it.try_into()?)))
            }
        }
    }
}

impl TryFrom<generated::nodes::IfExpr> for IfExpr {
    type Error = String;
    fn try_from(item: generated::nodes::IfExpr) -> Result<Self, Self::Error> {
        Ok(Self { 
            attrs:item
            .attrs()
            .into_iter()
            .map(Attr::try_from)
            .collect::<Result<Vec<Attr>, String>>()?,
            if_token: item.if_token().is_some(),
            condition: Box::new(
                item.condition()
                    .ok_or(format!("{}", stringify!(condition)))
                    .map(|it| Expr::try_from(it))??),
            then_branch: Box::new(
                item.then_branch()
                    .ok_or(format!("{}", stringify!(then_branch)))
                    .map(|it| BlockExpr::try_from(it))??),
            else_token: item.else_token().is_some(),
            else_branch: match item.else_branch() {
                Some(it) => Some(Box::new(ElseBranch::try_from(it)?)),
                None => None,
            },
        })
    }
}

