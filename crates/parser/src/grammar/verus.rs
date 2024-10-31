use super::{items::ITEM_RECOVERY_SET, *};

// referenced atom::closure_expr
pub(crate) fn verus_closure_expr(p: &mut Parser<'_>) -> CompletedMarker {
    let m = p.start();
    p.eat(T![forall]);
    p.eat(T![exists]);
    p.eat(T![choose]);

    if !p.at(T![|]) {
        p.error("expected `|`");
        return m.complete(p, CLOSURE_EXPR);
    }
    params::param_list_closure(p);
    attributes::inner_attrs(p);
    expressions::expr(p);
    m.complete(p, CLOSURE_EXPR)
}

pub(crate) fn verus_ret_type(p: &mut Parser<'_>) -> () {
    if p.at(T![->]) {
        let m = p.start();
        p.bump(T![->]);
        if p.at(T![tracked]) {
            p.expect(T![tracked]);
        }
        if p.at(T!['(']) && p.nth_at(1, IDENT) && p.nth_at(2, T![:]) {
            // verus named param
            p.bump(T!['(']);
            patterns::pattern(p);
            p.expect(T![:]);
            types::type_no_bounds(p);
            p.expect(T![')']);
        } else {
            types::type_no_bounds(p);
        }
        m.complete(p, RET_TYPE);
    }
}

pub(crate) fn view_expr(p: &mut Parser<'_>, lhs: CompletedMarker) -> CompletedMarker {
    assert!(p.at(T![@]));
    let m = lhs.precede(p);
    p.bump(T![@]);
    m.complete(p, VIEW_EXPR)
}

pub(crate) fn is_expr(p: &mut Parser<'_>, lhs: CompletedMarker) -> CompletedMarker {
    assert!(p.at(T![is]));
    let m = lhs.precede(p);
    p.bump(T![is]);
    types::type_no_bounds(p);
    m.complete(p, IS_EXPR)
}

pub(crate) fn matches_expr(p: &mut Parser<'_>, lhs: CompletedMarker) -> CompletedMarker {
    assert!(p.at(T![matches]));
    let m = lhs.precede(p);
    p.bump(T![matches]);
    patterns::pattern(p);
    m.complete(p, MATCHES_EXPR)
}

pub(crate) fn arrow_expr(p: &mut Parser<'_>, lhs: CompletedMarker) -> CompletedMarker {
    assert!(p.at(T![->]));
    let m = lhs.precede(p);
    p.bump(T![->]);
    if p.at(IDENT) {
        p.bump(IDENT);
    } else if p.at(NAME_REF) {
        p.bump(NAME_REF);
    } else {
        p.expect(INT_NUMBER);
    }
    m.complete(p, ARROW_EXPR)
}

pub(crate) fn publish(p: &mut Parser<'_>) -> CompletedMarker {
    let m = p.start();
    if p.at(T![open]) {
        p.bump(T![open]);
        m.complete(p, PUBLISH)
    } else if p.at(T![closed]) {
        p.bump(T![closed]);
        m.complete(p, PUBLISH)
    } else {
        p.error("TODO: expected open or closed or publish.");
        m.complete(p, ERROR)
    }
}

pub(crate) fn fn_mode(p: &mut Parser<'_>) -> CompletedMarker {
    let m = p.start();
    if p.at(T![proof]) {
        p.bump(T![proof]);
        m.complete(p, FN_MODE)
    } else if p.at(T![exec]) {
        p.bump(T![exec]);
        m.complete(p, FN_MODE)
    } else if p.at(T![spec]) {
        p.bump(T![spec]);
        if p.at(T!['(']) {
            p.expect(T!['(']);
            p.expect(T![checked]);
            p.expect(T![')']);
        }
        m.complete(p, FN_MODE)
    } else {
        p.error("Expected spec/spec(checked)/proof/exec.");
        m.complete(p, ERROR)
    }
}

