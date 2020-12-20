use syntax::{AstNode, ast};

use crate::{AssistId, AssistKind, assist_context::{AssistContext, Assists}};


// Assist: replace_filter_map_next_with_find_map
//
// Replaces `.filter_map(..).next()` with `.find_map()`.
//
// ```
// fn main() {
//    let m = [1, 2, 3]
//        .iter()
//        .<|>filter_map(|x| if *x == 2 { Some (4) } else { None })
//        .next();
//}
// ```
// ->
// ```
// fn main() {
//    let m = [1, 2, 3]
//        .iter()
//        .<|>find_map(|x| if *x == 2 { Some (4) } else { None });
//}
// ```
pub(crate) fn replace_filter_map_next_with_find_map(
    acc: &mut Assists,
    ctx: &AssistContext,
) -> Option<()> {
    let call: ast::MethodCallExpr = ctx.find_node_at_offset()?;
    let method = call.name_ref()?;
    if method.text().as_str() != "filter_map" {
        return None;
    }
    let method_args = method.syntax().next_sibling();
    let method_args = method_args.map(ast::ArgList::cast)??;

    let succ = call.syntax().next_sibling();
    let succ = succ.map(ast::NameRef::cast)??;
    if succ.text().as_str() != "next" {
        return None;
    }
    let succ_args = succ.syntax().next_sibling();
    let succ_args = succ_args.map(ast::ArgList::cast)??;

    let target = method.syntax().text_range().cover(succ_args.syntax().text_range());
    acc.add(
        AssistId("replace_filter_map_next_with_find_map", AssistKind::RefactorRewrite),
        "Replace with find_map",
        target,
        |edit| {
            let replacement = format!("find_map{}", method_args.to_string());
            edit.replace(target, replacement);
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn replace_filter_map_next_with_find_map_on_valid_match() {
        check_assist(
            replace_filter_map_next_with_find_map,
            r#"
fn main() {
    let m = [1, 2, 3]
        .iter()
        .<|>filter_map(|x| if *x == 2 { Some (4) } else { None })
        .next();
}
            "#,
            r#"
fn main() {
    let m = [1, 2, 3]
        .iter()
        .find_map(|x| if *x == 2 { Some (4) } else { None });
}
            "#,
        )
    }

    #[test]
    fn test_replace_filter_map_next_with_find_map_not_applicable_without_next() {
        check_assist_not_applicable(
            replace_filter_map_next_with_find_map,
            r#"
fn main() {
    let m = [1, 2, 3]
        .iter()
        .<|>filter_map(|x| if *x == 2 { Some (4) } else { None })
        .len();
}
            "#
        )
    }

    #[test]
    fn test_replace_filter_map_next_with_find_map_not_applicable_with_intervening_methods() {
        check_assist_not_applicable(
            replace_filter_map_next_with_find_map,
            r#"
fn main() {
    let m = [1, 2, 3]
        .iter()
        .<|>filter_map(|x| if *x == 2 { Some (4) } else { None })
        .map(|x| x)
        .next();
}
            "#
        )
    }
}
