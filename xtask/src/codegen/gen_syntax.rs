//! This module generate AST datatype used by rust-analyzer.
//!
//! Specifically, it generates the `SyntaxKind` enum and a number of newtype
//! wrappers around `SyntaxNode` which implement `ra_syntax::AstNode`.

use std::collections::HashSet;
use std::fs;

use proc_macro2::{Punct, Spacing};
use quote::{format_ident, quote, ToTokens};
use ron;
use serde::Deserialize;

use crate::{
    codegen::{self, update, Mode},
    project_root, Result,
};

use asdl::*;

const PROVIDED_TYPES: [&'static str; 1] = ["commentIter"];

struct AstStruct {
    asdl: Asdl,
    punct: HashSet<String>,
    keywords: HashSet<String>,
}

impl AstStruct {
    pub fn is_token(&self, type_id: &str) -> bool {
        self.punct.contains(type_id) || self.keywords.contains(type_id)
    }
}

pub fn generate_syntax(mode: Mode) -> Result<()> {
    let grammar = project_root().join(codegen::GRAMMAR);
    let grammar: Grammar = {
        let text = fs::read_to_string(grammar)?;
        ron::de::from_str(&text)?
    };

    let syntax_kinds_file = project_root().join(codegen::SYNTAX_KINDS);
    let syntax_kinds = generate_syntax_kinds(&grammar)?;
    update(syntax_kinds_file.as_path(), &syntax_kinds, mode)?;

    let asdl_file = project_root().join(codegen::ASDL);
    let asdl = Asdl::parse(&fs::read_to_string(asdl_file)?)?;
    let punct = grammar.punct.iter().map(|(_, p)| p.to_ascii_lowercase()).collect();
    let keywords = grammar.keywords.iter().map(|kw| format!("{}_kw", kw)).collect();
    let ast_struct = AstStruct { asdl, punct, keywords };
    let ast = generate_ast(&ast_struct)?;
    let ast_file = project_root().join(codegen::AST);
    update(ast_file.as_path(), &ast, mode)?;

    Ok(())
}

fn generate_sum_type(sty: &SumType, ast_struct: &AstStruct) -> impl ToTokens {
    let name = format_ident!("{}", capitalize(&sty.id));
    let variants = sty.constructors.iter().map(|c| format_ident!("{}", c.id)).collect::<Vec<_>>();
    let variants_builders =
        sty.constructors.iter().map(|c| format_ident!("{}Builder", c.id)).collect::<Vec<_>>();
    let kinds = variants
        .iter()
        .map(|name| format_ident!("{}", to_upper_snake_case(&name.to_string())))
        .collect::<Vec<_>>();

    let constructors = sty
        .constructors
        .iter()
        .filter(|c| {
            // do not generate special type if constructor has only one required field
            c.fields.is_empty()
                || c.fields.len() > 1
                || c.fields[0].arity != Arity::Required
                || ast_struct.is_token(&c.fields[0].type_id)
        })
        .map(|c| generate_type(&c.id, &c.fields, ast_struct));

    let attributes = generate_fields(&sty.id, &sty.attributes, ast_struct);

    let builder_name = format_ident!("{}Builder", capitalize(&sty.id));

    quote! {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum #name {
            #(#variants(#variants),)*
        }


        #(
        impl From<#variants> for #name {
            fn from(node: #variants) -> #name {
                #name::#variants(node)
            }
        }
        )*

        impl AstNode for #name {
            fn can_cast(kind: SyntaxKind) -> bool {
                match kind {
                    #(#kinds)|* => true,
                    _ => false,
                }
            }
            fn cast(syntax: SyntaxNode) -> Option<Self> {
                let res = match syntax.kind() {
                    #(
                    #kinds => #name::#variants(#variants { syntax }),
                    )*
                    _ => return None,
                };
                Some(res)
            }
            fn syntax(&self) -> &SyntaxNode {
                match self {
                    #(
                    #name::#variants(it) => &it.syntax,
                    )*
                }
            }
        }

        #attributes

        pub enum #builder_name {
            #(#variants_builders(Box<#variants_builders>),)*
        }


        #(
        impl From<#variants_builders> for #builder_name {
            fn from(builder: #variants_builders) -> #builder_name {
                #builder_name::#variants_builders(Box::new(builder))
            }
        }
        )*

        impl AstNodeBuilder for #builder_name {
            type Node = #name;
            fn make(self, builder: &mut SyntaxTreeBuilder) {
                match self {
                    #(
                    #builder_name::#variants_builders(b) => b.make(builder),
                    )*
                }
            }
        }

        #(#constructors)*
    }
}