pub(crate) fn broadcast_group(p: &mut Parser<'_>, m: Marker) -> CompletedMarker {
    //p.bump(T![group]);
    p.expect(IDENT); // group name
    p.expect(T!['{']);
    while !p.at(EOF) && !p.at(T!['}']) {
        attributes::inner_attrs(p);
        paths::use_path(p);

        if p.at(T!['}']) {
            break;
        }
        if p.at(T![,]) {
            p.bump(T![,]);
        } 
    }
    p.expect(T!['}']);
    m.complete(p, BROADCAST_GROUP)
}

pub(crate) fn broadcast_use_list(p: &mut Parser<'_>, m: Marker) -> CompletedMarker {
    p.expect(T![use]);
    while !p.at(EOF) && !p.at(T![;]) {
        paths::use_path(p);

        if p.at(T![;]) {
            break;
        }
        if p.at(T![,]) {
            p.bump(T![,]);
        }
    }
    p.expect(T![;]);
    m.complete(p, BROADCAST_USE_LIST)
}

pub(crate) fn data_mode(p: &mut Parser<'_>) -> CompletedMarker {
    let m = p.start();
    if p.at(T![ghost]) {
        p.bump(T![ghost]);
        m.complete(p, DATA_MODE)
    } else if p.at(T![tracked]) {
        p.bump(T![tracked]);
        m.complete(p, DATA_MODE)
    } else {
        p.error("Err: expected ghost/tracked");
        m.complete(p, ERROR)
    }
}

pub(crate) fn assume(p: &mut Parser<'_>, m: Marker) -> CompletedMarker {
    p.expect(T![assume]);
    p.expect(T!['(']);
    expressions::expr(p);
    p.expect(T![')']);
    m.complete(p, ASSUME_EXPR)
}

// AssertExpr =
//   'assert' '(' Expr ')' 'by'? ( '(' Name ')' )?  RequiresClause? BlockExpr?
pub(crate) fn assert(p: &mut Parser<'_>, m: Marker) -> CompletedMarker {
    if p.nth_at(1, T![forall]) {
        return assert_forall(p, m);
    }

    p.expect(T![assert]);
    if p.at(T!['(']) {
        // parse expression here
        p.expect(T!['(']);
        expressions::expr(p);
        p.expect(T![')']);
    } else {
        p.error("assert must be followed by left parenthesis or forall");
    }

    // parse optional `by`
    // bit_vector, nonlinear_artih ...
    if p.at(T![by]) {
        p.expect(T![by]);
        if p.at(T!['(']) {
            p.expect(T!['(']);
            // p.bump_any();
            name_r(p, ITEM_RECOVERY_SET);
            p.expect(T![')']);
        }
    }

    // parse optional 'requires`
    if p.at(T![requires]) {
        requires(p);
    }

    if p.at(T![;]) || p.at(T![,]) {
        // end of assert_expr
    } else {
        // parse optional 'proof block'
        expressions::block_expr(p);
    }

    m.complete(p, ASSERT_EXPR)
}

pub(crate) fn assert_forall(p: &mut Parser<'_>, m: Marker) -> CompletedMarker {
    p.expect(T![assert]);

    if !p.at(T![forall]) {
        p.error("assert forall must start with forall");
    }

    verus_closure_expr(p);
    if p.at(T![implies]) {
        p.bump(T![implies]);
        expressions::expr(p);
    }

    p.expect(T![by]);
    expressions::block_expr(p);
    m.complete(p, ASSERT_FORALL_EXPR)
}

pub(crate) fn prover(p: &mut Parser<'_>) -> CompletedMarker {
    let m = p.start();
    p.expect(T![by]);
    p.expect(T!['(']);
    name_r(p, ITEM_RECOVERY_SET);
    p.expect(T![')']);
    m.complete(p, PROVER)
}

