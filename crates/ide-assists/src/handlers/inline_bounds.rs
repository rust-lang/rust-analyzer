use either::Either;
use syntax::{
    AstNode, Direction, SyntaxKind, SyntaxToken, T,
    ast::{self, HasGenericParams, HasName, HasTypeBounds, syntax_factory::SyntaxFactory},
    syntax_editor::{Position, SyntaxEditor},
};

use crate::{
    AssistId,
    assist_context::{AssistContext, Assists},
};

// Assist: inline_bounds
//
// Inline where clause bound to generic param.
//
// ```
// pub fn serialize<S>(num: u32, serializer: S) -> Result<S::Ok, S::Error>
// where
//     $0S: Serializer,
// {
//     Ok(())
// }
// ```
// ->
// ```
// pub fn serialize<S: Serializer>(num: u32, serializer: S) -> Result<S::Ok, S::Error> {
//     Ok(())
// }
// ```
pub(crate) fn inline_bounds(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let pred = ctx.find_node_at_offset::<ast::WherePred>()?;
    let where_clause = ast::WhereClause::cast(pred.syntax().parent()?)?;
    let adt = ast::AnyHasGenericParams::cast(where_clause.syntax().parent()?)?;
    let arg = extract_pred_arg(&pred)?;
    let (has_colon, last_bound) = find_param(&adt, &arg)?;
    let bounds = pred.type_bound_list()?;

    if pred.for_binder().is_some() {
        return None;
    }

    let target = pred.syntax().text_range();
    acc.add(AssistId::refactor_rewrite("inline_bounds"), "Inline bounds", target, |builder| {
        let mut edit = builder.make_editor(pred.syntax());
        let make = SyntaxFactory::with_mappings();

        edit.insert(Position::after(&last_bound), bounds.clone_for_update().syntax());

        if has_colon {
            edit.insert_all(
                Position::after(&last_bound),
                vec![
                    make.whitespace(" ").into(),
                    make.token(T![+]).into(),
                    make.whitespace(" ").into(),
                ],
            );
        } else {
            edit.insert_all(
                Position::after(&last_bound),
                vec![make.token(T![:]).into(), make.whitespace(" ").into()],
            );
        }

        if where_clause.predicates().count() == 1 {
            delete_whitespace(&where_clause, Direction::Next, &mut edit);
            delete_whitespace(&where_clause, Direction::Prev, &mut edit);
            edit.replace(where_clause.syntax(), make.whitespace(" "));
        } else {
            delete_whitespace(&pred, Direction::Prev, &mut edit);
            delete_token(&pred, T![,], Direction::Next, &mut edit);
            edit.delete(pred.syntax());
        }

        edit.add_mappings(make.finish_with_mappings());
        builder.add_file_edits(ctx.vfs_file_id(), edit);
    })
}

fn delete_whitespace(node: &impl AstNode, dir: Direction, edit: &mut SyntaxEditor) {
    delete_token(node, SyntaxKind::WHITESPACE, dir, edit);
}

fn delete_token(node: &impl AstNode, kind: SyntaxKind, dir: Direction, edit: &mut SyntaxEditor) {
    let token = match dir {
        Direction::Next => node.syntax().next_sibling_or_token(),
        Direction::Prev => node.syntax().prev_sibling_or_token(),
    };
    if let Some(prev) = token
        && prev.kind() == kind
    {
        edit.delete(prev);
    }
}

fn find_param(
    adt: &ast::AnyHasGenericParams,
    arg: &Either<ast::NameRef, ast::Lifetime>,
) -> Option<(bool, SyntaxToken)> {
    let param = adt.generic_param_list()?.generic_params().find(|param| match param {
        ast::GenericParam::ConstParam(_) => false,
        ast::GenericParam::LifetimeParam(lifetime_param) => {
            lifetime_param.lifetime().is_some_and(|it| it.syntax().text() == arg.syntax().text())
        }
        ast::GenericParam::TypeParam(type_param) => {
            type_param.name().is_some_and(|it| it.syntax().text() == arg.syntax().text())
        }
    })?;

    let (has_colon, token) = match param {
        ast::GenericParam::ConstParam(_) => return None,
        ast::GenericParam::TypeParam(it) => (it.colon_token().is_some(), it.syntax().last_token()?),
        ast::GenericParam::LifetimeParam(it) => {
            (it.colon_token().is_some(), it.syntax().last_token()?)
        }
    };

    Some((has_colon, token))
}

fn extract_pred_arg(pred: &ast::WherePred) -> Option<Either<ast::NameRef, ast::Lifetime>> {
    pred.ty()
        .and_then(|ty| match ty {
            ast::Type::PathType(path_type) => {
                path_type.path()?.as_single_name_ref().map(Either::Left)
            }
            _ => None,
        })
        .or_else(|| pred.lifetime().map(Either::Right))
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn test_inline_bound() {
        check_assist(
            inline_bounds,
            "
pub fn serialize<S>(num: u32, serializer: S) -> Result<S::Ok, S::Error>
where
    $0S: Serializer,
{
    Ok(())
}
            ",
            "
pub fn serialize<S: Serializer>(num: u32, serializer: S) -> Result<S::Ok, S::Error> {
    Ok(())
}
            ",
        );

        check_assist(
            inline_bounds,
            "
pub struct Foo<S>
where
    $0S: Serializer,
{
    foo: S,
}
            ",
            "
pub struct Foo<S: Serializer> {
    foo: S,
}
            ",
        );
    }

    #[test]
    fn test_inline_bound_with_other_predicate() {
        check_assist(
            inline_bounds,
            "
pub fn serialize<S, T>(num: T, serializer: S) -> Result<S::Ok, S::Error>
where
    $0S: Serializer,
    T: Copy,
{
    Ok(())
}
            ",
            "
pub fn serialize<S: Serializer, T>(num: T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Copy,
{
    Ok(())
}
            ",
        );
    }

    #[test]
    fn test_inline_lifetime_bound() {
        check_assist(
            inline_bounds,
            "
pub fn serialize<S>(num: u32, serializer: S) -> Result<S::Ok, S::Error>
where
    $0S: 'static,
{
    Ok(())
}
            ",
            "
pub fn serialize<S: 'static>(num: u32, serializer: S) -> Result<S::Ok, S::Error> {
    Ok(())
}
            ",
        );
    }

    #[test]
    fn test_inline_bound_lifetime() {
        check_assist(
            inline_bounds,
            "
pub fn serialize<'a, 'b>(s: &'a mut &'b str)
where
    $0'b: 'a,
{
}
            ",
            "
pub fn serialize<'a, 'b: 'a>(s: &'a mut &'b str) {
}
            ",
        );
    }

    #[test]
    fn test_inline_extra_bound() {
        check_assist(
            inline_bounds,
            "
pub fn serialize<S: Clone>(num: u32, serializer: S) -> Result<S::Ok, S::Error>
where
    $0S: Serializer + Send,
{
    Ok(())
}
            ",
            "
pub fn serialize<S: Clone + Serializer + Send>(num: u32, serializer: S) -> Result<S::Ok, S::Error> {
    Ok(())
}
            ",
        );
    }

    #[test]
    fn inline_bound_not_applicable_with_for() {
        check_assist_not_applicable(
            inline_bounds,
            "
pub fn serialize<S>(num: u32, serializer: S) -> Result<S::Ok, S::Error>
where
    for<'a> $0S: Serializer,
{
    Ok(())
}
            ",
        );
    }
}