fn generate_type(name: &str, fields: &Vec<Field>, ast_struct: &AstStruct) -> impl ToTokens {
    let type_name = format_ident!("{}", capitalize(name));
    let kind = format_ident!("{}", to_upper_snake_case(name));
    let gen_fields = generate_fields(name, fields, ast_struct);
    let builder = generate_builder(name, fields, ast_struct);
    quote! {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct #type_name {
            pub(crate) syntax: SyntaxNode,
        }

        impl AstNode for #type_name {
            fn can_cast(kind: SyntaxKind) -> bool {
                match kind {
                    #kind => true,
                    _ => false,
                }
            }
            fn cast(syntax: SyntaxNode) -> Option<Self> {
                if Self::can_cast(syntax.kind()) { Some(Self { syntax }) } else { None }
            }
            fn syntax(&self) -> &SyntaxNode { &self.syntax }
        }

        #gen_fields

        #builder
    }
}

fn generate_builder(name: &str, fields: &Vec<Field>, ast_struct: &AstStruct) -> impl ToTokens {
    if fields.is_empty() {
        return quote! {};
    }
    let type_name = format_ident!("{}", capitalize(name));
    let kind = format_ident!("{}", to_upper_snake_case(name));
    let builder_name = format_ident!("{}Builder", capitalize(name));
    let builder_fields = fields
        .iter()
        .filter(|f| !ast_struct.is_token(&f.type_id) && !is_provided_type(&f.type_id))
        .map(|f| {
            let field_name = format_ident!("{}", to_lower_snake_case(&f.id));
            let ty = format_ident!("{}Builder", capitalize(&f.type_id));
            match f.arity {
                Arity::Optional | Arity::Required => {
                    quote! {
                        #field_name: Option<Box<#ty>>,
                    }
                }
                Arity::Repeated => {
                    quote! {
                        #field_name: Vec<Box<#ty>>,
                    }
                }
            }
        });
    let builder_methods = fields
        .iter()
        .filter(|f| !ast_struct.is_token(&f.type_id) && !is_provided_type(&f.type_id))
        .map(|f| {
            let method_name = format_ident!("{}", depluralize(to_lower_snake_case(&f.id).as_str()));
            let field_name = format_ident!("{}", to_lower_snake_case(&f.id));
            let ty = format_ident!("{}Builder", capitalize(&f.type_id));
            match f.arity {
                Arity::Optional | Arity::Required => {
                    quote! {
                        pub fn #method_name(mut self,  f: #ty) -> Self {
                            self.#field_name = Some(Box::new(f));
                            self
                        }
                    }
                }
                Arity::Repeated => {
                    quote! {
                        pub fn #method_name(mut self,  f: #ty) -> Self {
                            self.#field_name.push(Box::new(f));
                            self
                        }
                    }
                }
            }
        });

    // If token filed follows the child node field they have the same arity we will process them as one tuple.
    // This allows to specify delimiter tokens for repeated children in ASDL.
    let mut tuples: Vec<(&Field, Option<&Field>)> = Vec::new();
    let mut i = 0;
    while i < fields.len() {
        let curr = fields.get(i).unwrap();
        if is_provided_type(&curr.type_id) {
            i += 1;
            continue;
        }
        if let Some(next) = fields.get(i + 1) {
            if ast_struct.is_token(&next.type_id)
                && !ast_struct.is_token(&curr.type_id)
                && curr.arity == next.arity
            {
                tuples.push((curr, Some(next)));
                i += 2;
            } else {
                tuples.push((curr, None));
                i += 1;
            }
        } else {
            tuples.push((curr, None));
            i += 1;
        }
    }
    let make_contents = tuples.iter().map(|(f, t)| {
        let field_name = format_ident!("{}", to_lower_snake_case(&f.id));
        match f.arity {
            Arity::Optional => {
                quote! {
                    if let Some(b) = self.#field_name {
                        b.make(builder);
                    }
                }
            }
            Arity::Repeated => {
                if let Some(token) = t {
                    let token_kind = format_ident!("{}", to_upper_snake_case(&token.id));
                    quote! {
                        for b in self.#field_name {
                            b.make(builder);
                            builder.token(SyntaxKind::#token_kind, SmolStr::new(T_STR!(#token_kind)));
                        }
                    }
                } else {
                    quote! {
                        self.#field_name.into_iter().for_each(|b| b.make(builder));
                    }
                }
            }
            Arity::Required => {
                if ast_struct.is_token(&f.id) {
                    let token_kind = format_ident!("{}", to_upper_snake_case(&f.id));
                    quote! {
                        builder.token(SyntaxKind::#token_kind, SmolStr::new(T_STR!(#token_kind)));
                    }
                } else {
                    quote! {
                        self.#field_name.unwrap().make(builder);
                    }
                }
            }
        }
    });
    quote! {

        impl #type_name {
            pub fn new() -> #builder_name {
                #builder_name::default()
            }
        }

        #[derive(Default)]
        pub struct #builder_name {
            #(#builder_fields)*
        }

        impl #builder_name {
            #(#builder_methods)*
        }

        impl AstNodeBuilder for #builder_name {
            type Node = #type_name;

            fn make(self, builder: &mut SyntaxTreeBuilder) {
                builder.start_node(SyntaxKind::#kind);
                #(#make_contents)*
                builder.finish_node();
            }
        }
    }
}

