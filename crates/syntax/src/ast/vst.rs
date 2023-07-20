// defines VST handwritten nodes

pub use crate::ast::{self, generated::vst_nodes::*, operators::BinaryOp};

pub use super::{generated, HasAttrs};

pub(crate) fn token_ascii(name: &String) -> &str {
    match name.as_str() {
        "semicolon" => ";",
        "thin_arrow" => "->",
        "l_curly" => "{",
        "r_curly" => "}",
        "l_paren" => "(",
        "r_paren" => ")",
        "l_brack" => "[",
        "r_brack" => "]",
        "l_angle" => "<",
        "r_angle" => ">",
        "eq" => "=",
        "excl" => "!",
        "star" => "*",
        "amp" => "&",
        "minus" => "-",
        "underscore" => "_",
        "dot" => ".",
        "dotdot" => "..",
        "dotdotdot" => "...",
        "dotdoteq" => "..=",
        "fat_arrow" => "=>",
        "at" => "@",
        "colon" => ":",
        "coloncolon" => "::",
        "pound" => "#",
        "question_mark" => "?",
        "comma" => ",",
        "pipe" => "|",
        "tilde" => "~",
        _ => name.as_str(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinExpr {
    pub attrs: Vec<Attr>,
    pub lhs: Box<Expr>,
    pub op: BinaryOp,
    pub rhs: Box<Expr>,
    pub cst: Option<generated::nodes::BinExpr>,
}

impl std::fmt::Display for BinExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s.push_str(&self.attrs.iter().map(|it| it.to_string()).collect::<Vec<String>>().join(" "));
        s.push_str(&self.lhs.to_string());
        s.push_str(&self.op.to_string());
        s.push_str(&self.rhs.to_string());
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IfExpr {
    pub attrs: Vec<Attr>,
    if_token: bool,
    pub condition: Box<Expr>,
    pub then_branch: Box<BlockExpr>,
    else_token: bool,
    pub else_branch: Option<Box<ElseBranch>>,
    pub cst: Option<generated::nodes::IfExpr>,
}

impl std::fmt::Display for IfExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s.push_str(&self.attrs.iter().map(|it| it.to_string()).collect::<Vec<String>>().join(" "));
        if self.if_token {
            s.push_str("if ");
        }
        s.push_str(&self.condition.to_string());
        s.push_str(&self.then_branch.to_string());
        if self.else_token {
            s.push_str("else ");
        }
        if let Some(it) = &self.else_branch {
            s.push_str(&it.to_string());
        }
        write!(f, "{s}")
    }
}

impl IfExpr {
    pub fn new<ET0>(condition: ET0, then_branch: BlockExpr) -> Self
    where ET0: Into<Expr>,
    {
        IfExpr {
            attrs: vec![],
            if_token: true,
            condition: Box::new(condition.into()),
            then_branch: Box::new(then_branch),
            else_token: false,
            else_branch: None,
            cst: None,
        }
    }

    pub fn set_else_branch(&mut self, else_branch: ElseBranch) {
        self.else_token = true;
        self.else_branch = Some(Box::new(else_branch));
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElseBranch {
    Block(Box<BlockExpr>),
    IfExpr(Box<IfExpr>),
}

impl std::fmt::Display for ElseBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElseBranch::Block(it) => write!(f, "{}", it),
            ElseBranch::IfExpr(it) => write!(f, "{}", it),
        }
    }
}

