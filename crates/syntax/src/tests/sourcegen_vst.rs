//! This module generates VST datatype used by verus-analyzer.
//!
//! The VST datatype is generated from the ungrammar file.

use itertools::Itertools;
use quote::{format_ident, quote};
use crate::tests::ast_src::{
    AstSrc, KindsSrc, KINDS_SRC,
};

use crate::tests::sourcegen_ast::*;

// From sourcegen_ast::extract_struct_traits
const special_items: &[(&str, &[&str])] = &[
    ("HasAttrs", &["attrs"]),
    ("HasName", &["name"]),
    ("HasVisibility", &["visibility"]),
    ("HasGenericParams", &["generic_param_list", "where_clause"]),
    ("HasTypeBounds", &["type_bound_list", "colon_token"]),
    ("HasModuleItem", &["items"]),
    ("HasLoopBody", &["label", "loop_body"]),
    ("HasArgList", &["arg_list"]),
];

#[test]
fn sourcegen_vst() {
    let grammar =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/rust.ungram")).parse().unwrap();
    let ast = lower(&grammar, true);

    let ast_nodes = generate_vst(KINDS_SRC, &ast);
    let ast_nodes_file = sourcegen::project_root().join("crates/syntax/src/ast/generated/vst_nodes.rs");
    sourcegen::ensure_file_contents(ast_nodes_file.as_path(), &ast_nodes);
}

pub(crate) fn generate_vst(kinds: KindsSrc<'_>, grammar: &AstSrc) -> String {
    // TODO: add "Comment" item

    // generate struct definitions
    let node_defs: Vec<_> = grammar
        .nodes
        .iter()
        .map(|node| {
            let name = format_ident!("{}", node.name);
            let fields = node.fields.iter().map(|field| {
                let name = field.method_name();
                let ty = field.ty();

                if field.is_many() {
                    quote! {
                        pub #name : Vec<#ty>,
                    }
                } else if let Some(token_kind) = field.token_kind() {
                    // hacky for now
                    // maybe special-case identifier to "#name : Option<String>"
                    // 'ident, 'int_number', and 'lifetime_ident'.
                    if token_kind.to_string() == "T ! [ident]"
                        || token_kind.to_string() == "T ! [int_number]"
                        || token_kind.to_string() == "T ! [lifetime_ident]"
                    {
                        quote! {
                            #name : Option<String>,
                        }
                    } else {
                        quote! {
                            #name : bool,
                        }
                    }
                } else {
                    // As source code can be incomplete, we use Option even if the field is not optional in ungrammar.
                    // TODO:
                    // As source code can be incomplete, we use might use `Option` even if the field is not optional in ungrammar.
                    // instead, however, since proof action might choose to be available when syntax is complete
                    // therefore, we do not use `Option` for VST.
                    // we only use `Option` when the syntax item is optional in ungrammar.
                    quote! {
                        pub #name : Option<Box<#ty>>,
                    }
                }
            });

            quote! {
                #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                pub struct #name {
                    #(#fields)*
                }
            }
        })
        .collect_vec();

    // CST -> VST
    // impl From (eventually `TryFrom` to remove all the options around every fields) for each node
    let from_node_to_vnode_struct: Vec<_> = grammar
        .nodes
        .iter()
        .map(|node| {
            let name = format_ident!("{}", node.name);
            let fields = node.fields.iter().map(|field| {
                let name = field.method_name();
                let ty = field.ty();

                if field.is_many() {
                    quote! {
                        #name : item.#name().into_iter().map(#ty::from).collect(),
                    }
                } else if let Some(token_kind) = field.token_kind() {
                    // hacky for now
                    // maybe special-case identifier to "#name : Option<String>"
                    // 'ident, 'int_number', and 'lifetime_ident'.
                    if token_kind.to_string() == "T ! [ident]"
                        || token_kind.to_string() == "T ! [int_number]"
                        || token_kind.to_string() == "T ! [lifetime_ident]"
                    {
                        // #name : Option<String>,
                        quote! {
                            #name : item.#name().map(|it| it.text().to_string()),
                        }
                    } else {
                        // #name : bool,
                        quote! {
                            #name : item.#name().is_some(),
                        }
                    }
                } else {
                    // pub #name : Option<Box<#ty>>,
                    quote! {
                        #name : item.#name().map(#ty::from).map(Box::new),
                    }
                }
            });

            quote! {
                impl From<super::nodes::#name> for #name {
                    fn from(item: super::nodes::#name) -> Self {
                        Self {
                            #(#fields)*
                        }
                    }
                }
            }
        })
        .collect_vec();

    // generate enum definitions
    let enum_defs: Vec<_> = grammar
        .enums
        .iter()
        .map(|en| {
            let variants: Vec<_> = en.variants.iter().map(|var| format_ident!("{}", var)).collect();
            let name = format_ident!("{}", en.name);
            let kinds: Vec<_> = variants
                .iter()
                .map(|name| format_ident!("{}", to_upper_snake_case(&name.to_string())))
                .collect();

            let traits = en.traits.iter().map(|trait_name| {
                let trait_name = format_ident!("{}", trait_name);
                quote!(impl ast::#trait_name for #name {})
            });

            quote! {
                #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                pub enum #name {
                    #(#variants(Box<#variants>),)*
                }
            }
        })
        .collect_vec();

    // CST to VST
    let from_node_to_vnode_enum:  Vec<_> = grammar
    .enums
    .iter()
    .map(|en| {
        let variants: Vec<_> = en.variants.iter().map(|var| format_ident!("{}", var)).collect();
        let name = format_ident!("{}", en.name);
        let kinds: Vec<_> = variants
            .iter()
            .map(|name| format_ident!("{}", to_upper_snake_case(&name.to_string())))
            .collect();
        
        let traits = en.traits.iter().map(|trait_name| {
            let trait_name = format_ident!("{}", trait_name);
            quote!(impl ast::#trait_name for #name {})
        });

        quote! {
            impl From<super::nodes::#name> for #name {
                fn from(item: super::nodes::#name) -> Self {
                    match item {
                        #(
                            super::nodes::#name::#variants(it) => Self::#variants(Box::new(it.into())),
                        )*
                    }
                }
            }
        }  
    })
    .collect_vec();

    let ast = quote! {
        #![allow(non_snake_case)]
        use crate::{
            SyntaxNode, SyntaxToken, SyntaxKind::{self, *},
            ast::{self, AstNode, AstChildren, support, traits::*},
            T,
        };

        #(#node_defs)*
        #(#enum_defs)*
        #(#from_node_to_vnode_struct)*
        #(#from_node_to_vnode_enum)*
    };

    // TODO: expr_ext
    // this file contains manual `impl`s that are not auto-generated.
    // VST should have all corresponding `impl`s

    // VST -> CST
    // TODO: generate display impls (this is to print VST and parse into CST)
    // #(#display_impls)*

    sourcegen::add_preamble("sourcegen_vst", sourcegen::reformat(ast.to_string()))
}

