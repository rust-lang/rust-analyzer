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
    fn type_by_name(&self, type_name: &str) -> AstType {
        if is_provided_type(type_name) {
            AstType::Provided
        } else if self.is_token(type_name) {
            AstType::Token
        } else {
            if let Some(ty) = self.asdl.get_type_by_name(type_name) {
                self.type_of(ty)
            } else {
                AstType::Undefined
            }
        }
    }

    fn type_of(&self, ty: &asdl::Type) -> AstType {
        match ty {
            Type::SumType(sty) => {
                if self.is_token_set(sty) {
                    AstType::TokenSet
                } else {
                    AstType::Node
                }
            }
            Type::ProdType(pty) => {
                if Self::is_trait(pty) {
                    AstType::Trait
                } else {
                    AstType::Node
                }
            }
        }
    }

    // fn types(&self) -> impl Iterator<Item = (&asdl::Type, AstType)> {
    //     self.asdl
    //         .types
    //         .iter()
    //         .map(|ty| (ty, self.type_of(ty)))
    //         .collect::<Vec<(&asdl::Type, AstType)>>()
    //         .into_iter()
    // }

    pub fn is_token(&self, type_id: &str) -> bool {
        self.punct.contains(type_id) || self.keywords.contains(type_id)
    }

    fn is_token_set(&self, ty: &SumType) -> bool {
        ty.constructors.iter().all(|c| {
            if c.fields.len() == 1 {
                let f = c.fields.get(0).unwrap();
                f.arity == asdl::Arity::Required && self.is_token(&f.type_id)
            } else {
                false
            }
        })
    }

    fn infer_traits(&self, fields: &[FieldAndType]) -> Vec<&ProdType> {
        let fields = fields.iter().map(|ft| ft.field).collect::<Vec<&Field>>();
        self.asdl
            .types
            .iter()
            .filter_map(to_prod_type)
            .filter(|t| Self::is_trait(*t))
            .filter(|tr| fields.iter().any(|f| tr.fields.iter().any(|ff| ff == *f)))
            .collect()
    }

    fn is_trait(ty: &ProdType) -> bool {
        ty.id.ends_with("Owner")
    }
}

#[derive(Eq, PartialEq)]
enum AstType {
    Node,
    Token,
    TokenSet,
    Provided,
    Trait,
    Undefined,
}

struct FieldAndType<'a> {
    field: &'a Field,
    ty: AstType,
}

impl<'a> FieldAndType<'a> {
    fn cast(&self, ty: AstType) -> Option<&Field> {
        if self.ty == ty {
            Some(self.field)
        } else {
            None
        }
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

fn generate_token_set(sty: &SumType, _ast_struct: &AstStruct) -> impl ToTokens {
    let name = format_ident!("{}", capitalize(&sty.id));
    let variants = sty.constructors.iter().map(|c| format_ident!("{}", c.id)).collect::<Vec<_>>();
    let kinds = sty
        .constructors
        .iter()
        .map(|c| format_ident!("{}", to_upper_snake_case(&c.fields.get(0).unwrap().type_id)))
        .collect::<Vec<_>>();
    quote! {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub enum #name {
             #(#variants,)*
        }

        impl #name {
            fn from_token(t: &SyntaxToken) -> Option<#name> {
                match t.kind() {
                   #(#kinds => Some(#name::#variants),)*
                    _ => return None,
                }
            }
        }

        impl AstMake for #name {
            type I = Self;
            fn make(self, builder: &mut SyntaxTreeBuilder) {
                let (kind, token) = match self {
                     #(#name::#variants => (#kinds, T_STR!(#kinds)),)*
                };
                builder.token(kind, SmolStr::new(token));
            }
        }
    }
}

