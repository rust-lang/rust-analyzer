use ast::make;
use ide_db::FxHashSet;
use syntax::{
    SyntaxKind::{self, COMMENT},
    SyntaxNode, T, TextRange,
    ast::{self, AstNode, AstToken, HasName, RangeItem},
};

use crate::{
    AssistId,
    assist_context::{AssistContext, Assists},
};

// Assist: merge_let_stmts
//
// Merge multiple lets into a let tuple pattern,
// allowing for the use of multiple identical let-else.
//
// ```
// fn main() {
//     $0let a = 2;
//     let b = 3;$0
// }
// ```
// ->
// ```
// fn main() {
//     let (a, b) = (2, 3);
// }
// ```
//
// If multiple lets are not selected, merge the current let and next let.
//
// ```
// fn main() {
//     $0let a: i32 = 2;
//     let b = 3;
// }
// ```
// ->
// ```
// fn main() {
//     let (a, b): (i32, _) = (2, 3);
// }
// ```
pub(crate) fn merge_let_stmts(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let let_stmts = get_target_let_stmts(ctx)?;
    let range = stmts_range(&let_stmts);

    let pats = let_stmts.iter().map(|let_stmt| let_stmt.pat()).collect::<Option<Vec<_>>>()?;
    let has_ty = let_stmts.iter().any(|let_stmt| let_stmt.ty().is_some());
    let types = has_ty.then_some(
        let_stmts.iter().map(|let_stmt| let_stmt.ty().unwrap_or(make::ty_placeholder())),
    );
    let initializers =
        let_stmts.iter().map(|let_stmt| let_stmt.initializer()).collect::<Option<Vec<_>>>()?;
    let idents = pats.iter().cloned().flat_map(pat_vars).collect::<Vec<_>>();
    check_duplicate_pat(&idents)?;

    let let_else = extract_let_else(&let_stmts)?;

    acc.add(
        AssistId::refactor_rewrite("merge_let_stmts"),
        "Merge let statements",
        range,
        |builder| {
            let pattern = ast::Pat::from(make::tuple_pat(pats));
            let ty = types.map(make::ty_tuple);
            let expr = ast::Expr::from(make::expr_tuple(initializers));

            let output = if let Some(let_else) = let_else {
                make::let_else_stmt(pattern, ty, expr, let_else)
            } else {
                make::let_stmt(pattern, ty, Some(expr))
            };

            builder.replace(range, output.to_string());
        },
    )
}

fn get_target_let_stmts(ctx: &AssistContext<'_>) -> Option<Vec<ast::LetStmt>> {
    let has_next = ctx.has_empty_selection();
    let selected = ctx.selection_trimmed();
    let node = ctx.covering_element();

    if matches!(node.kind(), T!['{'] | T!['}'] | T!['('] | T![')'] | T!['['] | T![']']) {
        return None;
    }

    if node.kind() == COMMENT {
        return None;
    }

    let mut node = match node {
        syntax::NodeOrToken::Node(n) => n,
        syntax::NodeOrToken::Token(t) => t.parent()?,
    };
    while let SyntaxKind::LET_STMT = node.kind() {
        node = node.parent()?;
    }

    let let_stmts = get_let_stmts(&node, selected, has_next)?;
    if let_stmts.len() <= 1
        || let_stmts
            .iter()
            .any(|let_stmt| let_stmt.initializer().is_none() || let_stmt.pat().is_none())
    {
        return None;
    }
    Some(let_stmts)
}

fn stmts_range(stmts: &[ast::LetStmt]) -> TextRange {
    debug_assert_ne!(stmts.len(), 0);
    let first_rng = stmts[0].syntax().text_range();
    let last_rng = stmts.last().unwrap().syntax().text_range();
    TextRange::new(first_rng.start(), last_rng.end())
}

