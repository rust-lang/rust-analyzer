mod adt;
mod consts;
mod traits;
mod use_item;

pub(crate) use self::{
    adt::{record_field_list, variant_list},
    expressions::{match_arm_list, record_expr_field_list},
    traits::assoc_item_list,
    use_item::use_tree_list,
};
use super::*;

// test mod_contents
// fn foo() {}
// macro_rules! foo {}
// foo::bar!();
// super::baz! {}
// struct S;
pub(super) fn mod_contents(p: &mut Parser<'_>, stop_on_r_curly: bool) {
    attributes::inner_attrs(p);
    while !(p.at(EOF) || (p.at(T!['}']) && stop_on_r_curly)) {
        let pos = p.pos();
        item_or_macro(p, stop_on_r_curly);
        ensure_progress(p, pos);
    }
}

/// Backstop for the item loops: [`item_or_macro`] must always consume at least one token,
/// otherwise its callers spin until the parser's step limit trips and takes the whole
/// process down. Item parsing commits to a node based on lookahead predicates, so a
/// predicate that admits more than its consumer eats can silently produce a zero-token
/// item; catch that here rather than hanging on malformed input.
pub(super) fn ensure_progress(p: &mut Parser<'_>, prev_pos: usize) {
    if p.pos() == prev_pos {
        // `stdx::never!` would be the house style, but `stdx` is only a dev-dependency of
        // this crate and `parser` is deliberately dependency-light. `debug_assert!` gets
        // the same effect: loud in tests and CI, safe recovery in the shipped build.
        debug_assert!(false, "item parsing made no progress at {:?}", p.current());
        p.err_and_bump("expected an item");
    }
}

pub(super) const ITEM_RECOVERY_SET: TokenSet = TokenSet::new(&[
    T![fn],
    T![struct],
    T![enum],
    T![impl],
    T![trait],
    T![const],
    T![async],
    T![unsafe],
    T![extern],
    T![static],
    T![let],
    T![mod],
    T![pub],
    T![crate],
    T![use],
    T![macro],
    T![;],
]);

pub(super) fn item_or_macro(p: &mut Parser<'_>, stop_on_r_curly: bool) {
    let m = p.start();
    attributes::outer_attrs(p);

    let m = match opt_item(p, m) {
        Ok(()) => {
            if p.at(T![;]) {
                p.err_and_bump(
                    "expected item, found `;`\n\
                     consider removing this semicolon",
                );
            }
            return;
        }
        Err(m) => m,
    };

    // test macro_rules_as_macro_name
    // macro_rules! {}
    // macro_rules! ();
    // macro_rules! [];
    // fn main() {
    //     let foo = macro_rules!();
    // }

    // test_err macro_rules_as_macro_name
    // macro_rules! {};
    // macro_rules! ()
    // macro_rules! []
    if paths::is_use_path_start(p) {
        paths::use_path(p);
        // Do not create a MACRO_CALL node here if this isn't a macro call, this causes problems with completion.

        // test_err path_item_without_excl
        // foo
        if p.at(T![!]) {
            macro_call(p, m);
            return;
        } else {
            m.complete(p, ERROR);
            p.error("expected an item");
            return;
        }
    }

    m.abandon(p);
    match p.current() {
        T!['{'] => error_block(p, "expected an item"),
        T!['}'] if !stop_on_r_curly => {
            let e = p.start();
            p.error("unmatched `}`");
            p.bump(T!['}']);
            e.complete(p, ERROR);
        }
        EOF | T!['}'] => p.error("expected an item"),
        T![let] => error_let_stmt(p, "expected an item"),
        _ => p.err_and_bump("expected an item"),
    }
}