pub(crate) fn requires(p: &mut Parser<'_>) -> CompletedMarker {
    let m = p.start();
    p.expect(T![requires]);
    expressions::expr_no_struct(p);

    while !p.at(EOF)
        && !p.at(T![recommends])
        && !p.at(T![ensures])
        && !p.at(T![decreases])
        && !p.at(T!['{'])
        && !p.at(T![;])
    {
        if p.at(T![recommends]) || p.at(T![ensures]) || p.at(T![decreases]) || p.at(T!['{']) {
            break;
        }
        if p.at(T![,]) {
            if p.nth_at(1, T![recommends])
                || p.nth_at(1, T![ensures])
                || p.nth_at(1, T![decreases])
                || p.nth_at(1, T!['{'])
            {
                break;
            } else {
                comma_cond(p);
            }
        } else {
            dbg!("requires parse error");
            p.error("TODO: please add COMMA after each requires clause.");
            return m.complete(p, ERROR);
        }
    }
    if p.at(T![,]) {
        p.expect(T![,]);
    }
    m.complete(p, REQUIRES_CLAUSE)
}

pub(crate) fn recommends(p: &mut Parser<'_>) -> CompletedMarker {
    let m = p.start();
    p.expect(T![recommends]);
    expressions::expr_no_struct(p);
    while !p.at(EOF) && !p.at(T![ensures]) && !p.at(T![decreases]) && !p.at(T!['{']) && !p.at(T![;])
    {
        if p.at(T![recommends])
            || p.at(T![ensures])
            || p.at(T![decreases])
            || p.at(T!['{'])
            || p.at(T![via])
        {
            break;
        }
        if p.at(T![,]) {
            if p.nth_at(1, T![recommends])
                || p.nth_at(1, T![ensures])
                || p.nth_at(1, T![decreases])
                || p.nth_at(1, T!['{'])
                || p.nth_at(1, T![via])
            {
                break;
            } else {
                comma_cond(p);
            }
        } else {
            dbg!("recommends parse error");
            p.error("TODO: please add COMMA after each recommends clause.");
            return m.complete(p, ERROR);
        }
    }
    if p.at(T![,]) {
        p.expect(T![,]);
    }
    if p.at(T![via]) {
        p.expect(T![via]);
        expressions::expr_no_struct(p);
    }
    m.complete(p, RECOMMENDS_CLAUSE)
}

pub(crate) fn ensures(p: &mut Parser<'_>) -> CompletedMarker {
    let m = p.start();
    p.expect(T![ensures]);
    expressions::expr_no_struct(p);

    while !p.at(EOF) && !p.at(T![decreases]) && !p.at(T!['{']) && !p.at(T![;]) {
        if p.at(T![recommends]) || p.at(T![ensures]) || p.at(T![decreases]) || p.at(T!['{']) {
            break;
        }
        if p.at(T![,]) {
            if p.nth_at(1, T![recommends])
                || p.nth_at(1, T![ensures])
                || p.nth_at(1, T![decreases])
                || p.nth_at(1, T!['{'])
            {
                break;
            } else {
                comma_cond(p);
            }
        } else {
            dbg!("ensures parse error");
            p.error("TODO: please add COMMA after each ensures clause.");
            return m.complete(p, ERROR);
        }
    }
    if p.at(T![,]) {
        p.expect(T![,]);
    }
    m.complete(p, ENSURES_CLAUSE)
}

pub(crate) fn invariants_except_break(p: &mut Parser<'_>) -> CompletedMarker {
    let m = p.start();
    p.expect(T![invariant_except_break]);
    expressions::expr_no_struct(p);

    while !p.at(EOF) && !p.at(T![decreases]) && !p.at(T!['{']) && !p.at(T![;]) {
        if p.at(T![invariant])
            || p.at(T![recommends])
            || p.at(T![ensures])
            || p.at(T![decreases])
            || p.at(T!['{'])
        {
            break;
        }
        if p.at(T![,]) {
            if p.nth_at(1, T![recommends])
                || p.nth_at(1, T![invariant])
                || p.nth_at(1, T![ensures])
                || p.nth_at(1, T![decreases])
                || p.nth_at(1, T!['{'])
            {
                break;
            } else {
                comma_cond(p);
            }
        } else {
            dbg!("invariant_except_breakss parse error");
            p.error("TODO: please add COMMA after each invariant_except_break clause.");
            return m.complete(p, ERROR);
        }
    }
    if p.at(T![,]) {
        p.expect(T![,]);
    }
    m.complete(p, INVARIANT_EXCEPT_BREAK_CLAUSE)
}

