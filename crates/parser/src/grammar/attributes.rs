use super::*;

pub(super) const ATTRIBUTE_FIRST: TokenSet = TokenSet::new(&[T![#]]);

pub(super) fn inner_attrs(p: &mut Parser<'_>) {
    while p.at(T![#]) && p.nth(1) == T![!] {
        attr(p, true);
    }
}

pub(super) fn outer_attrs(p: &mut Parser<'_>) {
    while p.at(T![#]) {
        attr(p, false);
    }
}

fn attr(p: &mut Parser<'_>, inner: bool) {
    assert!(p.at(T![#]));

    //verus custom trigger attribute
    // verus trigger attribute is custom syntax
    // Reference https://github.com/verus-lang/verus/blob/4ef61030aadc4fd66b62f3614f36e3b64e89b855/source/builtin_macros/src/syntax.rs#L1808
    // Note that verus! macro compares the attributes name with "trigger", and process it
    // However, the rust-analyzer parser does not have access to string's content.
    // Therefore, to make a new syntax kind (e.g. TriggerAttribute),
    // we need to register `trigger` as a reserved keyword.
    // however, by registering `trigger` as a reserved keyword,
    // single trigger attribute, which is `#[trigger]` becomes invalid syntax.
    // therefore, we just special-case `#[trigger]`
    if p.nth(1) == T![!] && p.nth(2) == T!['['] && p.nth(3) == T![trigger] {
        verus::trigger_attribute(p);
        return;
    }
    // verus -- #[trigger]
    if p.nth(1) == T!['['] && p.nth(2) == T![trigger] {
        let attr = p.start();
        p.expect(T![#]);
        p.expect(T!['[']);
        p.expect(T![trigger]);
        p.expect(T![']']);
        attr.complete(p, ERROR);
        return;
    }

    let attr = p.start();
    p.bump(T![#]);

    if inner {
        p.bump(T![!]);
    }

    if p.eat(T!['[']) {
        meta(p);

        if !p.eat(T![']']) {
            p.error("expected `]`");
        }
    } else {
        p.error("expected `[`");
    }
    attr.complete(p, ATTR);
}

// test metas
// #![simple_ident]
// #![simple::path]
// #![simple_ident_expr = ""]
// #![simple::path::Expr = ""]
// #![simple_ident_tt(a b c)]
// #![simple_ident_tt[a b c]]
// #![simple_ident_tt{a b c}]
// #![simple::path::tt(a b c)]
// #![simple::path::tt[a b c]]
// #![simple::path::tt{a b c}]
// #![unsafe(simple_ident)]
// #![unsafe(simple::path)]
// #![unsafe(simple_ident_expr = "")]
// #![unsafe(simple::path::Expr = "")]
// #![unsafe(simple_ident_tt(a b c))]
// #![unsafe(simple_ident_tt[a b c])]
// #![unsafe(simple_ident_tt{a b c})]
// #![unsafe(simple::path::tt(a b c))]
// #![unsafe(simple::path::tt[a b c])]
// #![unsafe(simple::path::tt{a b c})]
pub(super) fn meta(p: &mut Parser<'_>) {
    let meta = p.start();
    let is_unsafe = p.eat(T![unsafe]);
    if is_unsafe {
        p.expect(T!['(']);
    }
    paths::use_path(p);

    match p.current() {
        T![=] => {
            p.bump(T![=]);
            if expressions::expr(p).is_none() {
                p.error("expected expression");
            }
        }
        T!['('] | T!['['] | T!['{'] => items::token_tree(p),
        _ => {}
    }
    if is_unsafe {
        p.expect(T![')']);
    }

    meta.complete(p, META);
}
