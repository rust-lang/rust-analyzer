//! This module generates VST datatype used by verus-analyzer.
//!
//! The VST datatype is generated from the ungrammar file.

use std::{
    collections::{BTreeSet, HashSet},
    fmt::Write,
};

use itertools::Itertools;
use proc_macro2::{Punct, Spacing};
use quote::{format_ident, quote};
use ungrammar::{Grammar, Rule};

use crate::tests::ast_src::{
    AstEnumSrc, AstNodeSrc, AstSrc, Cardinality, Field, KindsSrc, KINDS_SRC,
};

use crate::tests::sourcegen_ast::*;

#[test]
fn sourcegen_vst() {
    let grammar =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/rust.ungram")).parse().unwrap();
    // dbg!(&grammar);
    let ast = lower(&grammar);
    dbg!(&ast);

    let ast_nodes = generate_vst(KINDS_SRC, &ast);
    let ast_nodes_file = sourcegen::project_root().join("crates/syntax/src/ast/generated/vst.rs");
    sourcegen::ensure_file_contents(ast_nodes_file.as_path(), &ast_nodes);
}

pub(crate) fn generate_vst(kinds: KindsSrc<'_>, grammar: &AstSrc) -> String {
    let node_defs:Vec<_> = grammar
        .nodes
        .iter()
        .map(|node| {
            let name = format_ident!("{}", node.name);
            let kind = format_ident!("{}", to_upper_snake_case(&node.name));

            let methods = node.fields.iter().map(|field| {
                let name = field.method_name();
                let ty = field.ty();

                if field.is_many() {
                    quote! {
                        pub #name : Vec<#ty>,
                    }
                } else if let Some(token_kind) = field.token_kind() {
                    // ignore token
                    quote! {}
                    // quote! {
                    //     #name : #token_kind,
                    // }
                } else {
                    quote! {
                        pub #name : Option<Box<#ty>>,
                    }
                }
            });
            
            quote! {
                #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                pub struct #name {
                    #(#methods)*
                }
            }

        }).collect_vec();

        let enum_defs:  Vec<_> = grammar
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
            // qote! {
            //     #(
            //         impl From<#variants> for #name {
            //             fn from(node: #variants) -> #name {
            //                 #name::#variants(node)
            //             }
            //         }
            //     )*
            //     #ast_node
            // },
            
        })
        .collect_vec();

    dbg!(&node_defs);
    dbg!(&enum_defs);

    let ast = quote! {
        #![allow(non_snake_case)]
        use crate::{
            SyntaxNode, SyntaxToken, SyntaxKind::{self, *},
            ast::{self, AstNode, AstChildren, support},
            T,
        };

        #(#node_defs)*
        #(#enum_defs)*
    };
    // #(#any_node_defs)*
    // #(#node_boilerplate_impls)*
    // #(#enum_boilerplate_impls)*
    // #(#any_node_boilerplate_impls)*
    // #(#display_impls)*

    // let structs = node_defs.into_iter().map(|it| it.to_string()).join("\n\n");
    // let enums = enum_defs.into_iter().map(|it| it.to_string()).join("\n\n");

    sourcegen::add_preamble("sourcegen_vst", sourcegen::reformat(ast.to_string()))
}