/// Try to parse an item, completing `m` in case of success.
///
/// Structured to mirror rustc's `parse_item_common`/`parse_item_kind`
/// (`rustc_parse::parser::item`): visibility and defaultness are parsed once, up front,
/// then [`opt_item_kind`] dispatches on the remaining token stream without ever
/// speculatively consuming a modifier before knowing which item it belongs to. Each
/// branch owns parsing of its own front matter (constness, safety, `extern`, ...).
pub(super) fn opt_item(p: &mut Parser<'_>, m: Marker) -> Result<(), Marker> {
    // test_err pub_expr
    // fn foo() { pub 92; }

    // test_err item_modifier_recovery
    // pub const
    // mod m { pub const }
    // const static X: u8 = 0;
    // pub unsafe 92;
    let has_visibility = opt_visibility(p, false);
    let has_default = opt_defaultness(p);

    match opt_item_kind(p, m, !has_default, has_visibility) {
        Ok(()) => Ok(()),
        // rustc: `VisibilityNotFollowedByItem` / `DefaultNotFollowedByItem`
        // (`parse_item_common`)
        Err(m) if has_visibility || has_default => {
            // No known item kind matched, but we already committed to an item by consuming
            // a visibility or `default`. Greedily absorb any further qualifier-looking
            // tokens into this same ERROR node (e.g. `pub unsafe $0`, still being typed)
            // rather than leaving them to be re-tokenized as their own separate,
            // sibling ERROR nodes one token at a time. IDE completion
            // (ide-completion/src/context/analysis.rs) reconstructs which qualifiers were
            // typed so far from a single such node's children.
            while p.eat(T![unsafe])
                || p.eat(T![async])
                || p.eat(T![gen])
                || p.eat(T![const])
                || p.eat_contextual_kw(T![safe])
            {}
            if p.at(T![extern]) {
                abi(p);
            }
            p.error("expected an item");
            m.complete(p, ERROR);
            Ok(())
        }
        Err(m) => Err(m),
    }
}

