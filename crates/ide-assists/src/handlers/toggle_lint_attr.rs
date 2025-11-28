use ide_db::assists::AssistId;
use syntax::{AstNode, ast};

use crate::{AssistContext, Assists};

// Assist: toggle_lint_attr
//
// Change lint attribute level.
//
// ```
// #[$0allow(dead_code)]
// fn foo() {}
// ```
// ->
// ```
// #[expect(dead_code)]
// fn foo() {}
// ```
//
// ---
//
// ```
// #[$0forbid(dead_code)]
// fn foo() {}
// ```
// ->
// ```
// #[deny(dead_code)]
// fn foo() {}
// ```
pub(crate) fn toggle_lint_attr(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let path = ctx.find_node_at_offset::<ast::Path>()?;
    let name = path.as_single_name_ref()?;
    let _meta = ast::Meta::cast(path.syntax().parent()?)?;

    let target = name.syntax().text_range();
    for &toggled_name in toggled_lint(&name)? {
        acc.add(
            AssistId::refactor("toggle_lint_attr"),
            format!("Replace `{name}` to `{toggled_name}`"),
            target,
            |builder| {
                builder.replace(target, toggled_name);
            },
        )?;
    }
    Some(())
}

fn toggled_lint(name: &ast::NameRef) -> Option<&[&'static str]> {
    Some(match name.text().as_str() {
        "allow" => &["expect", "warn"],
        "expect" => &["allow"],
        "warn" => &["deny", "allow"],
        "deny" => &["forbid", "warn"],
        "forbid" => &["deny"],
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn test_toggle_lint_attr() {
        check_assist(
            toggle_lint_attr,
            "
#[$0allow(unused_mut)]
fn foo() {}
            ",
            "
#[expect(unused_mut)]
fn foo() {}
            ",
        );

        check_assist(
            toggle_lint_attr,
            "
#[$0expect(unused_mut)]
fn foo() {}
            ",
            "
#[allow(unused_mut)]
fn foo() {}
            ",
        );

        check_assist(
            toggle_lint_attr,
            "
#[$0forbid(unused_mut)]
fn foo() {}
            ",
            "
#[deny(unused_mut)]
fn foo() {}
            ",
        );
    }

    #[test]
    fn test_toggle_lint_attr_not_applicable_other_attr() {
        check_assist_not_applicable(
            toggle_lint_attr,
            "
#[$0inline(never)]
fn foo() {}
            ",
        );
    }
}
