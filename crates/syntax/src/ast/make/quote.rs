//! A `quote!`-like API for crafting AST nodes.

pub(crate) use rowan::{GreenNode, GreenToken, NodeOrToken, SyntaxKind as RSyntaxKind};

macro_rules! quote_impl_ {
    ( @append $children:ident ) => {}; // Base case.

    ( @append $children:ident
        $node:ident {
            $($tree:tt)*
        }
        $($rest:tt)*
    ) => {
        {
            #[allow(unused_mut)]
            let mut inner_children = ::std::vec::Vec::<$crate::ast::make::quote::NodeOrToken<
                $crate::ast::make::quote::GreenNode,
                $crate::ast::make::quote::GreenToken,
            >>::new();
            $crate::ast::make::quote::quote_impl!( @append inner_children
                $($tree)*
            );
            let kind = <$crate::ast::$node as $crate::ast::AstNode>::kind();
            let node = $crate::ast::make::quote::GreenNode::new($crate::ast::make::quote::RSyntaxKind(kind as u16), inner_children);
            $children.push($crate::ast::make::quote::NodeOrToken::Node(node));
        }
        $crate::ast::make::quote::quote_impl!( @append $children $($rest)* );
    };

    ( @append $children:ident
        [ $token_kind:ident $token_text:expr ]
        $($rest:tt)*
    ) => {
        $children.push($crate::ast::make::quote::NodeOrToken::Token(
            $crate::ast::make::quote::GreenToken::new(
                $crate::ast::make::quote::RSyntaxKind($crate::SyntaxKind::$token_kind as u16),
                &$token_text,
            ),
        ));
        $crate::ast::make::quote::quote_impl!( @append $children $($rest)* );
    };

    ( @append $children:ident
        [$($token:tt)+]
        $($rest:tt)*
    ) => {
        $children.push($crate::ast::make::quote::NodeOrToken::Token(
            $crate::ast::make::quote::GreenToken::new(
                $crate::ast::make::quote::RSyntaxKind($crate::T![ $($token)+ ] as u16),
                const { $crate::T![ $($token)+ ].text() },
            ),
        ));
        $crate::ast::make::quote::quote_impl!( @append $children $($rest)* );
    };

    ( @append $children:ident
        $whitespace:literal
        $($rest:tt)*
    ) => {
        const { $crate::ast::make::quote::verify_only_whitespaces($whitespace) };
        $crate::ast::make::quote::push_whitespace(&mut $children, $whitespace);
        $crate::ast::make::quote::quote_impl!( @append $children $($rest)* );
    };

    ( @append $children:ident
        # $var:ident
        $($rest:tt)*
    ) => {
        $crate::ast::make::quote::ToNodeChild::append_node_child($var, &mut $children);
        $crate::ast::make::quote::quote_impl!( @append $children $($rest)* );
    };

    ( @append $children:ident
        #( $($repetition:tt)+ )*
        $($rest:tt)*
    ) => {
        $crate::ast::make::quote::quote_impl!( @extract_pounded_in_repetition $children
            [] [] $($repetition)*
        );
        $crate::ast::make::quote::quote_impl!( @append $children $($rest)* );
    };

    // Base case - no repetition var.
    ( @extract_pounded_in_repetition $children:ident
        [ $($repetition:tt)* ] [ ]
    ) => {
        ::std::compile_error!("repetition in `ast::make::quote!()` without variable");
    };

    // Base case - repetition var found.
    ( @extract_pounded_in_repetition $children:ident
        [ $($repetition:tt)* ] [ $repetition_var:ident ]
    ) => {
        ::std::iter::IntoIterator::into_iter($repetition_var).for_each(|$repetition_var| {
            $crate::ast::make::quote::quote_impl!( @append $children $($repetition)* );
        });
    };

    ( @extract_pounded_in_repetition $children:ident
        [ $($repetition:tt)* ] [ $repetition_var1:ident ] # $repetition_var2:ident $($rest:tt)*
    ) => {
        ::std::compile_error!("repetition in `ast::make::quote!()` with more than one variable");
    };

    ( @extract_pounded_in_repetition $children:ident
        [ $($repetition:tt)* ] [ ] # $repetition_var:ident $($rest:tt)*
    ) => {
        $crate::ast::make::quote::quote_impl!( @extract_pounded_in_repetition $children
            [ $($repetition)* # $repetition_var ] [ $repetition_var ] $($rest)*
        );
    };

    ( @extract_pounded_in_repetition $children:ident
        [ $($repetition:tt)* ] [ $($repetition_var:tt)* ] $non_repetition_var:tt $($rest:tt)*
    ) => {
        $crate::ast::make::quote::quote_impl!( @extract_pounded_in_repetition $children
            [ $($repetition)* $non_repetition_var ] [ $($repetition_var)* ] $($rest)*
        );
    };
}
pub(crate) use quote_impl_ as quote_impl;

/// A `quote!`-like API for crafting AST nodes.
///
/// Syntax: AST nodes are created with `Node { children }`, where `Node` is the node name in `ast` (`ast::Node`).
/// Tokens are creates with their syntax enclosed by brackets, e.g. `[::]` or `['{']`. Alternatively, tokens can
/// be created with the syntax `[token_kind token_text]`, where `token_kind` is a variant of `SyntaxKind` (e.g.
/// `IDENT`) and `token_text` is an expression producing `String` or `&str`. Whitespaces can be added
/// as string literals (i.e. `"\n    "` is a whitespace token). Interpolation is allowed with `#` (`#variable`),
/// from `AstNode`s and `Option`s of them. Repetition is also supported, with only one repeating variable
/// and no separator (`#("\n" #variable [>])*`), for any `IntoIterator`. Note that `Option`s are also `IntoIterator`,
/// which can help when you want to conditionally include something along with an optional node.
///
/// There needs to be one root node, and its type is returned.
///
/// Be careful to closely match the Ungrammar AST, there is no validation for this!
macro_rules! quote_ {
    ( $root:ident { $($tree:tt)* } ) => {{
        #[allow(unused_mut)]
        let mut root = ::std::vec::Vec::<$crate::ast::make::quote::NodeOrToken<
            $crate::ast::make::quote::GreenNode,
            $crate::ast::make::quote::GreenToken,
        >>::with_capacity(1);
        $crate::ast::make::quote::quote_impl!( @append root $root { $($tree)* } );
        let root = root.into_iter().next().unwrap();
        let root = $crate::ast::make::quote::attach_trivia(&root.into_node().unwrap());
        let root = $crate::algo::strip_blank_edges(&$crate::SyntaxNode::new_root(root), true);
        <$crate::ast::$root as $crate::ast::AstNode>::cast(root).unwrap()
    }};
}
pub(crate) use quote_ as quote;

use std::mem;

use rowan::{GreenNodeData, GreenTokenData};

use super::tokens;
use crate::{AstNode, SyntaxKind, SyntaxKind::NEWLINE, algo::strip_blank_edges};

pub(crate) fn push_whitespace(children: &mut Vec<NodeOrToken<GreenNode, GreenToken>>, text: &str) {
    children.extend(
        tokens::trivia(text)
            .into_iter()
            .map(|(kind, text)| GreenToken::new(RSyntaxKind(kind as u16), text).into()),
    );
}

type Attached = (GreenToken, Vec<GreenToken>, Vec<GreenToken>);

fn kind(token: &GreenTokenData) -> SyntaxKind {
    SyntaxKind::from(token.kind().0)
}

fn collect(node: &GreenNodeData, tokens: &mut Vec<Attached>, leading: &mut Vec<GreenToken>) {
    for child in node.children() {
        match child {
            NodeOrToken::Node(node) => collect(node, tokens, leading),
            NodeOrToken::Token(token) if !kind(token).is_trivia() => {
                leading.extend(token.leading_trivia().iter().cloned());
                let trailing = token.trailing_trivia().to_vec();
                tokens.push((token.to_owned(), mem::take(leading), trailing));
            }
            NodeOrToken::Token(trivia) => match tokens.last_mut() {
                Some((_, _, trailing))
                    if !trailing.last().is_some_and(|it| kind(it) == NEWLINE) =>
                {
                    trailing.push(trivia.to_owned())
                }
                _ => leading.push(trivia.to_owned()),
            },
        }
    }
}

fn rebuild(node: &GreenNodeData, tokens: &mut impl Iterator<Item = GreenToken>) -> GreenNode {
    let children: Vec<_> = node
        .children()
        .filter_map(|child| match child {
            NodeOrToken::Node(node) => Some(rebuild(node, tokens).into()),
            NodeOrToken::Token(token) if kind(token).is_trivia() => None,
            NodeOrToken::Token(_) => tokens.next().map(Into::into),
        })
        .collect();
    GreenNode::new(node.kind(), children)
}

pub(crate) fn attach_trivia(root: &GreenNode) -> GreenNode {
    let (mut tokens, mut leading) = (Vec::new(), Vec::new());
    collect(root, &mut tokens, &mut leading);
    debug_assert!(leading.is_empty(), "trivia needs a following token");
    let mut tokens = tokens.into_iter().map(|(token, leading, trailing)| {
        GreenToken::with_trivia(token.kind(), token.text(), leading, trailing)
    });
    rebuild(root, &mut tokens)
}

pub(crate) trait ToNodeChild {
    fn append_node_child(self, children: &mut Vec<NodeOrToken<GreenNode, GreenToken>>);
}

impl<N: AstNode> ToNodeChild for N {
    fn append_node_child(self, children: &mut Vec<NodeOrToken<GreenNode, GreenToken>>) {
        children.push(strip_blank_edges(self.syntax(), false).green().to_owned().into());
    }
}

impl<C: ToNodeChild> ToNodeChild for Option<C> {
    fn append_node_child(self, children: &mut Vec<NodeOrToken<GreenNode, GreenToken>>) {
        if let Some(child) = self {
            child.append_node_child(children);
        }
    }
}

// This is useful when you want conditionally, based on some `bool`, to emit some code.
impl ToNodeChild for () {
    fn append_node_child(self, _children: &mut Vec<NodeOrToken<GreenNode, GreenToken>>) {}
}

pub(crate) const fn verify_only_whitespaces(text: &str) {
    let text = text.as_bytes();
    let mut i = 0;
    while i < text.len() {
        if !text[i].is_ascii_whitespace() {
            panic!("non-whitespace found in whitespace token");
        }
        i += 1;
    }
}
