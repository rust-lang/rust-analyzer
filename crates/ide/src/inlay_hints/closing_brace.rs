//! Implementation of "closing brace" inlay hints:
//! ```no_run
//! fn g() {
//! } /* fn g */
//! ```
use hir::{HirDisplay, Semantics};
use ide_db::{base_db::FileRange, RootDatabase};
use syntax::{
    ast::{self, AstNode, HasName, HasVisibility},
    match_ast, SyntaxKind, SyntaxNode, T,
};

use crate::{FileId, InlayHint, InlayHintLabel, InlayHintPosition, InlayHintsConfig, InlayKind};

pub(super) fn hints(
    acc: &mut Vec<InlayHint>,
    sema: &Semantics<'_, RootDatabase>,
    config: &InlayHintsConfig,
    file_id: FileId,
    node: SyntaxNode,
) -> Option<()> {
    let min_lines = config.closing_brace_hints_min_lines?;

    let name = |it: ast::Name| it.syntax().text_range();

    let mut closing_token;
    let (label, name_range) = if let Some(item_list) = ast::AssocItemList::cast(node.clone()) {
        closing_token = item_list.r_curly_token()?;

        let parent = item_list.syntax().parent()?;
        match_ast! {
            match parent {
                ast::Impl(imp) => {
                    let imp = sema.to_def(&imp)?;
                    let ty = imp.self_ty(sema.db);
                    let trait_ = imp.trait_(sema.db);
                    let hint_text = match trait_ {
                        Some(tr) => format!("impl {} for {}", tr.name(sema.db).display(sema.db), ty.display_truncated(sema.db, config.max_length)),
                        None => format!("impl {}", ty.display_truncated(sema.db, config.max_length)),
                    };
                    (hint_text, None)
                },
                ast::Trait(tr) => {
                    let vis = get_visibility(&tr);
                    (format!("{}trait {}", vis, tr.name()?), tr.name().map(name))
                },
                _ => return None,
            }
        }
    } else if let Some(list) = ast::ItemList::cast(node.clone()) {
        closing_token = list.r_curly_token()?;

        let module = ast::Module::cast(list.syntax().parent()?)?;
        let vis = get_visibility(&module);
        (format!("{}mod {}", vis, module.name()?), module.name().map(name))
    } else if let Some(block) = ast::BlockExpr::cast(node.clone()) {
        closing_token = block.stmt_list()?.r_curly_token()?;

        let parent = block.syntax().parent()?;
        match_ast! {
            match parent {
                ast::Fn(it) => {
                    // FIXME: this could include parameters, but `HirDisplay` prints too much info
                    // and doesn't respect the max length either, so the hints end up way too long
                    let vis = get_visibility(&it);
                    (format!("{}fn {}", vis,  it.name()?), it.name().map(name))
                },
                ast::Static(it) => {
                    let vis = get_visibility(&it);
                    (format!("{}static {}", vis, it.name()?), it.name().map(name))
                },
                ast::Const(it) => {
                    let vis = get_visibility(&it);
                    if it.underscore_token().is_some() {
                        (format!("{}const _", vis), None)
                    } else {
                        (format!("{}const {}", vis, it.name()?), it.name().map(name))
                    }
                },
                _ => return None,
            }
        }
    } else if let Some(mac) = ast::MacroCall::cast(node.clone()) {
        let last_token = mac.syntax().last_token()?;
        if last_token.kind() != T![;] && last_token.kind() != SyntaxKind::R_CURLY {
            return None;
        }
        closing_token = last_token;

        (
            format!("{}!", mac.path()?),
            mac.path().and_then(|it| it.segment()).map(|it| it.syntax().text_range()),
        )
    } else {
        return None;
    };

    if let Some(mut next) = closing_token.next_token() {
        if next.kind() == T![;] {
            if let Some(tok) = next.next_token() {
                closing_token = next;
                next = tok;
            }
        }
        if !(next.kind() == SyntaxKind::WHITESPACE && next.text().contains('\n')) {
            // Only display the hint if the `}` is the last token on the line
            return None;
        }
    }

    let mut lines = 1;
    node.text().for_each_chunk(|s| lines += s.matches('\n').count());
    if lines < min_lines {
        return None;
    }

    let linked_location = name_range.map(|range| FileRange { file_id, range });
    acc.push(InlayHint {
        range: closing_token.text_range(),
        kind: InlayKind::ClosingBrace,
        label: InlayHintLabel::simple(label, None, linked_location),
        text_edit: None,
        position: InlayHintPosition::After,
        pad_left: true,
        pad_right: false,
    });

    None
}

fn get_visibility(it: &impl HasVisibility) -> String {
    it.visibility()
        .and_then(|v| v.pub_token())
        .map_or_else(String::new, |token| token.to_string() + " ")
}

#[cfg(test)]
mod tests {
    use crate::{
        inlay_hints::tests::{check_with_config, DISABLED_CONFIG},
        InlayHintsConfig,
    };

    #[test]
    fn hints_closing_brace() {
        check_with_config(
            InlayHintsConfig { closing_brace_hints_min_lines: Some(2), ..DISABLED_CONFIG },
            r#"
fn a() {}

fn f() {
} // no hint unless `}` is the last token on the line

fn g() {
  }
//^ fn g

fn h<T>(with: T, arguments: u8, ...) {
  }
//^ fn h

trait Tr {
    fn f();
    fn g() {
    }
  //^ fn g
  }
//^ trait Tr
impl Tr for () {
  }
//^ impl Tr for ()
impl dyn Tr {
  }
//^ impl dyn Tr

static S0: () = 0;
static S1: () = {};
static S2: () = {
 };
//^ static S2
const _: () = {
 };
//^ const _

mod m {
  }
//^ mod m

m! {}
m!();
m!(
 );
//^ m!

m! {
  }
//^ m!

fn f() {
    let v = vec![
    ];
  }
//^ fn f
"#,
        );
    }

    #[test]
    fn hints_closing_brace_pub() {
        check_with_config(
            InlayHintsConfig { closing_brace_hints_min_lines: Some(2), ..DISABLED_CONFIG },
            r#"
pub fn g() {
  }
//^ pub fn g

pub fn h<T>(with: T, arguments: u8, ...) {
  }
//^ pub fn h

pub trait Tr {
    fn f();
    fn g() {
    }
  //^ fn g
  }
//^ pub trait Tr

pub static S1: () = {
 };
//^ pub static S1
pub const S2: () = {
 };
//^ pub const S2
"#,
        );
    }
}
