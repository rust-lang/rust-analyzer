//! A simplified version of quote-crate like quasi quote macro
#![allow(clippy::crate_in_macro_def)]

use intern::{Symbol, sym};
use span::Span;
use syntax::ToSmolStr;
use tt::IdentIsRaw;

use crate::{name::Name, tt::TopSubtreeBuilder};

pub(crate) fn dollar_crate(span: Span) -> tt::SpannedLeaf<tt::Ident> {
    tt::Ident::new_sym(sym::dollar_crate, tt::IdentIsRaw::No, span)
}

// A helper macro quote macro
// FIXME:
// 1. Not all puncts are handled
// 2. #()* pattern repetition not supported now
//    * But we can do it manually, see `test_quote_derive_copy_hack`
#[doc(hidden)]
#[macro_export]
macro_rules! quote_impl__ {
    ($span:ident $builder:ident) => {};

    ( @SUBTREE($span:ident $builder:ident) $delim:ident $($tt:tt)* ) => {
        {
            $builder.open($crate::tt::DelimiterKind::$delim, $span);
            $crate::builtin::quote::__quote!($span $builder  $($tt)*);
            $builder.close($span);
        }
    };

    ( @PUNCT($span:ident $builder:ident) $first:literal ) => {
        $builder.push($crate::tt::Punct::new(
            $first,
            $crate::tt::Spacing::Alone,
            $span,
        ));
    };

    ( @PUNCT($span:ident $builder:ident) $first:literal, $sec:literal ) => {
        $builder.extend([
            $crate::tt::Punct::new(
                $first,
                $crate::tt::Spacing::Joint,
                $span,
            )
            .map($crate::tt::Leaf::Punct),
            $crate::tt::Punct::new(
                $sec,
                $crate::tt::Spacing::Alone,
                $span,
            )
            .map($crate::tt::Leaf::Punct),
        ]);
    };

    // hash variable
    ($span:ident $builder:ident # $first:ident $($tail:tt)* ) => {
        $crate::builtin::quote::ToTokenTree::to_tokens($first, $span, $builder);
        $crate::builtin::quote::__quote!($span $builder $($tail)*);
    };

    ($span:ident $builder:ident # # $first:ident $($tail:tt)* ) => {{
        ::std::iter::IntoIterator::into_iter($first).for_each(|it| $crate::builtin::quote::ToTokenTree::to_tokens(it, $span, $builder));
        $crate::builtin::quote::__quote!($span $builder $($tail)*);
    }};

    // Brace
    ($span:ident $builder:ident { $($tt:tt)* } ) => { $crate::builtin::quote::__quote!(@SUBTREE($span $builder) Brace $($tt)*) };
    // Bracket
    ($span:ident $builder:ident [ $($tt:tt)* ] ) => { $crate::builtin::quote::__quote!(@SUBTREE($span $builder) Bracket $($tt)*) };
    // Parenthesis
    ($span:ident $builder:ident ( $($tt:tt)* ) ) => { $crate::builtin::quote::__quote!(@SUBTREE($span $builder) Parenthesis $($tt)*) };

    // Literal
    ($span:ident $builder:ident $tt:literal ) => { $crate::builtin::quote::ToTokenTree::to_tokens($tt, $span, $builder) };
    // Ident
    ($span:ident $builder:ident $tt:ident ) => {
        $builder.push($crate::tt::Ident::new_sym(
            intern::Symbol::intern(stringify!($tt)),
            tt::IdentIsRaw::No,
            $span,
        ));
    };

    // Puncts
    // FIXME: Not all puncts are handled
    ($span:ident $builder:ident -> ) => {$crate::builtin::quote::__quote!(@PUNCT($span $builder) '-', '>')};
    ($span:ident $builder:ident => ) => {$crate::builtin::quote::__quote!(@PUNCT($span $builder) '=', '>')};
    ($span:ident $builder:ident & ) => {$crate::builtin::quote::__quote!(@PUNCT($span $builder) '&')};
    ($span:ident $builder:ident , ) => {$crate::builtin::quote::__quote!(@PUNCT($span $builder) ',')};
    ($span:ident $builder:ident : ) => {$crate::builtin::quote::__quote!(@PUNCT($span $builder) ':')};
    ($span:ident $builder:ident ; ) => {$crate::builtin::quote::__quote!(@PUNCT($span $builder) ';')};
    ($span:ident $builder:ident :: ) => {$crate::builtin::quote::__quote!(@PUNCT($span $builder) ':', ':')};
    ($span:ident $builder:ident . ) => {$crate::builtin::quote::__quote!(@PUNCT($span $builder) '.')};
    ($span:ident $builder:ident < ) => {$crate::builtin::quote::__quote!(@PUNCT($span $builder) '<')};
    ($span:ident $builder:ident > ) => {$crate::builtin::quote::__quote!(@PUNCT($span $builder) '>')};
    ($span:ident $builder:ident ! ) => {$crate::builtin::quote::__quote!(@PUNCT($span $builder) '!')};
    ($span:ident $builder:ident # ) => {$crate::builtin::quote::__quote!(@PUNCT($span $builder) '#')};
    ($span:ident $builder:ident $ ) => {$crate::builtin::quote::__quote!(@PUNCT($span $builder) '$')};
    ($span:ident $builder:ident * ) => {$crate::builtin::quote::__quote!(@PUNCT($span $builder) '*')};
    ($span:ident $builder:ident = ) => {$crate::builtin::quote::__quote!(@PUNCT($span $builder) '=')};

    ($span:ident $builder:ident $first:tt $($tail:tt)+ ) => {{
        $crate::builtin::quote::__quote!($span $builder $first);
        $crate::builtin::quote::__quote!($span $builder $($tail)*);
    }};
}
pub use quote_impl__ as __quote;