fn generate_fields(name: &str, fields: &Vec<Field>, ast_struct: &AstStruct) -> impl ToTokens {
    if fields.is_empty() {
        quote! {}
    } else {
        let type_name = format_ident!("{}", capitalize(name));
        let methods = fields.iter().filter(|f| !ast_struct.is_token(&f.type_id)).map(|f| {
            let method_name = format_ident!("{}", to_lower_snake_case(&f.id));
            let ty = format_ident!("{}", capitalize(&f.type_id));
            match f.arity {
                Arity::Optional => {
                    quote! {
                        pub fn #method_name(&self) -> Option<#ty> {
                           super::child_opt(self)
                        }
                    }
                }
                Arity::Repeated => {
                    quote! {
                        pub fn #method_name(&self) -> AstChildren<#ty> {
                            super::children(self)
                        }
                    }
                }
                Arity::Required => {
                    if is_provided_type(&f.type_id.as_str()) {
                        quote! {
                            pub fn #method_name(&self) -> #ty {
                                #ty::new(self.syntax().children_with_tokens())
                            }
                        }
                    } else {
                        quote! {
                            // not implemented generation for required field #method_name and type #ty
                        }
                    }
                }
            }
        });
        let traits = generate_traits(name, fields, ast_struct);
        quote! {
            impl #type_name {
                #(#methods)*
            }

            #traits
        }
    }
}

fn is_provided_type(type_id: &str) -> bool {
    PROVIDED_TYPES.contains(&type_id)
}

