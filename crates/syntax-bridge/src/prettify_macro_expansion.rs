//! Utilities for formatting macro expanded nodes until we get a proper formatter.
use rustc_hash::FxHashMap;
use syntax::{
    NodeOrToken,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, T, WalkEvent,
    ast::{make, syntax_factory::SyntaxFactory},
    syntax_editor::{Position, SyntaxEditor},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrettifyWsKind {
    Space,
    Indent(usize),
    Newline,
}

/// Renders a [`SyntaxNode`] with whitespace inserted between tokens that require them.
///
/// This is an internal API that is only exported because `mbe` needs it for tests and cannot depend
/// on `hir-expand`. For any purpose other than tests, you are supposed to use the `prettify_macro_expansion`
/// from `hir-expand` that handles `$crate` for you.
#[deprecated = "use `hir_expand::prettify_macro_expansion()` instead"]
pub fn prettify_macro_expansion(
    syn: SyntaxNode,
    dollar_crate_replacement: &mut dyn FnMut(&SyntaxToken, &SyntaxFactory) -> Option<SyntaxToken>,
    inspect_mods: impl FnOnce(&[(Position, PrettifyWsKind)]),
) -> SyntaxNode {
    let mut indent = 0;
    let mut last: Option<SyntaxKind> = None;
    let mut mods = Vec::new();
    let mut dollar_crate_replacements = Vec::new();
    let (editor, syn) = SyntaxEditor::new(syn);

    let before = false;
    let after = true;

    let do_indent = |side: bool, token: &SyntaxToken, indent| {
        (token.clone(), side, PrettifyWsKind::Indent(indent))
    };
    let do_ws = |side: bool, token: &SyntaxToken| (token.clone(), side, PrettifyWsKind::Space);
    let do_nl = |side: bool, token: &SyntaxToken| (token.clone(), side, PrettifyWsKind::Newline);

    for event in syn.preorder_with_tokens() {
        let token = match event {
            WalkEvent::Enter(NodeOrToken::Token(token)) => token,
            WalkEvent::Leave(NodeOrToken::Node(node)) => {
                let is_last_child =
                    node.parent().is_some_and(|parent| parent.last_child().as_ref() == Some(&node));
                let is_always_newline = matches!(node.kind(), ATTR);
                let is_non_last_newline = match node.kind() {
                    MATCH_ARM | STRUCT | ENUM | UNION | FN | IMPL | MACRO_RULES | EXTERN_BLOCK
                    | EXTERN_CRATE | MODULE => true,
                    EXPR_STMT
                        if Some(R_CURLY) == node.last_non_trivia_token().map(|it| it.kind()) =>
                    {
                        true
                    }
                    _ => false,
                };
                if (!is_last_child && is_non_last_newline) || is_always_newline {
                    let Some(last) = node.last_non_trivia_token() else { continue };
                    if node.parent().is_some() {
                        mods.push(do_nl(after, &last));
                    }
                    mods.push(do_indent(after, &last, indent));
                }
                continue;
            }
            _ => continue,
        };
        if token.kind() == SyntaxKind::IDENT
            && token.text() == "$crate"
            && let Some(replacement) = dollar_crate_replacement(&token, editor.make())
        {
            dollar_crate_replacements.push((token.clone(), replacement));
        }
        let tok = &token;

        let is_next = |f: fn(SyntaxKind) -> bool, default| -> bool {
            tok.next_non_trivia_token()
                .filter(|it| it.kind() != SyntaxKind::EOF)
                .map(|it| f(it.kind()))
                .unwrap_or(default)
        };
        let spaced = tok.trailing_trivia().next().is_some();
        let is_last =
            |f: fn(SyntaxKind) -> bool, default| -> bool { last.map(f).unwrap_or(default) };

        match tok.kind() {
            k if is_text(k)
                && is_next(|it| !it.is_punct() || matches!(it, T![_] | T![#] | L_CURLY), false) =>
            {
                mods.push(do_ws(after, tok));
            }
            L_CURLY if is_next(|it| it != R_CURLY, true) => {
                indent += 1;
                mods.push(do_nl(after, tok));
                mods.push(do_indent(after, tok, indent));
            }
            R_CURLY if is_last(|it| it != L_CURLY, true) => {
                indent = indent.saturating_sub(1);

                mods.push(do_nl(before, tok));
                mods.push(do_indent(before, tok, indent));
            }
            R_CURLY if is_next(|it| it == T![else], false) => {
                mods.push(do_nl(before, tok));
                mods.push(do_indent(before, tok, indent));
            }
            LIFETIME_IDENT if is_next(is_text, true) => {
                mods.push(do_ws(after, tok));
            }
            AS_KW | DYN_KW | IMPL_KW | CONST_KW | MUT_KW | LET_KW | MATCH_KW => {
                mods.push(do_ws(after, tok));
            }
            T![;] if is_next(|it| it != R_CURLY, true) => {
                if tok.text_range().end() != syn.text_range().end() {
                    mods.push(do_nl(after, tok));
                }
                mods.push(do_indent(after, tok, indent));
            }
            T![=] if let Some((last, next)) = last.zip(tok.next_non_trivia_token()) => {
                // FIXME: this branch is for `=>` in macro_rules!, which is currently parsed as
                // two separate symbols.
                match (last, next.kind()) {
                    (T![=], _) | (_, T![=]) => (),
                    // catch ..= += etc
                    #[rustfmt::skip]
                    (
                        T![!] | T![%] | T![&] | T![*] | T![+] | T![-] |
                        T![/] | T![<] | T![>] | T![^] | T![|] | T![.],
                        _,
                    ) => (),
                    (_, T![>]) => {
                        mods.push(do_ws(before, tok));
                        mods.push(do_ws(after, &next));
                    }
                    _ => {
                        mods.push(do_ws(before, tok));
                        mods.push(do_ws(after, tok));
                    }
                }
            }
            T![->] | T![=>] => {
                mods.push(do_ws(before, tok));
                mods.push(do_ws(after, tok));
            }
            T![:]
                if !spaced
                    && is_next(|it| it != T![:], false)
                    && is_last(|it| it != T![:], false) =>
            {
                mods.push(do_ws(after, tok));
            }
            T![!] if is_last(|it| it == MACRO_RULES_KW, false) && is_next(is_text, false) => {
                mods.push(do_ws(after, tok));
            }
            T![,] if !spaced && tok.parent().is_some_and(|it| it.kind() != MATCH_ARM) => {
                if is_next(|it| !matches!(it, R_BRACK | R_PAREN | R_CURLY | T![,]), false) {
                    mods.push(do_ws(after, tok));
                }
            }
            _ => (),
        }

        last = Some(tok.kind());
    }

    let positions: Vec<_> = mods
        .iter()
        .map(|(token, side, kind)| {
            let position = match side {
                true => Position::after(token.clone()),
                false => Position::before(token.clone()),
            };
            (position, *kind)
        })
        .collect();
    inspect_mods(&positions);

    let mut leading: FxHashMap<SyntaxToken, String> = FxHashMap::default();
    let mut trailing: FxHashMap<SyntaxToken, String> = FxHashMap::default();
    for (token, side, insert) in mods {
        let text = match insert {
            PrettifyWsKind::Space => " ".to_owned(),
            PrettifyWsKind::Indent(0) => continue,
            PrettifyWsKind::Indent(indent) => " ".repeat(4 * indent),
            PrettifyWsKind::Newline => "\n".to_owned(),
        };
        match side {
            true => trailing.entry(token).or_default().push_str(&text),
            false => leading.entry(token).or_default().push_str(&text),
        }
    }
    for (token, text) in &mut trailing {
        if let Some(index) = text.find('\n') {
            let rest = text.split_off(index + 1);
            if !rest.is_empty()
                && let Some(next) = token.next_non_trivia_token()
            {
                leading.entry(next).or_default().insert_str(0, &rest);
            }
        }
    }
    for (token, text) in leading {
        if !text.is_empty() {
            editor.splice_leading_trivia(&token, ..0, make::tokens::trivia(&text));
        }
    }
    for (token, text) in trailing {
        if !text.is_empty() {
            let end = token.trailing_trivia().len();
            editor.splice_trailing_trivia(&token, end.., make::tokens::trivia(&text));
        }
    }
    for (old, new) in dollar_crate_replacements {
        editor.replace(old, new);
    }

    if let Some(it) = syn.descendants_with_tokens().filter_map(|it| it.into_token()).last() {
        editor.splice_trailing_trivia(&it, .., []);
    }

    editor.finish().new_root().clone()
}

fn is_text(k: SyntaxKind) -> bool {
    // Consider all keywords in all editions.
    k.is_any_identifier() || k.is_literal() || k == UNDERSCORE
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::{Expect, expect};

    #[expect(deprecated)]
    fn check_pretty(#[rust_analyzer::rust_fixture] ra_fixture: &str, expect: Expect) {
        let ra_fixture = stdx::trim_indent(ra_fixture);
        let source_file = syntax::ast::SourceFile::parse(&ra_fixture, span::Edition::CURRENT);
        let syn = remove_whitespaces(&source_file.syntax_node());

        let pretty = prettify_macro_expansion(syn, &mut |_, _| None, |_| ());
        let mut pretty = pretty.to_string();
        if pretty.contains('\n') {
            pretty.push('\n');
        }
        expect.assert_eq(&pretty);

        fn remove_whitespaces(node: &SyntaxNode) -> SyntaxNode {
            let (editor, node) = SyntaxEditor::new(node.clone());
            for token in node.descendants_with_tokens().filter_map(|it| it.into_token()) {
                editor.splice_leading_trivia(&token, .., []);
                editor.splice_trailing_trivia(&token, .., []);
            }
            editor.finish().new_root().clone()
        }
    }

    #[test]
    fn test_in_macro() {
        check_pretty(
            r#"
            const X: i32 = x::y::z;
            macro_rules! foo {
                () => {
                    $crate::foo::bar!();
                    (1..2, 1..=2);
                    (a==b, a!=b, a<=b, a>=b, x+=2, x<<=2);
                };
            }
            "#,
            expect![[r#"
                const X: i32 = x::y::z;
                macro_rules! foo {
                    () => {
                        $crate::foo::bar!();
                        (1..2, 1..=2);
                        (a==b, a!=b, a<=b, a>=b, x+=2, x<<=2);
                    };
                }
            "#]],
        );
    }

    #[test]
    fn test_curly_indent() {
        check_pretty(
            r#"
            const _: () = {
                {
                    2;
                    3
                }
            };
            "#,
            expect![[r#"
                const _: () = {
                    {
                        2;
                        3
                    }
                };
            "#]],
        );
    }

    #[test]
    fn test_pats() {
        check_pretty(
            r#"
            const _: () = {
                let x = 2;
                let mut y = 3;
                let ref mut z @ 0..5 = 4;
                let ref mut t @ 0..=5 = 4;
                let (x, ref y) = (5, 6);
                let (Foo { x, y }, Bar(z, t));
                let (&mut x, (y | y));
                match () {}
            };
            "#,
            expect![[r#"
                const _: () = {
                    let x = 2;
                    let mut y = 3;
                    let ref mut z@0..5 = 4;
                    let ref mut t@0..=5 = 4;
                    let (x, ref y) = (5, 6);
                    let (Foo {
                        x, y
                    }, Bar(z, t));
                    let (&mut x, (y|y));
                    match (){}
                };
            "#]],
        );
    }

    #[test]
    fn test_attrs() {
        check_pretty(
            r#"
            #[attr1]
            #[attr2]
            const _: () = {};
            #[attr1]
            const _: () = {
                #[attr2]
                {}
            };
            "#,
            expect![[r#"
                #[attr1]
                #[attr2]
                const _: () = {};
                #[attr1]
                const _: () = {
                    #[attr2]
                    {}
                };
            "#]],
        );
    }

    #[test]
    fn test_items() {
        check_pretty(
            r#"
            fn foo() {}
            struct Foo {}
            struct Foo;
            struct Bar {
                x: i32,
            }
            enum Foo {}
            impl Foo {}
            const _: () = {};
            static S: () = {};
            extern {}
            mod x {}
            mod x;
            type X = 2;
            use a;
            use b::{c, d};
            macro_rules! foo { () => {}; }
            "#,
            expect![[r#"
                fn foo(){}
                struct Foo {}
                struct Foo;

                struct Bar {
                    x: i32,
                }
                enum Foo {}
                impl Foo {}
                const _: () = {};
                static S: () = {};
                extern {}
                mod x {}
                mod x;

                type X = 2;
                use a;
                use b::{
                    c, d
                };
                macro_rules! foo {
                    () => {};
                }
            "#]],
        );
    }

    #[test]
    fn test_exprs() {
        check_pretty(
            r#"
            const _: () = {
                let _ = 1+2;
                let _ = !true && false;
                let _ = foo() + !bar() + dbg!(2) + *x;
                let _ = async move || {};
                let _ = async move {};
                let _ = x.await;
                let _ = (1..2, 1..=2);
                let _ = (3,);
                'lab: for _ in 0..5 {
                    loop { }
                    break 'lab expr;
                    if let pat = expr {
                        foo()
                    } else if true {
                        bar()
                    } else {}
                    if true {} else if true {} else {}
                    fun()
                }
            };
            "#,
            expect![[r#"
                const _: () = {
                    let _ = 1+2;
                    let _ = !true&&false;
                    let _ = foo()+!bar()+dbg!(2)+*x;
                    let _ = async move||{};
                    let _ = async move {};
                    let _ = x.await;
                    let _ = (1..2, 1..=2);
                    let _ = (3,);
                    'lab: for _ in 0..5 {
                        loop {}
                        break 'lab expr;
                        if let pat = expr {
                            foo()
                        }else if true {
                            bar()
                        }else {}
                        if true {
                        }else if true {
                        }else {}
                        fun()
                    }
                };
            "#]],
        );
    }

    #[test]
    fn test_match_arm() {
        check_pretty(
            r#"
            const _: () = {
                match 2 {
                    tmp => foo!(),
                };
            };
            "#,
            expect![[r#"
                const _: () = {
                    match 2 {
                        tmp => foo!(),
                    };
                };
            "#]],
        );

        check_pretty(
            r#"
            const _: () = {
                match 2 {
                    tmp => {}
                };
            };
            "#,
            expect![[r#"
                const _: () = {
                    match 2 {
                        tmp => {}
                    };
                };
            "#]],
        );

        check_pretty(
            r#"
            const _: () = {
                match 2 {
                    1 => {}
                    2 => foo(),
                    _ => {},
                };
            };
            "#,
            expect![[r#"
                const _: () = {
                    match 2 {
                        1 => {}
                        2 => foo(),
                        _ => {},
                    };
                };
            "#]],
        );
    }
}
