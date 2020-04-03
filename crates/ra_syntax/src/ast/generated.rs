//! Generated file, do not edit by hand, see `xtask/src/codegen`

#[allow(unused_imports)]
use crate::{
    ast::{self, AstChildElements, AstChildTokens, AstChildren, AstElement, AstNode, AstToken},
    NodeOrToken, SyntaxElement,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken,
};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Semi(SyntaxToken);
impl std::fmt::Display for Semi {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Semi {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SEMI => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Semi {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            SEMI => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Comma(SyntaxToken);
impl std::fmt::Display for Comma {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Comma {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            COMMA => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Comma {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            COMMA => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LParen(SyntaxToken);
impl std::fmt::Display for LParen {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for LParen {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            L_PAREN => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for LParen {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            L_PAREN => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RParen(SyntaxToken);
impl std::fmt::Display for RParen {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for RParen {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            R_PAREN => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for RParen {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            R_PAREN => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LCurly(SyntaxToken);
impl std::fmt::Display for LCurly {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for LCurly {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            L_CURLY => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for LCurly {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            L_CURLY => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RCurly(SyntaxToken);
impl std::fmt::Display for RCurly {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for RCurly {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            R_CURLY => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for RCurly {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            R_CURLY => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LBrack(SyntaxToken);
impl std::fmt::Display for LBrack {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for LBrack {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            L_BRACK => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for LBrack {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            L_BRACK => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RBrack(SyntaxToken);
impl std::fmt::Display for RBrack {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for RBrack {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            R_BRACK => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for RBrack {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            R_BRACK => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LAngle(SyntaxToken);
impl std::fmt::Display for LAngle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for LAngle {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            L_ANGLE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for LAngle {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            L_ANGLE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RAngle(SyntaxToken);
impl std::fmt::Display for RAngle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for RAngle {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            R_ANGLE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for RAngle {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            R_ANGLE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct At(SyntaxToken);
impl std::fmt::Display for At {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for At {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            AT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for At {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            AT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pound(SyntaxToken);
impl std::fmt::Display for Pound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Pound {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            POUND => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Pound {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            POUND => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tilde(SyntaxToken);
impl std::fmt::Display for Tilde {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Tilde {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TILDE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Tilde {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TILDE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Question(SyntaxToken);
impl std::fmt::Display for Question {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Question {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            QUESTION => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Question {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            QUESTION => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dollar(SyntaxToken);
impl std::fmt::Display for Dollar {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Dollar {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            DOLLAR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Dollar {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            DOLLAR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Amp(SyntaxToken);
impl std::fmt::Display for Amp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Amp {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            AMP => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Amp {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            AMP => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pipe(SyntaxToken);
impl std::fmt::Display for Pipe {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Pipe {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PIPE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Pipe {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PIPE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Plus(SyntaxToken);
impl std::fmt::Display for Plus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Plus {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PLUS => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Plus {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PLUS => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Star(SyntaxToken);
impl std::fmt::Display for Star {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Star {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            STAR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Star {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            STAR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Slash(SyntaxToken);
impl std::fmt::Display for Slash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Slash {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SLASH => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Slash {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            SLASH => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Caret(SyntaxToken);
impl std::fmt::Display for Caret {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Caret {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CARET => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Caret {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            CARET => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Percent(SyntaxToken);
impl std::fmt::Display for Percent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Percent {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PERCENT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Percent {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PERCENT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Underscore(SyntaxToken);
impl std::fmt::Display for Underscore {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Underscore {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            UNDERSCORE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Underscore {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            UNDERSCORE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dot(SyntaxToken);
impl std::fmt::Display for Dot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Dot {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            DOT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Dot {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            DOT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dotdot(SyntaxToken);
impl std::fmt::Display for Dotdot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Dotdot {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            DOTDOT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Dotdot {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            DOTDOT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dotdotdot(SyntaxToken);
impl std::fmt::Display for Dotdotdot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Dotdotdot {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            DOTDOTDOT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Dotdotdot {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            DOTDOTDOT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dotdoteq(SyntaxToken);
impl std::fmt::Display for Dotdoteq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Dotdoteq {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            DOTDOTEQ => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Dotdoteq {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            DOTDOTEQ => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Colon(SyntaxToken);
impl std::fmt::Display for Colon {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Colon {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            COLON => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Colon {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            COLON => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Coloncolon(SyntaxToken);
impl std::fmt::Display for Coloncolon {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Coloncolon {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            COLONCOLON => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Coloncolon {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            COLONCOLON => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Eq(SyntaxToken);
impl std::fmt::Display for Eq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Eq {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            EQ => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Eq {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            EQ => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Eqeq(SyntaxToken);
impl std::fmt::Display for Eqeq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Eqeq {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            EQEQ => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Eqeq {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            EQEQ => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FatArrow(SyntaxToken);
impl std::fmt::Display for FatArrow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for FatArrow {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            FAT_ARROW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for FatArrow {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            FAT_ARROW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Excl(SyntaxToken);
impl std::fmt::Display for Excl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Excl {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            EXCL => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Excl {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            EXCL => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Neq(SyntaxToken);
impl std::fmt::Display for Neq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Neq {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            NEQ => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Neq {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            NEQ => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Minus(SyntaxToken);
impl std::fmt::Display for Minus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Minus {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MINUS => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Minus {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            MINUS => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThinArrow(SyntaxToken);
impl std::fmt::Display for ThinArrow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for ThinArrow {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            THIN_ARROW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for ThinArrow {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            THIN_ARROW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lteq(SyntaxToken);
impl std::fmt::Display for Lteq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Lteq {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LTEQ => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Lteq {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            LTEQ => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Gteq(SyntaxToken);
impl std::fmt::Display for Gteq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Gteq {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            GTEQ => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Gteq {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            GTEQ => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pluseq(SyntaxToken);
impl std::fmt::Display for Pluseq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Pluseq {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PLUSEQ => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Pluseq {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PLUSEQ => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Minuseq(SyntaxToken);
impl std::fmt::Display for Minuseq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Minuseq {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MINUSEQ => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Minuseq {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            MINUSEQ => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pipeeq(SyntaxToken);
impl std::fmt::Display for Pipeeq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Pipeeq {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PIPEEQ => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Pipeeq {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PIPEEQ => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ampeq(SyntaxToken);
impl std::fmt::Display for Ampeq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Ampeq {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            AMPEQ => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Ampeq {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            AMPEQ => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Careteq(SyntaxToken);
impl std::fmt::Display for Careteq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Careteq {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CARETEQ => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Careteq {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            CARETEQ => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Slasheq(SyntaxToken);
impl std::fmt::Display for Slasheq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Slasheq {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SLASHEQ => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Slasheq {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            SLASHEQ => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Stareq(SyntaxToken);
impl std::fmt::Display for Stareq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Stareq {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            STAREQ => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Stareq {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            STAREQ => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Percenteq(SyntaxToken);
impl std::fmt::Display for Percenteq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Percenteq {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PERCENTEQ => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Percenteq {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PERCENTEQ => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ampamp(SyntaxToken);
impl std::fmt::Display for Ampamp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Ampamp {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            AMPAMP => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Ampamp {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            AMPAMP => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pipepipe(SyntaxToken);
impl std::fmt::Display for Pipepipe {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Pipepipe {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PIPEPIPE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Pipepipe {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PIPEPIPE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shl(SyntaxToken);
impl std::fmt::Display for Shl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Shl {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SHL => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Shl {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            SHL => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shr(SyntaxToken);
impl std::fmt::Display for Shr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Shr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SHR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Shr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            SHR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shleq(SyntaxToken);
impl std::fmt::Display for Shleq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Shleq {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SHLEQ => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Shleq {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            SHLEQ => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shreq(SyntaxToken);
impl std::fmt::Display for Shreq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Shreq {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SHREQ => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Shreq {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            SHREQ => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AsKw(SyntaxToken);
impl std::fmt::Display for AsKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for AsKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            AS_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for AsKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            AS_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AsyncKw(SyntaxToken);
impl std::fmt::Display for AsyncKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for AsyncKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ASYNC_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for AsyncKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ASYNC_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AwaitKw(SyntaxToken);
impl std::fmt::Display for AwaitKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for AwaitKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            AWAIT_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for AwaitKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            AWAIT_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoxKw(SyntaxToken);
impl std::fmt::Display for BoxKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for BoxKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BOX_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for BoxKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            BOX_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BreakKw(SyntaxToken);
impl std::fmt::Display for BreakKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for BreakKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BREAK_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for BreakKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            BREAK_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConstKw(SyntaxToken);
impl std::fmt::Display for ConstKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for ConstKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CONST_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for ConstKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            CONST_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContinueKw(SyntaxToken);
impl std::fmt::Display for ContinueKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for ContinueKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CONTINUE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for ContinueKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            CONTINUE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrateKw(SyntaxToken);
impl std::fmt::Display for CrateKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for CrateKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CRATE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for CrateKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            CRATE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DynKw(SyntaxToken);
impl std::fmt::Display for DynKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for DynKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            DYN_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for DynKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            DYN_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ElseKw(SyntaxToken);
impl std::fmt::Display for ElseKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for ElseKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ELSE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for ElseKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ELSE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumKw(SyntaxToken);
impl std::fmt::Display for EnumKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for EnumKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ENUM_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for EnumKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ENUM_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternKw(SyntaxToken);
impl std::fmt::Display for ExternKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for ExternKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            EXTERN_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for ExternKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            EXTERN_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FalseKw(SyntaxToken);
impl std::fmt::Display for FalseKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for FalseKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            FALSE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for FalseKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            FALSE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnKw(SyntaxToken);
impl std::fmt::Display for FnKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for FnKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            FN_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for FnKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            FN_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForKw(SyntaxToken);
impl std::fmt::Display for ForKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for ForKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            FOR_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for ForKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            FOR_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IfKw(SyntaxToken);
impl std::fmt::Display for IfKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for IfKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            IF_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for IfKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            IF_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImplKw(SyntaxToken);
impl std::fmt::Display for ImplKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for ImplKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            IMPL_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for ImplKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            IMPL_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InKw(SyntaxToken);
impl std::fmt::Display for InKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for InKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            IN_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for InKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            IN_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LetKw(SyntaxToken);
impl std::fmt::Display for LetKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for LetKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LET_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for LetKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            LET_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoopKw(SyntaxToken);
impl std::fmt::Display for LoopKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for LoopKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LOOP_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for LoopKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            LOOP_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MacroKw(SyntaxToken);
impl std::fmt::Display for MacroKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for MacroKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MACRO_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for MacroKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            MACRO_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MatchKw(SyntaxToken);
impl std::fmt::Display for MatchKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for MatchKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MATCH_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for MatchKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            MATCH_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModKw(SyntaxToken);
impl std::fmt::Display for ModKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for ModKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MOD_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for ModKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            MOD_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MoveKw(SyntaxToken);
impl std::fmt::Display for MoveKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for MoveKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MOVE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for MoveKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            MOVE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MutKw(SyntaxToken);
impl std::fmt::Display for MutKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for MutKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MUT_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for MutKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            MUT_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PubKw(SyntaxToken);
impl std::fmt::Display for PubKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for PubKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PUB_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for PubKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PUB_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RefKw(SyntaxToken);
impl std::fmt::Display for RefKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for RefKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            REF_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for RefKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            REF_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReturnKw(SyntaxToken);
impl std::fmt::Display for ReturnKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for ReturnKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RETURN_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for ReturnKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            RETURN_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SelfKw(SyntaxToken);
impl std::fmt::Display for SelfKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for SelfKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SELF_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for SelfKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            SELF_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StaticKw(SyntaxToken);
impl std::fmt::Display for StaticKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for StaticKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            STATIC_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for StaticKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            STATIC_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructKw(SyntaxToken);
impl std::fmt::Display for StructKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for StructKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            STRUCT_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for StructKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            STRUCT_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SuperKw(SyntaxToken);
impl std::fmt::Display for SuperKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for SuperKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SUPER_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for SuperKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            SUPER_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraitKw(SyntaxToken);
impl std::fmt::Display for TraitKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for TraitKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TRAIT_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for TraitKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TRAIT_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrueKw(SyntaxToken);
impl std::fmt::Display for TrueKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for TrueKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TRUE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for TrueKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TRUE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TryKw(SyntaxToken);
impl std::fmt::Display for TryKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for TryKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TRY_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for TryKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TRY_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeKw(SyntaxToken);
impl std::fmt::Display for TypeKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for TypeKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for TypeKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnsafeKw(SyntaxToken);
impl std::fmt::Display for UnsafeKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for UnsafeKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            UNSAFE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for UnsafeKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            UNSAFE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UseKw(SyntaxToken);
impl std::fmt::Display for UseKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for UseKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            USE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for UseKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            USE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WhereKw(SyntaxToken);
impl std::fmt::Display for WhereKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for WhereKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            WHERE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for WhereKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            WHERE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WhileKw(SyntaxToken);
impl std::fmt::Display for WhileKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for WhileKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            WHILE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for WhileKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            WHILE_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AutoKw(SyntaxToken);
impl std::fmt::Display for AutoKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for AutoKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            AUTO_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for AutoKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            AUTO_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefaultKw(SyntaxToken);
impl std::fmt::Display for DefaultKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for DefaultKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            DEFAULT_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for DefaultKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            DEFAULT_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExistentialKw(SyntaxToken);
impl std::fmt::Display for ExistentialKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for ExistentialKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            EXISTENTIAL_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for ExistentialKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            EXISTENTIAL_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionKw(SyntaxToken);
impl std::fmt::Display for UnionKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for UnionKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            UNION_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for UnionKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            UNION_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawKw(SyntaxToken);
impl std::fmt::Display for RawKw {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for RawKw {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RAW_KW => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for RawKw {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            RAW_KW => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntNumber(SyntaxToken);
impl std::fmt::Display for IntNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for IntNumber {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            INT_NUMBER => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for IntNumber {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            INT_NUMBER => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FloatNumber(SyntaxToken);
impl std::fmt::Display for FloatNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for FloatNumber {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            FLOAT_NUMBER => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for FloatNumber {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            FLOAT_NUMBER => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Char(SyntaxToken);
impl std::fmt::Display for Char {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Char {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CHAR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Char {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            CHAR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Byte(SyntaxToken);
impl std::fmt::Display for Byte {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Byte {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BYTE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Byte {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            BYTE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct String(SyntaxToken);
impl std::fmt::Display for String {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for String {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            STRING => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for String {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            STRING => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawString(SyntaxToken);
impl std::fmt::Display for RawString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for RawString {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RAW_STRING => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for RawString {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            RAW_STRING => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ByteString(SyntaxToken);
impl std::fmt::Display for ByteString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for ByteString {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BYTE_STRING => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for ByteString {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            BYTE_STRING => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawByteString(SyntaxToken);
impl std::fmt::Display for RawByteString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for RawByteString {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RAW_BYTE_STRING => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for RawByteString {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            RAW_BYTE_STRING => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Error(SyntaxToken);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Error {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ERROR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Error {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ERROR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident(SyntaxToken);
impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Ident {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            IDENT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Ident {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            IDENT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Whitespace(SyntaxToken);
impl std::fmt::Display for Whitespace {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Whitespace {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            WHITESPACE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Whitespace {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            WHITESPACE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lifetime(SyntaxToken);
impl std::fmt::Display for Lifetime {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Lifetime {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LIFETIME => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Lifetime {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            LIFETIME => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Comment(SyntaxToken);
impl std::fmt::Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Comment {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            COMMENT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Comment {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            COMMENT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shebang(SyntaxToken);
impl std::fmt::Display for Shebang {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for Shebang {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SHEBANG => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for Shebang {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            SHEBANG => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LDollar(SyntaxToken);
impl std::fmt::Display for LDollar {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for LDollar {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            L_DOLLAR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for LDollar {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            L_DOLLAR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RDollar(SyntaxToken);
impl std::fmt::Display for RDollar {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstToken for RDollar {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            R_DOLLAR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self(syntax))
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
    fn into_syntax(self) -> SyntaxToken {
        self.0
    }
}
impl AstElement for RDollar {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            R_DOLLAR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self(syntax.into_token().unwrap()))
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Token(&self.0)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Token(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceFile {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for SourceFile {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SOURCE_FILE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for SourceFile {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            SOURCE_FILE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::ModuleItemOwner for SourceFile {}
impl ast::FnDefOwner for SourceFile {}
impl ast::AttrsOwner for SourceFile {}
impl SourceFile {
    pub fn modules(&self) -> impl Iterator<Item = Module> + Clone {
        self.syntax.children().filter_map(Module::cast)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnDef {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for FnDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for FnDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            FN_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for FnDef {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            FN_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::VisibilityOwner for FnDef {}
impl ast::NameOwner for FnDef {}
impl ast::TypeParamsOwner for FnDef {}
impl ast::DocCommentsOwner for FnDef {}
impl ast::AttrsOwner for FnDef {}
impl FnDef {
    pub fn abi(&self) -> Option<Abi> {
        self.syntax.children().filter_map(Abi::cast).next()
    }
    pub fn const_kw(&self) -> Option<ConstKw> {
        self.syntax.children_with_tokens().filter_map(ConstKw::cast_element).next()
    }
    pub fn default_kw(&self) -> Option<DefaultKw> {
        self.syntax.children_with_tokens().filter_map(DefaultKw::cast_element).next()
    }
    pub fn async_kw(&self) -> Option<AsyncKw> {
        self.syntax.children_with_tokens().filter_map(AsyncKw::cast_element).next()
    }
    pub fn unsafe_kw(&self) -> Option<UnsafeKw> {
        self.syntax.children_with_tokens().filter_map(UnsafeKw::cast_element).next()
    }
    pub fn fn_kw(&self) -> Option<FnKw> {
        self.syntax.children_with_tokens().filter_map(FnKw::cast_element).next()
    }
    pub fn param_list(&self) -> Option<ParamList> {
        self.syntax.children().filter_map(ParamList::cast).next()
    }
    pub fn ret_type(&self) -> Option<RetType> {
        self.syntax.children().filter_map(RetType::cast).next()
    }
    pub fn body(&self) -> Option<BlockExpr> {
        self.syntax.children().filter_map(BlockExpr::cast).next()
    }
    pub fn semi(&self) -> Option<Semi> {
        self.syntax.children_with_tokens().filter_map(Semi::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RetType {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for RetType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for RetType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RET_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for RetType {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            RET_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl RetType {
    pub fn thin_arrow(&self) -> Option<ThinArrow> {
        self.syntax.children_with_tokens().filter_map(ThinArrow::cast_element).next()
    }
    pub fn type_ref(&self) -> Option<TypeRef> {
        self.syntax.children().filter_map(TypeRef::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructDef {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for StructDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for StructDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            STRUCT_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for StructDef {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            STRUCT_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::VisibilityOwner for StructDef {}
impl ast::NameOwner for StructDef {}
impl ast::TypeParamsOwner for StructDef {}
impl ast::AttrsOwner for StructDef {}
impl ast::DocCommentsOwner for StructDef {}
impl StructDef {
    pub fn struct_kw(&self) -> Option<StructKw> {
        self.syntax.children_with_tokens().filter_map(StructKw::cast_element).next()
    }
    pub fn field_def_list(&self) -> Option<FieldDefList> {
        self.syntax.children().filter_map(FieldDefList::cast).next()
    }
    pub fn semi(&self) -> Option<Semi> {
        self.syntax.children_with_tokens().filter_map(Semi::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionDef {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for UnionDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for UnionDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            UNION_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for UnionDef {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            UNION_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::VisibilityOwner for UnionDef {}
impl ast::NameOwner for UnionDef {}
impl ast::TypeParamsOwner for UnionDef {}
impl ast::AttrsOwner for UnionDef {}
impl ast::DocCommentsOwner for UnionDef {}
impl UnionDef {
    pub fn union_kw(&self) -> Option<UnionKw> {
        self.syntax.children_with_tokens().filter_map(UnionKw::cast_element).next()
    }
    pub fn record_field_def_list(&self) -> Option<RecordFieldDefList> {
        self.syntax.children().filter_map(RecordFieldDefList::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordFieldDefList {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for RecordFieldDefList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for RecordFieldDefList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD_DEF_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for RecordFieldDefList {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD_DEF_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl RecordFieldDefList {
    pub fn l_curly(&self) -> Option<LCurly> {
        self.syntax.children_with_tokens().filter_map(LCurly::cast_element).next()
    }
    pub fn fields(&self) -> impl Iterator<Item = RecordFieldDef> + Clone {
        self.syntax.children().filter_map(RecordFieldDef::cast)
    }
    pub fn r_curly(&self) -> Option<RCurly> {
        self.syntax.children_with_tokens().filter_map(RCurly::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordFieldDef {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for RecordFieldDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for RecordFieldDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for RecordFieldDef {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::VisibilityOwner for RecordFieldDef {}
impl ast::NameOwner for RecordFieldDef {}
impl ast::AttrsOwner for RecordFieldDef {}
impl ast::DocCommentsOwner for RecordFieldDef {}
impl ast::TypeAscriptionOwner for RecordFieldDef {}
impl RecordFieldDef {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TupleFieldDefList {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for TupleFieldDefList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for TupleFieldDefList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_FIELD_DEF_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for TupleFieldDefList {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_FIELD_DEF_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl TupleFieldDefList {
    pub fn l_paren(&self) -> Option<LParen> {
        self.syntax.children_with_tokens().filter_map(LParen::cast_element).next()
    }
    pub fn fields(&self) -> impl Iterator<Item = TupleFieldDef> + Clone {
        self.syntax.children().filter_map(TupleFieldDef::cast)
    }
    pub fn r_paren(&self) -> Option<RParen> {
        self.syntax.children_with_tokens().filter_map(RParen::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TupleFieldDef {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for TupleFieldDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for TupleFieldDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_FIELD_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for TupleFieldDef {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_FIELD_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::VisibilityOwner for TupleFieldDef {}
impl ast::AttrsOwner for TupleFieldDef {}
impl TupleFieldDef {
    pub fn type_ref(&self) -> Option<TypeRef> {
        self.syntax.children().filter_map(TypeRef::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumDef {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for EnumDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for EnumDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ENUM_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for EnumDef {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ENUM_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::VisibilityOwner for EnumDef {}
impl ast::NameOwner for EnumDef {}
impl ast::TypeParamsOwner for EnumDef {}
impl ast::AttrsOwner for EnumDef {}
impl ast::DocCommentsOwner for EnumDef {}
impl EnumDef {
    pub fn enum_kw(&self) -> Option<EnumKw> {
        self.syntax.children_with_tokens().filter_map(EnumKw::cast_element).next()
    }
    pub fn variant_list(&self) -> Option<EnumVariantList> {
        self.syntax.children().filter_map(EnumVariantList::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumVariantList {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for EnumVariantList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for EnumVariantList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ENUM_VARIANT_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for EnumVariantList {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ENUM_VARIANT_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl EnumVariantList {
    pub fn l_curly(&self) -> Option<LCurly> {
        self.syntax.children_with_tokens().filter_map(LCurly::cast_element).next()
    }
    pub fn variants(&self) -> impl Iterator<Item = EnumVariant> + Clone {
        self.syntax.children().filter_map(EnumVariant::cast)
    }
    pub fn r_curly(&self) -> Option<RCurly> {
        self.syntax.children_with_tokens().filter_map(RCurly::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumVariant {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for EnumVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for EnumVariant {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ENUM_VARIANT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for EnumVariant {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ENUM_VARIANT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::VisibilityOwner for EnumVariant {}
impl ast::NameOwner for EnumVariant {}
impl ast::DocCommentsOwner for EnumVariant {}
impl ast::AttrsOwner for EnumVariant {}
impl EnumVariant {
    pub fn field_def_list(&self) -> Option<FieldDefList> {
        self.syntax.children().filter_map(FieldDefList::cast).next()
    }
    pub fn eq(&self) -> Option<Eq> {
        self.syntax.children_with_tokens().filter_map(Eq::cast_element).next()
    }
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraitDef {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for TraitDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for TraitDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TRAIT_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for TraitDef {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TRAIT_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::VisibilityOwner for TraitDef {}
impl ast::NameOwner for TraitDef {}
impl ast::AttrsOwner for TraitDef {}
impl ast::DocCommentsOwner for TraitDef {}
impl ast::TypeParamsOwner for TraitDef {}
impl ast::TypeBoundsOwner for TraitDef {}
impl TraitDef {
    pub fn unsafe_kw(&self) -> Option<UnsafeKw> {
        self.syntax.children_with_tokens().filter_map(UnsafeKw::cast_element).next()
    }
    pub fn auto_kw(&self) -> Option<AutoKw> {
        self.syntax.children_with_tokens().filter_map(AutoKw::cast_element).next()
    }
    pub fn trait_kw(&self) -> Option<TraitKw> {
        self.syntax.children_with_tokens().filter_map(TraitKw::cast_element).next()
    }
    pub fn item_list(&self) -> Option<ItemList> {
        self.syntax.children().filter_map(ItemList::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Module {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for Module {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MODULE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for Module {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            MODULE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::VisibilityOwner for Module {}
impl ast::NameOwner for Module {}
impl ast::AttrsOwner for Module {}
impl ast::DocCommentsOwner for Module {}
impl Module {
    pub fn mod_kw(&self) -> Option<ModKw> {
        self.syntax.children_with_tokens().filter_map(ModKw::cast_element).next()
    }
    pub fn item_list(&self) -> Option<ItemList> {
        self.syntax.children().filter_map(ItemList::cast).next()
    }
    pub fn semi(&self) -> Option<Semi> {
        self.syntax.children_with_tokens().filter_map(Semi::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemList {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ItemList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ItemList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ITEM_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ItemList {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ITEM_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::FnDefOwner for ItemList {}
impl ast::ModuleItemOwner for ItemList {}
impl ItemList {
    pub fn l_curly(&self) -> Option<LCurly> {
        self.syntax.children_with_tokens().filter_map(LCurly::cast_element).next()
    }
    pub fn impl_items(&self) -> impl Iterator<Item = ImplItem> + Clone {
        self.syntax.children().filter_map(ImplItem::cast)
    }
    pub fn r_curly(&self) -> Option<RCurly> {
        self.syntax.children_with_tokens().filter_map(RCurly::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConstDef {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ConstDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ConstDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CONST_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ConstDef {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            CONST_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::VisibilityOwner for ConstDef {}
impl ast::NameOwner for ConstDef {}
impl ast::TypeParamsOwner for ConstDef {}
impl ast::AttrsOwner for ConstDef {}
impl ast::DocCommentsOwner for ConstDef {}
impl ast::TypeAscriptionOwner for ConstDef {}
impl ConstDef {
    pub fn default_kw(&self) -> Option<DefaultKw> {
        self.syntax.children_with_tokens().filter_map(DefaultKw::cast_element).next()
    }
    pub fn const_kw(&self) -> Option<ConstKw> {
        self.syntax.children_with_tokens().filter_map(ConstKw::cast_element).next()
    }
    pub fn eq(&self) -> Option<Eq> {
        self.syntax.children_with_tokens().filter_map(Eq::cast_element).next()
    }
    pub fn body(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
    pub fn semi(&self) -> Option<Semi> {
        self.syntax.children_with_tokens().filter_map(Semi::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StaticDef {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for StaticDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for StaticDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            STATIC_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for StaticDef {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            STATIC_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::VisibilityOwner for StaticDef {}
impl ast::NameOwner for StaticDef {}
impl ast::TypeParamsOwner for StaticDef {}
impl ast::AttrsOwner for StaticDef {}
impl ast::DocCommentsOwner for StaticDef {}
impl ast::TypeAscriptionOwner for StaticDef {}
impl StaticDef {
    pub fn static_kw(&self) -> Option<StaticKw> {
        self.syntax.children_with_tokens().filter_map(StaticKw::cast_element).next()
    }
    pub fn mut_kw(&self) -> Option<MutKw> {
        self.syntax.children_with_tokens().filter_map(MutKw::cast_element).next()
    }
    pub fn eq(&self) -> Option<Eq> {
        self.syntax.children_with_tokens().filter_map(Eq::cast_element).next()
    }
    pub fn body(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
    pub fn semi(&self) -> Option<Semi> {
        self.syntax.children_with_tokens().filter_map(Semi::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeAliasDef {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for TypeAliasDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for TypeAliasDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_ALIAS_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for TypeAliasDef {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_ALIAS_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::VisibilityOwner for TypeAliasDef {}
impl ast::NameOwner for TypeAliasDef {}
impl ast::TypeParamsOwner for TypeAliasDef {}
impl ast::AttrsOwner for TypeAliasDef {}
impl ast::DocCommentsOwner for TypeAliasDef {}
impl ast::TypeBoundsOwner for TypeAliasDef {}
impl TypeAliasDef {
    pub fn default_kw(&self) -> Option<DefaultKw> {
        self.syntax.children_with_tokens().filter_map(DefaultKw::cast_element).next()
    }
    pub fn type_kw(&self) -> Option<TypeKw> {
        self.syntax.children_with_tokens().filter_map(TypeKw::cast_element).next()
    }
    pub fn eq(&self) -> Option<Eq> {
        self.syntax.children_with_tokens().filter_map(Eq::cast_element).next()
    }
    pub fn type_ref(&self) -> Option<TypeRef> {
        self.syntax.children().filter_map(TypeRef::cast).next()
    }
    pub fn semi(&self) -> Option<Semi> {
        self.syntax.children_with_tokens().filter_map(Semi::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImplDef {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ImplDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ImplDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            IMPL_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ImplDef {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            IMPL_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::TypeParamsOwner for ImplDef {}
impl ast::AttrsOwner for ImplDef {}
impl ImplDef {
    pub fn default_kw(&self) -> Option<DefaultKw> {
        self.syntax.children_with_tokens().filter_map(DefaultKw::cast_element).next()
    }
    pub fn const_kw(&self) -> Option<ConstKw> {
        self.syntax.children_with_tokens().filter_map(ConstKw::cast_element).next()
    }
    pub fn unsafe_kw(&self) -> Option<UnsafeKw> {
        self.syntax.children_with_tokens().filter_map(UnsafeKw::cast_element).next()
    }
    pub fn impl_kw(&self) -> Option<ImplKw> {
        self.syntax.children_with_tokens().filter_map(ImplKw::cast_element).next()
    }
    pub fn excl(&self) -> Option<Excl> {
        self.syntax.children_with_tokens().filter_map(Excl::cast_element).next()
    }
    pub fn for_kw(&self) -> Option<ForKw> {
        self.syntax.children_with_tokens().filter_map(ForKw::cast_element).next()
    }
    pub fn item_list(&self) -> Option<ItemList> {
        self.syntax.children().filter_map(ItemList::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParenType {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ParenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ParenType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PAREN_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ParenType {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PAREN_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ParenType {
    pub fn l_paren(&self) -> Option<LParen> {
        self.syntax.children_with_tokens().filter_map(LParen::cast_element).next()
    }
    pub fn type_ref(&self) -> Option<TypeRef> {
        self.syntax.children().filter_map(TypeRef::cast).next()
    }
    pub fn r_paren(&self) -> Option<RParen> {
        self.syntax.children_with_tokens().filter_map(RParen::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TupleType {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for TupleType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for TupleType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for TupleType {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl TupleType {
    pub fn l_paren(&self) -> Option<LParen> {
        self.syntax.children_with_tokens().filter_map(LParen::cast_element).next()
    }
    pub fn fields(&self) -> impl Iterator<Item = TypeRef> + Clone {
        self.syntax.children().filter_map(TypeRef::cast)
    }
    pub fn r_paren(&self) -> Option<RParen> {
        self.syntax.children_with_tokens().filter_map(RParen::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NeverType {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for NeverType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for NeverType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            NEVER_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for NeverType {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            NEVER_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl NeverType {
    pub fn excl(&self) -> Option<Excl> {
        self.syntax.children_with_tokens().filter_map(Excl::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathType {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for PathType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for PathType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PATH_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for PathType {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PATH_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl PathType {
    pub fn path(&self) -> Option<Path> {
        self.syntax.children().filter_map(Path::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PointerType {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for PointerType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for PointerType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            POINTER_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for PointerType {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            POINTER_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl PointerType {
    pub fn star(&self) -> Option<Star> {
        self.syntax.children_with_tokens().filter_map(Star::cast_element).next()
    }
    pub fn const_kw(&self) -> Option<ConstKw> {
        self.syntax.children_with_tokens().filter_map(ConstKw::cast_element).next()
    }
    pub fn type_ref(&self) -> Option<TypeRef> {
        self.syntax.children().filter_map(TypeRef::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayType {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ArrayType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ArrayType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ARRAY_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ArrayType {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ARRAY_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ArrayType {
    pub fn l_brack(&self) -> Option<LBrack> {
        self.syntax.children_with_tokens().filter_map(LBrack::cast_element).next()
    }
    pub fn type_ref(&self) -> Option<TypeRef> {
        self.syntax.children().filter_map(TypeRef::cast).next()
    }
    pub fn semi(&self) -> Option<Semi> {
        self.syntax.children_with_tokens().filter_map(Semi::cast_element).next()
    }
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
    pub fn r_brack(&self) -> Option<RBrack> {
        self.syntax.children_with_tokens().filter_map(RBrack::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SliceType {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for SliceType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for SliceType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SLICE_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for SliceType {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            SLICE_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl SliceType {
    pub fn l_brack(&self) -> Option<LBrack> {
        self.syntax.children_with_tokens().filter_map(LBrack::cast_element).next()
    }
    pub fn type_ref(&self) -> Option<TypeRef> {
        self.syntax.children().filter_map(TypeRef::cast).next()
    }
    pub fn r_brack(&self) -> Option<RBrack> {
        self.syntax.children_with_tokens().filter_map(RBrack::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceType {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ReferenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ReferenceType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            REFERENCE_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ReferenceType {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            REFERENCE_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ReferenceType {
    pub fn amp(&self) -> Option<Amp> {
        self.syntax.children_with_tokens().filter_map(Amp::cast_element).next()
    }
    pub fn lifetime(&self) -> Option<Lifetime> {
        self.syntax.children_with_tokens().filter_map(Lifetime::cast_element).next()
    }
    pub fn mut_kw(&self) -> Option<MutKw> {
        self.syntax.children_with_tokens().filter_map(MutKw::cast_element).next()
    }
    pub fn type_ref(&self) -> Option<TypeRef> {
        self.syntax.children().filter_map(TypeRef::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlaceholderType {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for PlaceholderType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for PlaceholderType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PLACEHOLDER_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for PlaceholderType {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PLACEHOLDER_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl PlaceholderType {
    pub fn underscore(&self) -> Option<Underscore> {
        self.syntax.children_with_tokens().filter_map(Underscore::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnPointerType {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for FnPointerType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for FnPointerType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            FN_POINTER_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for FnPointerType {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            FN_POINTER_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl FnPointerType {
    pub fn abi(&self) -> Option<Abi> {
        self.syntax.children().filter_map(Abi::cast).next()
    }
    pub fn unsafe_kw(&self) -> Option<UnsafeKw> {
        self.syntax.children_with_tokens().filter_map(UnsafeKw::cast_element).next()
    }
    pub fn fn_kw(&self) -> Option<FnKw> {
        self.syntax.children_with_tokens().filter_map(FnKw::cast_element).next()
    }
    pub fn param_list(&self) -> Option<ParamList> {
        self.syntax.children().filter_map(ParamList::cast).next()
    }
    pub fn ret_type(&self) -> Option<RetType> {
        self.syntax.children().filter_map(RetType::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForType {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ForType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ForType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            FOR_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ForType {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            FOR_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ForType {
    pub fn for_kw(&self) -> Option<ForKw> {
        self.syntax.children_with_tokens().filter_map(ForKw::cast_element).next()
    }
    pub fn type_param_list(&self) -> Option<TypeParamList> {
        self.syntax.children().filter_map(TypeParamList::cast).next()
    }
    pub fn type_ref(&self) -> Option<TypeRef> {
        self.syntax.children().filter_map(TypeRef::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImplTraitType {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ImplTraitType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ImplTraitType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            IMPL_TRAIT_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ImplTraitType {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            IMPL_TRAIT_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::TypeBoundsOwner for ImplTraitType {}
impl ImplTraitType {
    pub fn impl_kw(&self) -> Option<ImplKw> {
        self.syntax.children_with_tokens().filter_map(ImplKw::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DynTraitType {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for DynTraitType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for DynTraitType {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            DYN_TRAIT_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for DynTraitType {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            DYN_TRAIT_TYPE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::TypeBoundsOwner for DynTraitType {}
impl DynTraitType {
    pub fn dyn_kw(&self) -> Option<DynKw> {
        self.syntax.children_with_tokens().filter_map(DynKw::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TupleExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for TupleExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for TupleExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for TupleExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for TupleExpr {}
impl TupleExpr {
    pub fn l_paren(&self) -> Option<LParen> {
        self.syntax.children_with_tokens().filter_map(LParen::cast_element).next()
    }
    pub fn exprs(&self) -> impl Iterator<Item = Expr> + Clone {
        self.syntax.children().filter_map(Expr::cast)
    }
    pub fn r_paren(&self) -> Option<RParen> {
        self.syntax.children_with_tokens().filter_map(RParen::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ArrayExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ArrayExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ARRAY_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ArrayExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ARRAY_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for ArrayExpr {}
impl ArrayExpr {
    pub fn l_brack(&self) -> Option<LBrack> {
        self.syntax.children_with_tokens().filter_map(LBrack::cast_element).next()
    }
    pub fn exprs(&self) -> impl Iterator<Item = Expr> + Clone {
        self.syntax.children().filter_map(Expr::cast)
    }
    pub fn semi(&self) -> Option<Semi> {
        self.syntax.children_with_tokens().filter_map(Semi::cast_element).next()
    }
    pub fn r_brack(&self) -> Option<RBrack> {
        self.syntax.children_with_tokens().filter_map(RBrack::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParenExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ParenExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ParenExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PAREN_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ParenExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PAREN_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for ParenExpr {}
impl ParenExpr {
    pub fn l_paren(&self) -> Option<LParen> {
        self.syntax.children_with_tokens().filter_map(LParen::cast_element).next()
    }
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
    pub fn r_paren(&self) -> Option<RParen> {
        self.syntax.children_with_tokens().filter_map(RParen::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for PathExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for PathExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PATH_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for PathExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PATH_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl PathExpr {
    pub fn path(&self) -> Option<Path> {
        self.syntax.children().filter_map(Path::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LambdaExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for LambdaExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for LambdaExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LAMBDA_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for LambdaExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            LAMBDA_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for LambdaExpr {}
impl LambdaExpr {
    pub fn static_kw(&self) -> Option<StaticKw> {
        self.syntax.children_with_tokens().filter_map(StaticKw::cast_element).next()
    }
    pub fn async_kw(&self) -> Option<AsyncKw> {
        self.syntax.children_with_tokens().filter_map(AsyncKw::cast_element).next()
    }
    pub fn move_kw(&self) -> Option<MoveKw> {
        self.syntax.children_with_tokens().filter_map(MoveKw::cast_element).next()
    }
    pub fn param_list(&self) -> Option<ParamList> {
        self.syntax.children().filter_map(ParamList::cast).next()
    }
    pub fn ret_type(&self) -> Option<RetType> {
        self.syntax.children().filter_map(RetType::cast).next()
    }
    pub fn body(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IfExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for IfExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for IfExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            IF_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for IfExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            IF_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for IfExpr {}
impl IfExpr {
    pub fn if_kw(&self) -> Option<IfKw> {
        self.syntax.children_with_tokens().filter_map(IfKw::cast_element).next()
    }
    pub fn condition(&self) -> Option<Condition> {
        self.syntax.children().filter_map(Condition::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoopExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for LoopExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for LoopExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LOOP_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for LoopExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            LOOP_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for LoopExpr {}
impl ast::LoopBodyOwner for LoopExpr {}
impl LoopExpr {
    pub fn loop_kw(&self) -> Option<LoopKw> {
        self.syntax.children_with_tokens().filter_map(LoopKw::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TryBlockExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for TryBlockExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for TryBlockExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TRY_BLOCK_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for TryBlockExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TRY_BLOCK_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for TryBlockExpr {}
impl TryBlockExpr {
    pub fn try_kw(&self) -> Option<TryKw> {
        self.syntax.children_with_tokens().filter_map(TryKw::cast_element).next()
    }
    pub fn body(&self) -> Option<BlockExpr> {
        self.syntax.children().filter_map(BlockExpr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ForExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ForExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            FOR_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ForExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            FOR_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for ForExpr {}
impl ast::LoopBodyOwner for ForExpr {}
impl ForExpr {
    pub fn for_kw(&self) -> Option<ForKw> {
        self.syntax.children_with_tokens().filter_map(ForKw::cast_element).next()
    }
    pub fn pat(&self) -> Option<Pat> {
        self.syntax.children().filter_map(Pat::cast).next()
    }
    pub fn in_kw(&self) -> Option<InKw> {
        self.syntax.children_with_tokens().filter_map(InKw::cast_element).next()
    }
    pub fn iterable(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WhileExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for WhileExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for WhileExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            WHILE_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for WhileExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            WHILE_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for WhileExpr {}
impl ast::LoopBodyOwner for WhileExpr {}
impl WhileExpr {
    pub fn while_kw(&self) -> Option<WhileKw> {
        self.syntax.children_with_tokens().filter_map(WhileKw::cast_element).next()
    }
    pub fn condition(&self) -> Option<Condition> {
        self.syntax.children().filter_map(Condition::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContinueExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ContinueExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ContinueExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CONTINUE_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ContinueExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            CONTINUE_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for ContinueExpr {}
impl ContinueExpr {
    pub fn continue_kw(&self) -> Option<ContinueKw> {
        self.syntax.children_with_tokens().filter_map(ContinueKw::cast_element).next()
    }
    pub fn lifetime(&self) -> Option<Lifetime> {
        self.syntax.children_with_tokens().filter_map(Lifetime::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BreakExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for BreakExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for BreakExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BREAK_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for BreakExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            BREAK_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for BreakExpr {}
impl BreakExpr {
    pub fn break_kw(&self) -> Option<BreakKw> {
        self.syntax.children_with_tokens().filter_map(BreakKw::cast_element).next()
    }
    pub fn lifetime(&self) -> Option<Lifetime> {
        self.syntax.children_with_tokens().filter_map(Lifetime::cast_element).next()
    }
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for Label {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LABEL => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for Label {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            LABEL => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl Label {
    pub fn lifetime(&self) -> Option<Lifetime> {
        self.syntax.children_with_tokens().filter_map(Lifetime::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for BlockExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for BlockExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BLOCK_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for BlockExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            BLOCK_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for BlockExpr {}
impl BlockExpr {
    pub fn label(&self) -> Option<Label> {
        self.syntax.children().filter_map(Label::cast).next()
    }
    pub fn unsafe_kw(&self) -> Option<UnsafeKw> {
        self.syntax.children_with_tokens().filter_map(UnsafeKw::cast_element).next()
    }
    pub fn block(&self) -> Option<Block> {
        self.syntax.children().filter_map(Block::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReturnExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ReturnExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ReturnExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RETURN_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ReturnExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            RETURN_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for ReturnExpr {}
impl ReturnExpr {
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CallExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for CallExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for CallExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CALL_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for CallExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            CALL_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::ArgListOwner for CallExpr {}
impl CallExpr {
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MethodCallExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for MethodCallExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for MethodCallExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            METHOD_CALL_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for MethodCallExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            METHOD_CALL_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for MethodCallExpr {}
impl ast::ArgListOwner for MethodCallExpr {}
impl MethodCallExpr {
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
    pub fn dot(&self) -> Option<Dot> {
        self.syntax.children_with_tokens().filter_map(Dot::cast_element).next()
    }
    pub fn name_ref(&self) -> Option<NameRef> {
        self.syntax.children().filter_map(NameRef::cast).next()
    }
    pub fn type_arg_list(&self) -> Option<TypeArgList> {
        self.syntax.children().filter_map(TypeArgList::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndexExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for IndexExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for IndexExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            INDEX_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for IndexExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            INDEX_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for IndexExpr {}
impl IndexExpr {
    pub fn l_brack(&self) -> Option<LBrack> {
        self.syntax.children_with_tokens().filter_map(LBrack::cast_element).next()
    }
    pub fn r_brack(&self) -> Option<RBrack> {
        self.syntax.children_with_tokens().filter_map(RBrack::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for FieldExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for FieldExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            FIELD_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for FieldExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            FIELD_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for FieldExpr {}
impl FieldExpr {
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
    pub fn dot(&self) -> Option<Dot> {
        self.syntax.children_with_tokens().filter_map(Dot::cast_element).next()
    }
    pub fn name_ref(&self) -> Option<NameRef> {
        self.syntax.children().filter_map(NameRef::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AwaitExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for AwaitExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for AwaitExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            AWAIT_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for AwaitExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            AWAIT_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for AwaitExpr {}
impl AwaitExpr {
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
    pub fn dot(&self) -> Option<Dot> {
        self.syntax.children_with_tokens().filter_map(Dot::cast_element).next()
    }
    pub fn await_kw(&self) -> Option<AwaitKw> {
        self.syntax.children_with_tokens().filter_map(AwaitKw::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TryExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for TryExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for TryExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TRY_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for TryExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TRY_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for TryExpr {}
impl TryExpr {
    pub fn try_kw(&self) -> Option<TryKw> {
        self.syntax.children_with_tokens().filter_map(TryKw::cast_element).next()
    }
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CastExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for CastExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for CastExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CAST_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for CastExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            CAST_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for CastExpr {}
impl CastExpr {
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
    pub fn as_kw(&self) -> Option<AsKw> {
        self.syntax.children_with_tokens().filter_map(AsKw::cast_element).next()
    }
    pub fn type_ref(&self) -> Option<TypeRef> {
        self.syntax.children().filter_map(TypeRef::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RefExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for RefExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for RefExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            REF_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for RefExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            REF_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for RefExpr {}
impl RefExpr {
    pub fn amp(&self) -> Option<Amp> {
        self.syntax.children_with_tokens().filter_map(Amp::cast_element).next()
    }
    pub fn raw_kw(&self) -> Option<RawKw> {
        self.syntax.children_with_tokens().filter_map(RawKw::cast_element).next()
    }
    pub fn mut_kw(&self) -> Option<MutKw> {
        self.syntax.children_with_tokens().filter_map(MutKw::cast_element).next()
    }
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrefixExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for PrefixExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for PrefixExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PREFIX_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for PrefixExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PREFIX_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for PrefixExpr {}
impl PrefixExpr {
    pub fn prefix_op(&self) -> Option<PrefixOp> {
        self.syntax.children_with_tokens().filter_map(PrefixOp::cast_element).next()
    }
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoxExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for BoxExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for BoxExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BOX_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for BoxExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            BOX_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for BoxExpr {}
impl BoxExpr {
    pub fn box_kw(&self) -> Option<BoxKw> {
        self.syntax.children_with_tokens().filter_map(BoxKw::cast_element).next()
    }
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RangeExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for RangeExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for RangeExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RANGE_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for RangeExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            RANGE_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for RangeExpr {}
impl RangeExpr {
    pub fn range_op(&self) -> Option<RangeOp> {
        self.syntax.children_with_tokens().filter_map(RangeOp::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for BinExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for BinExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BIN_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for BinExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            BIN_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for BinExpr {}
impl BinExpr {
    pub fn bin_op(&self) -> Option<BinOp> {
        self.syntax.children_with_tokens().filter_map(BinOp::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Literal {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for Literal {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LITERAL => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for Literal {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            LITERAL => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl Literal {
    pub fn literal_token(&self) -> Option<LiteralToken> {
        self.syntax.children_with_tokens().filter_map(LiteralToken::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MatchExpr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for MatchExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for MatchExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MATCH_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for MatchExpr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            MATCH_EXPR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for MatchExpr {}
impl MatchExpr {
    pub fn match_kw(&self) -> Option<MatchKw> {
        self.syntax.children_with_tokens().filter_map(MatchKw::cast_element).next()
    }
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
    pub fn match_arm_list(&self) -> Option<MatchArmList> {
        self.syntax.children().filter_map(MatchArmList::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MatchArmList {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for MatchArmList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for MatchArmList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MATCH_ARM_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for MatchArmList {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            MATCH_ARM_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for MatchArmList {}
impl MatchArmList {
    pub fn l_curly(&self) -> Option<LCurly> {
        self.syntax.children_with_tokens().filter_map(LCurly::cast_element).next()
    }
    pub fn arms(&self) -> impl Iterator<Item = MatchArm> + Clone {
        self.syntax.children().filter_map(MatchArm::cast)
    }
    pub fn r_curly(&self) -> Option<RCurly> {
        self.syntax.children_with_tokens().filter_map(RCurly::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MatchArm {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for MatchArm {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for MatchArm {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MATCH_ARM => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for MatchArm {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            MATCH_ARM => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for MatchArm {}
impl MatchArm {
    pub fn pat(&self) -> Option<Pat> {
        self.syntax.children().filter_map(Pat::cast).next()
    }
    pub fn guard(&self) -> Option<MatchGuard> {
        self.syntax.children().filter_map(MatchGuard::cast).next()
    }
    pub fn fat_arrow(&self) -> Option<FatArrow> {
        self.syntax.children_with_tokens().filter_map(FatArrow::cast_element).next()
    }
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MatchGuard {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for MatchGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for MatchGuard {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MATCH_GUARD => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for MatchGuard {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            MATCH_GUARD => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl MatchGuard {
    pub fn if_kw(&self) -> Option<IfKw> {
        self.syntax.children_with_tokens().filter_map(IfKw::cast_element).next()
    }
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordLit {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for RecordLit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for RecordLit {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_LIT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for RecordLit {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_LIT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl RecordLit {
    pub fn path(&self) -> Option<Path> {
        self.syntax.children().filter_map(Path::cast).next()
    }
    pub fn record_field_list(&self) -> Option<RecordFieldList> {
        self.syntax.children().filter_map(RecordFieldList::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordFieldList {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for RecordFieldList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for RecordFieldList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for RecordFieldList {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl RecordFieldList {
    pub fn l_curly(&self) -> Option<LCurly> {
        self.syntax.children_with_tokens().filter_map(LCurly::cast_element).next()
    }
    pub fn fields(&self) -> impl Iterator<Item = RecordField> + Clone {
        self.syntax.children().filter_map(RecordField::cast)
    }
    pub fn dotdot(&self) -> Option<Dotdot> {
        self.syntax.children_with_tokens().filter_map(Dotdot::cast_element).next()
    }
    pub fn spread(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
    pub fn r_curly(&self) -> Option<RCurly> {
        self.syntax.children_with_tokens().filter_map(RCurly::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordField {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for RecordField {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for RecordField {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for RecordField {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for RecordField {}
impl RecordField {
    pub fn name_ref(&self) -> Option<NameRef> {
        self.syntax.children().filter_map(NameRef::cast).next()
    }
    pub fn colon(&self) -> Option<Colon> {
        self.syntax.children_with_tokens().filter_map(Colon::cast_element).next()
    }
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrPat {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for OrPat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for OrPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            OR_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for OrPat {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            OR_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl OrPat {
    pub fn pats(&self) -> impl Iterator<Item = Pat> + Clone {
        self.syntax.children().filter_map(Pat::cast)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParenPat {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ParenPat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ParenPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PAREN_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ParenPat {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PAREN_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ParenPat {
    pub fn l_paren(&self) -> Option<LParen> {
        self.syntax.children_with_tokens().filter_map(LParen::cast_element).next()
    }
    pub fn pat(&self) -> Option<Pat> {
        self.syntax.children().filter_map(Pat::cast).next()
    }
    pub fn r_paren(&self) -> Option<RParen> {
        self.syntax.children_with_tokens().filter_map(RParen::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RefPat {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for RefPat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for RefPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            REF_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for RefPat {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            REF_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl RefPat {
    pub fn amp(&self) -> Option<Amp> {
        self.syntax.children_with_tokens().filter_map(Amp::cast_element).next()
    }
    pub fn mut_kw(&self) -> Option<MutKw> {
        self.syntax.children_with_tokens().filter_map(MutKw::cast_element).next()
    }
    pub fn pat(&self) -> Option<Pat> {
        self.syntax.children().filter_map(Pat::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoxPat {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for BoxPat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for BoxPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BOX_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for BoxPat {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            BOX_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl BoxPat {
    pub fn box_kw(&self) -> Option<BoxKw> {
        self.syntax.children_with_tokens().filter_map(BoxKw::cast_element).next()
    }
    pub fn pat(&self) -> Option<Pat> {
        self.syntax.children().filter_map(Pat::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindPat {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for BindPat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for BindPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BIND_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for BindPat {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            BIND_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for BindPat {}
impl ast::NameOwner for BindPat {}
impl BindPat {
    pub fn ref_kw(&self) -> Option<RefKw> {
        self.syntax.children_with_tokens().filter_map(RefKw::cast_element).next()
    }
    pub fn mut_kw(&self) -> Option<MutKw> {
        self.syntax.children_with_tokens().filter_map(MutKw::cast_element).next()
    }
    pub fn pat(&self) -> Option<Pat> {
        self.syntax.children().filter_map(Pat::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlaceholderPat {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for PlaceholderPat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for PlaceholderPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PLACEHOLDER_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for PlaceholderPat {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PLACEHOLDER_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl PlaceholderPat {
    pub fn underscore(&self) -> Option<Underscore> {
        self.syntax.children_with_tokens().filter_map(Underscore::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DotDotPat {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for DotDotPat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for DotDotPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            DOT_DOT_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for DotDotPat {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            DOT_DOT_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl DotDotPat {
    pub fn dotdot(&self) -> Option<Dotdot> {
        self.syntax.children_with_tokens().filter_map(Dotdot::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathPat {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for PathPat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for PathPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PATH_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for PathPat {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PATH_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl PathPat {
    pub fn path(&self) -> Option<Path> {
        self.syntax.children().filter_map(Path::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SlicePat {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for SlicePat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for SlicePat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SLICE_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for SlicePat {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            SLICE_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl SlicePat {
    pub fn l_brack(&self) -> Option<LBrack> {
        self.syntax.children_with_tokens().filter_map(LBrack::cast_element).next()
    }
    pub fn args(&self) -> impl Iterator<Item = Pat> + Clone {
        self.syntax.children().filter_map(Pat::cast)
    }
    pub fn r_brack(&self) -> Option<RBrack> {
        self.syntax.children_with_tokens().filter_map(RBrack::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RangePat {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for RangePat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for RangePat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RANGE_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for RangePat {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            RANGE_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl RangePat {
    pub fn range_separator(&self) -> Option<RangeSeparator> {
        self.syntax.children_with_tokens().filter_map(RangeSeparator::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LiteralPat {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for LiteralPat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for LiteralPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LITERAL_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for LiteralPat {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            LITERAL_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl LiteralPat {
    pub fn literal(&self) -> Option<Literal> {
        self.syntax.children().filter_map(Literal::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordPat {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for RecordPat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for RecordPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for RecordPat {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl RecordPat {
    pub fn record_field_pat_list(&self) -> Option<RecordFieldPatList> {
        self.syntax.children().filter_map(RecordFieldPatList::cast).next()
    }
    pub fn path(&self) -> Option<Path> {
        self.syntax.children().filter_map(Path::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordFieldPatList {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for RecordFieldPatList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for RecordFieldPatList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD_PAT_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for RecordFieldPatList {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD_PAT_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl RecordFieldPatList {
    pub fn l_curly(&self) -> Option<LCurly> {
        self.syntax.children_with_tokens().filter_map(LCurly::cast_element).next()
    }
    pub fn pats(&self) -> impl Iterator<Item = RecordInnerPat> + Clone {
        self.syntax.children().filter_map(RecordInnerPat::cast)
    }
    pub fn record_field_pats(&self) -> impl Iterator<Item = RecordFieldPat> + Clone {
        self.syntax.children().filter_map(RecordFieldPat::cast)
    }
    pub fn bind_pats(&self) -> impl Iterator<Item = BindPat> + Clone {
        self.syntax.children().filter_map(BindPat::cast)
    }
    pub fn dotdot(&self) -> Option<Dotdot> {
        self.syntax.children_with_tokens().filter_map(Dotdot::cast_element).next()
    }
    pub fn r_curly(&self) -> Option<RCurly> {
        self.syntax.children_with_tokens().filter_map(RCurly::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordFieldPat {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for RecordFieldPat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for RecordFieldPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for RecordFieldPat {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for RecordFieldPat {}
impl ast::NameOwner for RecordFieldPat {}
impl RecordFieldPat {
    pub fn colon(&self) -> Option<Colon> {
        self.syntax.children_with_tokens().filter_map(Colon::cast_element).next()
    }
    pub fn pat(&self) -> Option<Pat> {
        self.syntax.children().filter_map(Pat::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TupleStructPat {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for TupleStructPat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for TupleStructPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_STRUCT_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for TupleStructPat {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_STRUCT_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl TupleStructPat {
    pub fn path(&self) -> Option<Path> {
        self.syntax.children().filter_map(Path::cast).next()
    }
    pub fn l_paren(&self) -> Option<LParen> {
        self.syntax.children_with_tokens().filter_map(LParen::cast_element).next()
    }
    pub fn args(&self) -> impl Iterator<Item = Pat> + Clone {
        self.syntax.children().filter_map(Pat::cast)
    }
    pub fn r_paren(&self) -> Option<RParen> {
        self.syntax.children_with_tokens().filter_map(RParen::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TuplePat {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for TuplePat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for TuplePat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for TuplePat {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TUPLE_PAT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl TuplePat {
    pub fn l_paren(&self) -> Option<LParen> {
        self.syntax.children_with_tokens().filter_map(LParen::cast_element).next()
    }
    pub fn args(&self) -> impl Iterator<Item = Pat> + Clone {
        self.syntax.children().filter_map(Pat::cast)
    }
    pub fn r_paren(&self) -> Option<RParen> {
        self.syntax.children_with_tokens().filter_map(RParen::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Visibility {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for Visibility {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            VISIBILITY => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for Visibility {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            VISIBILITY => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl Visibility {
    pub fn pub_kw(&self) -> Option<PubKw> {
        self.syntax.children_with_tokens().filter_map(PubKw::cast_element).next()
    }
    pub fn super_kw(&self) -> Option<SuperKw> {
        self.syntax.children_with_tokens().filter_map(SuperKw::cast_element).next()
    }
    pub fn self_kw(&self) -> Option<SelfKw> {
        self.syntax.children_with_tokens().filter_map(SelfKw::cast_element).next()
    }
    pub fn crate_kw(&self) -> Option<CrateKw> {
        self.syntax.children_with_tokens().filter_map(CrateKw::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for Name {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            NAME => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for Name {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            NAME => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl Name {
    pub fn ident(&self) -> Option<Ident> {
        self.syntax.children_with_tokens().filter_map(Ident::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NameRef {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for NameRef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for NameRef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            NAME_REF => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for NameRef {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            NAME_REF => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl NameRef {
    pub fn name_ref_token(&self) -> Option<NameRefToken> {
        self.syntax.children_with_tokens().filter_map(NameRefToken::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MacroCall {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for MacroCall {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for MacroCall {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MACRO_CALL => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for MacroCall {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            MACRO_CALL => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::NameOwner for MacroCall {}
impl ast::AttrsOwner for MacroCall {}
impl ast::DocCommentsOwner for MacroCall {}
impl MacroCall {
    pub fn path(&self) -> Option<Path> {
        self.syntax.children().filter_map(Path::cast).next()
    }
    pub fn excl(&self) -> Option<Excl> {
        self.syntax.children_with_tokens().filter_map(Excl::cast_element).next()
    }
    pub fn token_tree(&self) -> Option<TokenTree> {
        self.syntax.children().filter_map(TokenTree::cast).next()
    }
    pub fn semi(&self) -> Option<Semi> {
        self.syntax.children_with_tokens().filter_map(Semi::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Attr {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for Attr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for Attr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ATTR => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for Attr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ATTR => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl Attr {
    pub fn pound(&self) -> Option<Pound> {
        self.syntax.children_with_tokens().filter_map(Pound::cast_element).next()
    }
    pub fn excl(&self) -> Option<Excl> {
        self.syntax.children_with_tokens().filter_map(Excl::cast_element).next()
    }
    pub fn l_brack(&self) -> Option<LBrack> {
        self.syntax.children_with_tokens().filter_map(LBrack::cast_element).next()
    }
    pub fn path(&self) -> Option<Path> {
        self.syntax.children().filter_map(Path::cast).next()
    }
    pub fn eq(&self) -> Option<Eq> {
        self.syntax.children_with_tokens().filter_map(Eq::cast_element).next()
    }
    pub fn input(&self) -> Option<AttrInput> {
        self.syntax.children().filter_map(AttrInput::cast).next()
    }
    pub fn r_brack(&self) -> Option<RBrack> {
        self.syntax.children_with_tokens().filter_map(RBrack::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenTree {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for TokenTree {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for TokenTree {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TOKEN_TREE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for TokenTree {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TOKEN_TREE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl TokenTree {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeParamList {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for TypeParamList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for TypeParamList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_PARAM_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for TypeParamList {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_PARAM_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl TypeParamList {
    pub fn l_angle(&self) -> Option<LAngle> {
        self.syntax.children_with_tokens().filter_map(LAngle::cast_element).next()
    }
    pub fn generic_params(&self) -> impl Iterator<Item = GenericParam> + Clone {
        self.syntax.children().filter_map(GenericParam::cast)
    }
    pub fn type_params(&self) -> impl Iterator<Item = TypeParam> + Clone {
        self.syntax.children().filter_map(TypeParam::cast)
    }
    pub fn lifetime_params(&self) -> impl Iterator<Item = LifetimeParam> + Clone {
        self.syntax.children().filter_map(LifetimeParam::cast)
    }
    pub fn const_params(&self) -> impl Iterator<Item = ConstParam> + Clone {
        self.syntax.children().filter_map(ConstParam::cast)
    }
    pub fn r_angle(&self) -> Option<RAngle> {
        self.syntax.children_with_tokens().filter_map(RAngle::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeParam {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for TypeParam {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for TypeParam {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_PARAM => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for TypeParam {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_PARAM => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::NameOwner for TypeParam {}
impl ast::AttrsOwner for TypeParam {}
impl ast::TypeBoundsOwner for TypeParam {}
impl TypeParam {
    pub fn eq(&self) -> Option<Eq> {
        self.syntax.children_with_tokens().filter_map(Eq::cast_element).next()
    }
    pub fn default_type(&self) -> Option<TypeRef> {
        self.syntax.children().filter_map(TypeRef::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConstParam {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ConstParam {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ConstParam {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CONST_PARAM => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ConstParam {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            CONST_PARAM => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::NameOwner for ConstParam {}
impl ast::AttrsOwner for ConstParam {}
impl ast::TypeAscriptionOwner for ConstParam {}
impl ConstParam {
    pub fn eq(&self) -> Option<Eq> {
        self.syntax.children_with_tokens().filter_map(Eq::cast_element).next()
    }
    pub fn default_val(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LifetimeParam {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for LifetimeParam {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for LifetimeParam {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LIFETIME_PARAM => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for LifetimeParam {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            LIFETIME_PARAM => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for LifetimeParam {}
impl LifetimeParam {
    pub fn lifetime(&self) -> Option<Lifetime> {
        self.syntax.children_with_tokens().filter_map(Lifetime::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeBound {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for TypeBound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for TypeBound {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_BOUND => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for TypeBound {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_BOUND => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl TypeBound {
    pub fn lifetime(&self) -> Option<Lifetime> {
        self.syntax.children_with_tokens().filter_map(Lifetime::cast_element).next()
    }
    pub fn const_kw(&self) -> Option<ConstKw> {
        self.syntax.children_with_tokens().filter_map(ConstKw::cast_element).next()
    }
    pub fn type_ref(&self) -> Option<TypeRef> {
        self.syntax.children().filter_map(TypeRef::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeBoundList {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for TypeBoundList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for TypeBoundList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_BOUND_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for TypeBoundList {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_BOUND_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl TypeBoundList {
    pub fn bounds(&self) -> impl Iterator<Item = TypeBound> + Clone {
        self.syntax.children().filter_map(TypeBound::cast)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WherePred {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for WherePred {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for WherePred {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            WHERE_PRED => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for WherePred {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            WHERE_PRED => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::TypeBoundsOwner for WherePred {}
impl WherePred {
    pub fn lifetime(&self) -> Option<Lifetime> {
        self.syntax.children_with_tokens().filter_map(Lifetime::cast_element).next()
    }
    pub fn type_ref(&self) -> Option<TypeRef> {
        self.syntax.children().filter_map(TypeRef::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WhereClause {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for WhereClause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for WhereClause {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            WHERE_CLAUSE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for WhereClause {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            WHERE_CLAUSE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl WhereClause {
    pub fn where_kw(&self) -> Option<WhereKw> {
        self.syntax.children_with_tokens().filter_map(WhereKw::cast_element).next()
    }
    pub fn predicates(&self) -> impl Iterator<Item = WherePred> + Clone {
        self.syntax.children().filter_map(WherePred::cast)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Abi {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for Abi {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for Abi {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ABI => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for Abi {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ABI => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl Abi {
    pub fn string(&self) -> Option<String> {
        self.syntax.children_with_tokens().filter_map(String::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExprStmt {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ExprStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ExprStmt {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            EXPR_STMT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ExprStmt {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            EXPR_STMT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for ExprStmt {}
impl ExprStmt {
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
    pub fn semi(&self) -> Option<Semi> {
        self.syntax.children_with_tokens().filter_map(Semi::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LetStmt {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for LetStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for LetStmt {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LET_STMT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for LetStmt {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            LET_STMT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for LetStmt {}
impl ast::TypeAscriptionOwner for LetStmt {}
impl LetStmt {
    pub fn let_kw(&self) -> Option<LetKw> {
        self.syntax.children_with_tokens().filter_map(LetKw::cast_element).next()
    }
    pub fn pat(&self) -> Option<Pat> {
        self.syntax.children().filter_map(Pat::cast).next()
    }
    pub fn eq(&self) -> Option<Eq> {
        self.syntax.children_with_tokens().filter_map(Eq::cast_element).next()
    }
    pub fn initializer(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Condition {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for Condition {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CONDITION => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for Condition {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            CONDITION => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl Condition {
    pub fn let_kw(&self) -> Option<LetKw> {
        self.syntax.children_with_tokens().filter_map(LetKw::cast_element).next()
    }
    pub fn pat(&self) -> Option<Pat> {
        self.syntax.children().filter_map(Pat::cast).next()
    }
    pub fn eq(&self) -> Option<Eq> {
        self.syntax.children_with_tokens().filter_map(Eq::cast_element).next()
    }
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for Block {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BLOCK => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for Block {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            BLOCK => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for Block {}
impl ast::ModuleItemOwner for Block {}
impl Block {
    pub fn l_curly(&self) -> Option<LCurly> {
        self.syntax.children_with_tokens().filter_map(LCurly::cast_element).next()
    }
    pub fn statements(&self) -> impl Iterator<Item = Stmt> + Clone {
        self.syntax.children().filter_map(Stmt::cast)
    }
    pub fn statements_or_semi(&self) -> impl Iterator<Item = StmtOrSemi> + Clone {
        self.syntax.children_with_tokens().filter_map(StmtOrSemi::cast_element)
    }
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
    pub fn r_curly(&self) -> Option<RCurly> {
        self.syntax.children_with_tokens().filter_map(RCurly::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParamList {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ParamList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ParamList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PARAM_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ParamList {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PARAM_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ParamList {
    pub fn l_paren(&self) -> Option<LParen> {
        self.syntax.children_with_tokens().filter_map(LParen::cast_element).next()
    }
    pub fn self_param(&self) -> Option<SelfParam> {
        self.syntax.children().filter_map(SelfParam::cast).next()
    }
    pub fn params(&self) -> impl Iterator<Item = Param> + Clone {
        self.syntax.children().filter_map(Param::cast)
    }
    pub fn r_paren(&self) -> Option<RParen> {
        self.syntax.children_with_tokens().filter_map(RParen::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SelfParam {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for SelfParam {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for SelfParam {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SELF_PARAM => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for SelfParam {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            SELF_PARAM => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::TypeAscriptionOwner for SelfParam {}
impl ast::AttrsOwner for SelfParam {}
impl SelfParam {
    pub fn amp(&self) -> Option<Amp> {
        self.syntax.children_with_tokens().filter_map(Amp::cast_element).next()
    }
    pub fn lifetime(&self) -> Option<Lifetime> {
        self.syntax.children_with_tokens().filter_map(Lifetime::cast_element).next()
    }
    pub fn self_kw(&self) -> Option<SelfKw> {
        self.syntax.children_with_tokens().filter_map(SelfKw::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Param {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for Param {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PARAM => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for Param {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PARAM => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::TypeAscriptionOwner for Param {}
impl ast::AttrsOwner for Param {}
impl Param {
    pub fn pat(&self) -> Option<Pat> {
        self.syntax.children().filter_map(Pat::cast).next()
    }
    pub fn dotdotdot(&self) -> Option<Dotdotdot> {
        self.syntax.children_with_tokens().filter_map(Dotdotdot::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UseItem {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for UseItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for UseItem {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            USE_ITEM => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for UseItem {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            USE_ITEM => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for UseItem {}
impl ast::VisibilityOwner for UseItem {}
impl UseItem {
    pub fn use_kw(&self) -> Option<UseKw> {
        self.syntax.children_with_tokens().filter_map(UseKw::cast_element).next()
    }
    pub fn use_tree(&self) -> Option<UseTree> {
        self.syntax.children().filter_map(UseTree::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UseTree {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for UseTree {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for UseTree {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            USE_TREE => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for UseTree {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            USE_TREE => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl UseTree {
    pub fn path(&self) -> Option<Path> {
        self.syntax.children().filter_map(Path::cast).next()
    }
    pub fn star(&self) -> Option<Star> {
        self.syntax.children_with_tokens().filter_map(Star::cast_element).next()
    }
    pub fn use_tree_list(&self) -> Option<UseTreeList> {
        self.syntax.children().filter_map(UseTreeList::cast).next()
    }
    pub fn alias(&self) -> Option<Alias> {
        self.syntax.children().filter_map(Alias::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Alias {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for Alias {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for Alias {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ALIAS => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for Alias {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ALIAS => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::NameOwner for Alias {}
impl Alias {
    pub fn as_kw(&self) -> Option<AsKw> {
        self.syntax.children_with_tokens().filter_map(AsKw::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UseTreeList {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for UseTreeList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for UseTreeList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            USE_TREE_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for UseTreeList {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            USE_TREE_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl UseTreeList {
    pub fn l_curly(&self) -> Option<LCurly> {
        self.syntax.children_with_tokens().filter_map(LCurly::cast_element).next()
    }
    pub fn use_trees(&self) -> impl Iterator<Item = UseTree> + Clone {
        self.syntax.children().filter_map(UseTree::cast)
    }
    pub fn r_curly(&self) -> Option<RCurly> {
        self.syntax.children_with_tokens().filter_map(RCurly::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternCrateItem {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ExternCrateItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ExternCrateItem {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            EXTERN_CRATE_ITEM => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ExternCrateItem {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            EXTERN_CRATE_ITEM => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::AttrsOwner for ExternCrateItem {}
impl ast::VisibilityOwner for ExternCrateItem {}
impl ExternCrateItem {
    pub fn extern_kw(&self) -> Option<ExternKw> {
        self.syntax.children_with_tokens().filter_map(ExternKw::cast_element).next()
    }
    pub fn crate_kw(&self) -> Option<CrateKw> {
        self.syntax.children_with_tokens().filter_map(CrateKw::cast_element).next()
    }
    pub fn name_ref(&self) -> Option<NameRef> {
        self.syntax.children().filter_map(NameRef::cast).next()
    }
    pub fn alias(&self) -> Option<Alias> {
        self.syntax.children().filter_map(Alias::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArgList {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ArgList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ArgList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ARG_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ArgList {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ARG_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ArgList {
    pub fn l_paren(&self) -> Option<LParen> {
        self.syntax.children_with_tokens().filter_map(LParen::cast_element).next()
    }
    pub fn args(&self) -> impl Iterator<Item = Expr> + Clone {
        self.syntax.children().filter_map(Expr::cast)
    }
    pub fn r_paren(&self) -> Option<RParen> {
        self.syntax.children_with_tokens().filter_map(RParen::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for Path {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PATH => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for Path {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PATH => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl Path {
    pub fn segment(&self) -> Option<PathSegment> {
        self.syntax.children().filter_map(PathSegment::cast).next()
    }
    pub fn qualifier(&self) -> Option<Path> {
        self.syntax.children().filter_map(Path::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathSegment {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for PathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for PathSegment {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            PATH_SEGMENT => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for PathSegment {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            PATH_SEGMENT => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl PathSegment {
    pub fn coloncolon(&self) -> Option<Coloncolon> {
        self.syntax.children_with_tokens().filter_map(Coloncolon::cast_element).next()
    }
    pub fn l_angle(&self) -> Option<LAngle> {
        self.syntax.children_with_tokens().filter_map(LAngle::cast_element).next()
    }
    pub fn name_ref(&self) -> Option<NameRef> {
        self.syntax.children().filter_map(NameRef::cast).next()
    }
    pub fn type_arg_list(&self) -> Option<TypeArgList> {
        self.syntax.children().filter_map(TypeArgList::cast).next()
    }
    pub fn param_list(&self) -> Option<ParamList> {
        self.syntax.children().filter_map(ParamList::cast).next()
    }
    pub fn ret_type(&self) -> Option<RetType> {
        self.syntax.children().filter_map(RetType::cast).next()
    }
    pub fn path_type(&self) -> Option<PathType> {
        self.syntax.children().filter_map(PathType::cast).next()
    }
    pub fn r_angle(&self) -> Option<RAngle> {
        self.syntax.children_with_tokens().filter_map(RAngle::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeArgList {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for TypeArgList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for TypeArgList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_ARG_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for TypeArgList {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_ARG_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl TypeArgList {
    pub fn coloncolon(&self) -> Option<Coloncolon> {
        self.syntax.children_with_tokens().filter_map(Coloncolon::cast_element).next()
    }
    pub fn l_angle(&self) -> Option<LAngle> {
        self.syntax.children_with_tokens().filter_map(LAngle::cast_element).next()
    }
    pub fn generic_args(&self) -> impl Iterator<Item = GenericArg> + Clone {
        self.syntax.children().filter_map(GenericArg::cast)
    }
    pub fn type_args(&self) -> impl Iterator<Item = TypeArg> + Clone {
        self.syntax.children().filter_map(TypeArg::cast)
    }
    pub fn lifetime_args(&self) -> impl Iterator<Item = LifetimeArg> + Clone {
        self.syntax.children().filter_map(LifetimeArg::cast)
    }
    pub fn assoc_type_args(&self) -> impl Iterator<Item = AssocTypeArg> + Clone {
        self.syntax.children().filter_map(AssocTypeArg::cast)
    }
    pub fn const_args(&self) -> impl Iterator<Item = ConstArg> + Clone {
        self.syntax.children().filter_map(ConstArg::cast)
    }
    pub fn r_angle(&self) -> Option<RAngle> {
        self.syntax.children_with_tokens().filter_map(RAngle::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeArg {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for TypeArg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for TypeArg {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_ARG => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for TypeArg {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            TYPE_ARG => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl TypeArg {
    pub fn type_ref(&self) -> Option<TypeRef> {
        self.syntax.children().filter_map(TypeRef::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssocTypeArg {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for AssocTypeArg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for AssocTypeArg {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ASSOC_TYPE_ARG => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for AssocTypeArg {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ASSOC_TYPE_ARG => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::TypeBoundsOwner for AssocTypeArg {}
impl AssocTypeArg {
    pub fn name_ref(&self) -> Option<NameRef> {
        self.syntax.children().filter_map(NameRef::cast).next()
    }
    pub fn eq(&self) -> Option<Eq> {
        self.syntax.children_with_tokens().filter_map(Eq::cast_element).next()
    }
    pub fn type_ref(&self) -> Option<TypeRef> {
        self.syntax.children().filter_map(TypeRef::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LifetimeArg {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for LifetimeArg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for LifetimeArg {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LIFETIME_ARG => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for LifetimeArg {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            LIFETIME_ARG => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl LifetimeArg {
    pub fn lifetime(&self) -> Option<Lifetime> {
        self.syntax.children_with_tokens().filter_map(Lifetime::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConstArg {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ConstArg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ConstArg {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CONST_ARG => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ConstArg {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            CONST_ARG => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ConstArg {
    pub fn literal(&self) -> Option<Literal> {
        self.syntax.children().filter_map(Literal::cast).next()
    }
    pub fn eq(&self) -> Option<Eq> {
        self.syntax.children_with_tokens().filter_map(Eq::cast_element).next()
    }
    pub fn block_expr(&self) -> Option<BlockExpr> {
        self.syntax.children().filter_map(BlockExpr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MacroItems {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for MacroItems {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for MacroItems {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MACRO_ITEMS => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for MacroItems {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            MACRO_ITEMS => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::ModuleItemOwner for MacroItems {}
impl ast::FnDefOwner for MacroItems {}
impl MacroItems {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MacroStmts {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for MacroStmts {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for MacroStmts {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MACRO_STMTS => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for MacroStmts {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            MACRO_STMTS => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl MacroStmts {
    pub fn statements(&self) -> impl Iterator<Item = Stmt> + Clone {
        self.syntax.children().filter_map(Stmt::cast)
    }
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.children().filter_map(Expr::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternItemList {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ExternItemList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ExternItemList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            EXTERN_ITEM_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ExternItemList {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            EXTERN_ITEM_LIST => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ast::FnDefOwner for ExternItemList {}
impl ast::ModuleItemOwner for ExternItemList {}
impl ExternItemList {
    pub fn l_curly(&self) -> Option<LCurly> {
        self.syntax.children_with_tokens().filter_map(LCurly::cast_element).next()
    }
    pub fn extern_items(&self) -> impl Iterator<Item = ExternItem> + Clone {
        self.syntax.children().filter_map(ExternItem::cast)
    }
    pub fn r_curly(&self) -> Option<RCurly> {
        self.syntax.children_with_tokens().filter_map(RCurly::cast_element).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternBlock {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for ExternBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for ExternBlock {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            EXTERN_BLOCK => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for ExternBlock {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            EXTERN_BLOCK => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl ExternBlock {
    pub fn abi(&self) -> Option<Abi> {
        self.syntax.children().filter_map(Abi::cast).next()
    }
    pub fn extern_item_list(&self) -> Option<ExternItemList> {
        self.syntax.children().filter_map(ExternItemList::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetaItem {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for MetaItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for MetaItem {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            META_ITEM => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for MetaItem {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            META_ITEM => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl MetaItem {
    pub fn path(&self) -> Option<Path> {
        self.syntax.children().filter_map(Path::cast).next()
    }
    pub fn eq(&self) -> Option<Eq> {
        self.syntax.children_with_tokens().filter_map(Eq::cast_element).next()
    }
    pub fn attr_input(&self) -> Option<AttrInput> {
        self.syntax.children().filter_map(AttrInput::cast).next()
    }
    pub fn nested_meta_items(&self) -> impl Iterator<Item = MetaItem> + Clone {
        self.syntax.children().filter_map(MetaItem::cast)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MacroDef {
    pub(crate) syntax: SyntaxNode,
}
impl std::fmt::Display for MacroDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl AstNode for MacroDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            MACRO_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        if Self::can_cast(syntax.kind()) {
            Ok(Self { syntax })
        } else {
            Err(syntax)
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl AstElement for MacroDef {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            MACRO_DEF => true,
            _ => false,
        }
    }
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        if Self::can_cast_element(syntax.kind()) {
            Ok(Self { syntax: syntax.into_node().unwrap() })
        } else {
            Err(syntax)
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        NodeOrToken::Node(&self.syntax)
    }
    fn into_syntax_element(self) -> SyntaxElement {
        NodeOrToken::Node(self.syntax)
    }
}
impl MacroDef {
    pub fn name(&self) -> Option<Name> {
        self.syntax.children().filter_map(Name::cast).next()
    }
    pub fn token_tree(&self) -> Option<TokenTree> {
        self.syntax.children().filter_map(TokenTree::cast).next()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NominalDef {
    StructDef(StructDef),
    EnumDef(EnumDef),
    UnionDef(UnionDef),
}
impl From<StructDef> for NominalDef {
    fn from(node: StructDef) -> NominalDef {
        NominalDef::StructDef(node)
    }
}
impl From<EnumDef> for NominalDef {
    fn from(node: EnumDef) -> NominalDef {
        NominalDef::EnumDef(node)
    }
}
impl From<UnionDef> for NominalDef {
    fn from(node: UnionDef) -> NominalDef {
        NominalDef::UnionDef(node)
    }
}
impl std::fmt::Display for NominalDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NominalDef::StructDef(it) => std::fmt::Display::fmt(it, f),
            NominalDef::EnumDef(it) => std::fmt::Display::fmt(it, f),
            NominalDef::UnionDef(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstNode for NominalDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ENUM_DEF | STRUCT_DEF | UNION_DEF => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        match syntax.kind() {
            STRUCT_DEF => StructDef::cast_or_return(syntax).map(|x| NominalDef::StructDef(x)),
            ENUM_DEF => EnumDef::cast_or_return(syntax).map(|x| NominalDef::EnumDef(x)),
            UNION_DEF => UnionDef::cast_or_return(syntax).map(|x| NominalDef::UnionDef(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            NominalDef::StructDef(it) => it.syntax(),
            NominalDef::EnumDef(it) => it.syntax(),
            NominalDef::UnionDef(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            NominalDef::StructDef(it) => it.into_syntax(),
            NominalDef::EnumDef(it) => it.into_syntax(),
            NominalDef::UnionDef(it) => it.into_syntax(),
        }
    }
}
impl AstElement for NominalDef {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ENUM_DEF | STRUCT_DEF | UNION_DEF => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            STRUCT_DEF => {
                StructDef::cast_or_return_element(syntax).map(|x| NominalDef::StructDef(x))
            }
            ENUM_DEF => EnumDef::cast_or_return_element(syntax).map(|x| NominalDef::EnumDef(x)),
            UNION_DEF => UnionDef::cast_or_return_element(syntax).map(|x| NominalDef::UnionDef(x)),
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            NominalDef::StructDef(it) => it.syntax_element(),
            NominalDef::EnumDef(it) => it.syntax_element(),
            NominalDef::UnionDef(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            NominalDef::StructDef(it) => it.into_syntax_element(),
            NominalDef::EnumDef(it) => it.into_syntax_element(),
            NominalDef::UnionDef(it) => it.into_syntax_element(),
        }
    }
}
impl ast::NameOwner for NominalDef {}
impl ast::TypeParamsOwner for NominalDef {}
impl ast::AttrsOwner for NominalDef {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericParam {
    LifetimeParam(LifetimeParam),
    TypeParam(TypeParam),
    ConstParam(ConstParam),
}
impl From<LifetimeParam> for GenericParam {
    fn from(node: LifetimeParam) -> GenericParam {
        GenericParam::LifetimeParam(node)
    }
}
impl From<TypeParam> for GenericParam {
    fn from(node: TypeParam) -> GenericParam {
        GenericParam::TypeParam(node)
    }
}
impl From<ConstParam> for GenericParam {
    fn from(node: ConstParam) -> GenericParam {
        GenericParam::ConstParam(node)
    }
}
impl std::fmt::Display for GenericParam {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GenericParam::LifetimeParam(it) => std::fmt::Display::fmt(it, f),
            GenericParam::TypeParam(it) => std::fmt::Display::fmt(it, f),
            GenericParam::ConstParam(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstNode for GenericParam {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CONST_PARAM | LIFETIME_PARAM | TYPE_PARAM => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        match syntax.kind() {
            LIFETIME_PARAM => {
                LifetimeParam::cast_or_return(syntax).map(|x| GenericParam::LifetimeParam(x))
            }
            TYPE_PARAM => TypeParam::cast_or_return(syntax).map(|x| GenericParam::TypeParam(x)),
            CONST_PARAM => ConstParam::cast_or_return(syntax).map(|x| GenericParam::ConstParam(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            GenericParam::LifetimeParam(it) => it.syntax(),
            GenericParam::TypeParam(it) => it.syntax(),
            GenericParam::ConstParam(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            GenericParam::LifetimeParam(it) => it.into_syntax(),
            GenericParam::TypeParam(it) => it.into_syntax(),
            GenericParam::ConstParam(it) => it.into_syntax(),
        }
    }
}
impl AstElement for GenericParam {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            CONST_PARAM | LIFETIME_PARAM | TYPE_PARAM => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            LIFETIME_PARAM => LifetimeParam::cast_or_return_element(syntax)
                .map(|x| GenericParam::LifetimeParam(x)),
            TYPE_PARAM => {
                TypeParam::cast_or_return_element(syntax).map(|x| GenericParam::TypeParam(x))
            }
            CONST_PARAM => {
                ConstParam::cast_or_return_element(syntax).map(|x| GenericParam::ConstParam(x))
            }
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            GenericParam::LifetimeParam(it) => it.syntax_element(),
            GenericParam::TypeParam(it) => it.syntax_element(),
            GenericParam::ConstParam(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            GenericParam::LifetimeParam(it) => it.into_syntax_element(),
            GenericParam::TypeParam(it) => it.into_syntax_element(),
            GenericParam::ConstParam(it) => it.into_syntax_element(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericArg {
    LifetimeArg(LifetimeArg),
    TypeArg(TypeArg),
    ConstArg(ConstArg),
    AssocTypeArg(AssocTypeArg),
}
impl From<LifetimeArg> for GenericArg {
    fn from(node: LifetimeArg) -> GenericArg {
        GenericArg::LifetimeArg(node)
    }
}
impl From<TypeArg> for GenericArg {
    fn from(node: TypeArg) -> GenericArg {
        GenericArg::TypeArg(node)
    }
}
impl From<ConstArg> for GenericArg {
    fn from(node: ConstArg) -> GenericArg {
        GenericArg::ConstArg(node)
    }
}
impl From<AssocTypeArg> for GenericArg {
    fn from(node: AssocTypeArg) -> GenericArg {
        GenericArg::AssocTypeArg(node)
    }
}
impl std::fmt::Display for GenericArg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GenericArg::LifetimeArg(it) => std::fmt::Display::fmt(it, f),
            GenericArg::TypeArg(it) => std::fmt::Display::fmt(it, f),
            GenericArg::ConstArg(it) => std::fmt::Display::fmt(it, f),
            GenericArg::AssocTypeArg(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstNode for GenericArg {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ASSOC_TYPE_ARG | CONST_ARG | LIFETIME_ARG | TYPE_ARG => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        match syntax.kind() {
            LIFETIME_ARG => LifetimeArg::cast_or_return(syntax).map(|x| GenericArg::LifetimeArg(x)),
            TYPE_ARG => TypeArg::cast_or_return(syntax).map(|x| GenericArg::TypeArg(x)),
            CONST_ARG => ConstArg::cast_or_return(syntax).map(|x| GenericArg::ConstArg(x)),
            ASSOC_TYPE_ARG => {
                AssocTypeArg::cast_or_return(syntax).map(|x| GenericArg::AssocTypeArg(x))
            }
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            GenericArg::LifetimeArg(it) => it.syntax(),
            GenericArg::TypeArg(it) => it.syntax(),
            GenericArg::ConstArg(it) => it.syntax(),
            GenericArg::AssocTypeArg(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            GenericArg::LifetimeArg(it) => it.into_syntax(),
            GenericArg::TypeArg(it) => it.into_syntax(),
            GenericArg::ConstArg(it) => it.into_syntax(),
            GenericArg::AssocTypeArg(it) => it.into_syntax(),
        }
    }
}
impl AstElement for GenericArg {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ASSOC_TYPE_ARG | CONST_ARG | LIFETIME_ARG | TYPE_ARG => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            LIFETIME_ARG => {
                LifetimeArg::cast_or_return_element(syntax).map(|x| GenericArg::LifetimeArg(x))
            }
            TYPE_ARG => TypeArg::cast_or_return_element(syntax).map(|x| GenericArg::TypeArg(x)),
            CONST_ARG => ConstArg::cast_or_return_element(syntax).map(|x| GenericArg::ConstArg(x)),
            ASSOC_TYPE_ARG => {
                AssocTypeArg::cast_or_return_element(syntax).map(|x| GenericArg::AssocTypeArg(x))
            }
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            GenericArg::LifetimeArg(it) => it.syntax_element(),
            GenericArg::TypeArg(it) => it.syntax_element(),
            GenericArg::ConstArg(it) => it.syntax_element(),
            GenericArg::AssocTypeArg(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            GenericArg::LifetimeArg(it) => it.into_syntax_element(),
            GenericArg::TypeArg(it) => it.into_syntax_element(),
            GenericArg::ConstArg(it) => it.into_syntax_element(),
            GenericArg::AssocTypeArg(it) => it.into_syntax_element(),
        }
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
impl std::fmt::Display for TypeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TypeRef::ParenType(it) => std::fmt::Display::fmt(it, f),
            TypeRef::TupleType(it) => std::fmt::Display::fmt(it, f),
            TypeRef::NeverType(it) => std::fmt::Display::fmt(it, f),
            TypeRef::PathType(it) => std::fmt::Display::fmt(it, f),
            TypeRef::PointerType(it) => std::fmt::Display::fmt(it, f),
            TypeRef::ArrayType(it) => std::fmt::Display::fmt(it, f),
            TypeRef::SliceType(it) => std::fmt::Display::fmt(it, f),
            TypeRef::ReferenceType(it) => std::fmt::Display::fmt(it, f),
            TypeRef::PlaceholderType(it) => std::fmt::Display::fmt(it, f),
            TypeRef::FnPointerType(it) => std::fmt::Display::fmt(it, f),
            TypeRef::ForType(it) => std::fmt::Display::fmt(it, f),
            TypeRef::ImplTraitType(it) => std::fmt::Display::fmt(it, f),
            TypeRef::DynTraitType(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstNode for TypeRef {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ARRAY_TYPE | DYN_TRAIT_TYPE | FN_POINTER_TYPE | FOR_TYPE | IMPL_TRAIT_TYPE
            | NEVER_TYPE | PAREN_TYPE | PATH_TYPE | PLACEHOLDER_TYPE | POINTER_TYPE
            | REFERENCE_TYPE | SLICE_TYPE | TUPLE_TYPE => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        match syntax.kind() {
            PAREN_TYPE => ParenType::cast_or_return(syntax).map(|x| TypeRef::ParenType(x)),
            TUPLE_TYPE => TupleType::cast_or_return(syntax).map(|x| TypeRef::TupleType(x)),
            NEVER_TYPE => NeverType::cast_or_return(syntax).map(|x| TypeRef::NeverType(x)),
            PATH_TYPE => PathType::cast_or_return(syntax).map(|x| TypeRef::PathType(x)),
            POINTER_TYPE => PointerType::cast_or_return(syntax).map(|x| TypeRef::PointerType(x)),
            ARRAY_TYPE => ArrayType::cast_or_return(syntax).map(|x| TypeRef::ArrayType(x)),
            SLICE_TYPE => SliceType::cast_or_return(syntax).map(|x| TypeRef::SliceType(x)),
            REFERENCE_TYPE => {
                ReferenceType::cast_or_return(syntax).map(|x| TypeRef::ReferenceType(x))
            }
            PLACEHOLDER_TYPE => {
                PlaceholderType::cast_or_return(syntax).map(|x| TypeRef::PlaceholderType(x))
            }
            FN_POINTER_TYPE => {
                FnPointerType::cast_or_return(syntax).map(|x| TypeRef::FnPointerType(x))
            }
            FOR_TYPE => ForType::cast_or_return(syntax).map(|x| TypeRef::ForType(x)),
            IMPL_TRAIT_TYPE => {
                ImplTraitType::cast_or_return(syntax).map(|x| TypeRef::ImplTraitType(x))
            }
            DYN_TRAIT_TYPE => {
                DynTraitType::cast_or_return(syntax).map(|x| TypeRef::DynTraitType(x))
            }
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            TypeRef::ParenType(it) => it.syntax(),
            TypeRef::TupleType(it) => it.syntax(),
            TypeRef::NeverType(it) => it.syntax(),
            TypeRef::PathType(it) => it.syntax(),
            TypeRef::PointerType(it) => it.syntax(),
            TypeRef::ArrayType(it) => it.syntax(),
            TypeRef::SliceType(it) => it.syntax(),
            TypeRef::ReferenceType(it) => it.syntax(),
            TypeRef::PlaceholderType(it) => it.syntax(),
            TypeRef::FnPointerType(it) => it.syntax(),
            TypeRef::ForType(it) => it.syntax(),
            TypeRef::ImplTraitType(it) => it.syntax(),
            TypeRef::DynTraitType(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            TypeRef::ParenType(it) => it.into_syntax(),
            TypeRef::TupleType(it) => it.into_syntax(),
            TypeRef::NeverType(it) => it.into_syntax(),
            TypeRef::PathType(it) => it.into_syntax(),
            TypeRef::PointerType(it) => it.into_syntax(),
            TypeRef::ArrayType(it) => it.into_syntax(),
            TypeRef::SliceType(it) => it.into_syntax(),
            TypeRef::ReferenceType(it) => it.into_syntax(),
            TypeRef::PlaceholderType(it) => it.into_syntax(),
            TypeRef::FnPointerType(it) => it.into_syntax(),
            TypeRef::ForType(it) => it.into_syntax(),
            TypeRef::ImplTraitType(it) => it.into_syntax(),
            TypeRef::DynTraitType(it) => it.into_syntax(),
        }
    }
}
impl AstElement for TypeRef {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ARRAY_TYPE | DYN_TRAIT_TYPE | FN_POINTER_TYPE | FOR_TYPE | IMPL_TRAIT_TYPE
            | NEVER_TYPE | PAREN_TYPE | PATH_TYPE | PLACEHOLDER_TYPE | POINTER_TYPE
            | REFERENCE_TYPE | SLICE_TYPE | TUPLE_TYPE => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            PAREN_TYPE => ParenType::cast_or_return_element(syntax).map(|x| TypeRef::ParenType(x)),
            TUPLE_TYPE => TupleType::cast_or_return_element(syntax).map(|x| TypeRef::TupleType(x)),
            NEVER_TYPE => NeverType::cast_or_return_element(syntax).map(|x| TypeRef::NeverType(x)),
            PATH_TYPE => PathType::cast_or_return_element(syntax).map(|x| TypeRef::PathType(x)),
            POINTER_TYPE => {
                PointerType::cast_or_return_element(syntax).map(|x| TypeRef::PointerType(x))
            }
            ARRAY_TYPE => ArrayType::cast_or_return_element(syntax).map(|x| TypeRef::ArrayType(x)),
            SLICE_TYPE => SliceType::cast_or_return_element(syntax).map(|x| TypeRef::SliceType(x)),
            REFERENCE_TYPE => {
                ReferenceType::cast_or_return_element(syntax).map(|x| TypeRef::ReferenceType(x))
            }
            PLACEHOLDER_TYPE => {
                PlaceholderType::cast_or_return_element(syntax).map(|x| TypeRef::PlaceholderType(x))
            }
            FN_POINTER_TYPE => {
                FnPointerType::cast_or_return_element(syntax).map(|x| TypeRef::FnPointerType(x))
            }
            FOR_TYPE => ForType::cast_or_return_element(syntax).map(|x| TypeRef::ForType(x)),
            IMPL_TRAIT_TYPE => {
                ImplTraitType::cast_or_return_element(syntax).map(|x| TypeRef::ImplTraitType(x))
            }
            DYN_TRAIT_TYPE => {
                DynTraitType::cast_or_return_element(syntax).map(|x| TypeRef::DynTraitType(x))
            }
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            TypeRef::ParenType(it) => it.syntax_element(),
            TypeRef::TupleType(it) => it.syntax_element(),
            TypeRef::NeverType(it) => it.syntax_element(),
            TypeRef::PathType(it) => it.syntax_element(),
            TypeRef::PointerType(it) => it.syntax_element(),
            TypeRef::ArrayType(it) => it.syntax_element(),
            TypeRef::SliceType(it) => it.syntax_element(),
            TypeRef::ReferenceType(it) => it.syntax_element(),
            TypeRef::PlaceholderType(it) => it.syntax_element(),
            TypeRef::FnPointerType(it) => it.syntax_element(),
            TypeRef::ForType(it) => it.syntax_element(),
            TypeRef::ImplTraitType(it) => it.syntax_element(),
            TypeRef::DynTraitType(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            TypeRef::ParenType(it) => it.into_syntax_element(),
            TypeRef::TupleType(it) => it.into_syntax_element(),
            TypeRef::NeverType(it) => it.into_syntax_element(),
            TypeRef::PathType(it) => it.into_syntax_element(),
            TypeRef::PointerType(it) => it.into_syntax_element(),
            TypeRef::ArrayType(it) => it.into_syntax_element(),
            TypeRef::SliceType(it) => it.into_syntax_element(),
            TypeRef::ReferenceType(it) => it.into_syntax_element(),
            TypeRef::PlaceholderType(it) => it.into_syntax_element(),
            TypeRef::FnPointerType(it) => it.into_syntax_element(),
            TypeRef::ForType(it) => it.into_syntax_element(),
            TypeRef::ImplTraitType(it) => it.into_syntax_element(),
            TypeRef::DynTraitType(it) => it.into_syntax_element(),
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
    ImplDef(ImplDef),
    UseItem(UseItem),
    ExternCrateItem(ExternCrateItem),
    ConstDef(ConstDef),
    StaticDef(StaticDef),
    Module(Module),
    MacroCall(MacroCall),
    ExternBlock(ExternBlock),
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
impl From<ImplDef> for ModuleItem {
    fn from(node: ImplDef) -> ModuleItem {
        ModuleItem::ImplDef(node)
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
impl From<MacroCall> for ModuleItem {
    fn from(node: MacroCall) -> ModuleItem {
        ModuleItem::MacroCall(node)
    }
}
impl From<ExternBlock> for ModuleItem {
    fn from(node: ExternBlock) -> ModuleItem {
        ModuleItem::ExternBlock(node)
    }
}
impl std::fmt::Display for ModuleItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ModuleItem::StructDef(it) => std::fmt::Display::fmt(it, f),
            ModuleItem::UnionDef(it) => std::fmt::Display::fmt(it, f),
            ModuleItem::EnumDef(it) => std::fmt::Display::fmt(it, f),
            ModuleItem::FnDef(it) => std::fmt::Display::fmt(it, f),
            ModuleItem::TraitDef(it) => std::fmt::Display::fmt(it, f),
            ModuleItem::TypeAliasDef(it) => std::fmt::Display::fmt(it, f),
            ModuleItem::ImplDef(it) => std::fmt::Display::fmt(it, f),
            ModuleItem::UseItem(it) => std::fmt::Display::fmt(it, f),
            ModuleItem::ExternCrateItem(it) => std::fmt::Display::fmt(it, f),
            ModuleItem::ConstDef(it) => std::fmt::Display::fmt(it, f),
            ModuleItem::StaticDef(it) => std::fmt::Display::fmt(it, f),
            ModuleItem::Module(it) => std::fmt::Display::fmt(it, f),
            ModuleItem::MacroCall(it) => std::fmt::Display::fmt(it, f),
            ModuleItem::ExternBlock(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstNode for ModuleItem {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CONST_DEF | ENUM_DEF | EXTERN_BLOCK | EXTERN_CRATE_ITEM | FN_DEF | IMPL_DEF
            | MACRO_CALL | MODULE | STATIC_DEF | STRUCT_DEF | TRAIT_DEF | TYPE_ALIAS_DEF
            | UNION_DEF | USE_ITEM => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        match syntax.kind() {
            STRUCT_DEF => StructDef::cast_or_return(syntax).map(|x| ModuleItem::StructDef(x)),
            UNION_DEF => UnionDef::cast_or_return(syntax).map(|x| ModuleItem::UnionDef(x)),
            ENUM_DEF => EnumDef::cast_or_return(syntax).map(|x| ModuleItem::EnumDef(x)),
            FN_DEF => FnDef::cast_or_return(syntax).map(|x| ModuleItem::FnDef(x)),
            TRAIT_DEF => TraitDef::cast_or_return(syntax).map(|x| ModuleItem::TraitDef(x)),
            TYPE_ALIAS_DEF => {
                TypeAliasDef::cast_or_return(syntax).map(|x| ModuleItem::TypeAliasDef(x))
            }
            IMPL_DEF => ImplDef::cast_or_return(syntax).map(|x| ModuleItem::ImplDef(x)),
            USE_ITEM => UseItem::cast_or_return(syntax).map(|x| ModuleItem::UseItem(x)),
            EXTERN_CRATE_ITEM => {
                ExternCrateItem::cast_or_return(syntax).map(|x| ModuleItem::ExternCrateItem(x))
            }
            CONST_DEF => ConstDef::cast_or_return(syntax).map(|x| ModuleItem::ConstDef(x)),
            STATIC_DEF => StaticDef::cast_or_return(syntax).map(|x| ModuleItem::StaticDef(x)),
            MODULE => Module::cast_or_return(syntax).map(|x| ModuleItem::Module(x)),
            MACRO_CALL => MacroCall::cast_or_return(syntax).map(|x| ModuleItem::MacroCall(x)),
            EXTERN_BLOCK => ExternBlock::cast_or_return(syntax).map(|x| ModuleItem::ExternBlock(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            ModuleItem::StructDef(it) => it.syntax(),
            ModuleItem::UnionDef(it) => it.syntax(),
            ModuleItem::EnumDef(it) => it.syntax(),
            ModuleItem::FnDef(it) => it.syntax(),
            ModuleItem::TraitDef(it) => it.syntax(),
            ModuleItem::TypeAliasDef(it) => it.syntax(),
            ModuleItem::ImplDef(it) => it.syntax(),
            ModuleItem::UseItem(it) => it.syntax(),
            ModuleItem::ExternCrateItem(it) => it.syntax(),
            ModuleItem::ConstDef(it) => it.syntax(),
            ModuleItem::StaticDef(it) => it.syntax(),
            ModuleItem::Module(it) => it.syntax(),
            ModuleItem::MacroCall(it) => it.syntax(),
            ModuleItem::ExternBlock(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            ModuleItem::StructDef(it) => it.into_syntax(),
            ModuleItem::UnionDef(it) => it.into_syntax(),
            ModuleItem::EnumDef(it) => it.into_syntax(),
            ModuleItem::FnDef(it) => it.into_syntax(),
            ModuleItem::TraitDef(it) => it.into_syntax(),
            ModuleItem::TypeAliasDef(it) => it.into_syntax(),
            ModuleItem::ImplDef(it) => it.into_syntax(),
            ModuleItem::UseItem(it) => it.into_syntax(),
            ModuleItem::ExternCrateItem(it) => it.into_syntax(),
            ModuleItem::ConstDef(it) => it.into_syntax(),
            ModuleItem::StaticDef(it) => it.into_syntax(),
            ModuleItem::Module(it) => it.into_syntax(),
            ModuleItem::MacroCall(it) => it.into_syntax(),
            ModuleItem::ExternBlock(it) => it.into_syntax(),
        }
    }
}
impl AstElement for ModuleItem {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            CONST_DEF | ENUM_DEF | EXTERN_BLOCK | EXTERN_CRATE_ITEM | FN_DEF | IMPL_DEF
            | MACRO_CALL | MODULE | STATIC_DEF | STRUCT_DEF | TRAIT_DEF | TYPE_ALIAS_DEF
            | UNION_DEF | USE_ITEM => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            STRUCT_DEF => {
                StructDef::cast_or_return_element(syntax).map(|x| ModuleItem::StructDef(x))
            }
            UNION_DEF => UnionDef::cast_or_return_element(syntax).map(|x| ModuleItem::UnionDef(x)),
            ENUM_DEF => EnumDef::cast_or_return_element(syntax).map(|x| ModuleItem::EnumDef(x)),
            FN_DEF => FnDef::cast_or_return_element(syntax).map(|x| ModuleItem::FnDef(x)),
            TRAIT_DEF => TraitDef::cast_or_return_element(syntax).map(|x| ModuleItem::TraitDef(x)),
            TYPE_ALIAS_DEF => {
                TypeAliasDef::cast_or_return_element(syntax).map(|x| ModuleItem::TypeAliasDef(x))
            }
            IMPL_DEF => ImplDef::cast_or_return_element(syntax).map(|x| ModuleItem::ImplDef(x)),
            USE_ITEM => UseItem::cast_or_return_element(syntax).map(|x| ModuleItem::UseItem(x)),
            EXTERN_CRATE_ITEM => ExternCrateItem::cast_or_return_element(syntax)
                .map(|x| ModuleItem::ExternCrateItem(x)),
            CONST_DEF => ConstDef::cast_or_return_element(syntax).map(|x| ModuleItem::ConstDef(x)),
            STATIC_DEF => {
                StaticDef::cast_or_return_element(syntax).map(|x| ModuleItem::StaticDef(x))
            }
            MODULE => Module::cast_or_return_element(syntax).map(|x| ModuleItem::Module(x)),
            MACRO_CALL => {
                MacroCall::cast_or_return_element(syntax).map(|x| ModuleItem::MacroCall(x))
            }
            EXTERN_BLOCK => {
                ExternBlock::cast_or_return_element(syntax).map(|x| ModuleItem::ExternBlock(x))
            }
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            ModuleItem::StructDef(it) => it.syntax_element(),
            ModuleItem::UnionDef(it) => it.syntax_element(),
            ModuleItem::EnumDef(it) => it.syntax_element(),
            ModuleItem::FnDef(it) => it.syntax_element(),
            ModuleItem::TraitDef(it) => it.syntax_element(),
            ModuleItem::TypeAliasDef(it) => it.syntax_element(),
            ModuleItem::ImplDef(it) => it.syntax_element(),
            ModuleItem::UseItem(it) => it.syntax_element(),
            ModuleItem::ExternCrateItem(it) => it.syntax_element(),
            ModuleItem::ConstDef(it) => it.syntax_element(),
            ModuleItem::StaticDef(it) => it.syntax_element(),
            ModuleItem::Module(it) => it.syntax_element(),
            ModuleItem::MacroCall(it) => it.syntax_element(),
            ModuleItem::ExternBlock(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            ModuleItem::StructDef(it) => it.into_syntax_element(),
            ModuleItem::UnionDef(it) => it.into_syntax_element(),
            ModuleItem::EnumDef(it) => it.into_syntax_element(),
            ModuleItem::FnDef(it) => it.into_syntax_element(),
            ModuleItem::TraitDef(it) => it.into_syntax_element(),
            ModuleItem::TypeAliasDef(it) => it.into_syntax_element(),
            ModuleItem::ImplDef(it) => it.into_syntax_element(),
            ModuleItem::UseItem(it) => it.into_syntax_element(),
            ModuleItem::ExternCrateItem(it) => it.into_syntax_element(),
            ModuleItem::ConstDef(it) => it.into_syntax_element(),
            ModuleItem::StaticDef(it) => it.into_syntax_element(),
            ModuleItem::Module(it) => it.into_syntax_element(),
            ModuleItem::MacroCall(it) => it.into_syntax_element(),
            ModuleItem::ExternBlock(it) => it.into_syntax_element(),
        }
    }
}
impl ast::NameOwner for ModuleItem {}
impl ast::AttrsOwner for ModuleItem {}
impl ast::VisibilityOwner for ModuleItem {}
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
impl std::fmt::Display for ImplItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ImplItem::FnDef(it) => std::fmt::Display::fmt(it, f),
            ImplItem::TypeAliasDef(it) => std::fmt::Display::fmt(it, f),
            ImplItem::ConstDef(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstNode for ImplItem {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CONST_DEF | FN_DEF | TYPE_ALIAS_DEF => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        match syntax.kind() {
            FN_DEF => FnDef::cast_or_return(syntax).map(|x| ImplItem::FnDef(x)),
            TYPE_ALIAS_DEF => {
                TypeAliasDef::cast_or_return(syntax).map(|x| ImplItem::TypeAliasDef(x))
            }
            CONST_DEF => ConstDef::cast_or_return(syntax).map(|x| ImplItem::ConstDef(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            ImplItem::FnDef(it) => it.syntax(),
            ImplItem::TypeAliasDef(it) => it.syntax(),
            ImplItem::ConstDef(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            ImplItem::FnDef(it) => it.into_syntax(),
            ImplItem::TypeAliasDef(it) => it.into_syntax(),
            ImplItem::ConstDef(it) => it.into_syntax(),
        }
    }
}
impl AstElement for ImplItem {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            CONST_DEF | FN_DEF | TYPE_ALIAS_DEF => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            FN_DEF => FnDef::cast_or_return_element(syntax).map(|x| ImplItem::FnDef(x)),
            TYPE_ALIAS_DEF => {
                TypeAliasDef::cast_or_return_element(syntax).map(|x| ImplItem::TypeAliasDef(x))
            }
            CONST_DEF => ConstDef::cast_or_return_element(syntax).map(|x| ImplItem::ConstDef(x)),
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            ImplItem::FnDef(it) => it.syntax_element(),
            ImplItem::TypeAliasDef(it) => it.syntax_element(),
            ImplItem::ConstDef(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            ImplItem::FnDef(it) => it.into_syntax_element(),
            ImplItem::TypeAliasDef(it) => it.into_syntax_element(),
            ImplItem::ConstDef(it) => it.into_syntax_element(),
        }
    }
}
impl ast::NameOwner for ImplItem {}
impl ast::AttrsOwner for ImplItem {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExternItem {
    FnDef(FnDef),
    StaticDef(StaticDef),
}
impl From<FnDef> for ExternItem {
    fn from(node: FnDef) -> ExternItem {
        ExternItem::FnDef(node)
    }
}
impl From<StaticDef> for ExternItem {
    fn from(node: StaticDef) -> ExternItem {
        ExternItem::StaticDef(node)
    }
}
impl std::fmt::Display for ExternItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExternItem::FnDef(it) => std::fmt::Display::fmt(it, f),
            ExternItem::StaticDef(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstNode for ExternItem {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            FN_DEF | STATIC_DEF => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        match syntax.kind() {
            FN_DEF => FnDef::cast_or_return(syntax).map(|x| ExternItem::FnDef(x)),
            STATIC_DEF => StaticDef::cast_or_return(syntax).map(|x| ExternItem::StaticDef(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            ExternItem::FnDef(it) => it.syntax(),
            ExternItem::StaticDef(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            ExternItem::FnDef(it) => it.into_syntax(),
            ExternItem::StaticDef(it) => it.into_syntax(),
        }
    }
}
impl AstElement for ExternItem {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            FN_DEF | STATIC_DEF => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            FN_DEF => FnDef::cast_or_return_element(syntax).map(|x| ExternItem::FnDef(x)),
            STATIC_DEF => {
                StaticDef::cast_or_return_element(syntax).map(|x| ExternItem::StaticDef(x))
            }
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            ExternItem::FnDef(it) => it.syntax_element(),
            ExternItem::StaticDef(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            ExternItem::FnDef(it) => it.into_syntax_element(),
            ExternItem::StaticDef(it) => it.into_syntax_element(),
        }
    }
}
impl ast::NameOwner for ExternItem {}
impl ast::AttrsOwner for ExternItem {}
impl ast::VisibilityOwner for ExternItem {}
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
    RangeExpr(RangeExpr),
    BinExpr(BinExpr),
    Literal(Literal),
    MacroCall(MacroCall),
    BoxExpr(BoxExpr),
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
impl From<BoxExpr> for Expr {
    fn from(node: BoxExpr) -> Expr {
        Expr::BoxExpr(node)
    }
}
impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::TupleExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::ArrayExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::ParenExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::PathExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::LambdaExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::IfExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::LoopExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::ForExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::WhileExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::ContinueExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::BreakExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::Label(it) => std::fmt::Display::fmt(it, f),
            Expr::BlockExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::ReturnExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::MatchExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::RecordLit(it) => std::fmt::Display::fmt(it, f),
            Expr::CallExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::IndexExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::MethodCallExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::FieldExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::AwaitExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::TryExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::TryBlockExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::CastExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::RefExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::PrefixExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::RangeExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::BinExpr(it) => std::fmt::Display::fmt(it, f),
            Expr::Literal(it) => std::fmt::Display::fmt(it, f),
            Expr::MacroCall(it) => std::fmt::Display::fmt(it, f),
            Expr::BoxExpr(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstNode for Expr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ARRAY_EXPR | AWAIT_EXPR | BIN_EXPR | BLOCK_EXPR | BOX_EXPR | BREAK_EXPR | CALL_EXPR
            | CAST_EXPR | CONTINUE_EXPR | FIELD_EXPR | FOR_EXPR | IF_EXPR | INDEX_EXPR | LABEL
            | LAMBDA_EXPR | LITERAL | LOOP_EXPR | MACRO_CALL | MATCH_EXPR | METHOD_CALL_EXPR
            | PAREN_EXPR | PATH_EXPR | PREFIX_EXPR | RANGE_EXPR | RECORD_LIT | REF_EXPR
            | RETURN_EXPR | TRY_BLOCK_EXPR | TRY_EXPR | TUPLE_EXPR | WHILE_EXPR => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        match syntax.kind() {
            TUPLE_EXPR => TupleExpr::cast_or_return(syntax).map(|x| Expr::TupleExpr(x)),
            ARRAY_EXPR => ArrayExpr::cast_or_return(syntax).map(|x| Expr::ArrayExpr(x)),
            PAREN_EXPR => ParenExpr::cast_or_return(syntax).map(|x| Expr::ParenExpr(x)),
            PATH_EXPR => PathExpr::cast_or_return(syntax).map(|x| Expr::PathExpr(x)),
            LAMBDA_EXPR => LambdaExpr::cast_or_return(syntax).map(|x| Expr::LambdaExpr(x)),
            IF_EXPR => IfExpr::cast_or_return(syntax).map(|x| Expr::IfExpr(x)),
            LOOP_EXPR => LoopExpr::cast_or_return(syntax).map(|x| Expr::LoopExpr(x)),
            FOR_EXPR => ForExpr::cast_or_return(syntax).map(|x| Expr::ForExpr(x)),
            WHILE_EXPR => WhileExpr::cast_or_return(syntax).map(|x| Expr::WhileExpr(x)),
            CONTINUE_EXPR => ContinueExpr::cast_or_return(syntax).map(|x| Expr::ContinueExpr(x)),
            BREAK_EXPR => BreakExpr::cast_or_return(syntax).map(|x| Expr::BreakExpr(x)),
            LABEL => Label::cast_or_return(syntax).map(|x| Expr::Label(x)),
            BLOCK_EXPR => BlockExpr::cast_or_return(syntax).map(|x| Expr::BlockExpr(x)),
            RETURN_EXPR => ReturnExpr::cast_or_return(syntax).map(|x| Expr::ReturnExpr(x)),
            MATCH_EXPR => MatchExpr::cast_or_return(syntax).map(|x| Expr::MatchExpr(x)),
            RECORD_LIT => RecordLit::cast_or_return(syntax).map(|x| Expr::RecordLit(x)),
            CALL_EXPR => CallExpr::cast_or_return(syntax).map(|x| Expr::CallExpr(x)),
            INDEX_EXPR => IndexExpr::cast_or_return(syntax).map(|x| Expr::IndexExpr(x)),
            METHOD_CALL_EXPR => {
                MethodCallExpr::cast_or_return(syntax).map(|x| Expr::MethodCallExpr(x))
            }
            FIELD_EXPR => FieldExpr::cast_or_return(syntax).map(|x| Expr::FieldExpr(x)),
            AWAIT_EXPR => AwaitExpr::cast_or_return(syntax).map(|x| Expr::AwaitExpr(x)),
            TRY_EXPR => TryExpr::cast_or_return(syntax).map(|x| Expr::TryExpr(x)),
            TRY_BLOCK_EXPR => TryBlockExpr::cast_or_return(syntax).map(|x| Expr::TryBlockExpr(x)),
            CAST_EXPR => CastExpr::cast_or_return(syntax).map(|x| Expr::CastExpr(x)),
            REF_EXPR => RefExpr::cast_or_return(syntax).map(|x| Expr::RefExpr(x)),
            PREFIX_EXPR => PrefixExpr::cast_or_return(syntax).map(|x| Expr::PrefixExpr(x)),
            RANGE_EXPR => RangeExpr::cast_or_return(syntax).map(|x| Expr::RangeExpr(x)),
            BIN_EXPR => BinExpr::cast_or_return(syntax).map(|x| Expr::BinExpr(x)),
            LITERAL => Literal::cast_or_return(syntax).map(|x| Expr::Literal(x)),
            MACRO_CALL => MacroCall::cast_or_return(syntax).map(|x| Expr::MacroCall(x)),
            BOX_EXPR => BoxExpr::cast_or_return(syntax).map(|x| Expr::BoxExpr(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Expr::TupleExpr(it) => it.syntax(),
            Expr::ArrayExpr(it) => it.syntax(),
            Expr::ParenExpr(it) => it.syntax(),
            Expr::PathExpr(it) => it.syntax(),
            Expr::LambdaExpr(it) => it.syntax(),
            Expr::IfExpr(it) => it.syntax(),
            Expr::LoopExpr(it) => it.syntax(),
            Expr::ForExpr(it) => it.syntax(),
            Expr::WhileExpr(it) => it.syntax(),
            Expr::ContinueExpr(it) => it.syntax(),
            Expr::BreakExpr(it) => it.syntax(),
            Expr::Label(it) => it.syntax(),
            Expr::BlockExpr(it) => it.syntax(),
            Expr::ReturnExpr(it) => it.syntax(),
            Expr::MatchExpr(it) => it.syntax(),
            Expr::RecordLit(it) => it.syntax(),
            Expr::CallExpr(it) => it.syntax(),
            Expr::IndexExpr(it) => it.syntax(),
            Expr::MethodCallExpr(it) => it.syntax(),
            Expr::FieldExpr(it) => it.syntax(),
            Expr::AwaitExpr(it) => it.syntax(),
            Expr::TryExpr(it) => it.syntax(),
            Expr::TryBlockExpr(it) => it.syntax(),
            Expr::CastExpr(it) => it.syntax(),
            Expr::RefExpr(it) => it.syntax(),
            Expr::PrefixExpr(it) => it.syntax(),
            Expr::RangeExpr(it) => it.syntax(),
            Expr::BinExpr(it) => it.syntax(),
            Expr::Literal(it) => it.syntax(),
            Expr::MacroCall(it) => it.syntax(),
            Expr::BoxExpr(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            Expr::TupleExpr(it) => it.into_syntax(),
            Expr::ArrayExpr(it) => it.into_syntax(),
            Expr::ParenExpr(it) => it.into_syntax(),
            Expr::PathExpr(it) => it.into_syntax(),
            Expr::LambdaExpr(it) => it.into_syntax(),
            Expr::IfExpr(it) => it.into_syntax(),
            Expr::LoopExpr(it) => it.into_syntax(),
            Expr::ForExpr(it) => it.into_syntax(),
            Expr::WhileExpr(it) => it.into_syntax(),
            Expr::ContinueExpr(it) => it.into_syntax(),
            Expr::BreakExpr(it) => it.into_syntax(),
            Expr::Label(it) => it.into_syntax(),
            Expr::BlockExpr(it) => it.into_syntax(),
            Expr::ReturnExpr(it) => it.into_syntax(),
            Expr::MatchExpr(it) => it.into_syntax(),
            Expr::RecordLit(it) => it.into_syntax(),
            Expr::CallExpr(it) => it.into_syntax(),
            Expr::IndexExpr(it) => it.into_syntax(),
            Expr::MethodCallExpr(it) => it.into_syntax(),
            Expr::FieldExpr(it) => it.into_syntax(),
            Expr::AwaitExpr(it) => it.into_syntax(),
            Expr::TryExpr(it) => it.into_syntax(),
            Expr::TryBlockExpr(it) => it.into_syntax(),
            Expr::CastExpr(it) => it.into_syntax(),
            Expr::RefExpr(it) => it.into_syntax(),
            Expr::PrefixExpr(it) => it.into_syntax(),
            Expr::RangeExpr(it) => it.into_syntax(),
            Expr::BinExpr(it) => it.into_syntax(),
            Expr::Literal(it) => it.into_syntax(),
            Expr::MacroCall(it) => it.into_syntax(),
            Expr::BoxExpr(it) => it.into_syntax(),
        }
    }
}
impl AstElement for Expr {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ARRAY_EXPR | AWAIT_EXPR | BIN_EXPR | BLOCK_EXPR | BOX_EXPR | BREAK_EXPR | CALL_EXPR
            | CAST_EXPR | CONTINUE_EXPR | FIELD_EXPR | FOR_EXPR | IF_EXPR | INDEX_EXPR | LABEL
            | LAMBDA_EXPR | LITERAL | LOOP_EXPR | MACRO_CALL | MATCH_EXPR | METHOD_CALL_EXPR
            | PAREN_EXPR | PATH_EXPR | PREFIX_EXPR | RANGE_EXPR | RECORD_LIT | REF_EXPR
            | RETURN_EXPR | TRY_BLOCK_EXPR | TRY_EXPR | TUPLE_EXPR | WHILE_EXPR => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            TUPLE_EXPR => TupleExpr::cast_or_return_element(syntax).map(|x| Expr::TupleExpr(x)),
            ARRAY_EXPR => ArrayExpr::cast_or_return_element(syntax).map(|x| Expr::ArrayExpr(x)),
            PAREN_EXPR => ParenExpr::cast_or_return_element(syntax).map(|x| Expr::ParenExpr(x)),
            PATH_EXPR => PathExpr::cast_or_return_element(syntax).map(|x| Expr::PathExpr(x)),
            LAMBDA_EXPR => LambdaExpr::cast_or_return_element(syntax).map(|x| Expr::LambdaExpr(x)),
            IF_EXPR => IfExpr::cast_or_return_element(syntax).map(|x| Expr::IfExpr(x)),
            LOOP_EXPR => LoopExpr::cast_or_return_element(syntax).map(|x| Expr::LoopExpr(x)),
            FOR_EXPR => ForExpr::cast_or_return_element(syntax).map(|x| Expr::ForExpr(x)),
            WHILE_EXPR => WhileExpr::cast_or_return_element(syntax).map(|x| Expr::WhileExpr(x)),
            CONTINUE_EXPR => {
                ContinueExpr::cast_or_return_element(syntax).map(|x| Expr::ContinueExpr(x))
            }
            BREAK_EXPR => BreakExpr::cast_or_return_element(syntax).map(|x| Expr::BreakExpr(x)),
            LABEL => Label::cast_or_return_element(syntax).map(|x| Expr::Label(x)),
            BLOCK_EXPR => BlockExpr::cast_or_return_element(syntax).map(|x| Expr::BlockExpr(x)),
            RETURN_EXPR => ReturnExpr::cast_or_return_element(syntax).map(|x| Expr::ReturnExpr(x)),
            MATCH_EXPR => MatchExpr::cast_or_return_element(syntax).map(|x| Expr::MatchExpr(x)),
            RECORD_LIT => RecordLit::cast_or_return_element(syntax).map(|x| Expr::RecordLit(x)),
            CALL_EXPR => CallExpr::cast_or_return_element(syntax).map(|x| Expr::CallExpr(x)),
            INDEX_EXPR => IndexExpr::cast_or_return_element(syntax).map(|x| Expr::IndexExpr(x)),
            METHOD_CALL_EXPR => {
                MethodCallExpr::cast_or_return_element(syntax).map(|x| Expr::MethodCallExpr(x))
            }
            FIELD_EXPR => FieldExpr::cast_or_return_element(syntax).map(|x| Expr::FieldExpr(x)),
            AWAIT_EXPR => AwaitExpr::cast_or_return_element(syntax).map(|x| Expr::AwaitExpr(x)),
            TRY_EXPR => TryExpr::cast_or_return_element(syntax).map(|x| Expr::TryExpr(x)),
            TRY_BLOCK_EXPR => {
                TryBlockExpr::cast_or_return_element(syntax).map(|x| Expr::TryBlockExpr(x))
            }
            CAST_EXPR => CastExpr::cast_or_return_element(syntax).map(|x| Expr::CastExpr(x)),
            REF_EXPR => RefExpr::cast_or_return_element(syntax).map(|x| Expr::RefExpr(x)),
            PREFIX_EXPR => PrefixExpr::cast_or_return_element(syntax).map(|x| Expr::PrefixExpr(x)),
            RANGE_EXPR => RangeExpr::cast_or_return_element(syntax).map(|x| Expr::RangeExpr(x)),
            BIN_EXPR => BinExpr::cast_or_return_element(syntax).map(|x| Expr::BinExpr(x)),
            LITERAL => Literal::cast_or_return_element(syntax).map(|x| Expr::Literal(x)),
            MACRO_CALL => MacroCall::cast_or_return_element(syntax).map(|x| Expr::MacroCall(x)),
            BOX_EXPR => BoxExpr::cast_or_return_element(syntax).map(|x| Expr::BoxExpr(x)),
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            Expr::TupleExpr(it) => it.syntax_element(),
            Expr::ArrayExpr(it) => it.syntax_element(),
            Expr::ParenExpr(it) => it.syntax_element(),
            Expr::PathExpr(it) => it.syntax_element(),
            Expr::LambdaExpr(it) => it.syntax_element(),
            Expr::IfExpr(it) => it.syntax_element(),
            Expr::LoopExpr(it) => it.syntax_element(),
            Expr::ForExpr(it) => it.syntax_element(),
            Expr::WhileExpr(it) => it.syntax_element(),
            Expr::ContinueExpr(it) => it.syntax_element(),
            Expr::BreakExpr(it) => it.syntax_element(),
            Expr::Label(it) => it.syntax_element(),
            Expr::BlockExpr(it) => it.syntax_element(),
            Expr::ReturnExpr(it) => it.syntax_element(),
            Expr::MatchExpr(it) => it.syntax_element(),
            Expr::RecordLit(it) => it.syntax_element(),
            Expr::CallExpr(it) => it.syntax_element(),
            Expr::IndexExpr(it) => it.syntax_element(),
            Expr::MethodCallExpr(it) => it.syntax_element(),
            Expr::FieldExpr(it) => it.syntax_element(),
            Expr::AwaitExpr(it) => it.syntax_element(),
            Expr::TryExpr(it) => it.syntax_element(),
            Expr::TryBlockExpr(it) => it.syntax_element(),
            Expr::CastExpr(it) => it.syntax_element(),
            Expr::RefExpr(it) => it.syntax_element(),
            Expr::PrefixExpr(it) => it.syntax_element(),
            Expr::RangeExpr(it) => it.syntax_element(),
            Expr::BinExpr(it) => it.syntax_element(),
            Expr::Literal(it) => it.syntax_element(),
            Expr::MacroCall(it) => it.syntax_element(),
            Expr::BoxExpr(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            Expr::TupleExpr(it) => it.into_syntax_element(),
            Expr::ArrayExpr(it) => it.into_syntax_element(),
            Expr::ParenExpr(it) => it.into_syntax_element(),
            Expr::PathExpr(it) => it.into_syntax_element(),
            Expr::LambdaExpr(it) => it.into_syntax_element(),
            Expr::IfExpr(it) => it.into_syntax_element(),
            Expr::LoopExpr(it) => it.into_syntax_element(),
            Expr::ForExpr(it) => it.into_syntax_element(),
            Expr::WhileExpr(it) => it.into_syntax_element(),
            Expr::ContinueExpr(it) => it.into_syntax_element(),
            Expr::BreakExpr(it) => it.into_syntax_element(),
            Expr::Label(it) => it.into_syntax_element(),
            Expr::BlockExpr(it) => it.into_syntax_element(),
            Expr::ReturnExpr(it) => it.into_syntax_element(),
            Expr::MatchExpr(it) => it.into_syntax_element(),
            Expr::RecordLit(it) => it.into_syntax_element(),
            Expr::CallExpr(it) => it.into_syntax_element(),
            Expr::IndexExpr(it) => it.into_syntax_element(),
            Expr::MethodCallExpr(it) => it.into_syntax_element(),
            Expr::FieldExpr(it) => it.into_syntax_element(),
            Expr::AwaitExpr(it) => it.into_syntax_element(),
            Expr::TryExpr(it) => it.into_syntax_element(),
            Expr::TryBlockExpr(it) => it.into_syntax_element(),
            Expr::CastExpr(it) => it.into_syntax_element(),
            Expr::RefExpr(it) => it.into_syntax_element(),
            Expr::PrefixExpr(it) => it.into_syntax_element(),
            Expr::RangeExpr(it) => it.into_syntax_element(),
            Expr::BinExpr(it) => it.into_syntax_element(),
            Expr::Literal(it) => it.into_syntax_element(),
            Expr::MacroCall(it) => it.into_syntax_element(),
            Expr::BoxExpr(it) => it.into_syntax_element(),
        }
    }
}
impl ast::AttrsOwner for Expr {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pat {
    OrPat(OrPat),
    ParenPat(ParenPat),
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
impl From<OrPat> for Pat {
    fn from(node: OrPat) -> Pat {
        Pat::OrPat(node)
    }
}
impl From<ParenPat> for Pat {
    fn from(node: ParenPat) -> Pat {
        Pat::ParenPat(node)
    }
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
impl std::fmt::Display for Pat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Pat::OrPat(it) => std::fmt::Display::fmt(it, f),
            Pat::ParenPat(it) => std::fmt::Display::fmt(it, f),
            Pat::RefPat(it) => std::fmt::Display::fmt(it, f),
            Pat::BoxPat(it) => std::fmt::Display::fmt(it, f),
            Pat::BindPat(it) => std::fmt::Display::fmt(it, f),
            Pat::PlaceholderPat(it) => std::fmt::Display::fmt(it, f),
            Pat::DotDotPat(it) => std::fmt::Display::fmt(it, f),
            Pat::PathPat(it) => std::fmt::Display::fmt(it, f),
            Pat::RecordPat(it) => std::fmt::Display::fmt(it, f),
            Pat::TupleStructPat(it) => std::fmt::Display::fmt(it, f),
            Pat::TuplePat(it) => std::fmt::Display::fmt(it, f),
            Pat::SlicePat(it) => std::fmt::Display::fmt(it, f),
            Pat::RangePat(it) => std::fmt::Display::fmt(it, f),
            Pat::LiteralPat(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstNode for Pat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BIND_PAT | BOX_PAT | DOT_DOT_PAT | LITERAL_PAT | OR_PAT | PAREN_PAT | PATH_PAT
            | PLACEHOLDER_PAT | RANGE_PAT | RECORD_PAT | REF_PAT | SLICE_PAT | TUPLE_PAT
            | TUPLE_STRUCT_PAT => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        match syntax.kind() {
            OR_PAT => OrPat::cast_or_return(syntax).map(|x| Pat::OrPat(x)),
            PAREN_PAT => ParenPat::cast_or_return(syntax).map(|x| Pat::ParenPat(x)),
            REF_PAT => RefPat::cast_or_return(syntax).map(|x| Pat::RefPat(x)),
            BOX_PAT => BoxPat::cast_or_return(syntax).map(|x| Pat::BoxPat(x)),
            BIND_PAT => BindPat::cast_or_return(syntax).map(|x| Pat::BindPat(x)),
            PLACEHOLDER_PAT => {
                PlaceholderPat::cast_or_return(syntax).map(|x| Pat::PlaceholderPat(x))
            }
            DOT_DOT_PAT => DotDotPat::cast_or_return(syntax).map(|x| Pat::DotDotPat(x)),
            PATH_PAT => PathPat::cast_or_return(syntax).map(|x| Pat::PathPat(x)),
            RECORD_PAT => RecordPat::cast_or_return(syntax).map(|x| Pat::RecordPat(x)),
            TUPLE_STRUCT_PAT => {
                TupleStructPat::cast_or_return(syntax).map(|x| Pat::TupleStructPat(x))
            }
            TUPLE_PAT => TuplePat::cast_or_return(syntax).map(|x| Pat::TuplePat(x)),
            SLICE_PAT => SlicePat::cast_or_return(syntax).map(|x| Pat::SlicePat(x)),
            RANGE_PAT => RangePat::cast_or_return(syntax).map(|x| Pat::RangePat(x)),
            LITERAL_PAT => LiteralPat::cast_or_return(syntax).map(|x| Pat::LiteralPat(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Pat::OrPat(it) => it.syntax(),
            Pat::ParenPat(it) => it.syntax(),
            Pat::RefPat(it) => it.syntax(),
            Pat::BoxPat(it) => it.syntax(),
            Pat::BindPat(it) => it.syntax(),
            Pat::PlaceholderPat(it) => it.syntax(),
            Pat::DotDotPat(it) => it.syntax(),
            Pat::PathPat(it) => it.syntax(),
            Pat::RecordPat(it) => it.syntax(),
            Pat::TupleStructPat(it) => it.syntax(),
            Pat::TuplePat(it) => it.syntax(),
            Pat::SlicePat(it) => it.syntax(),
            Pat::RangePat(it) => it.syntax(),
            Pat::LiteralPat(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            Pat::OrPat(it) => it.into_syntax(),
            Pat::ParenPat(it) => it.into_syntax(),
            Pat::RefPat(it) => it.into_syntax(),
            Pat::BoxPat(it) => it.into_syntax(),
            Pat::BindPat(it) => it.into_syntax(),
            Pat::PlaceholderPat(it) => it.into_syntax(),
            Pat::DotDotPat(it) => it.into_syntax(),
            Pat::PathPat(it) => it.into_syntax(),
            Pat::RecordPat(it) => it.into_syntax(),
            Pat::TupleStructPat(it) => it.into_syntax(),
            Pat::TuplePat(it) => it.into_syntax(),
            Pat::SlicePat(it) => it.into_syntax(),
            Pat::RangePat(it) => it.into_syntax(),
            Pat::LiteralPat(it) => it.into_syntax(),
        }
    }
}
impl AstElement for Pat {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            BIND_PAT | BOX_PAT | DOT_DOT_PAT | LITERAL_PAT | OR_PAT | PAREN_PAT | PATH_PAT
            | PLACEHOLDER_PAT | RANGE_PAT | RECORD_PAT | REF_PAT | SLICE_PAT | TUPLE_PAT
            | TUPLE_STRUCT_PAT => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            OR_PAT => OrPat::cast_or_return_element(syntax).map(|x| Pat::OrPat(x)),
            PAREN_PAT => ParenPat::cast_or_return_element(syntax).map(|x| Pat::ParenPat(x)),
            REF_PAT => RefPat::cast_or_return_element(syntax).map(|x| Pat::RefPat(x)),
            BOX_PAT => BoxPat::cast_or_return_element(syntax).map(|x| Pat::BoxPat(x)),
            BIND_PAT => BindPat::cast_or_return_element(syntax).map(|x| Pat::BindPat(x)),
            PLACEHOLDER_PAT => {
                PlaceholderPat::cast_or_return_element(syntax).map(|x| Pat::PlaceholderPat(x))
            }
            DOT_DOT_PAT => DotDotPat::cast_or_return_element(syntax).map(|x| Pat::DotDotPat(x)),
            PATH_PAT => PathPat::cast_or_return_element(syntax).map(|x| Pat::PathPat(x)),
            RECORD_PAT => RecordPat::cast_or_return_element(syntax).map(|x| Pat::RecordPat(x)),
            TUPLE_STRUCT_PAT => {
                TupleStructPat::cast_or_return_element(syntax).map(|x| Pat::TupleStructPat(x))
            }
            TUPLE_PAT => TuplePat::cast_or_return_element(syntax).map(|x| Pat::TuplePat(x)),
            SLICE_PAT => SlicePat::cast_or_return_element(syntax).map(|x| Pat::SlicePat(x)),
            RANGE_PAT => RangePat::cast_or_return_element(syntax).map(|x| Pat::RangePat(x)),
            LITERAL_PAT => LiteralPat::cast_or_return_element(syntax).map(|x| Pat::LiteralPat(x)),
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            Pat::OrPat(it) => it.syntax_element(),
            Pat::ParenPat(it) => it.syntax_element(),
            Pat::RefPat(it) => it.syntax_element(),
            Pat::BoxPat(it) => it.syntax_element(),
            Pat::BindPat(it) => it.syntax_element(),
            Pat::PlaceholderPat(it) => it.syntax_element(),
            Pat::DotDotPat(it) => it.syntax_element(),
            Pat::PathPat(it) => it.syntax_element(),
            Pat::RecordPat(it) => it.syntax_element(),
            Pat::TupleStructPat(it) => it.syntax_element(),
            Pat::TuplePat(it) => it.syntax_element(),
            Pat::SlicePat(it) => it.syntax_element(),
            Pat::RangePat(it) => it.syntax_element(),
            Pat::LiteralPat(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            Pat::OrPat(it) => it.into_syntax_element(),
            Pat::ParenPat(it) => it.into_syntax_element(),
            Pat::RefPat(it) => it.into_syntax_element(),
            Pat::BoxPat(it) => it.into_syntax_element(),
            Pat::BindPat(it) => it.into_syntax_element(),
            Pat::PlaceholderPat(it) => it.into_syntax_element(),
            Pat::DotDotPat(it) => it.into_syntax_element(),
            Pat::PathPat(it) => it.into_syntax_element(),
            Pat::RecordPat(it) => it.into_syntax_element(),
            Pat::TupleStructPat(it) => it.into_syntax_element(),
            Pat::TuplePat(it) => it.into_syntax_element(),
            Pat::SlicePat(it) => it.into_syntax_element(),
            Pat::RangePat(it) => it.into_syntax_element(),
            Pat::LiteralPat(it) => it.into_syntax_element(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RecordInnerPat {
    RecordFieldPat(RecordFieldPat),
    BindPat(BindPat),
}
impl From<RecordFieldPat> for RecordInnerPat {
    fn from(node: RecordFieldPat) -> RecordInnerPat {
        RecordInnerPat::RecordFieldPat(node)
    }
}
impl From<BindPat> for RecordInnerPat {
    fn from(node: BindPat) -> RecordInnerPat {
        RecordInnerPat::BindPat(node)
    }
}
impl std::fmt::Display for RecordInnerPat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RecordInnerPat::RecordFieldPat(it) => std::fmt::Display::fmt(it, f),
            RecordInnerPat::BindPat(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstNode for RecordInnerPat {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BIND_PAT | RECORD_FIELD_PAT => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        match syntax.kind() {
            RECORD_FIELD_PAT => {
                RecordFieldPat::cast_or_return(syntax).map(|x| RecordInnerPat::RecordFieldPat(x))
            }
            BIND_PAT => BindPat::cast_or_return(syntax).map(|x| RecordInnerPat::BindPat(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            RecordInnerPat::RecordFieldPat(it) => it.syntax(),
            RecordInnerPat::BindPat(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            RecordInnerPat::RecordFieldPat(it) => it.into_syntax(),
            RecordInnerPat::BindPat(it) => it.into_syntax(),
        }
    }
}
impl AstElement for RecordInnerPat {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            BIND_PAT | RECORD_FIELD_PAT => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            RECORD_FIELD_PAT => RecordFieldPat::cast_or_return_element(syntax)
                .map(|x| RecordInnerPat::RecordFieldPat(x)),
            BIND_PAT => BindPat::cast_or_return_element(syntax).map(|x| RecordInnerPat::BindPat(x)),
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            RecordInnerPat::RecordFieldPat(it) => it.syntax_element(),
            RecordInnerPat::BindPat(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            RecordInnerPat::RecordFieldPat(it) => it.into_syntax_element(),
            RecordInnerPat::BindPat(it) => it.into_syntax_element(),
        }
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
impl std::fmt::Display for AttrInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AttrInput::Literal(it) => std::fmt::Display::fmt(it, f),
            AttrInput::TokenTree(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstNode for AttrInput {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LITERAL | TOKEN_TREE => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        match syntax.kind() {
            LITERAL => Literal::cast_or_return(syntax).map(|x| AttrInput::Literal(x)),
            TOKEN_TREE => TokenTree::cast_or_return(syntax).map(|x| AttrInput::TokenTree(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            AttrInput::Literal(it) => it.syntax(),
            AttrInput::TokenTree(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            AttrInput::Literal(it) => it.into_syntax(),
            AttrInput::TokenTree(it) => it.into_syntax(),
        }
    }
}
impl AstElement for AttrInput {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            LITERAL | TOKEN_TREE => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            LITERAL => Literal::cast_or_return_element(syntax).map(|x| AttrInput::Literal(x)),
            TOKEN_TREE => {
                TokenTree::cast_or_return_element(syntax).map(|x| AttrInput::TokenTree(x))
            }
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            AttrInput::Literal(it) => it.syntax_element(),
            AttrInput::TokenTree(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            AttrInput::Literal(it) => it.into_syntax_element(),
            AttrInput::TokenTree(it) => it.into_syntax_element(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stmt {
    ModuleItem(ModuleItem),
    LetStmt(LetStmt),
    ExprStmt(ExprStmt),
}
impl From<ModuleItem> for Stmt {
    fn from(node: ModuleItem) -> Stmt {
        Stmt::ModuleItem(node)
    }
}
impl From<LetStmt> for Stmt {
    fn from(node: LetStmt) -> Stmt {
        Stmt::LetStmt(node)
    }
}
impl From<ExprStmt> for Stmt {
    fn from(node: ExprStmt) -> Stmt {
        Stmt::ExprStmt(node)
    }
}
impl std::fmt::Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Stmt::ModuleItem(it) => std::fmt::Display::fmt(it, f),
            Stmt::LetStmt(it) => std::fmt::Display::fmt(it, f),
            Stmt::ExprStmt(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstNode for Stmt {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            CONST_DEF | ENUM_DEF | EXPR_STMT | EXTERN_BLOCK | EXTERN_CRATE_ITEM | FN_DEF
            | IMPL_DEF | LET_STMT | MACRO_CALL | MODULE | STATIC_DEF | STRUCT_DEF | TRAIT_DEF
            | TYPE_ALIAS_DEF | UNION_DEF | USE_ITEM => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        match syntax.kind() {
            CONST_DEF | ENUM_DEF | EXTERN_BLOCK | EXTERN_CRATE_ITEM | FN_DEF | IMPL_DEF
            | MACRO_CALL | MODULE | STATIC_DEF | STRUCT_DEF | TRAIT_DEF | TYPE_ALIAS_DEF
            | UNION_DEF | USE_ITEM => {
                ModuleItem::cast_or_return(syntax).map(|x| Stmt::ModuleItem(x))
            }
            LET_STMT => LetStmt::cast_or_return(syntax).map(|x| Stmt::LetStmt(x)),
            EXPR_STMT => ExprStmt::cast_or_return(syntax).map(|x| Stmt::ExprStmt(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Stmt::ModuleItem(it) => it.syntax(),
            Stmt::LetStmt(it) => it.syntax(),
            Stmt::ExprStmt(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            Stmt::ModuleItem(it) => it.into_syntax(),
            Stmt::LetStmt(it) => it.into_syntax(),
            Stmt::ExprStmt(it) => it.into_syntax(),
        }
    }
}
impl AstElement for Stmt {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            CONST_DEF | ENUM_DEF | EXPR_STMT | EXTERN_BLOCK | EXTERN_CRATE_ITEM | FN_DEF
            | IMPL_DEF | LET_STMT | MACRO_CALL | MODULE | STATIC_DEF | STRUCT_DEF | TRAIT_DEF
            | TYPE_ALIAS_DEF | UNION_DEF | USE_ITEM => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            CONST_DEF | ENUM_DEF | EXTERN_BLOCK | EXTERN_CRATE_ITEM | FN_DEF | IMPL_DEF
            | MACRO_CALL | MODULE | STATIC_DEF | STRUCT_DEF | TRAIT_DEF | TYPE_ALIAS_DEF
            | UNION_DEF | USE_ITEM => {
                ModuleItem::cast_or_return_element(syntax).map(|x| Stmt::ModuleItem(x))
            }
            LET_STMT => LetStmt::cast_or_return_element(syntax).map(|x| Stmt::LetStmt(x)),
            EXPR_STMT => ExprStmt::cast_or_return_element(syntax).map(|x| Stmt::ExprStmt(x)),
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            Stmt::ModuleItem(it) => it.syntax_element(),
            Stmt::LetStmt(it) => it.syntax_element(),
            Stmt::ExprStmt(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            Stmt::ModuleItem(it) => it.into_syntax_element(),
            Stmt::LetStmt(it) => it.into_syntax_element(),
            Stmt::ExprStmt(it) => it.into_syntax_element(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StmtOrSemi {
    Stmt(Stmt),
    Semi(Semi),
}
impl From<Stmt> for StmtOrSemi {
    fn from(node: Stmt) -> StmtOrSemi {
        StmtOrSemi::Stmt(node)
    }
}
impl From<Semi> for StmtOrSemi {
    fn from(node: Semi) -> StmtOrSemi {
        StmtOrSemi::Semi(node)
    }
}
impl std::fmt::Display for StmtOrSemi {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StmtOrSemi::Stmt(it) => std::fmt::Display::fmt(it, f),
            StmtOrSemi::Semi(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstElement for StmtOrSemi {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            CONST_DEF | ENUM_DEF | EXPR_STMT | EXTERN_BLOCK | EXTERN_CRATE_ITEM | FN_DEF
            | IMPL_DEF | LET_STMT | MACRO_CALL | MODULE | SEMI | STATIC_DEF | STRUCT_DEF
            | TRAIT_DEF | TYPE_ALIAS_DEF | UNION_DEF | USE_ITEM => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            CONST_DEF | ENUM_DEF | EXPR_STMT | EXTERN_BLOCK | EXTERN_CRATE_ITEM | FN_DEF
            | IMPL_DEF | LET_STMT | MACRO_CALL | MODULE | STATIC_DEF | STRUCT_DEF | TRAIT_DEF
            | TYPE_ALIAS_DEF | UNION_DEF | USE_ITEM => {
                Stmt::cast_or_return_element(syntax).map(|x| StmtOrSemi::Stmt(x))
            }
            SEMI => Semi::cast_or_return_element(syntax).map(|x| StmtOrSemi::Semi(x)),
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            StmtOrSemi::Stmt(it) => it.syntax_element(),
            StmtOrSemi::Semi(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            StmtOrSemi::Stmt(it) => it.into_syntax_element(),
            StmtOrSemi::Semi(it) => it.into_syntax_element(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LeftDelimiter {
    LParen(LParen),
    LBrack(LBrack),
    LCurly(LCurly),
}
impl From<LParen> for LeftDelimiter {
    fn from(node: LParen) -> LeftDelimiter {
        LeftDelimiter::LParen(node)
    }
}
impl From<LBrack> for LeftDelimiter {
    fn from(node: LBrack) -> LeftDelimiter {
        LeftDelimiter::LBrack(node)
    }
}
impl From<LCurly> for LeftDelimiter {
    fn from(node: LCurly) -> LeftDelimiter {
        LeftDelimiter::LCurly(node)
    }
}
impl std::fmt::Display for LeftDelimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LeftDelimiter::LParen(it) => std::fmt::Display::fmt(it, f),
            LeftDelimiter::LBrack(it) => std::fmt::Display::fmt(it, f),
            LeftDelimiter::LCurly(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstToken for LeftDelimiter {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            L_BRACK | L_CURLY | L_PAREN => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        match syntax.kind() {
            L_PAREN => LParen::cast_or_return(syntax).map(|x| LeftDelimiter::LParen(x)),
            L_BRACK => LBrack::cast_or_return(syntax).map(|x| LeftDelimiter::LBrack(x)),
            L_CURLY => LCurly::cast_or_return(syntax).map(|x| LeftDelimiter::LCurly(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        match self {
            LeftDelimiter::LParen(it) => it.syntax(),
            LeftDelimiter::LBrack(it) => it.syntax(),
            LeftDelimiter::LCurly(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxToken {
        match self {
            LeftDelimiter::LParen(it) => it.into_syntax(),
            LeftDelimiter::LBrack(it) => it.into_syntax(),
            LeftDelimiter::LCurly(it) => it.into_syntax(),
        }
    }
}
impl AstElement for LeftDelimiter {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            L_BRACK | L_CURLY | L_PAREN => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            L_PAREN => LParen::cast_or_return_element(syntax).map(|x| LeftDelimiter::LParen(x)),
            L_BRACK => LBrack::cast_or_return_element(syntax).map(|x| LeftDelimiter::LBrack(x)),
            L_CURLY => LCurly::cast_or_return_element(syntax).map(|x| LeftDelimiter::LCurly(x)),
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            LeftDelimiter::LParen(it) => it.syntax_element(),
            LeftDelimiter::LBrack(it) => it.syntax_element(),
            LeftDelimiter::LCurly(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            LeftDelimiter::LParen(it) => it.into_syntax_element(),
            LeftDelimiter::LBrack(it) => it.into_syntax_element(),
            LeftDelimiter::LCurly(it) => it.into_syntax_element(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RightDelimiter {
    RParen(RParen),
    RBrack(RBrack),
    RCurly(RCurly),
}
impl From<RParen> for RightDelimiter {
    fn from(node: RParen) -> RightDelimiter {
        RightDelimiter::RParen(node)
    }
}
impl From<RBrack> for RightDelimiter {
    fn from(node: RBrack) -> RightDelimiter {
        RightDelimiter::RBrack(node)
    }
}
impl From<RCurly> for RightDelimiter {
    fn from(node: RCurly) -> RightDelimiter {
        RightDelimiter::RCurly(node)
    }
}
impl std::fmt::Display for RightDelimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RightDelimiter::RParen(it) => std::fmt::Display::fmt(it, f),
            RightDelimiter::RBrack(it) => std::fmt::Display::fmt(it, f),
            RightDelimiter::RCurly(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstToken for RightDelimiter {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            R_BRACK | R_CURLY | R_PAREN => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        match syntax.kind() {
            R_PAREN => RParen::cast_or_return(syntax).map(|x| RightDelimiter::RParen(x)),
            R_BRACK => RBrack::cast_or_return(syntax).map(|x| RightDelimiter::RBrack(x)),
            R_CURLY => RCurly::cast_or_return(syntax).map(|x| RightDelimiter::RCurly(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        match self {
            RightDelimiter::RParen(it) => it.syntax(),
            RightDelimiter::RBrack(it) => it.syntax(),
            RightDelimiter::RCurly(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxToken {
        match self {
            RightDelimiter::RParen(it) => it.into_syntax(),
            RightDelimiter::RBrack(it) => it.into_syntax(),
            RightDelimiter::RCurly(it) => it.into_syntax(),
        }
    }
}
impl AstElement for RightDelimiter {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            R_BRACK | R_CURLY | R_PAREN => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            R_PAREN => RParen::cast_or_return_element(syntax).map(|x| RightDelimiter::RParen(x)),
            R_BRACK => RBrack::cast_or_return_element(syntax).map(|x| RightDelimiter::RBrack(x)),
            R_CURLY => RCurly::cast_or_return_element(syntax).map(|x| RightDelimiter::RCurly(x)),
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            RightDelimiter::RParen(it) => it.syntax_element(),
            RightDelimiter::RBrack(it) => it.syntax_element(),
            RightDelimiter::RCurly(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            RightDelimiter::RParen(it) => it.into_syntax_element(),
            RightDelimiter::RBrack(it) => it.into_syntax_element(),
            RightDelimiter::RCurly(it) => it.into_syntax_element(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RangeSeparator {
    Dotdot(Dotdot),
    Dotdotdot(Dotdotdot),
    Dotdoteq(Dotdoteq),
}
impl From<Dotdot> for RangeSeparator {
    fn from(node: Dotdot) -> RangeSeparator {
        RangeSeparator::Dotdot(node)
    }
}
impl From<Dotdotdot> for RangeSeparator {
    fn from(node: Dotdotdot) -> RangeSeparator {
        RangeSeparator::Dotdotdot(node)
    }
}
impl From<Dotdoteq> for RangeSeparator {
    fn from(node: Dotdoteq) -> RangeSeparator {
        RangeSeparator::Dotdoteq(node)
    }
}
impl std::fmt::Display for RangeSeparator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RangeSeparator::Dotdot(it) => std::fmt::Display::fmt(it, f),
            RangeSeparator::Dotdotdot(it) => std::fmt::Display::fmt(it, f),
            RangeSeparator::Dotdoteq(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstToken for RangeSeparator {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            DOTDOT | DOTDOTDOT | DOTDOTEQ => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        match syntax.kind() {
            DOTDOT => Dotdot::cast_or_return(syntax).map(|x| RangeSeparator::Dotdot(x)),
            DOTDOTDOT => Dotdotdot::cast_or_return(syntax).map(|x| RangeSeparator::Dotdotdot(x)),
            DOTDOTEQ => Dotdoteq::cast_or_return(syntax).map(|x| RangeSeparator::Dotdoteq(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        match self {
            RangeSeparator::Dotdot(it) => it.syntax(),
            RangeSeparator::Dotdotdot(it) => it.syntax(),
            RangeSeparator::Dotdoteq(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxToken {
        match self {
            RangeSeparator::Dotdot(it) => it.into_syntax(),
            RangeSeparator::Dotdotdot(it) => it.into_syntax(),
            RangeSeparator::Dotdoteq(it) => it.into_syntax(),
        }
    }
}
impl AstElement for RangeSeparator {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            DOTDOT | DOTDOTDOT | DOTDOTEQ => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            DOTDOT => Dotdot::cast_or_return_element(syntax).map(|x| RangeSeparator::Dotdot(x)),
            DOTDOTDOT => {
                Dotdotdot::cast_or_return_element(syntax).map(|x| RangeSeparator::Dotdotdot(x))
            }
            DOTDOTEQ => {
                Dotdoteq::cast_or_return_element(syntax).map(|x| RangeSeparator::Dotdoteq(x))
            }
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            RangeSeparator::Dotdot(it) => it.syntax_element(),
            RangeSeparator::Dotdotdot(it) => it.syntax_element(),
            RangeSeparator::Dotdoteq(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            RangeSeparator::Dotdot(it) => it.into_syntax_element(),
            RangeSeparator::Dotdotdot(it) => it.into_syntax_element(),
            RangeSeparator::Dotdoteq(it) => it.into_syntax_element(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinOp {
    Pipepipe(Pipepipe),
    Ampamp(Ampamp),
    Eqeq(Eqeq),
    Neq(Neq),
    Lteq(Lteq),
    Gteq(Gteq),
    LAngle(LAngle),
    RAngle(RAngle),
    Plus(Plus),
    Star(Star),
    Minus(Minus),
    Slash(Slash),
    Percent(Percent),
    Shl(Shl),
    Shr(Shr),
    Caret(Caret),
    Pipe(Pipe),
    Amp(Amp),
    Eq(Eq),
    Pluseq(Pluseq),
    Slasheq(Slasheq),
    Stareq(Stareq),
    Percenteq(Percenteq),
    Shreq(Shreq),
    Shleq(Shleq),
    Minuseq(Minuseq),
    Pipeeq(Pipeeq),
    Ampeq(Ampeq),
    Careteq(Careteq),
}
impl From<Pipepipe> for BinOp {
    fn from(node: Pipepipe) -> BinOp {
        BinOp::Pipepipe(node)
    }
}
impl From<Ampamp> for BinOp {
    fn from(node: Ampamp) -> BinOp {
        BinOp::Ampamp(node)
    }
}
impl From<Eqeq> for BinOp {
    fn from(node: Eqeq) -> BinOp {
        BinOp::Eqeq(node)
    }
}
impl From<Neq> for BinOp {
    fn from(node: Neq) -> BinOp {
        BinOp::Neq(node)
    }
}
impl From<Lteq> for BinOp {
    fn from(node: Lteq) -> BinOp {
        BinOp::Lteq(node)
    }
}
impl From<Gteq> for BinOp {
    fn from(node: Gteq) -> BinOp {
        BinOp::Gteq(node)
    }
}
impl From<LAngle> for BinOp {
    fn from(node: LAngle) -> BinOp {
        BinOp::LAngle(node)
    }
}
impl From<RAngle> for BinOp {
    fn from(node: RAngle) -> BinOp {
        BinOp::RAngle(node)
    }
}
impl From<Plus> for BinOp {
    fn from(node: Plus) -> BinOp {
        BinOp::Plus(node)
    }
}
impl From<Star> for BinOp {
    fn from(node: Star) -> BinOp {
        BinOp::Star(node)
    }
}
impl From<Minus> for BinOp {
    fn from(node: Minus) -> BinOp {
        BinOp::Minus(node)
    }
}
impl From<Slash> for BinOp {
    fn from(node: Slash) -> BinOp {
        BinOp::Slash(node)
    }
}
impl From<Percent> for BinOp {
    fn from(node: Percent) -> BinOp {
        BinOp::Percent(node)
    }
}
impl From<Shl> for BinOp {
    fn from(node: Shl) -> BinOp {
        BinOp::Shl(node)
    }
}
impl From<Shr> for BinOp {
    fn from(node: Shr) -> BinOp {
        BinOp::Shr(node)
    }
}
impl From<Caret> for BinOp {
    fn from(node: Caret) -> BinOp {
        BinOp::Caret(node)
    }
}
impl From<Pipe> for BinOp {
    fn from(node: Pipe) -> BinOp {
        BinOp::Pipe(node)
    }
}
impl From<Amp> for BinOp {
    fn from(node: Amp) -> BinOp {
        BinOp::Amp(node)
    }
}
impl From<Eq> for BinOp {
    fn from(node: Eq) -> BinOp {
        BinOp::Eq(node)
    }
}
impl From<Pluseq> for BinOp {
    fn from(node: Pluseq) -> BinOp {
        BinOp::Pluseq(node)
    }
}
impl From<Slasheq> for BinOp {
    fn from(node: Slasheq) -> BinOp {
        BinOp::Slasheq(node)
    }
}
impl From<Stareq> for BinOp {
    fn from(node: Stareq) -> BinOp {
        BinOp::Stareq(node)
    }
}
impl From<Percenteq> for BinOp {
    fn from(node: Percenteq) -> BinOp {
        BinOp::Percenteq(node)
    }
}
impl From<Shreq> for BinOp {
    fn from(node: Shreq) -> BinOp {
        BinOp::Shreq(node)
    }
}
impl From<Shleq> for BinOp {
    fn from(node: Shleq) -> BinOp {
        BinOp::Shleq(node)
    }
}
impl From<Minuseq> for BinOp {
    fn from(node: Minuseq) -> BinOp {
        BinOp::Minuseq(node)
    }
}
impl From<Pipeeq> for BinOp {
    fn from(node: Pipeeq) -> BinOp {
        BinOp::Pipeeq(node)
    }
}
impl From<Ampeq> for BinOp {
    fn from(node: Ampeq) -> BinOp {
        BinOp::Ampeq(node)
    }
}
impl From<Careteq> for BinOp {
    fn from(node: Careteq) -> BinOp {
        BinOp::Careteq(node)
    }
}
impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BinOp::Pipepipe(it) => std::fmt::Display::fmt(it, f),
            BinOp::Ampamp(it) => std::fmt::Display::fmt(it, f),
            BinOp::Eqeq(it) => std::fmt::Display::fmt(it, f),
            BinOp::Neq(it) => std::fmt::Display::fmt(it, f),
            BinOp::Lteq(it) => std::fmt::Display::fmt(it, f),
            BinOp::Gteq(it) => std::fmt::Display::fmt(it, f),
            BinOp::LAngle(it) => std::fmt::Display::fmt(it, f),
            BinOp::RAngle(it) => std::fmt::Display::fmt(it, f),
            BinOp::Plus(it) => std::fmt::Display::fmt(it, f),
            BinOp::Star(it) => std::fmt::Display::fmt(it, f),
            BinOp::Minus(it) => std::fmt::Display::fmt(it, f),
            BinOp::Slash(it) => std::fmt::Display::fmt(it, f),
            BinOp::Percent(it) => std::fmt::Display::fmt(it, f),
            BinOp::Shl(it) => std::fmt::Display::fmt(it, f),
            BinOp::Shr(it) => std::fmt::Display::fmt(it, f),
            BinOp::Caret(it) => std::fmt::Display::fmt(it, f),
            BinOp::Pipe(it) => std::fmt::Display::fmt(it, f),
            BinOp::Amp(it) => std::fmt::Display::fmt(it, f),
            BinOp::Eq(it) => std::fmt::Display::fmt(it, f),
            BinOp::Pluseq(it) => std::fmt::Display::fmt(it, f),
            BinOp::Slasheq(it) => std::fmt::Display::fmt(it, f),
            BinOp::Stareq(it) => std::fmt::Display::fmt(it, f),
            BinOp::Percenteq(it) => std::fmt::Display::fmt(it, f),
            BinOp::Shreq(it) => std::fmt::Display::fmt(it, f),
            BinOp::Shleq(it) => std::fmt::Display::fmt(it, f),
            BinOp::Minuseq(it) => std::fmt::Display::fmt(it, f),
            BinOp::Pipeeq(it) => std::fmt::Display::fmt(it, f),
            BinOp::Ampeq(it) => std::fmt::Display::fmt(it, f),
            BinOp::Careteq(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstToken for BinOp {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            AMP | AMPAMP | AMPEQ | CARET | CARETEQ | EQ | EQEQ | GTEQ | LTEQ | L_ANGLE | MINUS
            | MINUSEQ | NEQ | PERCENT | PERCENTEQ | PIPE | PIPEEQ | PIPEPIPE | PLUS | PLUSEQ
            | R_ANGLE | SHL | SHLEQ | SHR | SHREQ | SLASH | SLASHEQ | STAR | STAREQ => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        match syntax.kind() {
            PIPEPIPE => Pipepipe::cast_or_return(syntax).map(|x| BinOp::Pipepipe(x)),
            AMPAMP => Ampamp::cast_or_return(syntax).map(|x| BinOp::Ampamp(x)),
            EQEQ => Eqeq::cast_or_return(syntax).map(|x| BinOp::Eqeq(x)),
            NEQ => Neq::cast_or_return(syntax).map(|x| BinOp::Neq(x)),
            LTEQ => Lteq::cast_or_return(syntax).map(|x| BinOp::Lteq(x)),
            GTEQ => Gteq::cast_or_return(syntax).map(|x| BinOp::Gteq(x)),
            L_ANGLE => LAngle::cast_or_return(syntax).map(|x| BinOp::LAngle(x)),
            R_ANGLE => RAngle::cast_or_return(syntax).map(|x| BinOp::RAngle(x)),
            PLUS => Plus::cast_or_return(syntax).map(|x| BinOp::Plus(x)),
            STAR => Star::cast_or_return(syntax).map(|x| BinOp::Star(x)),
            MINUS => Minus::cast_or_return(syntax).map(|x| BinOp::Minus(x)),
            SLASH => Slash::cast_or_return(syntax).map(|x| BinOp::Slash(x)),
            PERCENT => Percent::cast_or_return(syntax).map(|x| BinOp::Percent(x)),
            SHL => Shl::cast_or_return(syntax).map(|x| BinOp::Shl(x)),
            SHR => Shr::cast_or_return(syntax).map(|x| BinOp::Shr(x)),
            CARET => Caret::cast_or_return(syntax).map(|x| BinOp::Caret(x)),
            PIPE => Pipe::cast_or_return(syntax).map(|x| BinOp::Pipe(x)),
            AMP => Amp::cast_or_return(syntax).map(|x| BinOp::Amp(x)),
            EQ => Eq::cast_or_return(syntax).map(|x| BinOp::Eq(x)),
            PLUSEQ => Pluseq::cast_or_return(syntax).map(|x| BinOp::Pluseq(x)),
            SLASHEQ => Slasheq::cast_or_return(syntax).map(|x| BinOp::Slasheq(x)),
            STAREQ => Stareq::cast_or_return(syntax).map(|x| BinOp::Stareq(x)),
            PERCENTEQ => Percenteq::cast_or_return(syntax).map(|x| BinOp::Percenteq(x)),
            SHREQ => Shreq::cast_or_return(syntax).map(|x| BinOp::Shreq(x)),
            SHLEQ => Shleq::cast_or_return(syntax).map(|x| BinOp::Shleq(x)),
            MINUSEQ => Minuseq::cast_or_return(syntax).map(|x| BinOp::Minuseq(x)),
            PIPEEQ => Pipeeq::cast_or_return(syntax).map(|x| BinOp::Pipeeq(x)),
            AMPEQ => Ampeq::cast_or_return(syntax).map(|x| BinOp::Ampeq(x)),
            CARETEQ => Careteq::cast_or_return(syntax).map(|x| BinOp::Careteq(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        match self {
            BinOp::Pipepipe(it) => it.syntax(),
            BinOp::Ampamp(it) => it.syntax(),
            BinOp::Eqeq(it) => it.syntax(),
            BinOp::Neq(it) => it.syntax(),
            BinOp::Lteq(it) => it.syntax(),
            BinOp::Gteq(it) => it.syntax(),
            BinOp::LAngle(it) => it.syntax(),
            BinOp::RAngle(it) => it.syntax(),
            BinOp::Plus(it) => it.syntax(),
            BinOp::Star(it) => it.syntax(),
            BinOp::Minus(it) => it.syntax(),
            BinOp::Slash(it) => it.syntax(),
            BinOp::Percent(it) => it.syntax(),
            BinOp::Shl(it) => it.syntax(),
            BinOp::Shr(it) => it.syntax(),
            BinOp::Caret(it) => it.syntax(),
            BinOp::Pipe(it) => it.syntax(),
            BinOp::Amp(it) => it.syntax(),
            BinOp::Eq(it) => it.syntax(),
            BinOp::Pluseq(it) => it.syntax(),
            BinOp::Slasheq(it) => it.syntax(),
            BinOp::Stareq(it) => it.syntax(),
            BinOp::Percenteq(it) => it.syntax(),
            BinOp::Shreq(it) => it.syntax(),
            BinOp::Shleq(it) => it.syntax(),
            BinOp::Minuseq(it) => it.syntax(),
            BinOp::Pipeeq(it) => it.syntax(),
            BinOp::Ampeq(it) => it.syntax(),
            BinOp::Careteq(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxToken {
        match self {
            BinOp::Pipepipe(it) => it.into_syntax(),
            BinOp::Ampamp(it) => it.into_syntax(),
            BinOp::Eqeq(it) => it.into_syntax(),
            BinOp::Neq(it) => it.into_syntax(),
            BinOp::Lteq(it) => it.into_syntax(),
            BinOp::Gteq(it) => it.into_syntax(),
            BinOp::LAngle(it) => it.into_syntax(),
            BinOp::RAngle(it) => it.into_syntax(),
            BinOp::Plus(it) => it.into_syntax(),
            BinOp::Star(it) => it.into_syntax(),
            BinOp::Minus(it) => it.into_syntax(),
            BinOp::Slash(it) => it.into_syntax(),
            BinOp::Percent(it) => it.into_syntax(),
            BinOp::Shl(it) => it.into_syntax(),
            BinOp::Shr(it) => it.into_syntax(),
            BinOp::Caret(it) => it.into_syntax(),
            BinOp::Pipe(it) => it.into_syntax(),
            BinOp::Amp(it) => it.into_syntax(),
            BinOp::Eq(it) => it.into_syntax(),
            BinOp::Pluseq(it) => it.into_syntax(),
            BinOp::Slasheq(it) => it.into_syntax(),
            BinOp::Stareq(it) => it.into_syntax(),
            BinOp::Percenteq(it) => it.into_syntax(),
            BinOp::Shreq(it) => it.into_syntax(),
            BinOp::Shleq(it) => it.into_syntax(),
            BinOp::Minuseq(it) => it.into_syntax(),
            BinOp::Pipeeq(it) => it.into_syntax(),
            BinOp::Ampeq(it) => it.into_syntax(),
            BinOp::Careteq(it) => it.into_syntax(),
        }
    }
}
impl AstElement for BinOp {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            AMP | AMPAMP | AMPEQ | CARET | CARETEQ | EQ | EQEQ | GTEQ | LTEQ | L_ANGLE | MINUS
            | MINUSEQ | NEQ | PERCENT | PERCENTEQ | PIPE | PIPEEQ | PIPEPIPE | PLUS | PLUSEQ
            | R_ANGLE | SHL | SHLEQ | SHR | SHREQ | SLASH | SLASHEQ | STAR | STAREQ => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            PIPEPIPE => Pipepipe::cast_or_return_element(syntax).map(|x| BinOp::Pipepipe(x)),
            AMPAMP => Ampamp::cast_or_return_element(syntax).map(|x| BinOp::Ampamp(x)),
            EQEQ => Eqeq::cast_or_return_element(syntax).map(|x| BinOp::Eqeq(x)),
            NEQ => Neq::cast_or_return_element(syntax).map(|x| BinOp::Neq(x)),
            LTEQ => Lteq::cast_or_return_element(syntax).map(|x| BinOp::Lteq(x)),
            GTEQ => Gteq::cast_or_return_element(syntax).map(|x| BinOp::Gteq(x)),
            L_ANGLE => LAngle::cast_or_return_element(syntax).map(|x| BinOp::LAngle(x)),
            R_ANGLE => RAngle::cast_or_return_element(syntax).map(|x| BinOp::RAngle(x)),
            PLUS => Plus::cast_or_return_element(syntax).map(|x| BinOp::Plus(x)),
            STAR => Star::cast_or_return_element(syntax).map(|x| BinOp::Star(x)),
            MINUS => Minus::cast_or_return_element(syntax).map(|x| BinOp::Minus(x)),
            SLASH => Slash::cast_or_return_element(syntax).map(|x| BinOp::Slash(x)),
            PERCENT => Percent::cast_or_return_element(syntax).map(|x| BinOp::Percent(x)),
            SHL => Shl::cast_or_return_element(syntax).map(|x| BinOp::Shl(x)),
            SHR => Shr::cast_or_return_element(syntax).map(|x| BinOp::Shr(x)),
            CARET => Caret::cast_or_return_element(syntax).map(|x| BinOp::Caret(x)),
            PIPE => Pipe::cast_or_return_element(syntax).map(|x| BinOp::Pipe(x)),
            AMP => Amp::cast_or_return_element(syntax).map(|x| BinOp::Amp(x)),
            EQ => Eq::cast_or_return_element(syntax).map(|x| BinOp::Eq(x)),
            PLUSEQ => Pluseq::cast_or_return_element(syntax).map(|x| BinOp::Pluseq(x)),
            SLASHEQ => Slasheq::cast_or_return_element(syntax).map(|x| BinOp::Slasheq(x)),
            STAREQ => Stareq::cast_or_return_element(syntax).map(|x| BinOp::Stareq(x)),
            PERCENTEQ => Percenteq::cast_or_return_element(syntax).map(|x| BinOp::Percenteq(x)),
            SHREQ => Shreq::cast_or_return_element(syntax).map(|x| BinOp::Shreq(x)),
            SHLEQ => Shleq::cast_or_return_element(syntax).map(|x| BinOp::Shleq(x)),
            MINUSEQ => Minuseq::cast_or_return_element(syntax).map(|x| BinOp::Minuseq(x)),
            PIPEEQ => Pipeeq::cast_or_return_element(syntax).map(|x| BinOp::Pipeeq(x)),
            AMPEQ => Ampeq::cast_or_return_element(syntax).map(|x| BinOp::Ampeq(x)),
            CARETEQ => Careteq::cast_or_return_element(syntax).map(|x| BinOp::Careteq(x)),
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            BinOp::Pipepipe(it) => it.syntax_element(),
            BinOp::Ampamp(it) => it.syntax_element(),
            BinOp::Eqeq(it) => it.syntax_element(),
            BinOp::Neq(it) => it.syntax_element(),
            BinOp::Lteq(it) => it.syntax_element(),
            BinOp::Gteq(it) => it.syntax_element(),
            BinOp::LAngle(it) => it.syntax_element(),
            BinOp::RAngle(it) => it.syntax_element(),
            BinOp::Plus(it) => it.syntax_element(),
            BinOp::Star(it) => it.syntax_element(),
            BinOp::Minus(it) => it.syntax_element(),
            BinOp::Slash(it) => it.syntax_element(),
            BinOp::Percent(it) => it.syntax_element(),
            BinOp::Shl(it) => it.syntax_element(),
            BinOp::Shr(it) => it.syntax_element(),
            BinOp::Caret(it) => it.syntax_element(),
            BinOp::Pipe(it) => it.syntax_element(),
            BinOp::Amp(it) => it.syntax_element(),
            BinOp::Eq(it) => it.syntax_element(),
            BinOp::Pluseq(it) => it.syntax_element(),
            BinOp::Slasheq(it) => it.syntax_element(),
            BinOp::Stareq(it) => it.syntax_element(),
            BinOp::Percenteq(it) => it.syntax_element(),
            BinOp::Shreq(it) => it.syntax_element(),
            BinOp::Shleq(it) => it.syntax_element(),
            BinOp::Minuseq(it) => it.syntax_element(),
            BinOp::Pipeeq(it) => it.syntax_element(),
            BinOp::Ampeq(it) => it.syntax_element(),
            BinOp::Careteq(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            BinOp::Pipepipe(it) => it.into_syntax_element(),
            BinOp::Ampamp(it) => it.into_syntax_element(),
            BinOp::Eqeq(it) => it.into_syntax_element(),
            BinOp::Neq(it) => it.into_syntax_element(),
            BinOp::Lteq(it) => it.into_syntax_element(),
            BinOp::Gteq(it) => it.into_syntax_element(),
            BinOp::LAngle(it) => it.into_syntax_element(),
            BinOp::RAngle(it) => it.into_syntax_element(),
            BinOp::Plus(it) => it.into_syntax_element(),
            BinOp::Star(it) => it.into_syntax_element(),
            BinOp::Minus(it) => it.into_syntax_element(),
            BinOp::Slash(it) => it.into_syntax_element(),
            BinOp::Percent(it) => it.into_syntax_element(),
            BinOp::Shl(it) => it.into_syntax_element(),
            BinOp::Shr(it) => it.into_syntax_element(),
            BinOp::Caret(it) => it.into_syntax_element(),
            BinOp::Pipe(it) => it.into_syntax_element(),
            BinOp::Amp(it) => it.into_syntax_element(),
            BinOp::Eq(it) => it.into_syntax_element(),
            BinOp::Pluseq(it) => it.into_syntax_element(),
            BinOp::Slasheq(it) => it.into_syntax_element(),
            BinOp::Stareq(it) => it.into_syntax_element(),
            BinOp::Percenteq(it) => it.into_syntax_element(),
            BinOp::Shreq(it) => it.into_syntax_element(),
            BinOp::Shleq(it) => it.into_syntax_element(),
            BinOp::Minuseq(it) => it.into_syntax_element(),
            BinOp::Pipeeq(it) => it.into_syntax_element(),
            BinOp::Ampeq(it) => it.into_syntax_element(),
            BinOp::Careteq(it) => it.into_syntax_element(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrefixOp {
    Minus(Minus),
    Excl(Excl),
    Star(Star),
}
impl From<Minus> for PrefixOp {
    fn from(node: Minus) -> PrefixOp {
        PrefixOp::Minus(node)
    }
}
impl From<Excl> for PrefixOp {
    fn from(node: Excl) -> PrefixOp {
        PrefixOp::Excl(node)
    }
}
impl From<Star> for PrefixOp {
    fn from(node: Star) -> PrefixOp {
        PrefixOp::Star(node)
    }
}
impl std::fmt::Display for PrefixOp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PrefixOp::Minus(it) => std::fmt::Display::fmt(it, f),
            PrefixOp::Excl(it) => std::fmt::Display::fmt(it, f),
            PrefixOp::Star(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstToken for PrefixOp {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            EXCL | MINUS | STAR => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        match syntax.kind() {
            MINUS => Minus::cast_or_return(syntax).map(|x| PrefixOp::Minus(x)),
            EXCL => Excl::cast_or_return(syntax).map(|x| PrefixOp::Excl(x)),
            STAR => Star::cast_or_return(syntax).map(|x| PrefixOp::Star(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        match self {
            PrefixOp::Minus(it) => it.syntax(),
            PrefixOp::Excl(it) => it.syntax(),
            PrefixOp::Star(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxToken {
        match self {
            PrefixOp::Minus(it) => it.into_syntax(),
            PrefixOp::Excl(it) => it.into_syntax(),
            PrefixOp::Star(it) => it.into_syntax(),
        }
    }
}
impl AstElement for PrefixOp {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            EXCL | MINUS | STAR => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            MINUS => Minus::cast_or_return_element(syntax).map(|x| PrefixOp::Minus(x)),
            EXCL => Excl::cast_or_return_element(syntax).map(|x| PrefixOp::Excl(x)),
            STAR => Star::cast_or_return_element(syntax).map(|x| PrefixOp::Star(x)),
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            PrefixOp::Minus(it) => it.syntax_element(),
            PrefixOp::Excl(it) => it.syntax_element(),
            PrefixOp::Star(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            PrefixOp::Minus(it) => it.into_syntax_element(),
            PrefixOp::Excl(it) => it.into_syntax_element(),
            PrefixOp::Star(it) => it.into_syntax_element(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RangeOp {
    Dotdot(Dotdot),
    Dotdoteq(Dotdoteq),
}
impl From<Dotdot> for RangeOp {
    fn from(node: Dotdot) -> RangeOp {
        RangeOp::Dotdot(node)
    }
}
impl From<Dotdoteq> for RangeOp {
    fn from(node: Dotdoteq) -> RangeOp {
        RangeOp::Dotdoteq(node)
    }
}
impl std::fmt::Display for RangeOp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RangeOp::Dotdot(it) => std::fmt::Display::fmt(it, f),
            RangeOp::Dotdoteq(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstToken for RangeOp {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            DOTDOT | DOTDOTEQ => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        match syntax.kind() {
            DOTDOT => Dotdot::cast_or_return(syntax).map(|x| RangeOp::Dotdot(x)),
            DOTDOTEQ => Dotdoteq::cast_or_return(syntax).map(|x| RangeOp::Dotdoteq(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        match self {
            RangeOp::Dotdot(it) => it.syntax(),
            RangeOp::Dotdoteq(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxToken {
        match self {
            RangeOp::Dotdot(it) => it.into_syntax(),
            RangeOp::Dotdoteq(it) => it.into_syntax(),
        }
    }
}
impl AstElement for RangeOp {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            DOTDOT | DOTDOTEQ => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            DOTDOT => Dotdot::cast_or_return_element(syntax).map(|x| RangeOp::Dotdot(x)),
            DOTDOTEQ => Dotdoteq::cast_or_return_element(syntax).map(|x| RangeOp::Dotdoteq(x)),
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            RangeOp::Dotdot(it) => it.syntax_element(),
            RangeOp::Dotdoteq(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            RangeOp::Dotdot(it) => it.into_syntax_element(),
            RangeOp::Dotdoteq(it) => it.into_syntax_element(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LiteralToken {
    IntNumber(IntNumber),
    FloatNumber(FloatNumber),
    String(String),
    RawString(RawString),
    TrueKw(TrueKw),
    FalseKw(FalseKw),
    ByteString(ByteString),
    RawByteString(RawByteString),
    Char(Char),
    Byte(Byte),
}
impl From<IntNumber> for LiteralToken {
    fn from(node: IntNumber) -> LiteralToken {
        LiteralToken::IntNumber(node)
    }
}
impl From<FloatNumber> for LiteralToken {
    fn from(node: FloatNumber) -> LiteralToken {
        LiteralToken::FloatNumber(node)
    }
}
impl From<String> for LiteralToken {
    fn from(node: String) -> LiteralToken {
        LiteralToken::String(node)
    }
}
impl From<RawString> for LiteralToken {
    fn from(node: RawString) -> LiteralToken {
        LiteralToken::RawString(node)
    }
}
impl From<TrueKw> for LiteralToken {
    fn from(node: TrueKw) -> LiteralToken {
        LiteralToken::TrueKw(node)
    }
}
impl From<FalseKw> for LiteralToken {
    fn from(node: FalseKw) -> LiteralToken {
        LiteralToken::FalseKw(node)
    }
}
impl From<ByteString> for LiteralToken {
    fn from(node: ByteString) -> LiteralToken {
        LiteralToken::ByteString(node)
    }
}
impl From<RawByteString> for LiteralToken {
    fn from(node: RawByteString) -> LiteralToken {
        LiteralToken::RawByteString(node)
    }
}
impl From<Char> for LiteralToken {
    fn from(node: Char) -> LiteralToken {
        LiteralToken::Char(node)
    }
}
impl From<Byte> for LiteralToken {
    fn from(node: Byte) -> LiteralToken {
        LiteralToken::Byte(node)
    }
}
impl std::fmt::Display for LiteralToken {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LiteralToken::IntNumber(it) => std::fmt::Display::fmt(it, f),
            LiteralToken::FloatNumber(it) => std::fmt::Display::fmt(it, f),
            LiteralToken::String(it) => std::fmt::Display::fmt(it, f),
            LiteralToken::RawString(it) => std::fmt::Display::fmt(it, f),
            LiteralToken::TrueKw(it) => std::fmt::Display::fmt(it, f),
            LiteralToken::FalseKw(it) => std::fmt::Display::fmt(it, f),
            LiteralToken::ByteString(it) => std::fmt::Display::fmt(it, f),
            LiteralToken::RawByteString(it) => std::fmt::Display::fmt(it, f),
            LiteralToken::Char(it) => std::fmt::Display::fmt(it, f),
            LiteralToken::Byte(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstToken for LiteralToken {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BYTE | BYTE_STRING | CHAR | FALSE_KW | FLOAT_NUMBER | INT_NUMBER | RAW_BYTE_STRING
            | RAW_STRING | STRING | TRUE_KW => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        match syntax.kind() {
            INT_NUMBER => IntNumber::cast_or_return(syntax).map(|x| LiteralToken::IntNumber(x)),
            FLOAT_NUMBER => {
                FloatNumber::cast_or_return(syntax).map(|x| LiteralToken::FloatNumber(x))
            }
            STRING => String::cast_or_return(syntax).map(|x| LiteralToken::String(x)),
            RAW_STRING => RawString::cast_or_return(syntax).map(|x| LiteralToken::RawString(x)),
            TRUE_KW => TrueKw::cast_or_return(syntax).map(|x| LiteralToken::TrueKw(x)),
            FALSE_KW => FalseKw::cast_or_return(syntax).map(|x| LiteralToken::FalseKw(x)),
            BYTE_STRING => ByteString::cast_or_return(syntax).map(|x| LiteralToken::ByteString(x)),
            RAW_BYTE_STRING => {
                RawByteString::cast_or_return(syntax).map(|x| LiteralToken::RawByteString(x))
            }
            CHAR => Char::cast_or_return(syntax).map(|x| LiteralToken::Char(x)),
            BYTE => Byte::cast_or_return(syntax).map(|x| LiteralToken::Byte(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        match self {
            LiteralToken::IntNumber(it) => it.syntax(),
            LiteralToken::FloatNumber(it) => it.syntax(),
            LiteralToken::String(it) => it.syntax(),
            LiteralToken::RawString(it) => it.syntax(),
            LiteralToken::TrueKw(it) => it.syntax(),
            LiteralToken::FalseKw(it) => it.syntax(),
            LiteralToken::ByteString(it) => it.syntax(),
            LiteralToken::RawByteString(it) => it.syntax(),
            LiteralToken::Char(it) => it.syntax(),
            LiteralToken::Byte(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxToken {
        match self {
            LiteralToken::IntNumber(it) => it.into_syntax(),
            LiteralToken::FloatNumber(it) => it.into_syntax(),
            LiteralToken::String(it) => it.into_syntax(),
            LiteralToken::RawString(it) => it.into_syntax(),
            LiteralToken::TrueKw(it) => it.into_syntax(),
            LiteralToken::FalseKw(it) => it.into_syntax(),
            LiteralToken::ByteString(it) => it.into_syntax(),
            LiteralToken::RawByteString(it) => it.into_syntax(),
            LiteralToken::Char(it) => it.into_syntax(),
            LiteralToken::Byte(it) => it.into_syntax(),
        }
    }
}
impl AstElement for LiteralToken {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            BYTE | BYTE_STRING | CHAR | FALSE_KW | FLOAT_NUMBER | INT_NUMBER | RAW_BYTE_STRING
            | RAW_STRING | STRING | TRUE_KW => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            INT_NUMBER => {
                IntNumber::cast_or_return_element(syntax).map(|x| LiteralToken::IntNumber(x))
            }
            FLOAT_NUMBER => {
                FloatNumber::cast_or_return_element(syntax).map(|x| LiteralToken::FloatNumber(x))
            }
            STRING => String::cast_or_return_element(syntax).map(|x| LiteralToken::String(x)),
            RAW_STRING => {
                RawString::cast_or_return_element(syntax).map(|x| LiteralToken::RawString(x))
            }
            TRUE_KW => TrueKw::cast_or_return_element(syntax).map(|x| LiteralToken::TrueKw(x)),
            FALSE_KW => FalseKw::cast_or_return_element(syntax).map(|x| LiteralToken::FalseKw(x)),
            BYTE_STRING => {
                ByteString::cast_or_return_element(syntax).map(|x| LiteralToken::ByteString(x))
            }
            RAW_BYTE_STRING => RawByteString::cast_or_return_element(syntax)
                .map(|x| LiteralToken::RawByteString(x)),
            CHAR => Char::cast_or_return_element(syntax).map(|x| LiteralToken::Char(x)),
            BYTE => Byte::cast_or_return_element(syntax).map(|x| LiteralToken::Byte(x)),
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            LiteralToken::IntNumber(it) => it.syntax_element(),
            LiteralToken::FloatNumber(it) => it.syntax_element(),
            LiteralToken::String(it) => it.syntax_element(),
            LiteralToken::RawString(it) => it.syntax_element(),
            LiteralToken::TrueKw(it) => it.syntax_element(),
            LiteralToken::FalseKw(it) => it.syntax_element(),
            LiteralToken::ByteString(it) => it.syntax_element(),
            LiteralToken::RawByteString(it) => it.syntax_element(),
            LiteralToken::Char(it) => it.syntax_element(),
            LiteralToken::Byte(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            LiteralToken::IntNumber(it) => it.into_syntax_element(),
            LiteralToken::FloatNumber(it) => it.into_syntax_element(),
            LiteralToken::String(it) => it.into_syntax_element(),
            LiteralToken::RawString(it) => it.into_syntax_element(),
            LiteralToken::TrueKw(it) => it.into_syntax_element(),
            LiteralToken::FalseKw(it) => it.into_syntax_element(),
            LiteralToken::ByteString(it) => it.into_syntax_element(),
            LiteralToken::RawByteString(it) => it.into_syntax_element(),
            LiteralToken::Char(it) => it.into_syntax_element(),
            LiteralToken::Byte(it) => it.into_syntax_element(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NameRefToken {
    Ident(Ident),
    IntNumber(IntNumber),
}
impl From<Ident> for NameRefToken {
    fn from(node: Ident) -> NameRefToken {
        NameRefToken::Ident(node)
    }
}
impl From<IntNumber> for NameRefToken {
    fn from(node: IntNumber) -> NameRefToken {
        NameRefToken::IntNumber(node)
    }
}
impl std::fmt::Display for NameRefToken {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NameRefToken::Ident(it) => std::fmt::Display::fmt(it, f),
            NameRefToken::IntNumber(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstToken for NameRefToken {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            IDENT | INT_NUMBER => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxToken) -> Result<Self, SyntaxToken> {
        match syntax.kind() {
            IDENT => Ident::cast_or_return(syntax).map(|x| NameRefToken::Ident(x)),
            INT_NUMBER => IntNumber::cast_or_return(syntax).map(|x| NameRefToken::IntNumber(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        match self {
            NameRefToken::Ident(it) => it.syntax(),
            NameRefToken::IntNumber(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxToken {
        match self {
            NameRefToken::Ident(it) => it.into_syntax(),
            NameRefToken::IntNumber(it) => it.into_syntax(),
        }
    }
}
impl AstElement for NameRefToken {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            IDENT | INT_NUMBER => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            IDENT => Ident::cast_or_return_element(syntax).map(|x| NameRefToken::Ident(x)),
            INT_NUMBER => {
                IntNumber::cast_or_return_element(syntax).map(|x| NameRefToken::IntNumber(x))
            }
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            NameRefToken::Ident(it) => it.syntax_element(),
            NameRefToken::IntNumber(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            NameRefToken::Ident(it) => it.into_syntax_element(),
            NameRefToken::IntNumber(it) => it.into_syntax_element(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldDefList {
    RecordFieldDefList(RecordFieldDefList),
    TupleFieldDefList(TupleFieldDefList),
}
impl From<RecordFieldDefList> for FieldDefList {
    fn from(node: RecordFieldDefList) -> FieldDefList {
        FieldDefList::RecordFieldDefList(node)
    }
}
impl From<TupleFieldDefList> for FieldDefList {
    fn from(node: TupleFieldDefList) -> FieldDefList {
        FieldDefList::TupleFieldDefList(node)
    }
}
impl std::fmt::Display for FieldDefList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FieldDefList::RecordFieldDefList(it) => std::fmt::Display::fmt(it, f),
            FieldDefList::TupleFieldDefList(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstNode for FieldDefList {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD_DEF_LIST | TUPLE_FIELD_DEF_LIST => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return(syntax: SyntaxNode) -> Result<Self, SyntaxNode> {
        match syntax.kind() {
            RECORD_FIELD_DEF_LIST => RecordFieldDefList::cast_or_return(syntax)
                .map(|x| FieldDefList::RecordFieldDefList(x)),
            TUPLE_FIELD_DEF_LIST => TupleFieldDefList::cast_or_return(syntax)
                .map(|x| FieldDefList::TupleFieldDefList(x)),
            _ => Err(syntax),
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            FieldDefList::RecordFieldDefList(it) => it.syntax(),
            FieldDefList::TupleFieldDefList(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            FieldDefList::RecordFieldDefList(it) => it.into_syntax(),
            FieldDefList::TupleFieldDefList(it) => it.into_syntax(),
        }
    }
}
impl AstElement for FieldDefList {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            RECORD_FIELD_DEF_LIST | TUPLE_FIELD_DEF_LIST => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            RECORD_FIELD_DEF_LIST => RecordFieldDefList::cast_or_return_element(syntax)
                .map(|x| FieldDefList::RecordFieldDefList(x)),
            TUPLE_FIELD_DEF_LIST => TupleFieldDefList::cast_or_return_element(syntax)
                .map(|x| FieldDefList::TupleFieldDefList(x)),
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            FieldDefList::RecordFieldDefList(it) => it.syntax_element(),
            FieldDefList::TupleFieldDefList(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            FieldDefList::RecordFieldDefList(it) => it.into_syntax_element(),
            FieldDefList::TupleFieldDefList(it) => it.into_syntax_element(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttrOrComment {
    Attr(Attr),
    Comment(Comment),
}
impl From<Attr> for AttrOrComment {
    fn from(node: Attr) -> AttrOrComment {
        AttrOrComment::Attr(node)
    }
}
impl From<Comment> for AttrOrComment {
    fn from(node: Comment) -> AttrOrComment {
        AttrOrComment::Comment(node)
    }
}
impl std::fmt::Display for AttrOrComment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AttrOrComment::Attr(it) => std::fmt::Display::fmt(it, f),
            AttrOrComment::Comment(it) => std::fmt::Display::fmt(it, f),
        }
    }
}
impl AstElement for AttrOrComment {
    fn can_cast_element(kind: SyntaxKind) -> bool {
        match kind {
            ATTR | COMMENT => true,
            _ => false,
        }
    }
    #[allow(unreachable_patterns)]
    fn cast_or_return_element(syntax: SyntaxElement) -> Result<Self, SyntaxElement> {
        match syntax.kind() {
            ATTR => Attr::cast_or_return_element(syntax).map(|x| AttrOrComment::Attr(x)),
            COMMENT => Comment::cast_or_return_element(syntax).map(|x| AttrOrComment::Comment(x)),
            _ => Err(syntax),
        }
    }
    fn syntax_element(&self) -> NodeOrToken<&SyntaxNode, &SyntaxToken> {
        match self {
            AttrOrComment::Attr(it) => it.syntax_element(),
            AttrOrComment::Comment(it) => it.syntax_element(),
        }
    }
    fn into_syntax_element(self) -> SyntaxElement {
        match self {
            AttrOrComment::Attr(it) => it.into_syntax_element(),
            AttrOrComment::Comment(it) => it.into_syntax_element(),
        }
    }
}
