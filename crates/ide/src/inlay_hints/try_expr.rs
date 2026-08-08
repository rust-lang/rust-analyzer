//! Implementation of "try expr" inlay hints:
//! ```ignore
//! val.foo()/* .try */?.bar()/* .try */?
//! ```
//! This can avoid try-expr not being obvious enough
use ide_db::famous_defs::FamousDefs;
use syntax::{SyntaxToken, ast};

use crate::{InlayHint, InlayHintsConfig};

pub(super) fn hints(
    acc: &mut Vec<InlayHint>,
    FamousDefs(_sema, _): &FamousDefs<'_, '_>,
    config: &InlayHintsConfig<'_>,
    expr: ast::TryExpr,
) -> Option<()> {
    config
        .try_expr_hints
        .then(|| {
            expr.question_mark_token().map(|token| {
                acc.push(inlay_hint(token));
            })
        })
        .flatten()
}

fn inlay_hint(token: SyntaxToken) -> InlayHint {
    InlayHint {
        range: token.text_range(),
        position: crate::InlayHintPosition::Before,
        pad_left: false,
        pad_right: false,
        kind: crate::InlayKind::RangeExclusive,
        label: crate::InlayHintLabel::from(".try"),
        text_edit: None,
        resolve_parent: None,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        InlayHintsConfig,
        inlay_hints::tests::{DISABLED_CONFIG, check_with_config},
    };

    #[test]
    fn basic_works() {
        check_with_config(
            InlayHintsConfig { try_expr_hints: true, ..DISABLED_CONFIG },
            r#"
fn main() {
    foo.x()?
         //^ .try
        .bar()?
            //^ .try
        .await?;
            //^ .try
}"#,
        );
    }
}
