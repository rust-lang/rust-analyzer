/// `mbe` (short for Macro By Example) crate contains code for handling
/// `macro_rules` macros. It uses `TokenTree` (from `ra_tt` package) as the
/// interface, although it contains some code to bridge `SyntaxNode`s and
/// `TokenTree`s as well!

macro_rules! impl_froms {
    ($e:ident: $($v:ident), *) => {
        $(
            impl From<$v> for $e {
                fn from(it: $v) -> $e {
                    $e::$v(it)
                }
            }
        )*
    }
}

mod tt_cursor;
mod mbe_parser;
mod mbe_expander;
mod syntax_bridge;

use ra_syntax::SmolStr;

pub use tt::{Delimiter, Punct};

pub use crate::syntax_bridge::{
    ast_to_token_tree, parse_token_tree,
    TokenMap, RangesMap,
};

/// This struct contains AST for a single `macro_rules` defenition. What might
/// be very confusing is that AST has almost exactly the same shape as
/// `tt::TokenTree`, but there's a crucial difference: in macro rules, `$ident`
/// and `$()*` have special meaning (see `Var` and `Repeat` data structures)
#[derive(Debug, PartialEq, Eq)]
pub struct MacroRules {
    pub(crate) rules: Vec<Rule>,
}

impl MacroRules {
    pub fn parse(tt: &tt::Subtree) -> Option<MacroRules> {
        mbe_parser::parse(tt)
    }
    pub fn expand(&self, tt: &tt::Subtree) -> Option<tt::Subtree> {
        mbe_expander::exapnd(self, tt)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Rule {
    pub(crate) lhs: Subtree,
    pub(crate) rhs: Subtree,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TokenTree {
    Leaf(Leaf),
    Subtree(Subtree),
    Repeat(Repeat),
}
impl_froms!(TokenTree: Leaf, Subtree, Repeat);

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Leaf {
    Literal(Literal),
    Punct(Punct),
    Ident(Ident),
    Var(Var),
}
impl_froms!(Leaf: Literal, Punct, Ident, Var);

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Subtree {
    pub(crate) delimiter: Delimiter,
    pub(crate) token_trees: Vec<TokenTree>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Repeat {
    pub(crate) subtree: Subtree,
    pub(crate) kind: RepeatKind,
    pub(crate) separator: Option<char>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RepeatKind {
    ZeroOrMore,
    OneOrMore,
    ZeroOrOne,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Literal {
    pub(crate) text: SmolStr,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Ident {
    pub(crate) text: SmolStr,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Var {
    pub(crate) text: SmolStr,
    pub(crate) kind: Option<SmolStr>,
}

#[cfg(test)]
mod tests {
    use ra_syntax::{ast, AstNode, TextRange};

    use super::*;

    // Good first issue (although a slightly chellegning one):
    //
    // * Pick a random test from here
    //   https://github.com/intellij-rust/intellij-rust/blob/c4e9feee4ad46e7953b1948c112533360b6087bb/src/test/kotlin/org/rust/lang/core/macros/RsMacroExpansionTest.kt
    // * Port the test to rust and add it to this module
    // * Make it pass :-)

    #[test]
    fn test_convert_tt() {
        let macro_definition = r#"
macro_rules! impl_froms {
    ($e:ident: $($v:ident),*) => {
        $(
            impl From<$v> for $e {
                fn from(it: $v) -> $e {
                    $e::$v(it)
                }
            }
        )*
    }
}
"#;

        let macro_invocation = r#"
impl_froms!(TokenTree: Leaf, Subtree);
"#;

        let source_file = ast::SourceFile::parse(macro_definition);
        let macro_definition = source_file
            .syntax()
            .descendants()
            .find_map(ast::MacroCall::cast)
            .unwrap();

        let source_file = ast::SourceFile::parse(macro_invocation);
        let macro_invocation = source_file
            .syntax()
            .descendants()
            .find_map(ast::MacroCall::cast)
            .unwrap();

        let (definition_tt, _) = ast_to_token_tree(macro_definition.token_tree().unwrap()).unwrap();
        let (invocation_tt, token_map) =
            ast_to_token_tree(macro_invocation.token_tree().unwrap()).unwrap();
        let rules = crate::MacroRules::parse(&definition_tt).unwrap();
        let expansion = rules.expand(&invocation_tt).unwrap();
        let (file, ranges_map) = parse_token_tree(&expansion, &token_map);
        assert_eq!(
            file.syntax().text().to_string(),
            "impl From < Leaf > for TokenTree {fn from (it : Leaf) -> TokenTree {TokenTree :: Leaf (it)}} \
             impl From < Subtree > for TokenTree {fn from (it : Subtree) -> TokenTree {TokenTree :: Subtree (it)}}"
        );
        assert_eq!(
            ranges_map.map_forward(TextRange::from_to(1.into(), 10.into())),
            Some(TextRange::from_to(23.into(), 32.into())),
        )
    }

    #[test]
    fn dont_die_on_serde() {
        let macro_definition = r#"
macro_rules! tuple_impls {
    ($($len:expr => ($($n:tt $name:ident)+))+) => {
        $(
            impl<$($name),+> Serialize for ($($name,)+)
            where
                $($name: Serialize,)+
            {
                #[inline]
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    let mut tuple = try!(serializer.serialize_tuple($len));
                    $(
                        try!(tuple.serialize_element(&self.$n));
                    )+
                    tuple.end()
                }
            }
        )+
    }
}"#;
        let macro_invocation = r#"
tuple_impls! {
    1 => (0 T0)
    2 => (0 T0 1 T1)
    3 => (0 T0 1 T1 2 T2)
    4 => (0 T0 1 T1 2 T2 3 T3)
    5 => (0 T0 1 T1 2 T2 3 T3 4 T4)
    6 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5)
    7 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6)
    8 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7)
    9 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8)
    10 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9)
    11 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10)
    12 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11)
    13 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12)
    14 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13)
    15 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14)
    16 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15)
}
"#;
        let source_file = ast::SourceFile::parse(macro_definition);
        let macro_definition = source_file
            .syntax()
            .descendants()
            .find_map(ast::MacroCall::cast)
            .unwrap();

        let source_file = ast::SourceFile::parse(macro_invocation);
        let macro_invocation = source_file
            .syntax()
            .descendants()
            .find_map(ast::MacroCall::cast)
            .unwrap();

        let (definition_tt, _) = ast_to_token_tree(macro_definition.token_tree().unwrap()).unwrap();
        let (invocation_tt, token_map) =
            ast_to_token_tree(macro_invocation.token_tree().unwrap()).unwrap();
        let rules = crate::MacroRules::parse(&definition_tt).unwrap();
        let expansion = rules.expand(&invocation_tt);
        assert!(expansion.is_some())
    }
}
