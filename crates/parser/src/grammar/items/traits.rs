use super::*;

// test trait_item
// trait T { fn new() -> Self; }
pub(super) fn trait_(p: &mut Parser<'_>, m: Marker) {
    // test impl_restrictions
    // pub impl(crate) unsafe trait Foo {}
    // impl(in super::bar) trait Bar {}
    // impl () {}
    // impl (i32) {}
    if p.at(T![impl]) {
        let restriction = p.start();
        p.bump(T![impl]);
        if !opt_visibility_inner(p, false) {
            p.error("expected an impl restriction");
        }
        restriction.complete(p, IMPL_RESTRICTION);
    }

    // test const_trait_and_impl
    // const trait Trait {}
    // const impl Trait for Type {}
    p.eat(T![const]);
    p.eat(T![unsafe]);
    // test const_unsafe_auto_trait
    // const unsafe auto trait T {}
    if p.at_contextual_kw(T![auto]) && p.nth(1) == T![trait] {
        p.bump_remap(T![auto]);
    }

    // rustc: `parse_item_trait`, gated by `check_trait_front_matter`. That gate returns
    // `true` for `impl(in ..)` without checking that a trait actually follows — rustc does
    // the same and leans on its fallible parser to back out. We have no such recovery, so
    // bail here instead of asserting on `trait`.
    // The impl restriction above always consumed tokens on that path, so the caller still
    // makes progress.

    // test_err impl_restriction_without_trait
    // impl(in foo)
    // impl(in foo) struct S;
    // impl(in foo) const unsafe
    if !p.at(T![trait]) {
        p.error("expected `trait`");
        m.complete(p, TRAIT);
        return;
    }
    p.bump(T![trait]);
    name_r(p, ITEM_RECOVERY_SET);

    // test trait_item_generic_params
    // trait X<U: Debug + Display> {}
    generic_params::opt_generic_param_list(p);

    if p.eat(T![=]) {
        // test trait_alias
        // trait Z<U> = T<U>;
        generic_params::bounds_without_colon(p);

        // test trait_alias_where_clause
        // trait Z<U> = T<U> where U: Copy;
        // trait Z<U> = where Self: T<U>;
        generic_params::opt_where_clause(p);
        p.expect(T![;]);
        m.complete(p, TRAIT);
        return;
    }

    if p.at(T![:]) {
        // test trait_item_bounds
        // trait T: Hash + Clone {}
        generic_params::bounds(p);
    }

    // test trait_item_where_clause
    // trait T where Self: Copy {}
    generic_params::opt_where_clause(p);

    if p.at(T!['{']) {
        assoc_item_list(p);
    } else {
        p.error("expected `{`");
    }
    m.complete(p, TRAIT);
}

// test impl_item
// impl S {}
pub(super) fn impl_(p: &mut Parser<'_>, m: Marker) {
    // rustc: `parse_item_impl` parses constness then safety then expects `impl`, but its
    // gate `check_impl_frontmatter` accepts `const`/`unsafe` in *any* order and repeated
    // (an over-approximation rustc absorbs via its fallible parser). We match the gate's
    // whole language here, same as `parse_fn_front_matter` does for functions, and report
    // a misordered or duplicated qualifier instead of silently accepting it — otherwise
    // e.g. `unsafe const impl` reaches the `impl` bump while still sitting on `const`.

    // test impl_item_const
    // const impl Send for S {}

    // test_err impl_qualifier_recovery
    // unsafe const impl T for S {}
    // const const impl T for S {}
    // unsafe unsafe impl T for S {}
    const CONST: u8 = 0;
    const SAFETY: u8 = 1;
    let mut prev_rank = None;
    while p.at(T![const]) || p.at(T![unsafe]) {
        let rank = if p.at(T![const]) { CONST } else { SAFETY };
        match prev_rank {
            Some(prev) if rank < prev => p.error("wrong order of qualifiers"),
            Some(prev) if rank == prev => p.error("duplicate qualifier"),
            _ => (),
        }
        prev_rank = Some(rank);
        p.bump_any();
    }
    // `expect`, not `bump`: the gate only guarantees `impl` within the first three tokens,
    // so a repeated qualifier can leave us short of it. The loop above consumed at least
    // one token whenever `impl` is missing here, so the caller still makes progress.
    p.expect(T![impl]);
    if p.at(T![<]) && not_a_qualified_path(p) {
        generic_params::opt_generic_param_list(p);
    }

    // test impl_item_never_type
    // impl ! {}
    if p.at(T![!]) && !p.nth_at(1, T!['{']) {
        // test impl_item_neg
        // impl !Send for S {}
        p.eat(T![!]);
    }
    impl_type(p);
    if p.eat(T![for]) {
        impl_type(p);
    }
    generic_params::opt_where_clause(p);
    if p.at(T!['{']) {
        assoc_item_list(p);
    } else {
        p.error("expected `{`");
    }
    m.complete(p, IMPL);
}

// test assoc_item_list
// impl F {
//     type A = i32;
//     const B: i32 = 92;
//     fn foo() {}
//     fn bar(&self) {}
// }
pub(crate) fn assoc_item_list(p: &mut Parser<'_>) {
    assert!(p.at(T!['{']));

    let m = p.start();
    p.bump(T!['{']);
    // test assoc_item_list_inner_attrs
    // impl S { #![attr] }
    attributes::inner_attrs(p);

    while !p.at(EOF) && !p.at(T!['}']) {
        if p.at(T!['{']) {
            error_block(p, "expected an item");
            continue;
        }
        let pos = p.pos();
        item_or_macro(p, true);
        ensure_progress(p, pos);
    }
    p.expect(T!['}']);
    m.complete(p, ASSOC_ITEM_LIST);
}

// test impl_type_params
// impl<const N: u32> Bar<N> {}
fn not_a_qualified_path(p: &Parser<'_>) -> bool {
    // There's an ambiguity between generic parameters and qualified paths in impls.
    // If we see `<` it may start both, so we have to inspect some following tokens.
    // The following combinations can only start generics,
    // but not qualified paths (with one exception):
    //     `<` `>` - empty generic parameters
    //     `<` `#` - generic parameters with attributes
    //     `<` `const` - const generic parameters
    //     `<` (LIFETIME_IDENT|IDENT) `>` - single generic parameter
    //     `<` (LIFETIME_IDENT|IDENT) `,` - first generic parameter in a list
    //     `<` (LIFETIME_IDENT|IDENT) `:` - generic parameter with bounds
    //     `<` (LIFETIME_IDENT|IDENT) `=` - generic parameter with a default
    // The only truly ambiguous case is
    //     `<` IDENT `>` `::` IDENT ...
    // we disambiguate it in favor of generics (`impl<T> ::absolute::Path<T> { ... }`)
    // because this is what almost always expected in practice, qualified paths in impls
    // (`impl <Type>::AssocTy { ... }`) aren't even allowed by type checker at the moment.
    if [T![#], T![>], T![const]].contains(&p.nth(1)) {
        return true;
    }
    ([LIFETIME_IDENT, IDENT].contains(&p.nth(1)))
        && ([T![>], T![,], T![:], T![=]].contains(&p.nth(2)))
}

// test_err impl_type
// impl Type {}
// impl Trait1 for T {}
// impl impl NotType {}
// impl Trait2 for impl NotType {}
pub(crate) fn impl_type(p: &mut Parser<'_>) {
    if p.at(T![impl]) {
        p.error("expected trait or type");
        return;
    }
    types::type_(p);
}