impl ElseBranch {
    pub fn cst(&self) -> Option<crate::ast::ElseBranch> {
        match self {
            // Stmt::ExprStmt(it) => Some(super::nodes::Stmt::ExprStmt(it.cst.as_ref()?.clone())),
            // Stmt::Item(it) => Some(super::nodes::Stmt::Item(it.cst()?.clone())),
            // Stmt::LetStmt(it) => Some(super::nodes::Stmt::LetStmt(it.cst.as_ref()?.clone())),
            ElseBranch::Block(it) => Some(crate::ast::ElseBranch::Block(it.cst.as_ref()?.clone())),
            ElseBranch::IfExpr(it) => {
                Some(crate::ast::ElseBranch::IfExpr(it.cst.as_ref()?.clone()))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Literal {
    pub attrs: Vec<Attr>,
    pub literal: String,
    pub cst: Option<generated::nodes::Literal>,
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s.push_str(&self.attrs.iter().map(|it| it.to_string()).collect::<Vec<String>>().join(" "));
        s.push_str(&self.literal);
        write!(f, "{s}")
    }
}

impl Literal {
    pub fn new(id: String) -> Self {
        Literal
        {
            attrs: vec![],
            literal: id.clone(),
            cst: None,
        }
    }
}

impl TryFrom<generated::nodes::Literal> for Literal {
    type Error = String;
    fn try_from(item: generated::nodes::Literal) -> Result<Self, Self::Error> {
        use ast::expr_ext::LiteralKind;
        Ok(Self {
            attrs: item
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
            cst: Some(item.clone()),
        })
    }
}

impl TryFrom<generated::nodes::BinExpr> for BinExpr {
    type Error = String;
    fn try_from(item: generated::nodes::BinExpr) -> Result<Self, Self::Error> {
        Ok(Self {
            attrs: item
                .attrs()
                .into_iter()
                .map(Attr::try_from)
                .collect::<Result<Vec<Attr>, String>>()?,
            lhs: Box::new(
                item.lhs().ok_or(format!("{}", stringify!(lhs))).map(|it| Expr::try_from(it))??,
            ),
            op: item.op_details().ok_or(format!("{}", stringify!(op_details))).map(|it| it.1)?,
            rhs: Box::new(
                item.rhs().ok_or(format!("{}", stringify!(rhs))).map(|it| Expr::try_from(it))??,
            ),
            cst: Some(item.clone()),
        })
    }
}

impl BinExpr {
    pub fn new<ET0, ET1>(lhs: ET0, op: BinaryOp, rhs: ET1) -> Self 
        where ET0: Into<Expr>, ET1: Into<Expr>
    {
        BinExpr {
            attrs: vec![],
            lhs: Box::new(lhs.into()),
            op,
            rhs: Box::new(rhs.into()),
            cst: None,
        }
    }
}

impl TryFrom<super::expr_ext::ElseBranch> for ElseBranch {
    type Error = String;
    fn try_from(item: super::expr_ext::ElseBranch) -> Result<Self, Self::Error> {
        match item {
            super::expr_ext::ElseBranch::Block(it) => Ok(Self::Block(Box::new(it.try_into()?))),
            super::expr_ext::ElseBranch::IfExpr(it) => Ok(Self::IfExpr(Box::new(it.try_into()?))),
        }
    }
}

impl TryFrom<generated::nodes::IfExpr> for IfExpr {
    type Error = String;
    fn try_from(item: generated::nodes::IfExpr) -> Result<Self, Self::Error> {
        Ok(Self {
            attrs: item
                .attrs()
                .into_iter()
                .map(Attr::try_from)
                .collect::<Result<Vec<Attr>, String>>()?,
            if_token: item.if_token().is_some(),
            condition: Box::new(
                item.condition()
                    .ok_or(format!("{}", stringify!(condition)))
                    .map(|it| Expr::try_from(it))??,
            ),
            then_branch: Box::new(
                item.then_branch()
                    .ok_or(format!("{}", stringify!(then_branch)))
                    .map(|it| BlockExpr::try_from(it))??,
            ),
            else_token: item.else_token().is_some(),
            else_branch: match item.else_branch() {
                Some(it) => Some(Box::new(ElseBranch::try_from(it)?)),
                None => None,
            },
            cst: Some(item.clone()),
        })
    }
}

// display for HAND_WRITTEN_PRINT_ONLY
impl std::fmt::Display for ParamList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        if self.l_paren_token {
            let mut tmp = stringify!(l_paren_token).to_string();
            tmp.truncate(tmp.len() - 6);
            s.push_str(token_ascii(&tmp));
            s.push_str(" ");
        }
        if let Some(it) = &self.self_param {
            s.push_str(&it.to_string());
            s.push_str(" ");
        }
        if self.comma_token && self.self_param.is_some() {
            let mut tmp = stringify!(comma_token).to_string();
            tmp.truncate(tmp.len() - 6);
            s.push_str(token_ascii(&tmp));
            s.push_str(" ");
        }
        s.push_str(
            &self.params.iter().map(|it| it.to_string()).collect::<Vec<String>>().join(", "),
        );
        if self.r_paren_token {
            let mut tmp = stringify!(r_paren_token).to_string();
            tmp.truncate(tmp.len() - 6);
            s.push_str(token_ascii(&tmp));
            s.push_str(" ");
        }
        if self.pipe_token {
            let mut tmp = stringify!(pipe_token).to_string();
            tmp.truncate(tmp.len() - 6);
            s.push_str(token_ascii(&tmp));
            s.push_str(" ");
        }
        write!(f, "{s}")
    }
}

impl std::fmt::Display for ArgList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        if self.l_paren_token {
            let mut tmp = stringify!(l_paren_token).to_string();
            tmp.truncate(tmp.len() - 6);
            s.push_str(token_ascii(&tmp));
            s.push_str(" ");
        }
        s.push_str(&self.args.iter().map(|it| it.to_string()).collect::<Vec<String>>().join(", "));
        if self.r_paren_token {
            let mut tmp = stringify!(r_paren_token).to_string();
            tmp.truncate(tmp.len() - 6);
            s.push_str(token_ascii(&tmp));
            s.push_str(" ");
        }
        write!(f, "{s}")
    }
}

impl ExprStmt {
    pub fn new<ET0>(expr: ET0) -> Self
    where
        ET0: Into<Expr>,
    {
        Self { expr: Box::new(expr.into()), semicolon_token: true, cst: None } // semicolon default to true, where ungram has it optional
    }
}