fn generate_sum_type(sty: &SumType, ast_struct: &AstStruct) -> impl ToTokens {
    let name = format_ident!("{}", capitalize(&sty.id));
    let variants = sty.constructors.iter().map(|c| format_ident!("{}", c.id)).collect::<Vec<_>>();
    let variants_builders =
        sty.constructors.iter().map(|c| format_ident!("{}Make", c.id)).collect::<Vec<_>>();
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
    let attrs: Vec<FieldAndType> = sty
        .attributes
        .iter()
        .map(|field| FieldAndType { field, ty: ast_struct.type_by_name(&field.type_id) })
        .collect();
    let attributes = generate_fields(&sty.id, &attrs, ast_struct);

    let builder_name = format_ident!("{}Make", capitalize(&sty.id));

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

        impl AstMake for #builder_name {
            type I = #name;
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

fn generate_type(name: &str, fields: &[Field], ast_struct: &AstStruct) -> impl ToTokens {
    let type_name = format_ident!("{}", capitalize(name));
    let kind = format_ident!("{}", to_upper_snake_case(name));
    let fts: Vec<FieldAndType> = fields
        .iter()
        .map(|field| FieldAndType { field, ty: ast_struct.type_by_name(&field.type_id) })
        .collect();
    let gen_fields = generate_fields(name, &fts, ast_struct);
    let builder = generate_builder(name, &fts, ast_struct);
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

fn generate_builder(name: &str, fields: &[FieldAndType], _ast_struct: &AstStruct) -> impl ToTokens {
    if fields.is_empty() {
        return quote! {};
    }
    let type_name = format_ident!("{}", capitalize(name));
    let kind = format_ident!("{}", to_upper_snake_case(name));
    let builder_name = format_ident!("{}Make", capitalize(name));
    let builder_fields = fields.iter().map(|ft| {
        let field_name = format_ident!("{}", to_lower_snake_case(&ft.field.id));
        match ft.ty {
            AstType::Node => {
                let ty = format_ident!("{}Make", capitalize(&ft.field.type_id));
                match ft.field.arity {
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
            }
            AstType::TokenSet => {
                let ty = format_ident!("{}", capitalize(&ft.field.type_id));
                quote! {
                    #field_name: Option<#ty>,
                }
            }
            _ => {
                quote! {}
            }
        }
    });
    let builder_methods = fields.iter().map(|ft| {
        let method_name =
            format_ident!("{}", depluralize(to_lower_snake_case(&ft.field.id).as_str()));
        let field_name = format_ident!("{}", to_lower_snake_case(&ft.field.id));
        match ft.ty {
            AstType::Node => {
                let ty = format_ident!("{}Make", capitalize(&ft.field.type_id));
                match ft.field.arity {
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
            }
            AstType::TokenSet => {
                let ty = format_ident!("{}", capitalize(&ft.field.type_id));
                quote! {
                    pub fn #method_name(mut self,  f: #ty) -> Self {
                        self.#field_name = Some(f);
                        self
                    }
                }
            }
            _ => {
                quote! {}
            }
        }
    });

    // If token filed follows the child node field they have the same arity we will process them as one tuple.
    // This allows to specify delimiter tokens for repeated children in ASDL.
    let mut tuples: Vec<(&FieldAndType, Option<&FieldAndType>)> = Vec::new();
    let mut peek = fields.iter().peekable();
    while let Some(ft) = peek.next() {
        match ft.ty {
            AstType::Node => {
                if let Some(peeked) = peek.peek() {
                    if let AstType::Token = peeked.ty {
                        if ft.field.arity == peeked.field.arity {
                            tuples.push((ft, Some(*peeked)));
                            peek.next();
                            continue;
                        }
                    }
                }
            }
            _ => (),
        };
        tuples.push((ft, None));
    }

    let make_contents = tuples.iter().map(|(ft, tok)| {
        let field_name = format_ident!("{}", to_lower_snake_case(&ft.field.id));
        match ft.ty {
            AstType::Node => {
                let build_token = if let Some(token) = tok {
                    let token_kind = format_ident!("{}", to_upper_snake_case(&token.field.id));
                    quote! {
                        builder.token(SyntaxKind::#token_kind, SmolStr::new(T_STR!(#token_kind)));
                    }
                } else {
                    quote! {}
                };
                match ft.field.arity {
                    Arity::Optional => {
                        quote! {
                            if let Some(b) = self.#field_name {
                                b.make(builder);
                                #build_token
                            }
                        }
                    }
                    Arity::Repeated => {
                        quote! {
                            for b in self.#field_name {
                                b.make(builder);
                                #build_token
                            }
                        }
                    }
                    Arity::Required => {
                        quote! {
                            self.#field_name.unwrap().make(builder);
                            #build_token
                        }
                    }
                }
            }
            AstType::Token => {
                let token_kind = format_ident!("{}", to_upper_snake_case(&ft.field.id));
                quote! {
                    builder.token(SyntaxKind::#token_kind, SmolStr::new(T_STR!(#token_kind)));
                }
            }
            AstType::TokenSet => {
                quote! {
                    if let Some(b) = self.#field_name {
                        b.make(builder);
                    }
                }
            }
            _ => {
                quote! {}
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

        impl AstMake for #builder_name {
            type I = #type_name;

            fn make(self, builder: &mut SyntaxTreeBuilder) {
                builder.start_node(SyntaxKind::#kind);
                #(#make_contents)*
                builder.finish_node();
            }
        }
    }
}

fn is_same_types_fields(fts: &[&FieldAndType]) -> bool {
    if fts.len() <= 1 {
        false
    } else {
        let first_type = &fts.get(0).unwrap().field.type_id;
        fts.iter().all(|ft| &ft.field.type_id == first_type)
    }
}

fn generate_fields(name: &str, fields: &[FieldAndType], ast_struct: &AstStruct) -> impl ToTokens {
    let nodes_only =
        fields.iter().filter(|ft| ft.ty == AstType::Node).collect::<Vec<&FieldAndType>>();
    if nodes_only.is_empty() {
        quote! {}
    } else {
        let type_name = format_ident!("{}", capitalize(name));
        let traits = generate_traits(name, fields, ast_struct);
        if is_same_types_fields(&nodes_only) {
            let methods = generate_same_type_methods(fields);
            quote! {
                impl #type_name {
                    #methods
                }

                #traits
            }
        } else {
            let methods = generate_methods(fields);
            quote! {
                impl #type_name {
                    #methods
                }

                #traits
            }
        }
    }
}

fn generate_same_type_methods(fields: &[FieldAndType]) -> impl ToTokens {
    let methods =
        fields.iter().filter_map(|ft| ft.cast(AstType::Node)).enumerate().map(|(i, f)| {
            let method_name = format_ident!("{}", to_lower_snake_case(&f.id));
            let ty = format_ident!("{}", capitalize(&f.type_id));
            match f.arity {
                Arity::Optional => {
                    quote! {
                        pub fn #method_name(&self) -> Option<#ty> {
                            super::children(self).nth(#i)
                        }
                    }
                }
                Arity::Repeated => {
                    quote! {
                        // not implemented generation for Repeated fields with the same type #method_name and type #ty
                    }
                }
                Arity::Required => {
                    quote! {
                        // not implemented generation for Required fields with the same type #method_name and type #ty
                    }
                }
            }
        });
    let token_set_methods = fields.iter().filter_map(|ft| ft.cast(AstType::TokenSet)).map(|f| {
        let method = generate_token_set_field(f);
        quote! {
            #method
        }
    });
    quote! {
        #(#methods)*
        #(#token_set_methods)*
    }
}

fn generate_methods(fields: &[FieldAndType]) -> impl ToTokens {
    let methods = fields.iter().map(|ft| {
        let f = ft.field;
        let ty = format_ident!("{}", capitalize(&f.type_id));
        let method_name = format_ident!("{}", to_lower_snake_case(&f.id));
        match ft.ty {
            AstType::Node => {
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
                        quote! {
                            // not implemented generation for required field #method_name and type #ty
                        }
                    }
                }
            }
            AstType::TokenSet => {
                let method = generate_token_set_field(ft.field);
                quote! {
                    #method
                }
            }
            AstType::Provided => {
                quote! {
                    pub fn #method_name(&self) -> #ty {
                        #ty::new(self.syntax().children_with_tokens())
                    }
                }
            }
            _ => {
                quote! {}
            }
        }
    });
    quote! {
        #(#methods)*
    }
}