/// FIXME:
/// It probably should implement in proc-macro
#[macro_export]
macro_rules! quote {
    ($span:ident=> $($tt:tt)* ) => {
        {
            let mut builder = $crate::tt::TopSubtreeBuilder::new(
                $crate::tt::DelimiterKind::Invisible,
                $crate::tt::DelimSpan {
                    open: $span,
                    close: $span,
                },
            );
            #[allow(unused)]
            let builder_ref = &mut builder;
            $crate::builtin::quote::__quote!($span builder_ref $($tt)*);
            builder.build_skip_top_subtree()
        }
    }
}
pub use quote;

pub trait ToTokenTree {
    fn to_tokens(self, span: Span, builder: &mut TopSubtreeBuilder);
}

/// Wraps `TokenTreesView` with a delimiter (a subtree, but without allocating).
pub struct WithDelimiter<'a> {
    pub delimiter: crate::tt::SpannedDelimiter,
    pub token_trees: crate::tt::TokenTreesView<'a>,
}

impl ToTokenTree for WithDelimiter<'_> {
    fn to_tokens(self, span: Span, builder: &mut TopSubtreeBuilder) {
        builder.open(self.delimiter.kind, self.delimiter.open);
        self.token_trees.to_tokens(span, builder);
        builder.close(self.delimiter.close);
    }
}

impl ToTokenTree for crate::tt::TokenTreesView<'_> {
    fn to_tokens(self, _: Span, builder: &mut TopSubtreeBuilder) {
        builder.extend_with_tt(self);
    }
}

impl ToTokenTree for crate::tt::SubtreeView<'_> {
    fn to_tokens(self, _: Span, builder: &mut TopSubtreeBuilder) {
        builder.extend_with_tt(self.as_token_trees());
    }
}

impl ToTokenTree for crate::tt::TopSubtree {
    fn to_tokens(self, _: Span, builder: &mut TopSubtreeBuilder) {
        builder.extend_with_tt(self.as_token_trees());
    }
}

impl ToTokenTree for crate::tt::TtElement<'_> {
    fn to_tokens(self, _: Span, builder: &mut TopSubtreeBuilder) {
        match self {
            crate::tt::TtElement::Leaf(leaf) => match leaf {
                tt::SpannedLeafKind::Literal(leaf) => builder.push(leaf.cloned()),
                tt::SpannedLeafKind::Punct(leaf) => builder.push(leaf.cloned()),
                tt::SpannedLeafKind::Ident(leaf) => builder.push(leaf.cloned()),
            },
            crate::tt::TtElement::Subtree(subtree, subtree_iter) => {
                builder.open(subtree.delimiter.kind, subtree.open_span());
                builder.extend_with_tt(subtree_iter.remaining());
                builder.close(subtree.close_span());
            }
        }
    }
}

macro_rules! impl_to_to_tokentrees {
    ($($span:ident: $ty:ty => $this:ident $im:block;)*) => {
        $(
            impl ToTokenTree for $ty {
                fn to_tokens($this, $span: Span, builder: &mut TopSubtreeBuilder) {
                    let leaf: crate::tt::SpannedLeaf<crate::tt::Leaf> = $im.into();
                    builder.push(leaf);
                }
            }
        )*
    }
}
impl<T: ToTokenTree + Clone> ToTokenTree for &T {
    fn to_tokens(self, span: Span, builder: &mut TopSubtreeBuilder) {
        self.clone().to_tokens(span, builder);
    }
}