fn get_let_stmts(
    node: &SyntaxNode,
    selected: TextRange,
    has_next: bool,
) -> Option<Vec<ast::LetStmt>> {
    if let Some(stmt_list) = ast::StmtList::cast(node.clone()) {
        selected_statements(stmt_list, selected, has_next)
            .into_iter()
            .map(|stmt| if let ast::Stmt::LetStmt(let_stmt) = stmt { Some(let_stmt) } else { None })
            .collect()
    } else {
        None
    }
}

fn selected_statements(
    parent: ast::StmtList,
    selected: TextRange,
    mut has_next: bool,
) -> Vec<ast::Stmt> {
    let not_intersect = move |stmt: &ast::Stmt| {
        selected.intersect(stmt.syntax().text_range()).is_none_or(|rng| !has_next && rng.is_empty())
    };
    let is_intersect_contain_next = |stmt: &ast::Stmt| {
        !not_intersect(stmt)
            || has_next && {
                has_next = false;
                true
            }
    };
    parent.statements().skip_while(not_intersect).take_while(is_intersect_contain_next).collect()
}

fn check_duplicate_pat<'a>(idents: impl IntoIterator<Item = &'a ast::Ident>) -> Option<()> {
    let mut set = FxHashSet::default();
    for ident in idents {
        if !set.insert(ident.text()) {
            return None;
        }
    }
    Some(())
}

fn extract_let_else<'a>(
    let_stmts: impl IntoIterator<Item = &'a ast::LetStmt>,
) -> Option<Option<ast::BlockExpr>> {
    let_stmts.into_iter().try_fold(None::<ast::BlockExpr>, |found, let_stmt| {
        if let Some(let_else) = let_stmt.let_else() {
            let block = let_else.block_expr()?;
            if found.is_some_and(|found| block.syntax().text() != found.syntax().text()) {
                None
            } else {
                Some(Some(block))
            }
        } else {
            Some(found)
        }
    })
}