fn generate_token_set_field(f: &Field) -> impl ToTokens {
    let ty = format_ident!("{}", capitalize(&f.type_id));
    let details_method_name = format_ident!("{}_details", to_lower_snake_case(&f.id));
    let kind_method_name = format_ident!("{}_kind", to_lower_snake_case(&f.id));
    let token_method_name = format_ident!("{}_token", to_lower_snake_case(&f.id));
    match f.arity {
        Arity::Optional => {
            quote! {
                pub fn #details_method_name(&self) -> Option<(SyntaxToken, #ty)> {
                   self.syntax().children_with_tokens().filter_map(|it| it.into_token()).find_map(|tok| {
                       #ty::from_token(&tok).map(|ty| (tok, ty))
                   })
                }

                pub fn #kind_method_name(&self) -> Option<#ty> {
                    self.#details_method_name().map(|t| t.1)
                }

                pub fn #token_method_name(&self) -> Option<SyntaxToken> {
                    self.#details_method_name().map(|t| t.0)
                }
            }
        }
        Arity::Repeated => {
            quote! {
                // not implemented generation for Repeated token set methods
            }
        }
        Arity::Required => {
            quote! {
                // not implemented generation for Required token set methods
            }
        }
    }
}

fn is_provided_type(type_id: &str) -> bool {
    PROVIDED_TYPES.contains(&type_id)
}

fn generate_traits(name: &str, fields: &[FieldAndType], ast_struct: &AstStruct) -> impl ToTokens {
    let type_name = format_ident!("{}", capitalize(name));
    let inf_traits = ast_struct.infer_traits(fields);
    let traits = inf_traits.iter().map(|tr| {
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
        let trait_name = format_ident!("{}", capitalize(&tr.id));
        quote! {
            impl ast::#trait_name for #type_name {
                #(#trait_methods)*
            }
        }
    });
    quote! {
        #(#traits)*
    }
}

fn generate_ast(ast_struct: &AstStruct) -> Result<String> {
    let types = ast_struct.asdl.types.iter().map(|ty| {
        let ast_type = ast_struct.type_of(ty);
        match ty {
            Type::SumType(sty) => {
                if ast_type == AstType::TokenSet {
                    let gts = generate_token_set(sty, ast_struct);
                    quote! {#gts}
                } else {
                    let ty = generate_sum_type(sty, ast_struct);
                    quote! {#ty}
                }
            }
            Type::ProdType(pty) => {
                if ast_type == AstType::Node {
                    let ty = generate_type(&pty.id, &pty.fields, ast_struct);
                    quote! {#ty}
                } else {
                    quote! {}
                }
            }
        }
    });

    let ast = quote! {
        use crate::{
            SyntaxTreeBuilder, SyntaxNode, SyntaxKind::{self, *}, T_STR, SmolStr,
            ast::{self, AstNode, AstChildren, traits::CommentIter, builders::*},
            SyntaxToken
        };

        #(#types)*
    };

    let pretty = codegen::reformat(ast)?;
    Ok(pretty)
}

fn to_prod_type(ty: &Type) -> Option<&ProdType> {
    match ty {
        Type::ProdType(pty) => Some(pty),
        _ => None,
    }
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