// rustc: `parse_item_kind`. Kept as an if-else chain, in rustc's branch order, rather than
// a `match`, so it can be diffed against `parse_item_kind` branch by branch.
//
// Complete list of deliberate divergences from rustc, so this can be audited without
// leaving the file (each is also commented at its site):
//
//  * `default`/`final`: `opt_defaultness` accepts `default` before a raw identifier, which
//    rustc doesn't, and does not handle `final` at all. See `opt_defaultness`.
//  * Const-block items (`const {}` at item position): r-a has no such node, so the branch
//    falls through to expression parsing instead.
//  * `reuse`/delegation items: unsupported by r-a, branch omitted entirely.
//  * `type const` items: rustc removed this syntax (rust-lang/rust#162517), r-a still
//    parses it as a const item.
//  * Recovery-only rustc branches, all omitted: misordered-qualifier reordering
//    suggestions, `macro_rules` missing its `!`, `impl(path)` missing its `in`,
//    `import`/`using`/`include`/`require` misspellings, `field: Type` inside a trait body
//    (`recover_field_in_trait`), and the `Case::Insensitive` retry pass. r-a still parses
//    these inputs, just with its own generic errors.
//
// Everything else is intended to match rustc's accept/reject decisions exactly. When
// touching a `check_*`/`is_*` gate here, keep its consumer in sync: the gates are
// deliberate over-approximations (rustc absorbs the slack through its fallible parser,
// which we do not have), so a consumer must accept the gate's whole language and must
// always consume at least one token. See `ensure_progress`.
fn opt_item_kind(
    p: &mut Parser<'_>,
    m: Marker,
    check_pub: bool,
    has_visibility: bool,
) -> Result<(), Marker> {
    if p.at(T![use]) && !is_use_closure(p) {
        use_item::use_(p, m);
    } else if check_fn_front_matter(p, check_pub) {
        // FUNCTION ITEM
        fn_(p, m, has_visibility);
    } else if p.at(T![extern]) && p.nth(1) == T![crate] {
        // EXTERN CRATE
        extern_crate(p, m);
    } else if p.at(T![extern]) {
        // EXTERN BLOCK
        foreign_mod(p, m);
    } else if is_unsafe_foreign_mod(p) {
        // EXTERN BLOCK (`unsafe extern`)
        foreign_mod(p, m);
    } else if is_global_static_front_matter(p) {
        // STATIC ITEM
        consts::static_(p, m);
    } else if p.at(T![trait]) || check_trait_front_matter(p) {
        // TRAIT ITEM
        traits::trait_(p, m);
    } else if check_impl_frontmatter(p) {
        // IMPL ITEM
        traits::impl_(p, m);
    } else if is_const_block(p) {
        // CONST BLOCK ITEM: r-a has no such node (unlike rustc's `AllowConstBlockItems`);
        // fall through so it gets parsed as a const-block *expression* instead.
        return Err(m);
    } else if at_item_constness(p) {
        // CONST ITEM
        consts::konst(p, m);
    } else if p.at(T![mod]) || (p.at(T![unsafe]) && p.nth(1) == T![mod]) {
        // MODULE ITEM
        mod_item(p, m);
    } else if p.at(T![type]) {
        if p.nth(1) == T![const] {
            // Not in rustc anymore: rust-lang/rust#162517 removed `type const`, so there
            // `type` always starts a type alias.

            // test type_const
            // type const FOO: i32 = 2;
            consts::konst(p, m);
        } else {
            // TYPE ITEM
            type_alias(p, m);
        }
    } else if p.at(T![enum]) {
        // ENUM ITEM
        adt::enum_(p, m);
    } else if p.at(T![struct]) {
        // STRUCT ITEM
        adt::strukt(p, m);
    } else if p.at_contextual_kw(T![union]) && p.nth(1) == IDENT {
        // UNION ITEM
        adt::union(p, m);
    } else if p.at_contextual_kw(T![builtin])
        && p.nth_at(1, T![#])
        && p.nth_at_contextual_kw(2, T![global_asm])
    {
        // BUILTIN# ITEM
        p.bump_remap(T![builtin]);
        p.bump(T![#]);
        p.bump_remap(T![global_asm]);
        // test global_asm
        // builtin#global_asm("")
        expressions::parse_asm_expr(p, m);
    } else if p.at(T![macro]) {
        // MACROS 2.0 ITEM
        macro_def(p, m);
    } else if p.at_contextual_kw(T![macro_rules]) && p.nth_at(1, BANG) && p.nth_at(2, IDENT) {
        // MACRO_RULES ITEM
        // rustc also recovers `macro_rules foo { ... }` missing the `!` (`is_macro_rules_item`);
        // not ported, r-a falls through to the generic "expected an item" recovery in
        // `item_or_macro` for that case, as it always has.
        macro_rules(p, m);
    } else {
        // rustc also has recovery-only branches here: `import`/`using`/`include`/`require`
        // misspelling recovery, `pub`-without-item recovery (subsumed by `opt_item`'s
        // `has_visibility` check above), a `Case::Insensitive` retry pass, `field: Type`
        // recovery inside a trait body (`recover_field_in_trait`), and delegation items
        // (`reuse`, unsupported by r-a). None of these are ported.
        return Err(m);
    }
    Ok(())
}

/// rustc: `parse_defaultness`. `default` followed by any identifier or keyword except `as`,
/// the one keyword that can follow an expression (`default as Ty`). Only the `default` itself
/// is consumed; whatever follows is left to [`opt_item_kind`]'s dispatch.
///
/// This only ever runs in item position. rustc's statement parser handles a statement that
/// starts with a non-reserved identifier as a path before it tries item parsing, so a
/// `default` there never reaches `parse_defaultness`; r-a does the same via
/// [`is_stmt_path_start`].
///
/// rustc reports `default` on an item kind that can't take it (`error_on_unconsumed_default`)
/// after parsing the item. r-a does that in syntax validation instead, which can point at
/// the `default` token itself.
///
/// Divergences from rustc:
///
///  * rustc rejects a *raw* identifier after `default` (`default r#foo`). r-a's parser input
///    doesn't record raw-ness, so this accepts one. The input is an error either way.
///  * rustc also accepts `final` here (`Defaultness::Final`, feature
///    `final_associated_functions`). r-a does not support `final` on items at all — it is
///    absent from `rust.ungram`, so there is no token to attach — and `final fn` parses to
///    an `ERROR` node, exactly as it did before this function existed. Adding it is a
///    grammar + codegen change, tracked separately rather than smuggled in here.
fn opt_defaultness(p: &mut Parser<'_>) -> bool {
    // test default_item
    // default impl T for Foo {}

    // test default_unsafe_item
    // default unsafe impl T for Foo {
    //     default unsafe fn foo() {}
    // }

    // test default_async_fn
    // impl T for Foo {
    //     default async fn foo() {}
    // }

    // test default_async_unsafe_fn
    // impl T for Foo {
    //     default async unsafe fn foo() {}
    // }

    // test default_fn_front_matter
    // impl T for S {
    //     default unsafe extern "C" fn f() {}
    //     default extern "C" fn g() {}
    //     default const unsafe fn h() {}
    // }
    if p.at_contextual_kw(T![default]) && p.nth(1).is_any_identifier() && p.nth(1) != T![as] {
        p.bump_remap(T![default]);
        return true;
    }
    false
}

/// rustc: the path-statement branch of `parse_stmt_without_recovery`, which comes *before*
/// its item branch: a statement that starts with a path is parsed as one (an expression or
/// a macro call) unless `is_path_start_item`/`is_builtin` say it starts an item.
///
/// Only the case of a non-reserved identifier (`IDENT`, which includes contextual keywords)
/// is ported. It's the only one where [`opt_item`] could otherwise accept something that
/// rustc never tries as an item: a statement-leading `default` or `safe` stays an
/// expression, as in rustc, instead of becoming an item qualifier.
pub(super) fn is_stmt_path_start(p: &Parser<'_>) -> bool {
    // test_err default_and_safe_stmt_are_paths
    // fn f() {
    //     default fn g() {}
    //     safe fn h() {}
    // }

    // test default_and_safe_stmt_expr
    // fn f() {
    //     default;
    //     safe;
    //     default as u8;
    // }
    p.at(IDENT)
        && !is_path_start_item(p)
        && !(p.at_contextual_kw(T![builtin]) && p.nth_at(1, T![#]))
}

/// rustc: `is_path_start_item`, minus `reuse` items (unsupported by r-a) and 2015-edition
/// `async fn` (in 2015 r-a lexes `async` as an identifier and never parses it as a fn
/// qualifier, so there is nothing to divert).
fn is_path_start_item(p: &Parser<'_>) -> bool {
    // `union U { .. }`, not `union::b`
    (p.at_contextual_kw(T![union]) && p.nth(1) == IDENT)
        // `auto trait X { .. }`, not `auto::b`
        || check_trait_front_matter(p)
        // `macro_rules! mac`, not `macro_rules::b`
        || (p.at_contextual_kw(T![macro_rules]) && p.nth_at(1, BANG) && p.nth_at(2, IDENT))
}

/// rustc: `check_fn_front_matter`.
///
/// `check_pub` mirrors rustc's parameter of the same name: `pub` counts as a qualifier
/// only when no `default` was parsed, since `default pub` is invalid. Note this matters
/// even though [`opt_item`] already consumed any *leading* visibility — a misplaced `pub`
/// can still show up *after* another qualifier (`const pub fn`), which is exactly the case
/// rustc keeps `pub` in the list to recover as a single fn item.
fn check_fn_front_matter(p: &Parser<'_>, check_pub: bool) -> bool {
    let is_qual = |n: usize| {
        matches!(p.nth(n), T![gen] | T![const] | T![async] | T![unsafe] | T![extern])
            || p.nth_at_contextual_kw(n, T![safe])
            || (check_pub && p.nth(n) == T![pub])
    };
    // rustc requires the *second* qualifier to be `is_reserved()`, which excludes the
    // weak keyword `safe` (and rules out old-edition `const async: T = val`, since
    // `async`/`gen` are only real keywords, i.e. only match here, in edition 2018+/2024+).
    // `pub` is a reserved keyword, so unlike `safe` it does count in this position.
    let is_reserved_qual = |n: usize| {
        matches!(p.nth(n), T![gen] | T![const] | T![async] | T![unsafe] | T![extern])
            || (check_pub && p.nth(n) == T![pub])
    };
    // rustc's `ALL_QUALS`, i.e. including `pub` regardless of `check_pub`. rustc matches
    // these by name, but r-a only lexes `async`/`gen` as keywords from 2018/2024 on.
    let is_any_qual = |n: usize| {
        matches!(p.nth(n), T![pub] | T![gen] | T![const] | T![async] | T![unsafe] | T![extern])
            || p.nth_at_contextual_kw(n, T![safe])
    };

    p.at(T![fn])
        || (is_qual(0)
            && (p.nth(1) == T![fn]
                || (is_reserved_qual(1)
                    // Rule out `unsafe extern {`.
                    && !is_unsafe_foreign_mod(p)
                    // Rule out `async gen {` and `async gen move {`.
                    && !is_async_gen_block(p)
                    // Rule out `const unsafe auto` and `const unsafe trait` and `const unsafe impl`.
                    && !p.nth_at_contextual_kw(2, T![auto])
                    && !matches!(p.nth(2), T![trait] | T![impl]))))
        // `extern ABI fn`, e.g. `extern "C" fn`, plus rustc's recovery arm for qualifiers
        // misplaced after the ABI (`extern "C" unsafe fn`, `extern "C" const unsafe fn`),
        // which keeps that a single fn item rather than an extern block followed by a stray
        // fn. Like the `$qual fn` / `$qual $qual` rule above, the recovery arm accepts either
        // `fn` or another qualifier after the first one. rustc's version also handles the ABI
        // being a metavariable via `tree_look_ahead`; r-a has no such token category here.
        || (p.at(T![extern])
            && p.nth(1) == STRING
            && (p.nth(2) == T![fn]
                || (is_qual(2) && (p.nth(3) == T![fn] || is_any_qual(3)))))
}

/// rustc: `is_use_closure`.
fn is_use_closure(p: &Parser<'_>) -> bool {
    if !p.at(T![use]) {
        return false;
    }
    // Move or async here would be an error but still we're parsing a closure.
    let dist = if matches!(p.nth(1), T![move] | T![async]) { 2 } else { 1 };
    // A single `|` also matches the first half of `||`; `Parser::nth` doesn't merge
    // composite tokens, so checking the raw `|` covers both `|x|` and `||`.
    p.nth(dist) == T![|]
}

// Only a leading `unsafe` opens a foreign mod; `const`/`safe` before `extern` don't
// (rustc rejects them too, with "expected `fn`, found `{`" — `const extern "C" fn` is
// valid *function* front matter, but `const extern "C" { .. }` alone is not an item).

// test_err const_qualified_extern_block
// const extern "C" {}
// safe extern "C" {}
// const unsafe extern "C" {
//     fn item();
// }

/// rustc: `is_unsafe_foreign_mod`.
fn is_unsafe_foreign_mod(p: &Parser<'_>) -> bool {
    if !p.at(T![unsafe]) {
        return false;
    }
    if p.nth(1) != T![extern] {
        return false;
    }
    let n = if p.nth(2) == STRING { 3 } else { 2 };
    p.nth(n) == T!['{']
}

/// rustc: `is_async_gen_block` (`async gen {`, `async gen move {`).
fn is_async_gen_block(p: &Parser<'_>) -> bool {
    p.at(T![async]) && p.nth(1) == T![gen] && (p.nth(2) == T!['{'] || p.nth(2) == T![move])
}

// `async gen fn` must still be recognized as fn front matter, not misdetected by
// `is_async_gen_block` above as the start of an `async gen { .. }` block.

// test const_async_gen_fn
// const async gen fn f() {}

/// rustc: `parse_global_static_front_matter`.
fn is_global_static_front_matter(p: &Parser<'_>) -> bool {
    if p.at(T![static]) {
        // Not a closure: `static || …` / `static move || …` / `static use || …`.
        return !matches!(p.nth(1), T![move] | T![use] | T![|]);
    }
    (p.at(T![unsafe]) || p.at_contextual_kw(T![safe])) && p.nth(1) == T![static]
}

/// rustc: `check_trait_front_matter`. rustc's last branch — recovering
/// `impl(path::to::mod)` missing `in`, purely to suggest inserting `in` — is
/// diagnostics-only and not ported.
fn check_trait_front_matter(p: &Parser<'_>) -> bool {
    const SUFFIXES: &[&[SyntaxKind]] = &[
        &[T![trait]],
        &[T![auto], T![trait]],
        &[T![unsafe], T![trait]],
        &[T![unsafe], T![auto], T![trait]],
        &[T![const], T![trait]],
        &[T![const], T![auto], T![trait]],
        &[T![const], T![unsafe], T![trait]],
        &[T![const], T![unsafe], T![auto], T![trait]],
    ];
    // `auto` is a contextual keyword (raw kind `IDENT`), unlike the other, hard-keyword
    // entries in `SUFFIXES`, so it needs its own comparison.
    fn nth_is(p: &Parser<'_>, n: usize, kw: SyntaxKind) -> bool {
        if kw == T![auto] { p.nth_at_contextual_kw(n, T![auto]) } else { p.nth(n) == kw }
    }

    if p.at(T![impl]) && p.nth(1) == T!['('] {
        // `impl(in` unambiguously introduces an impl restriction: `in` cannot start a type,
        // so there's no ambiguity with a parenthesized-type `impl` block to disambiguate.
        if p.nth(2) == T![in] {
            return true;
        }
        // `impl(crate | self | super)` + SUFFIX.
        return matches!(p.nth(2), T![crate] | T![self] | T![super])
            && p.nth(3) == T![')']
            && SUFFIXES
                .iter()
                .any(|suffix| suffix.iter().enumerate().all(|(i, &kw)| nth_is(p, 4 + i, kw)));
    }
    SUFFIXES.iter().any(|suffix| suffix.iter().enumerate().all(|(i, &kw)| nth_is(p, i, kw)))
}

/// rustc: `check_impl_frontmatter`. rustc's `look_ahead` parameter is only
/// ever non-zero at a speculative delegation (`reuse`) call site we don't port, so it's
/// dropped here.
fn check_impl_frontmatter(p: &Parser<'_>) -> bool {
    for i in 0..2 {
        match p.nth(i) {
            T![impl] => return true,
            T![const] | T![unsafe] => continue,
            _ => return false,
        }
    }
    p.nth(2) == T![impl]
}

/// rustc: `check_inline_const`, `dist == 0` call site only (rustc's other
/// call sites are in expression/pattern parsing, out of scope here).
fn is_const_block(p: &Parser<'_>) -> bool {
    p.at(T![const]) && p.nth(1) == T!['{']
}

/// rustc: `check_const_closure`. rustc also accepts forced keywords there
/// (`const k#move ||`); r-a's lexer has no `k#` tokens, so there is nothing to port.
fn is_const_closure(p: &Parser<'_>) -> bool {
    // Mirrors rustc's FIXME(#146122): `const async ...`, `const gen ...` and
    // `const async gen ...` closures aren't parsed yet, only `const static async ...` etc.
    p.at(T![const]) && matches!(p.nth(1), T![move] | T![use] | T![static] | T![|])
}

// The rust-lang/rust-analyzer#23006 repro: r-a used to eat a bare `const` as an item
// modifier before even looking at what followed, so `const move |a| ...` (a valid
// standalone-statement closure, e.g. in `library/core/src/ops/try_trait.rs`) was
// misparsed as the start of a broken `const` item.

// test const_closure_not_item
// fn wrap() {
//     const move |a| NeverShortCircuit(f(a))
// }

// test const_static_closure_not_item
// fn wrap() {
//     const static || ();
//     const static move || ();
//     const static gen || ();
//     const static async gen move || ()
// }

/// rustc: `parse_constness_`, `is_closure: false` (i.e. the plain
/// `parse_constness` used for items, not `parse_closure_constness`).
fn at_item_constness(p: &Parser<'_>) -> bool {
    p.at(T![const]) && !is_const_closure(p) && p.nth(1) != T!['{']
}

// rustc: `parse_item_foreign_mod`. Handles both `extern <abi>? { .. }` and
// `unsafe extern <abi>? { .. }` (the leading `unsafe` is a no-op `eat` when absent).
fn foreign_mod(p: &mut Parser<'_>, m: Marker) {
    // test extern_block
    // unsafe extern "C" {}
    // extern {}
    p.eat(T![unsafe]);
    abi(p);
    if p.at(T!['{']) {
        extern_item_list(p);
    } else {
        p.error("expected `{`");
    }
    m.complete(p, EXTERN_BLOCK);
}

// test extern_crate
// extern crate foo;
// extern crate self;
fn extern_crate(p: &mut Parser<'_>, m: Marker) {
    p.bump(T![extern]);
    p.bump(T![crate]);

    name_ref_or_self(p);

    // test extern_crate_rename
    // extern crate foo as bar;
    // extern crate self as bar;
    opt_rename(p);
    p.expect(T![;]);
    m.complete(p, EXTERN_CRATE);
}

// test mod_item
// mod a;
pub(crate) fn mod_item(p: &mut Parser<'_>, m: Marker) {
    p.eat(T![unsafe]);
    p.bump(T![mod]);
    name(p);
    if p.at(T!['{']) {
        // test mod_item_curly
        // mod b { }
        item_list(p);
    } else if !p.eat(T![;]) {
        p.error("expected `;` or `{`");
    }
    m.complete(p, MODULE);
}

// test type_alias
// type Foo = Bar;
fn type_alias(p: &mut Parser<'_>, m: Marker) {
    p.bump(T![type]);

    name(p);

    // test type_item_type_params
    // type Result<T> = ();
    generic_params::opt_generic_param_list(p);

    if p.at(T![:]) {
        generic_params::bounds(p);
    }

    // test type_item_where_clause_deprecated
    // type Foo where Foo: Copy = ();
    generic_params::opt_where_clause(p);
    if p.eat(T![=]) {
        types::type_(p);
    }

    // test type_item_where_clause
    // type Foo = () where Foo: Copy;
    generic_params::opt_where_clause(p);

    p.expect(T![;]);
    m.complete(p, TYPE_ALIAS);
}

pub(crate) fn item_list(p: &mut Parser<'_>) {
    assert!(p.at(T!['{']));
    let m = p.start();
    p.bump(T!['{']);
    mod_contents(p, true);
    p.expect(T!['}']);
    m.complete(p, ITEM_LIST);
}

pub(crate) fn extern_item_list(p: &mut Parser<'_>) {
    assert!(p.at(T!['{']));
    let m = p.start();
    p.bump(T!['{']);
    mod_contents(p, true);
    p.expect(T!['}']);
    m.complete(p, EXTERN_ITEM_LIST);
}

// test try_macro_rules 2015
// macro_rules! try { () => {} }
fn macro_rules(p: &mut Parser<'_>, m: Marker) {
    assert!(p.at_contextual_kw(T![macro_rules]));
    p.bump_remap(T![macro_rules]);
    p.expect(T![!]);

    name(p);

    match p.current() {
        // test macro_rules_non_brace
        // macro_rules! m ( ($i:ident) => {} );
        // macro_rules! m [ ($i:ident) => {} ];
        T!['['] | T!['('] => {
            token_tree(p);
            p.expect(T![;]);
        }
        T!['{'] => token_tree(p),
        _ => p.error("expected `{`, `[`, `(`"),
    }
    m.complete(p, MACRO_RULES);
}

// test macro_def
// macro m($i:ident) {}
fn macro_def(p: &mut Parser<'_>, m: Marker) {
    p.expect(T![macro]);
    name_r(p, ITEM_RECOVERY_SET);
    if p.at(T!['{']) {
        // test macro_def_curly
        // macro m { ($i:ident) => {} }
        token_tree(p);
    } else if p.at(T!['(']) {
        token_tree(p);
        match p.current() {
            T!['{'] | T!['['] | T!['('] => token_tree(p),
            _ => p.error("expected `{`, `[`, `(`"),
        }
    } else {
        p.error("unmatched `(`");
    }

    m.complete(p, MACRO_DEF);
}

// test fn_
// fn foo() {}
fn fn_(p: &mut Parser<'_>, m: Marker, has_visibility: bool) {
    let consumed_front_matter = parse_fn_front_matter(p, has_visibility);
    if !p.eat(T![fn]) {
        // Each case is wrapped so the qualifier runs don't merge into one token stream.
        // Before these were handled, every one of them hung the enclosing item loop until
        // the parser step limit tripped.

        // test_err fn_qualifiers_without_fn
        // mod a { async gen }
        // mod b { gen const }
        // mod c { gen unsafe }
        // mod d { gen gen }
        // mod e { gen extern }
        // fn f() { async gen }

        // Qualifiers without the `fn` they promised, e.g. `async gen` mid-edit. Bail with a
        // single error instead of cascading through this function's remaining recovery
        // steps, each of which would also fail to match and pile on redundant errors.
        p.error("expected fn");
        if !consumed_front_matter {
            // Unreachable via `check_fn_front_matter` (it only fires at `fn` itself, which
            // `eat` above would have taken, or at a qualifier, which the front matter
            // always eats). Guard anyway: completing a node without consuming anything
            // would hang the caller's item loop.
            p.err_and_bump("expected an item");
        }
        m.complete(p, FN);
        return;
    }

    name_r(p, ITEM_RECOVERY_SET);
    // test function_type_params
    // fn foo<T: Clone + Copy>(){}
    generic_params::opt_generic_param_list(p);

    if p.at(T!['(']) {
        params::param_list_fn_def(p);
    } else {
        p.error("expected function arguments");
    }
    // test function_ret_type
    // fn foo() {}
    // fn bar() -> () {}
    if !opt_ret_type(p) {
        // test_err function_ret_type_missing_arrow
        // fn foo() usize {}
        // fn bar() super::Foo {}
        opt_no_arrow_ret_type(p);
    }

    // test_err fn_ret_recovery
    // fn foo() -> A>]) { let x = 1; }
    // fn foo() -> A>]) where T: Copy { let x = 1; }
    while p.at(T![')']) | p.at(T![']']) | p.at(T![>]) {
        // recover from unbalanced return type brackets
        p.err_and_bump("expected a curly brace");
    }

    // test function_where_clause
    // fn foo<T>() where T: Copy {}
    generic_params::opt_where_clause(p);

    // test fn_decl
    // trait T { fn foo(); }
    if !p.eat(T![;]) {
        expressions::block_expr(p);
    }
    m.complete(p, FN);
}

/// rustc: `parse_fn_front_matter`, whose canonical order is constness →
/// coroutine (`async`/`gen`) → safety (`unsafe`/`safe`) → `extern` + abi → `fn`.
///
/// Consumes qualifiers in *any* order rather than only that one, because
/// [`check_fn_front_matter`] admits any order too (`unsafe async fn`, `const pub fn`, ...).
/// Matching the gate's language is what keeps the two contracts in sync: every token the
/// gate accepted gets eaten here, so this never returns without having made progress, and
/// a misordered qualifier still yields one `FN` node the way rustc yields one fn item.
/// A wrong order is still *reported*, so accepting it here does not silently bless invalid
/// code; rustc's more specific "`async` must come before `unsafe`" wording and its
/// machine-applicable reordering suggestion are not ported.
///
/// Returns whether anything was consumed.
fn parse_fn_front_matter(p: &mut Parser<'_>, has_visibility: bool) -> bool {
    // test item_front_matter
    // const async fn first() {}
    // const unsafe extern "C" fn second() {}

    // test_err async_without_semicolon
    // fn foo() { let _ = async {} }

    // test_err gen_fn 2021
    // gen fn gen_fn() {}
    // async gen fn async_gen_fn() {}

    // test_err unsafe_block_in_mod
    // fn foo(){} unsafe { } fn bar(){}

    // test safe_outside_of_extern
    // fn foo() { safe = true; }

    // test_err fn_qualifier_order
    // const pub fn a() {}
    // async pub fn b() {}
    // extern "C" unsafe fn c() {}
    // unsafe async fn d() {}
    // const const fn e() {}
    // extern "C" const unsafe fn g() {}

    // test_err fn_duplicate_visibility
    // pub pub fn f() {}

    // Canonical order, from rustc's own diagnostic: "keyword order for functions
    // declaration is `pub`, `default`, `const`, `async`, `unsafe`, `extern`" (`gen` pairs
    // with `async`, `safe` is the counterpart of `unsafe`). Ranks must strictly increase;
    // anything else is a misordered or duplicated qualifier.
    const PUB: u8 = 0;
    const CONST: u8 = 1;
    const ASYNC: u8 = 2;
    const GEN: u8 = 3;
    const SAFETY: u8 = 4;
    const EXTERN: u8 = 5;

    let mut consumed = false;
    // A leading `pub` may already have been consumed by `opt_item` before this function
    // was even called; seed the rank so a second one, e.g. `pub pub fn`, is still caught
    // as a duplicate instead of looking like the first qualifier seen.
    let mut prev_rank = if has_visibility { Some(PUB) } else { None };
    loop {
        let rank = if p.at(T![extern]) {
            EXTERN
        } else if p.at(T![pub]) {
            PUB
        } else if p.at(T![const]) {
            CONST
        } else if p.at(T![async]) {
            ASYNC
        } else if p.at(T![gen]) {
            GEN
        } else if p.at(T![unsafe]) || p.at_contextual_kw(T![safe]) {
            SAFETY
        } else {
            return consumed;
        };

        match prev_rank {
            Some(prev) if rank < prev => p.error("wrong order of qualifiers"),
            Some(prev) if rank == prev => p.error("duplicate qualifier"),
            _ => (),
        }
        prev_rank = Some(rank);

        if p.at(T![extern]) {
            // `const extern "C" fn` is valid fn front matter (unlike `const extern "C" { .. }`
            // alone, see `err/const_qualified_extern_block.rs`).
            abi(p);
        } else if p.at(T![pub]) {
            // A misplaced visibility, e.g. `const pub fn`; a *leading* one was already
            // taken by `opt_item`.
            opt_visibility(p, false);
        } else if p.at_contextual_kw(T![safe]) {
            p.eat_contextual_kw(T![safe]);
        } else {
            // `const` / `async` / `gen` / `unsafe`
            p.bump_any();
        }
        consumed = true;
    }
}

fn macro_call(p: &mut Parser<'_>, m: Marker) {
    assert!(p.at(T![!]));
    match macro_call_after_excl(p) {
        BlockLike::Block => (),
        BlockLike::NotBlock => {
            p.expect(T![;]);
        }
    }
    m.complete(p, MACRO_CALL);
}

pub(super) fn macro_call_after_excl(p: &mut Parser<'_>) -> BlockLike {
    p.expect(T![!]);

    match p.current() {
        T!['{'] => {
            token_tree(p);
            BlockLike::Block
        }
        T!['('] | T!['['] => {
            token_tree(p);
            BlockLike::NotBlock
        }
        _ => {
            p.error("expected `{`, `[`, `(`");
            BlockLike::NotBlock
        }
    }
}

pub(crate) fn token_tree(p: &mut Parser<'_>) {
    let closing_paren_kind = match p.current() {
        T!['{'] => T!['}'],
        T!['('] => T![')'],
        T!['['] => T![']'],
        _ => unreachable!(),
    };
    let m = p.start();
    p.bump_any();
    while !p.at(EOF) && !p.at(closing_paren_kind) {
        match p.current() {
            T!['{'] | T!['('] | T!['['] => token_tree(p),
            T!['}'] => {
                p.error("unmatched `}`");
                m.complete(p, TOKEN_TREE);
                return;
            }
            T![')'] | T![']'] => p.err_and_bump("unmatched brace"),
            _ => p.bump_any(),
        }
    }
    p.expect(closing_paren_kind);
    m.complete(p, TOKEN_TREE);
}