pub(crate) fn invariants(p: &mut Parser<'_>) -> CompletedMarker {
    let m = p.start();
    p.expect(T![invariant]);
    expressions::expr_no_struct(p);

    while !p.at(EOF) && !p.at(T![decreases]) && !p.at(T!['{']) && !p.at(T![;]) {
        if p.at(T![recommends]) || p.at(T![ensures]) || p.at(T![decreases]) || p.at(T!['{']) {
            break;
        }
        if p.at(T![,]) {
            if p.nth_at(1, T![recommends])
                || p.nth_at(1, T![ensures])
                || p.nth_at(1, T![decreases])
                || p.nth_at(1, T!['{'])
            {
                break;
            } else {
                comma_cond(p);
            }
        } else {
            dbg!("invariants parse error");
            p.error("TODO: please add COMMA after each invariants clause.");
            return m.complete(p, ERROR);
        }
    }
    if p.at(T![,]) {
        p.expect(T![,]);
    }
    m.complete(p, INVARIANT_CLAUSE)
}

pub(crate) fn decreases(p: &mut Parser<'_>) -> CompletedMarker {
    let m = p.start();
    p.expect(T![decreases]);
    expressions::expr_no_struct(p);
    while !p.at(EOF) && !p.at(T!['{']) && !p.at(T![;]) {
        if p.at(T![recommends])
            || p.at(T![ensures])
            || p.at(T![decreases])
            || p.at(T!['{'])
            || p.at(T![via])
            || p.at(T![when])
        {
            break;
        }
        if p.at(T![,]) {
            if p.nth_at(1, T![recommends])
                || p.nth_at(1, T![ensures])
                || p.nth_at(1, T![decreases])
                || p.nth_at(1, T!['{'])
                || p.nth_at(1, T![via])
                || p.nth_at(1, T![when])
            {
                break;
            } else {
                comma_cond(p);
            }
        } else {
            dbg!("decreases parsing error");
            p.error("TODO: please add COMMA after each decreases clause.");
            return m.complete(p, ERROR);
        }
    }
    if p.at(T![,]) {
        p.expect(T![,]);
    }
    m.complete(p, DECREASES_CLAUSE)
}

pub(crate) fn signature_decreases(p: &mut Parser<'_>) -> CompletedMarker {
    let m = p.start();
    decreases(p);
    if p.at(T![when]) {
        p.expect(T![when]);
        expressions::expr_no_struct(p);
    }
    if p.at(T![via]) {
        p.expect(T![via]);
        expressions::expr_no_struct(p);
    }
    m.complete(p, SIGNATURE_DECREASES)
}

fn comma_cond(p: &mut Parser<'_>) -> () {
    p.expect(T![,]);
    expressions::expr_no_struct(p);
}

pub(crate) fn trigger_attribute(p: &mut Parser<'_>, inner:bool) -> CompletedMarker {
    let m = p.start();
    p.expect(T![trigger]);
    if inner {
        expressions::expr_no_struct(p);
        while !p.at(EOF) && !p.at(T![']']) {
            if !p.at(T![,]) {
                break;
            }
            p.expect(T![,]);
            expressions::expr_no_struct(p);
        }

        if p.at(T![,]) {
            p.expect(T![,]);
        }
    }

    m.complete(p, TRIGGER_ATTRIBUTE)
}

// pub(crate) fn global_clause(p: &mut Parser<'_>, m: Marker) {
//     println!("global_clause");
//     //global size_of usize == 8;
//     p.bump(T![global]);
//     // name(p);
//     // name(p);
//     p.expect(T![==]);
//     p.expect(INT_NUMBER);
//     p.expect(T![;]);

//     m.complete(p, VERUS_GLOBAL);
//     todo!("are we ehre yet?")
// }