fn generate_traits(name: &str, fields: &Vec<Field>, ast_struct: &AstStruct) -> impl ToTokens {
    let type_name = format_ident!("{}", capitalize(name));
    let inf_traits = infer_traits(fields, &ast_struct.asdl);
    let traits = inf_traits.iter().map(|trait_name| {
        if let Some(tr) = ast_struct.asdl.get_type_by_name(trait_name).and_then(to_prod_type) {
            let trait_methods = tr.fields.iter().map(|f| {
                let method_name = format_ident!("{}", to_lower_snake_case(&f.id));
                let ty = format_ident!("{}", capitalize(&f.type_id));
                match f.arity {
                    Arity::Optional => {
                        quote! {
                            fn #method_name(&self) -> Option<#ty> {
                                 self.#method_name()
                            }
                        }
                    }
                    Arity::Repeated => {
                        quote! {
                            fn #method_name(&self) -> AstChildren<#ty> {
                                 self.#method_name()
                            }
                        }
                    }
                    Arity::Required => {
                        quote! {
                            fn #method_name(&self) -> #ty {
                                 self.#method_name()
                            }
                        }
                    }
                }
            });
            let trait_name = format_ident!("{}", capitalize(trait_name));
            quote! {
                impl ast::#trait_name for #type_name {
                    #(#trait_methods)*
                }
            }
        } else {
            quote! {
                // can't get methods for trait #trait_name
            }
        }
    });
    quote! {
        #(#traits)*
    }
}

fn generate_ast(ast_struct: &AstStruct) -> Result<String> {
    let sum_types = ast_struct
        .asdl
        .types
        .iter()
        .filter_map(to_sum_type)
        .map(|sty| generate_sum_type(&sty, ast_struct));
    let prod_types = ast_struct
        .asdl
        .types
        .iter()
        .filter_map(to_prod_type)
        .filter(|ty| !is_trait(ty))
        .map(|pty| generate_type(&pty.id, &pty.fields, ast_struct));

    let ast = quote! {
        use crate::{
            SyntaxTreeBuilder, SyntaxNode, SyntaxKind::{self, *}, T_STR, SmolStr,
            ast::{self, AstNode, AstChildren, traits::CommentIter, builders::*},
        };

        #(#sum_types)*
        #(#prod_types)*
    };

    let pretty = codegen::reformat(ast)?;
    Ok(pretty)
}

fn is_trait(ty: &ProdType) -> bool {
    ty.id.ends_with("Owner")
}

fn to_prod_type(ty: &Type) -> Option<&ProdType> {
    match ty {
        Type::ProdType(pty) => Some(pty),
        _ => None,
    }
}

fn to_sum_type(ty: &Type) -> Option<&SumType> {
    match ty {
        Type::SumType(sty) => Some(sty),
        _ => None,
    }
}

fn infer_traits(fields: &Vec<Field>, asdl: &Asdl) -> Vec<String> {
    asdl.types
        .iter()
        .filter_map(to_prod_type)
        .filter(|t| is_trait(*t))
        .filter(|tr| contains_fields(fields, &tr.fields))
        .map(|tr| tr.id.clone())
        .collect()
}

fn contains_fields(outer: &Vec<Field>, inner: &Vec<Field>) -> bool {
    inner.iter().filter(|f| outer.contains(f)).count() == inner.len()
}

