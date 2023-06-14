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
    let ast = lower(&grammar);

    let ast_nodes = generate_vst(KINDS_SRC, &ast);
    let ast_nodes_file = sourcegen::project_root().join("crates/syntax/src/ast/generated/vst.rs");
    sourcegen::ensure_file_contents(ast_nodes_file.as_path(), &ast_nodes);
}

pub(crate) fn generate_vst(kinds: KindsSrc<'_>, grammar: &AstSrc) -> String {
    
    // generate struct definitions
    let node_defs:Vec<_> = grammar
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
                    if token_kind.to_string() == "T ! [ident]" ||
                        token_kind.to_string() == "T ! [int_number]" ||
                        token_kind.to_string() == "T ! [lifetime_ident]" 
                    {
                        quote! {
                            #name : String,
                        }
                    } else {
                        quote! {
                            #name : bool,
                        }
                    }
                } else {
                    // As source code can be incomplete, we use Option even if the field is not optional in ungrammar.
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

        }).collect_vec();


    // generate enum definitions
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
    })
    .collect_vec();


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
    // TODO: generate display impls
    // #(#display_impls)*

    sourcegen::add_preamble("sourcegen_vst", sourcegen::reformat(ast.to_string()))
}