/*
below stuff are removed in "sourcege_ast" with "remove_field"
through "extract_struct_traits"

("HasAttrs", &["attrs"]),
("HasName", &["name"]),
("HasVisibility", &["visibility"]),
("HasGenericParams", &["generic_param_list", "where_clause"]),
("HasTypeBounds", &["type_bound_list", "colon_token"]),
("HasModuleItem", &["items"]),
("HasLoopBody", &["label", "loop_body"]),
("HasArgList", &["arg_list"]),
 */

/*
impl From<super::nodes::AssertExpr> for AssertExpr {
    fn from(item: super::nodes::AssertExpr) -> Self {
        Self {
            assert_token: item.assert_token().is_some(),
            l_paren_token: item.l_paren_token().is_some(),
            expr: item.expr().map(Expr::from).map(Box::new),
            r_paren_token: item.r_paren_token().is_some(),
            by_token: item.by_token().is_some(),
            requires_clause: item.requires_clause().map(RequiresClause::from).map(Box::new),
            block_expr: item.block_expr().map(BlockExpr::from).map(Box::new),
        }
    }
}

impl From<super::nodes::Name> for Name {
    fn from(item: super::nodes::Name) -> Self {
        Self {
            ident_token: item.ident_token().map(|it| it.text().to_string()),
            self_token: item.self_token().is_some(),
        }
    }
}

impl TryFrom<super::nodes::AssertExpr> for AssertExpr {
    type Error = ();

    fn try_from(item: super::nodes::AssertExpr) -> Result<Self, Self::Error> {
        let res = Self {
            assert_token: item.assert_token().is_some(),
            l_paren_token: item.l_paren_token().is_some(),
            expr: Some(Box::new(item.expr().try_into()?)),
            r_paren_token: item.r_paren_token().is_some(),
            by_token: item.by_token().is_some(),
            requires_clause: item.requires_clause().map(RequiresClause::try_from).map(Box::new),
            block_expr: item.block_expr.map(Box::new),
        };
        Ok(res)
    }
}
 */
