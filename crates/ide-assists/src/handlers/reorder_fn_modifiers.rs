use ide_db::assists::AssistId;
use syntax::{AstNode, SyntaxKind, SyntaxToken, T};

use crate::{AssistContext, Assists};

// Assist: reorder_fn_modifiers
//
// Reorder and deduplicate function modifier keywords.
//
// ```
// $0const extern extern pub fn async "C" unsafe foo() {}
// ```
// ->
// ```
// pub const async unsafe extern "C" fn foo() {}
// ```
pub(crate) fn reorder_fn_modifiers(acc: &mut Assists, ctx: &AssistContext<'_, '_>) -> Option<()> {
    // FIXME: support pub(crate), pub(self)...
    // see: https://doc.rust-lang.org/reference/visibility-and-privacy.html#grammar-Visibility

    let tok = match ctx.token_at_offset() {
        syntax::TokenAtOffset::None => return None,
        syntax::TokenAtOffset::Single(tok) => {
            if is_fn_modifier(tok.kind()) {
                tok
            } else {
                return None;
            }
        }
        syntax::TokenAtOffset::Between(tok1, tok2) => {
            [tok1, tok2].into_iter().find(|t| is_fn_modifier(t.kind()))?
        }
    };

    let left = get_leftmost_modifier(tok);
    let mut collector = ModifierCollector::default();
    let right = collect_modifiers(left.clone(), &mut collector);

    if !collector.fn_kw_exist() {
        return None;
    }

    let target = left.text_range().cover(right.text_range());
    let new_text = collector.gen_new_text();
    if ctx.source_file().syntax().text().slice(target).to_string() == new_text {
        return None;
    }

    acc.add(
        AssistId::quick_fix("reorder_fn_modifiers"),
        "Fix function modifier order",
        target,
        |builder| {
            builder.replace(target, new_text);
        },
    )
}

fn get_leftmost_modifier(mut curr: SyntaxToken) -> SyntaxToken {
    while let Some(prev) = curr.prev_token() {
        if prev.kind() == SyntaxKind::WHITESPACE {
            if prev.text().contains('\n') {
                break;
            }
            curr = prev;
            continue;
        }
        if !is_fn_modifier(prev.kind()) {
            break;
        }
        curr = prev;
    }
    curr
}

fn collect_modifiers(mut curr: SyntaxToken, collector: &mut ModifierCollector) -> SyntaxToken {
    let mut rightmost = curr.clone();
    loop {
        if curr.kind() == SyntaxKind::WHITESPACE {
            if curr.text().contains('\n') {
                break;
            }
        } else {
            if !collector.collect(&curr) {
                break;
            }
            rightmost = curr.clone();
        }

        let Some(next) = curr.next_token() else { break };
        curr = next;
    }
    rightmost
}

fn is_fn_modifier(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        T![pub] | T![extern] | T![const] | T![async] | T![unsafe] | T![safe] | T![string] | T![fn]
    )
}

/// see <https://doc.rust-lang.org/reference/items/functions.html>
#[derive(Default)]
struct ModifierCollector {
    pub_kw: bool,
    const_kw: bool,
    async_kw: bool,
    safe_kw: bool,
    unsafe_kw: bool,
    extern_kw: bool,
    abi_str: Option<String>,
    fn_kw: bool,
}

impl ModifierCollector {
    fn collect(&mut self, tok: &syntax::SyntaxToken) -> bool {
        match tok.kind() {
            T![pub] => self.pub_kw = true,
            T![const] => self.const_kw = true,
            T![async] => self.async_kw = true,
            T![safe] => self.safe_kw = true,
            T![unsafe] => self.unsafe_kw = true,
            T![extern] => self.extern_kw = true,
            T![string] => {
                self.extern_kw = true;
                self.abi_str = Some(tok.text().to_owned());
            }
            T![fn] => self.fn_kw = true,
            _ => return false,
        }
        true
    }

    fn fn_kw_exist(&self) -> bool {
        self.fn_kw
    }

    fn gen_new_text(&self) -> String {
        let mut mods = Vec::new();
        if self.pub_kw {
            mods.push("pub");
        }
        if self.const_kw {
            mods.push("const");
        }
        if self.async_kw {
            mods.push("async");
        }
        if self.safe_kw {
            mods.push("safe");
        }
        if self.unsafe_kw {
            mods.push("unsafe");
        }
        if self.extern_kw {
            mods.push("extern");
        }
        if let Some(abi) = &self.abi_str {
            mods.push(abi);
        }
        if self.fn_kw {
            mods.push("fn");
        }
        mods.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{check_assist, check_assist_not_applicable};

    #[test]
    fn completion_extern_kw() {
        check_assist(reorder_fn_modifiers, r#"$0fn "C" foo() {}"#, r#"extern "C" fn foo() {}"#);
    }

    #[test]
    fn reorder_kw() {
        check_assist(
            reorder_fn_modifiers,
            r#"const extern pub fn$0 async "C" unsafe foo() {}"#,
            r#"pub const async unsafe extern "C" fn foo() {}"#,
        );

        check_assist(
            reorder_fn_modifiers,
            r#"async fn$0 const foo() {}"#,
            r#"const async fn foo() {}"#,
        );

        check_assist(
            reorder_fn_modifiers,
            r#"unsafe async fn$0 "C" foo() {}"#,
            r#"async unsafe extern "C" fn foo() {}"#,
        );
    }

    #[test]
    fn deduplicate_modifiers() {
        check_assist(
            reorder_fn_modifiers,
            r#"const async const unsafe async fn$0 foo() {}"#,
            r#"const async unsafe fn foo() {}"#,
        );

        check_assist(
            reorder_fn_modifiers,
            r#"unsafe unsafe unsafe unsafe unsafe fn$0 foo() {}"#,
            r#"unsafe fn foo() {}"#,
        );
    }

    #[test]
    fn not_applicable_already_correct() {
        check_assist_not_applicable(reorder_fn_modifiers, r#"const fn$0 foo() {}"#);
        check_assist_not_applicable(
            reorder_fn_modifiers,
            r#"pub const async unsafe f$0n foo() {}"#,
        );
    }

    #[test]
    fn not_applicable_different_line() {
        check_assist_not_applicable(
            reorder_fn_modifiers,
            r#"
        $0fn
        fn const foo() {}
        "#,
        );
    }
}
