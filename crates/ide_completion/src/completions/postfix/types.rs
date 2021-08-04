//! Postfix completions in a type context.
//!
//! For example: `T.wrapo` => `Option<T>`.

use syntax::{SyntaxKind, SyntaxNode};

use crate::{
    completions::{postfix::postfix_snippet, Completions},
    context::CompletionContext,
    patterns::ImmediateLocation,
};

fn find_wrapped_type(node: SyntaxNode) -> Option<SyntaxNode> {
    match node.kind() {
        SyntaxKind::TUPLE_TYPE
        | SyntaxKind::PATH
        | SyntaxKind::PTR_TYPE
        | SyntaxKind::ARRAY_TYPE
        | SyntaxKind::SLICE_TYPE => Some(node),
        SyntaxKind::PATH_SEGMENT | SyntaxKind::NAME_REF | SyntaxKind::GENERIC_ARG_LIST => {
            find_wrapped_type(node.parent()?)
        }
        _ => None,
    }
}

fn type_from_dot_path(token: &syntax::SyntaxToken) -> Option<SyntaxNode> {
    let type_end_token = match token.kind() {
        SyntaxKind::DOT => token.prev_token(),
        SyntaxKind::IDENT => {
            let prev = token.prev_token()?;
            if prev.kind() == SyntaxKind::DOT {
                prev.prev_token()
            } else {
                None
            }
        }
        _ => None,
    }?;

    find_wrapped_type(type_end_token.parent()?)
}

pub(super) fn complete_postfix_type(
    acc: &mut Completions,
    ctx: &CompletionContext,
    cap: ide_db::helpers::SnippetCap,
) -> bool {
    use ImmediateLocation as IM;
    match ctx.completion_location {
        Some(
            IM::RecordField
            | IM::TupleField
            | IM::TypeBound
            | IM::GenericArgList(_)
            | IM::ItemList
            | IM::Trait
            | IM::IdentPat,
        ) => {
            // Context can contain types, so continue.
        }
        _ => {
            // Context is not relevant for types (like expressions, use statements, ...).
            return false;
        }
    }

    let type_node = if let Some(ty) = type_from_dot_path(&ctx.original_token) {
        ty
    } else {
        return false;
    };
    let type_text = type_node.text();

    postfix_snippet(ctx, cap, &type_node, "wrap", "_<T>", format!("${{1:_}}<{}>", type_text))
        .add_to(acc);
    postfix_snippet(ctx, cap, &type_node, "wrapopt", "Option<T>", format!("Option<{}>", type_text))
        .add_to(acc);
    postfix_snippet(
        ctx,
        cap,
        &type_node,
        "wrapres",
        "Result<T, _>",
        format!("Result<{}, ${{1:_}}>", type_text),
    )
    .add_to(acc);

    true
}

#[cfg(test)]
mod tests {
    use crate::{
        tests::{check_edit, filtered_completion_list},
        CompletionKind,
    };
    use expect_test::{expect, Expect};

    fn check(ra_fixture: &str, expect: Expect) {
        let actual = filtered_completion_list(ra_fixture, CompletionKind::Postfix);
        expect.assert_eq(&actual)
    }

    #[test]
    fn postfix_type_suggestions() {
        let cases = &[
            "struct S { f: bool.$0 }",
            "struct S { f: bool.wrap$0, }",
            "struct S { f: Box<bool>.$0,, f2: String }",
            "struct S { f: Box<bool>.wrap$0, }",
            "impl S<bool.$0> {}",
            "impl S<[Rc<bool>; 20].$0> {}",
            "impl TRAIT<bool.$0> {}",
            "trait T { type X = bool.$0 }",
            // NOTE: works without semicolon, but not with
            // "trait T { type X = bool.$0; }",
            "fn f(x: bool.$0) {}",
            "fn f(x: bool.$0, y: bool) {}",
            // FAILING
            // "a::<bool.$0, String>(true)",
            // "fn f() {  let x: bool.$0 = 22 }",
            // "fn f() { x::<bool.$0>(true); }",
        ];

        for case in cases {
            check(
                case,
                expect![[r#"
                sn wrap    _<T>
                sn wrapopt Option<T>
                sn wrapres Result<T, _>
            "#]],
            );
        }
    }

    #[test]
    fn postfix_wrap() {
        check_edit(
            "wrap",
            r#"
struct S {
    f: bool.wrap$0,
}
"#,
            r#"
struct S {
    f: ${1:_}<bool>,
}
"#,
        );
    }

    #[test]
    fn postfix_wrapopt() {
        check_edit(
            "wrapopt",
            r#"
struct S {
    f: bool.wrapopt$0,
}
"#,
            r#"
struct S {
    f: Option<bool>,
}
"#,
        );
    }

    #[test]
    fn postfix_wrapres() {
        check_edit(
            "wrapres",
            r#"
struct S {
    f: bool.$0,
}
"#,
            r#"
struct S {
    f: Result<bool, ${1:_}>,
}
"#,
        );
    }

    // #[test]
    // fn postfix_wrap_in_type_bound() {
    //     check_edit(
    //         "wrap",
    //         r#"
    // let f: bool.wrap$0 = 0;
    // "#,
    //         r#"
    // let f: ${1:_}<bool> = 0;
    // "#,
    //     );
    // }
}
