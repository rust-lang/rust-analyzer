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
        let root = $crate::SyntaxNode::new_root(root);
        <$crate::ast::$root as $crate::ast::AstNode>::cast(root).unwrap()
    }};
}
pub(crate) use quote_ as quote;

use crate::{AstNode, SyntaxKind, SyntaxKind::NEWLINE};

pub(crate) fn push_whitespace(children: &mut Vec<NodeOrToken<GreenNode, GreenToken>>, text: &str) {
    let trivia = crate::ast::make::tokens::trivia(text);
    children.extend(
        trivia
            .into_iter()
            .map(|(kind, text)| GreenToken::new(RSyntaxKind(kind as u16), text).into()),
    );
}

enum Piece {
    Enter(RSyntaxKind),
    Exit,
    Token(GreenToken, Vec<GreenToken>, Vec<GreenToken>),
}

fn kind(token: &rowan::GreenTokenData) -> SyntaxKind {
    SyntaxKind::from(token.kind().0)
}

fn flatten(node: &rowan::GreenNodeData, pieces: &mut Vec<Piece>, leading: &mut Vec<GreenToken>) {
    pieces.push(Piece::Enter(node.kind()));
    for child in node.children() {
        match child {
            NodeOrToken::Node(node) => flatten(node, pieces, leading),
            NodeOrToken::Token(token) if !kind(token).is_trivia() => {
                leading.extend(token.leading_trivia().iter().cloned());
                let trailing = token.trailing_trivia().to_vec();
                pieces.push(Piece::Token(token.to_owned(), std::mem::take(leading), trailing));
            }
            NodeOrToken::Token(trivia) => {
                let open = pieces.iter_mut().rev().find_map(|piece| match piece {
                    Piece::Token(_, _, trailing) => Some(trailing),
                    _ => None,
                });
                match open {
                    Some(trailing) if !trailing.last().is_some_and(|it| kind(it) == NEWLINE) => {
                        trailing.push(trivia.to_owned())
                    }
                    _ => leading.push(trivia.to_owned()),
                }
            }
        }
    }
    pieces.push(Piece::Exit);
}

pub(crate) fn attach_trivia(root: &GreenNode) -> GreenNode {
    let (mut pieces, mut leading) = (Vec::new(), Vec::new());
    flatten(root, &mut pieces, &mut leading);
    debug_assert!(leading.is_empty(), "trivia needs a following token");

    let mut frames: Vec<(RSyntaxKind, Vec<NodeOrToken<GreenNode, GreenToken>>)> = Vec::new();
    for piece in pieces {
        match piece {
            Piece::Enter(kind) => frames.push((kind, Vec::new())),
            Piece::Token(token, leading, trailing) => {
                let token = GreenToken::with_trivia(token.kind(), token.text(), leading, trailing);
                frames.last_mut().expect("a token outside any node").1.push(token.into());
            }
            Piece::Exit => {
                let (kind, children) = frames.pop().expect("an exit for every enter");
                let node = GreenNode::new(kind, children);
                match frames.last_mut() {
                    Some((_, parent)) => parent.push(node.into()),
                    None => return node,
                }
            }
        }
    }
    unreachable!("the root exits last")
}

pub(crate) trait ToNodeChild {
    fn append_node_child(self, children: &mut Vec<NodeOrToken<GreenNode, GreenToken>>);
}

impl<N: AstNode> ToNodeChild for N {
    fn append_node_child(self, children: &mut Vec<NodeOrToken<GreenNode, GreenToken>>) {
        let detached = crate::algo::strip_outer_blank_trivia(self.syntax());
        children.push(detached.green().to_owned().into());
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