impl_to_to_tokentrees! {
    span: u32 => self { crate::tt::Literal::new(Symbol::integer(self as _), span, tt::LitKind::Integer, None) };
    span: usize => self { crate::tt::Literal::new(Symbol::integer(self as _), span, tt::LitKind::Integer, None) };
    span: i32 => self { crate::tt::Literal::new(Symbol::integer(self as _), span, tt::LitKind::Integer, None) };
    span: bool => self { crate::tt::Ident::new_sym(if self { sym::true_ } else { sym::false_ }, tt::IdentIsRaw::No, span) };
    _span: crate::tt::SpannedLeaf<crate::tt::Leaf> => self { self };
    _span: crate::tt::SpannedLeaf<crate::tt::Literal> => self { self };
    _span: crate::tt::SpannedLeaf<crate::tt::Ident> => self { self };
    _span: crate::tt::SpannedLeaf<crate::tt::Punct> => self { self };
    span: &str => self { crate::tt::Literal::new(Symbol::intern(&self.escape_default().to_smolstr()), span, tt::LitKind::Str, None) };
    span: String => self { crate::tt::Literal::new(Symbol::intern(&self.escape_default().to_smolstr()), span, tt::LitKind::Str, None) };
    span: Name => self {
        let (is_raw, s) = IdentIsRaw::split_from_symbol(self.as_str());
        crate::tt::Ident::new_sym(Symbol::intern(s), is_raw, span)
    };
    span: Symbol => self {
        let (is_raw, s) = IdentIsRaw::split_from_symbol(self.as_str());
        crate::tt::Ident::new_sym(Symbol::intern(s), is_raw, span)
    };
}

#[cfg(test)]
mod tests {
    use crate::tt;
    use ::tt::IdentIsRaw;
    use expect_test::expect;
    use intern::Symbol;
    use span::{Edition, ROOT_ERASED_FILE_AST_ID, SpanAnchor, SyntaxContext};
    use syntax::{TextRange, TextSize};

    const DUMMY: tt::Span = tt::Span {
        range: TextRange::empty(TextSize::new(0)),
        anchor: SpanAnchor {
            file_id: span::EditionedFileId::new(
                span::FileId::from_raw(0xe4e4e),
                span::Edition::CURRENT,
            ),
            ast_id: ROOT_ERASED_FILE_AST_ID,
        },
        ctx: SyntaxContext::root(Edition::CURRENT),
    };

    #[test]
    fn test_quote_delimiters() {
        assert_eq!(quote!(DUMMY =>{}).to_string(), "{}");
        assert_eq!(quote!(DUMMY =>()).to_string(), "()");
        assert_eq!(quote!(DUMMY =>[]).to_string(), "[]");
    }

    #[test]
    fn test_quote_idents() {
        assert_eq!(quote!(DUMMY =>32).to_string(), "32");
        assert_eq!(quote!(DUMMY =>struct).to_string(), "struct");
    }

    #[test]
    fn test_quote_hash_simple_literal() {
        let a = 20;
        assert_eq!(quote!(DUMMY =>#a).to_string(), "20");
        let s: String = "hello".into();
        assert_eq!(quote!(DUMMY =>#s).to_string(), "\"hello\"");
    }

    fn mk_ident(name: &str) -> tt::SpannedLeaf<tt::Ident> {
        let (is_raw, s) = IdentIsRaw::split_from_symbol(name);
        tt::Ident::new_sym(Symbol::intern(s), is_raw, DUMMY)
    }

    #[test]
    fn test_quote_hash_token_tree() {
        let a = mk_ident("hello");

        let quoted = quote!(DUMMY =>#a);
        assert_eq!(quoted.to_string(), "hello");
        let t = format!("{quoted:#?}");
        expect![[r#"
            SUBTREE $$ 937550:Root[0000, 0]@0..0#ROOT2024 937550:Root[0000, 0]@0..0#ROOT2024
              IDENT   hello 937550:Root[0000, 0]@0..0#ROOT2024"#]]
        .assert_eq(&t);
    }

    #[test]
    fn test_quote_simple_derive_copy() {
        let name = mk_ident("Foo");

        let quoted = quote! {DUMMY =>
            impl Clone for #name {
                fn clone(&self) -> Self {
                    Self {}
                }
            }
        };

        assert_eq!(quoted.to_string(), "impl Clone for Foo {fn clone (& self) -> Self {Self {}}}");
    }

    #[test]
    fn test_quote_derive_copy_hack() {
        // Assume the given struct is:
        // struct Foo {
        //  name: String,
        //  id: u32,
        // }
        let struct_name = mk_ident("Foo");
        let fields = [mk_ident("name"), mk_ident("id")];
        let fields = fields.iter().map(|it| quote!(DUMMY =>#it: self.#it.clone(), ));

        let mut builder = tt::TopSubtreeBuilder::new(
            tt::DelimiterKind::Brace,
            crate::tt::DelimSpan { open: DUMMY, close: DUMMY },
        );
        fields.for_each(|field| builder.extend_with_tt(field.view().as_token_trees()));
        let list = builder.build();

        let quoted = quote! {DUMMY =>
            impl Clone for #struct_name {
                fn clone(&self) -> Self {
                    Self #list
                }
            }
        };

        assert_eq!(
            quoted.to_string(),
            "impl Clone for Foo {fn clone (& self) -> Self {Self {name : self . name . clone () , id : self . id . clone () ,}}}"
        );
    }
}