fn generate_syntax_kinds(grammar: &Grammar) -> Result<String> {
    let (single_byte_tokens_values, single_byte_tokens): (Vec<_>, Vec<_>) = grammar
        .punct
        .iter()
        .filter(|(token, _name)| token.len() == 1)
        .map(|(token, name)| (token.chars().next().unwrap(), format_ident!("{}", name)))
        .unzip();

    let punctuation_values = grammar.punct.iter().map(|(token, _name)| {
        if "{}[]()".contains(token) {
            let c = token.chars().next().unwrap();
            quote! { #c }
        } else {
            let cs = token.chars().map(|c| Punct::new(c, Spacing::Joint));
            quote! { #(#cs)* }
        }
    });
    let punctuation_values_str =
        grammar.punct.iter().map(|(token, _name)| token).collect::<Vec<_>>();

    let punctuation =
        grammar.punct.iter().map(|(_token, name)| format_ident!("{}", name)).collect::<Vec<_>>();

    let full_keywords_values = &grammar.keywords;
    let full_keywords = full_keywords_values
        .iter()
        .map(|kw| format_ident!("{}_KW", to_upper_snake_case(&kw)))
        .collect::<Vec<_>>();

    let all_keywords_values =
        grammar.keywords.iter().chain(grammar.contextual_keywords.iter()).collect::<Vec<_>>();
    let all_keywords_idents = all_keywords_values.iter().map(|kw| format_ident!("{}", kw));
    let all_keywords = all_keywords_values
        .iter()
        .map(|name| format_ident!("{}_KW", to_upper_snake_case(&name)))
        .collect::<Vec<_>>();

    let literals =
        grammar.literals.iter().map(|name| format_ident!("{}", name)).collect::<Vec<_>>();

    let tokens = grammar.tokens.iter().map(|name| format_ident!("{}", name)).collect::<Vec<_>>();

    let nodes = grammar.nodes.iter().map(|name| format_ident!("{}", name)).collect::<Vec<_>>();

    let ast = quote! {
        #![allow(bad_style, missing_docs, unreachable_pub)]
        /// The kind of syntax node, e.g. `IDENT`, `USE_KW`, or `STRUCT_DEF`.
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        #[repr(u16)]
        pub enum SyntaxKind {
            // Technical SyntaxKinds: they appear temporally during parsing,
            // but never end up in the final tree
            #[doc(hidden)]
            TOMBSTONE,
            #[doc(hidden)]
            EOF,
            #(#punctuation,)*
            #(#all_keywords,)*
            #(#literals,)*
            #(#tokens,)*
            #(#nodes,)*

            // Technical kind so that we can cast from u16 safely
            #[doc(hidden)]
            __LAST,
        }
        use self::SyntaxKind::*;

        impl SyntaxKind {
            pub fn is_keyword(self) -> bool {
                match self {
                    #(#all_keywords)|* => true,
                    _ => false,
                }
            }

            pub fn is_punct(self) -> bool {
                match self {
                    #(#punctuation)|* => true,
                    _ => false,
                }
            }

            pub fn is_literal(self) -> bool {
                match self {
                    #(#literals)|* => true,
                    _ => false,
                }
            }

            pub fn from_keyword(ident: &str) -> Option<SyntaxKind> {
                let kw = match ident {
                    #(#full_keywords_values => #full_keywords,)*
                    _ => return None,
                };
                Some(kw)
            }

            pub fn from_char(c: char) -> Option<SyntaxKind> {
                let tok = match c {
                    #(#single_byte_tokens_values => #single_byte_tokens,)*
                    _ => return None,
                };
                Some(tok)
            }
        }

        #[macro_export]
        macro_rules! T {
            #((#punctuation_values) => { $crate::SyntaxKind::#punctuation };)*
            #((#all_keywords_idents) => { $crate::SyntaxKind::#all_keywords };)*
        }

        #[macro_export]
        macro_rules! T_STR {
            #((#punctuation) => { #punctuation_values_str };)*
            #((#all_keywords) => { #all_keywords_values };)*
        }
    };

    codegen::reformat(ast)
}

#[derive(Deserialize, Debug)]
struct Grammar {
    punct: Vec<(String, String)>,
    keywords: Vec<String>,
    contextual_keywords: Vec<String>,
    literals: Vec<String>,
    tokens: Vec<String>,
    nodes: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Attr {
    Type(String),
    NameType(String, String),
}

fn to_upper_snake_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev_is_upper = None;
    for c in s.chars() {
        if c.is_ascii_uppercase() && prev_is_upper == Some(false) {
            buf.push('_')
        }
        prev_is_upper = Some(c.is_ascii_uppercase());

        buf.push(c.to_ascii_uppercase());
    }
    buf
}

fn to_lower_snake_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev_is_upper = None;
    for c in s.chars() {
        if c.is_ascii_uppercase() && prev_is_upper == Some(false) {
            buf.push('_')
        }
        prev_is_upper = Some(c.is_ascii_uppercase());

        buf.push(c.to_ascii_lowercase());
    }
    buf
}

fn depluralize(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    if s.ends_with("ies") {
        buf.push_str(&s[0..s.len() - 3]);
        buf.push('y');
    } else if s.ends_with("s") {
        buf.push_str(&s[0..s.len() - 1]);
    } else {
        buf.push_str(s);
    }
    buf
}

fn capitalize(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    buf.push(s.chars().next().unwrap().to_ascii_uppercase());
    buf.push_str(&s[1..]);
    buf
}