fn pat_vars(pat: ast::Pat) -> Vec<ast::Ident> {
    match pat {
        ast::Pat::IdentPat(ident_pat) => ident_pat
            .name()
            .and_then(|name| name.ident_token())
            .and_then(ast::Ident::cast)
            .into_iter()
            .chain(ident_pat.pat().into_iter().flat_map(pat_vars))
            .collect(),
        ast::Pat::OrPat(or_pat) => or_pat.pats().flat_map(pat_vars).collect(),
        ast::Pat::ParenPat(paren_pat) => paren_pat.pat().map(pat_vars).unwrap_or_default(),
        ast::Pat::RangePat(range_pat) => range_pat
            .start()
            .into_iter()
            .flat_map(pat_vars)
            .chain(range_pat.end().into_iter().flat_map(pat_vars))
            .collect(),
        ast::Pat::BoxPat(box_pat) => box_pat.pat().into_iter().flat_map(pat_vars).collect(),
        ast::Pat::RefPat(ref_pat) => ref_pat.pat().into_iter().flat_map(pat_vars).collect(),
        ast::Pat::SlicePat(slice_pat) => slice_pat.pats().flat_map(pat_vars).collect(),
        ast::Pat::TuplePat(tuple_pat) => tuple_pat.fields().flat_map(pat_vars).collect(),
        ast::Pat::TupleStructPat(tuple_struct_pat) => {
            tuple_struct_pat.fields().flat_map(pat_vars).collect()
        }
        ast::Pat::RecordPat(record_pat) => record_pat
            .record_pat_field_list()
            .into_iter()
            .flat_map(|record_list| record_list.fields())
            .flat_map(|field| match (field.name_ref(), field.pat()) {
                (Some(name), None) => {
                    name.ident_token().and_then(ast::Ident::cast).into_iter().collect()
                }
                (_, Some(pat)) => pat_vars(pat),
                _ => vec![],
            })
            .collect(),
        ast::Pat::WildcardPat(_)
        | ast::Pat::ConstBlockPat(_)
        | ast::Pat::LiteralPat(_)
        | ast::Pat::MacroPat(_)
        | ast::Pat::RestPat(_)
        | ast::Pat::PathPat(_) => vec![],
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn merge_let_stmts_it_work() {
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a = 2;
                let b = 3;$0
            }
            "#,
            r#"
            fn main() {
                let (a, b) = (2, 3);
            }
            "#,
        );
    }

    #[test]
    fn merge_let_stmts_with_types() {
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a: i64 = 2;
                let b = 3;$0
            }
            "#,
            r#"
            fn main() {
                let (a, b): (i64, _) = (2, 3);
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a: i64 = 2;
                let b: = 3;$0
            }
            "#,
            r#"
            fn main() {
                let (a, b): (i64, _) = (2, 3);
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a: i64 = 2;
                let b: . = 3;$0
            }
            "#,
            r#"
            fn main() {
                let (a, b): (i64, _) = (2, 3);
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a: i64 = 2;
                let b: u32 = 3;$0
            }
            "#,
            r#"
            fn main() {
                let (a, b): (i64, u32) = (2, 3);
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a: i64 = 2;
                let b: (u32, i8) = (3, 4);$0
            }
            "#,
            r#"
            fn main() {
                let (a, b): (i64, (u32, i8)) = (2, (3, 4));
            }
            "#,
        );
    }

    #[test]
    fn merge_let_stmts_with_else_block() {
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a: i64 = 2;
                let b = 3 else { return };$0
            }
            "#,
            r#"
            fn main() {
                let (a, b): (i64, _) = (2, 3) else { return };
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a: i64 = 2;
                let b = 3 else { return; };$0
            }
            "#,
            r#"
            fn main() {
                let (a, b): (i64, _) = (2, 3) else { return; };
            }
            "#,
        );
    }

    #[test]
    fn merge_let_stmts_with_multiple_same_else_block() {
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a: i64 = 2 else { return };
                let b = 3 else { return };$0
            }
            "#,
            r#"
            fn main() {
                let (a, b): (i64, _) = (2, 3) else { return };
            }
            "#,
        );
    }

    #[test]
    fn merge_let_stmts_on_partial_selection() {
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                let a =$0 2;
                let b = 3;$0
            }
            "#,
            r#"
            fn main() {
                let (a, b) = (2, 3);
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                let a = 2$0;
                let b = 3;$0
            }
            "#,
            r#"
            fn main() {
                let (a, b) = (2, 3);
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                let a = 2$0;
                let b $0= 3;
            }
            "#,
            r#"
            fn main() {
                let (a, b) = (2, 3);
            }
            "#,
        );
    }

    #[test]
    fn merge_let_stmts_invalid_let_else_block() {
        check_assist_not_applicable(
            merge_let_stmts,
            r#"
            fn main() {
                let a =$0 2 else ;
                let b = 3;$0
            }
            "#,
        );
        check_assist_not_applicable(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a = 2 else {};
                let b = 3 else { panic!() };$0
            }
            "#,
        );
        check_assist_not_applicable(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a = 2 else {};
                let b = 3 else {/*...*/};$0
            }
            "#,
        );
    }

    #[test]
    fn merge_let_stmts_invalid_pattern() {
        check_assist_not_applicable(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a = 2;
                let  = 3;$0
            }
            "#,
        );
        check_assist_not_applicable(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a = 2;
                let . = 3;$0
            }
            "#,
        );
    }

    #[test]
    fn merge_let_stmts_duplicate_names() {
        check_assist_not_applicable(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a = 2;
                let a = 3;$0
            }
            "#,
        );
        check_assist_not_applicable(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a = 2;
                let (a,) = (3,);$0
            }
            "#,
        );
        check_assist_not_applicable(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a = 2;
                let (a @ 3,) = (3,);$0
            }
            "#,
        );
        check_assist_not_applicable(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a = 2;
                let Foo { a } = Foo { a: 2 };$0
            }
            "#,
        );
        check_assist_not_applicable(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a = 2;
                let Foo { a: a } = Foo { a: 2 };$0
            }
            "#,
        );
        check_assist_not_applicable(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a = 2;
                let Foo { a: a @ 2 } = Foo { a: 2 };$0
            }
            "#,
        );
        check_assist_not_applicable(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a = 2;
                let Foo { a: (a,) } = Foo { a: (2,) };$0
            }
            "#,
        );
    }

    #[test]
    fn merge_let_stmts_invalid_initializer() {
        check_assist_not_applicable(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a = 2;
                let b = ;$0
            }
            "#,
        );
        check_assist_not_applicable(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a = 2;
                let b = .;$0
            }
            "#,
        );
        check_assist_not_applicable(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a = .;
                let b = 2;$0
            }
            "#,
        );
    }

    #[test]
    fn merge_let_stmts_invalid_single_let() {
        check_assist_not_applicable(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a = 2;$0
                let b = 3;
            }
            "#,
        );
    }

    #[test]
    fn merge_let_stmts_with_comments() {
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a: i64 = {
                    // inner
                    2
                } else { return };
                let b = 3 else { return };$0
            }
            "#,
            r#"
            fn main() {
                let (a, b): (i64, _) = ({
                    // inner
                    2
                }, 3) else { return };
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a: i64 = {
                    /* inner */
                    2
                } else { return };
                let b = 3 else { return };$0
            }
            "#,
            r#"
            fn main() {
                let (a, b): (i64, _) = ({
                    /* inner */
                    2
                }, 3) else { return };
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a: i64 = 2 else { return /* inner */ };
                let b = 3 else { return /* inner */ };$0
            }
            "#,
            r#"
            fn main() {
                let (a, b): (i64, _) = (2, 3) else { return /* inner */ };
            }
            "#,
        );
    }

    #[test]
    fn merge_let_stmts_complex_patterns() {
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a: i64 = 2;
                let (b,) = (3,);$0
            }
            "#,
            r#"
            fn main() {
                let (a, (b,)): (i64, _) = (2, (3,));
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a: i64 = 2;
                let Foo { a: b } = todo!();$0
            }
            "#,
            r#"
            fn main() {
                let (a, Foo { a: b }): (i64, _) = (2, todo!());
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a: i64 = 2;
                let Foo { b } = todo!();$0
            }
            "#,
            r#"
            fn main() {
                let (a, Foo { b }): (i64, _) = (2, todo!());
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a: i64 = 2;
                let Foo { a: b @ 2 } = todo!();$0
            }
            "#,
            r#"
            fn main() {
                let (a, Foo { a: b @ 2 }): (i64, _) = (2, todo!());
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let a: i64 = 2;
                let _ = todo!();$0
            }
            "#,
            r#"
            fn main() {
                let (a, _): (i64, _) = (2, todo!());
            }
            "#,
        );
    }

    #[test]
    fn merge_let_stmts_with_mut_pattern() {
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let mut body = builder.translate(theta.body());
                let mut outputs = BTreeMap::new();
            }
            "#,
            r#"
            fn main() {
                let (mut body, mut outputs) = (builder.translate(theta.body()), BTreeMap::new());
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                $0let mut body = builder.translate(theta.body());
                let $0mut outputs = BTreeMap::new();
            }
            "#,
            r#"
            fn main() {
                let (mut body, mut outputs) = (builder.translate(theta.body()), BTreeMap::new());
            }
            "#,
        );
    }

    #[test]
    fn merge_let_stmts_some_other_statements_and_expressions() {
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                foo();
                {
                    $0let a: i64 = 2;
                    let (b,) = (3,);$0
                    bar();
                }
                let x = 3;
                ()
            }
            "#,
            r#"
            fn main() {
                foo();
                {
                    let (a, (b,)): (i64, _) = (2, (3,));
                    bar();
                }
                let x = 3;
                ()
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                foo();
                {
                    bar();
                    $0let a: i64 = 2;
                    let (b,) = (3,);$0
                }
                let x = 3;
                ()
            }
            "#,
            r#"
            fn main() {
                foo();
                {
                    bar();
                    let (a, (b,)): (i64, _) = (2, (3,));
                }
                let x = 3;
                ()
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                foo();
                {
                    bar();
                    $0let a: i64 = 2;
                    let (b,) = (3,);$0
                    ()
                }
                let x = 3;
                ()
            }
            "#,
            r#"
            fn main() {
                foo();
                {
                    bar();
                    let (a, (b,)): (i64, _) = (2, (3,));
                    ()
                }
                let x = 3;
                ()
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                foo();
                $0let a: i64 = 2;
                let (b,) = (3,);$0
                bar();
                let x = 3;
                ()
            }
            "#,
            r#"
            fn main() {
                foo();
                let (a, (b,)): (i64, _) = (2, (3,));
                bar();
                let x = 3;
                ()
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                foo();
                $0let a: i64 = 2;
                let (b,) = (3,);$0
                bar();
                let x = 3;
                ();
            }
            "#,
            r#"
            fn main() {
                foo();
                let (a, (b,)): (i64, _) = (2, (3,));
                bar();
                let x = 3;
                ();
            }
            "#,
        );
    }

    #[test]
    fn merge_let_stmts_in_inner_block() {
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                foo();
                {
                    $0let a: i64 = 2;
                    let b = {
                        3$0
                    };
                    bar();
                }
                let x = 3;
                ()
            }
            "#,
            r#"
            fn main() {
                foo();
                {
                    let (a, b): (i64, _) = (2, {
                        3
                    });
                    bar();
                }
                let x = 3;
                ()
            }
            "#,
        );
    }

    #[test]
    fn merge_let_stmts_no_selection() {
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                foo();
                $0let a: i64 = 2;
                let b = 3;
                let c = 4;
                bar();
                let x = 3;
                ()
            }
            "#,
            r#"
            fn main() {
                foo();
                let (a, b): (i64, _) = (2, 3);
                let c = 4;
                bar();
                let x = 3;
                ()
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                foo();
                le$0t a: i64 = 2;
                let b = 3;
                let c = 4;
                bar();
                let x = 3;
                ()
            }
            "#,
            r#"
            fn main() {
                foo();
                let (a, b): (i64, _) = (2, 3);
                let c = 4;
                bar();
                let x = 3;
                ()
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                foo();
                let$0 a: i64 = 2;
                let b = 3;
                let c = 4;
                bar();
                let x = 3;
                ()
            }
            "#,
            r#"
            fn main() {
                foo();
                let (a, b): (i64, _) = (2, 3);
                let c = 4;
                bar();
                let x = 3;
                ()
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                foo();
                le$0t a: i64 = 2;

                let b = 3;
                let c = 4;
                bar();
                let x = 3;
                ()
            }
            "#,
            r#"
            fn main() {
                foo();
                let (a, b): (i64, _) = (2, 3);
                let c = 4;
                bar();
                let x = 3;
                ()
            }
            "#,
        );
        check_assist(
            merge_let_stmts,
            r#"
            fn main() {
                foo();
                l$0et a: i64 = 2;

                let b = 3;
                let c = 4;
                bar();
                let x = 3;
                ()
            }
            "#,
            r#"
            fn main() {
                foo();
                let (a, b): (i64, _) = (2, 3);
                let c = 4;
                bar();
                let x = 3;
                ()
            }
            "#,
        );
    }

    #[test]
    fn merge_let_stmts_no_selection_invalid() {
        check_assist_not_applicable(
            merge_let_stmts,
            r#"
            fn main() {
                foo();$0
                let a: i64 = 2;
                let b = 3;
                let c = 4;
                bar();
                let x = 3;
                ()
            }
            "#,
        );
        check_assist_not_applicable(
            merge_let_stmts,
            r#"
            fn main() {
                foo();
             $0   let a: i64 = 2;
                let b = 3;
                let c = 4;
                bar();
                let x = 3;
                ()
            }
            "#,
        );
    }
}
