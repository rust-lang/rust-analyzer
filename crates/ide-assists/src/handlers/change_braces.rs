use syntax::{
    ast::{self, edit_in_place::Indent, syntax_factory::SyntaxFactory},
    AstNode, SyntaxNode,
};

use crate::{AssistContext, AssistId, AssistKind, Assists};

// Assist: add_braces
//
// Adds braces to lambda and match arm expressions.
//
// ```
// fn foo(n: i32) -> i32 {
//     match n {
//         1 =>$0 n + 1,
//         _ => 0
//     }
// }
// ```
// ->
// ```
// fn foo(n: i32) -> i32 {
//     match n {
//         1 => {
//             n + 1
//         },
//         _ => 0
//     }
// }
// ```

// Assist: remove_braces
//
// Removes braces from lambda and match arm expressions.
//
// ```
// fn foo(n: i32) -> i32 {
//     match n {
//         1 =>$0 {
//             n + 1
//         },
//         _ => 0
//     }
// }
// ```
// ->
// ```
// fn foo(n: i32) -> i32 {
//     match n {
//         1 => n + 1,
//         _ => 0
//     }
// }
// ```
pub(crate) fn change_braces(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let change = get_change(ctx)?;
    acc.add(change.assist_id(), change.label(), change.root_syntax().text_range(), |builder| {
        let mut editor = builder.make_editor(change.root_syntax());
        let make = SyntaxFactory::new();
        editor.replace(change.root_syntax(), change.replacement(&make));
        editor.add_mappings(make.finish_with_mappings());
        builder.add_file_edits(ctx.file_id(), editor);
    })
}

fn get_change(ctx: &AssistContext<'_>) -> Option<ChangeBraces> {
    let (expr, parent) = if let Some(match_arm) = ctx.find_node_at_offset::<ast::MatchArm>() {
        (match_arm.expr().unwrap(), ParentKind::MatchArmExpr(match_arm))
    } else if let Some(closure_expr) = ctx.find_node_at_offset::<ast::ClosureExpr>() {
        (closure_expr.body().unwrap(), ParentKind::ClosureExpr)
    } else {
        return None;
    };

    let change = match expr {
        ast::Expr::BlockExpr(block_expr) => {
            if block_expr.statements().next().is_some() {
                return None;
            }

            let tail_expr = block_expr.tail_expr()?;

            ChangeKind::RemoveBraces { block_expr, tail_expr }
        }
        other => ChangeKind::AddBraces(other),
    };

    Some(ChangeBraces { kind: change, parent })
}

struct ChangeBraces {
    kind: ChangeKind,
    parent: ParentKind,
}

impl ChangeBraces {
    fn assist_id(&self) -> AssistId {
        let s = match &self.kind {
            ChangeKind::AddBraces(_) => "add_braces",
            ChangeKind::RemoveBraces { .. } => "remove_braces",
        };

        AssistId(s, AssistKind::RefactorRewrite)
    }

    fn label(&self) -> String {
        let change_part = match &self.kind {
            ChangeKind::AddBraces(_) => "Add braces to",
            ChangeKind::RemoveBraces { .. } => "Remove braces from",
        };

        let parent_str = match &self.parent {
            ParentKind::MatchArmExpr(_) => "arm expression",
            ParentKind::ClosureExpr => "closure body",
        };

        format!("{change_part} {parent_str}")
    }

    fn root_syntax(&self) -> &SyntaxNode {
        match &self.kind {
            ChangeKind::AddBraces(expr) => expr.syntax(),
            ChangeKind::RemoveBraces { block_expr, .. } => match &self.parent {
                ParentKind::MatchArmExpr(match_arm) => match_arm.syntax(),
                _ => block_expr.syntax(),
            },
        }
    }

    fn replacement(&self, make: &SyntaxFactory) -> SyntaxNode {
        match &self.kind {
            ChangeKind::AddBraces(expr) => {
                let block_expr = make.block_expr([], Some(expr.clone()));
                block_expr.indent(expr.indent_level());
                block_expr.syntax().clone()
            }
            ChangeKind::RemoveBraces { tail_expr, .. } => match &self.parent {
                ParentKind::MatchArmExpr(match_arm) => make
                    .match_arm(match_arm.pat().unwrap(), match_arm.guard(), tail_expr.clone())
                    .syntax()
                    .clone(),
                _ => tail_expr.syntax().clone_for_update(),
            },
        }
    }
}

enum ChangeKind {
    AddBraces(ast::Expr),
    RemoveBraces { block_expr: ast::BlockExpr, tail_expr: ast::Expr },
}

// We have to keep track of the match arm in case we need to add a comma
enum ParentKind {
    MatchArmExpr(ast::MatchArm),
    ClosureExpr,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::{check_assist, check_assist_not_applicable};

    #[test]
    fn suggest_add_braces_for_closure() {
        check_assist(
            change_braces,
            r#"
fn foo() {
t(|n|$0 n + 100);
}
"#,
            r#"
fn foo() {
t(|n| {
    n + 100
});
}
"#,
        );
    }

    #[test]
    fn suggest_add_braces_for_match() {
        check_assist(
            change_braces,
            r#"
fn foo() {
match n {
    Some(n) $0=> 29,
    _ => ()
};
}
"#,
            r#"
fn foo() {
match n {
    Some(n) => {
        29
    },
    _ => ()
};
}
"#,
        );
    }

    #[test]
    pub(crate) fn suggest_remove_braces_for_closure() {
        check_assist(
            change_braces,
            r#"
fn foo() {
t(|n|$0 {
    n + 100
});
}
"#,
            r#"
fn foo() {
t(|n| n + 100);
}
"#,
        );
    }

    #[test]
    pub(crate) fn no_assist_for_closures_with_statements() {
        check_assist_not_applicable(
            change_braces,
            r#"
fn foo() {
t(|n|$0 {
    panic!();
    n + 100
});
}
"#,
        );
    }

    #[test]
    pub(crate) fn suggest_remove_braces_for_match() {
        check_assist(
            change_braces,
            r#"
fn foo() {
match n {
    Some(n) $0=> {
        29
    },
    _ => ()
};
}
"#,
            r#"
fn foo() {
match n {
    Some(n) => 29,
    _ => ()
};
}
"#,
        );
    }

    #[test]
    pub(crate) fn no_assist_for_match_with_statements() {
        check_assist_not_applicable(
            change_braces,
            r#"
fn foo() {
match n {
    Some(n) $0=> {
        panic!();
        29
    },
    _ => ()
};
}
"#,
        );
    }

    #[test]
    pub(crate) fn handle_adding_comma() {
        check_assist(
            change_braces,
            r#"
fn foo() {
match n {
    Some(n) $0=> {
        29
    }
    _ => ()
};
}
"#,
            r#"
fn foo() {
match n {
    Some(n) => 29,
    _ => ()
};
}
"#,
        );
    }
}
