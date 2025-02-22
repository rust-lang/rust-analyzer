//! Implementation of "closing brace" inlay hints:
//! ```no_run
//! fn g() {
//! } /* fn g */
//! ```
use hir::{HirDisplay, Semantics};
use ide_db::{FileRange, RootDatabase};
use span::EditionedFileId;
use syntax::{
    ast::{self, AstNode, HasGenericParams, HasLoopBody, HasName, HasVisibility},
    match_ast, SyntaxKind, SyntaxNode, SyntaxToken, T,
};

use crate::{
    inlay_hints::LazyProperty, InlayHint, InlayHintLabel, InlayHintPosition, InlayHintsConfig,
    InlayKind,
};

/// Applies closing-brace inlay hints to this syntax node.
pub(super) fn hints(
    acc: &mut Vec<InlayHint>,
    sema: &Semantics<'_, RootDatabase>,
    config: &InlayHintsConfig,
    file_id: EditionedFileId,
    original_node: SyntaxNode,
) -> Option<()> {
    // If `config.closing_brace_hints_min_lines == None`, closing brace hints are disabled.
    let min_lines = config.closing_brace_hints_min_lines?;

    // FIXME: `ast::WhereClause` members of functions, structs, etc. are not currently included in closing brace inlay hints,
    //   because formatting them was annoying.
    // It would be nice to include sufficiently-short where clauses in the output.

    // Check if `node` is block-like. If it is, display an inlay hint based on the type of its parent.
    let node = original_node.clone();
    let inlay_hint = match_ast! {
        match (node) {
            ast::AssocItemList(item_list) => {
                let closing_token = item_list.r_curly_token()?;

                let parent = item_list.syntax().parent()?;
                match_ast! {
                    match parent {
                        ast::Impl(imp) => {
                            let imp = sema.to_def(&imp)?;
                            let ty = imp.self_ty(sema.db);
                            let trait_ = imp.trait_(sema.db);
                            let text = match trait_ {
                                Some(tr) => format!(
                                    "impl {} for {}",
                                    tr.name(sema.db).display(sema.db, file_id.edition()),
                                    ty.display_truncated(sema.db, config.max_length, file_id.edition(),
                                )),
                                None => format!("impl {}", ty.display_truncated(sema.db, config.max_length, file_id.edition())),
                            };
                            to_inlay_hint(config, min_lines, file_id, node, closing_token, &text, None)
                        },
                        ast::Trait(tr) => {
                            let text = format!("trait {}", tr.name()?);
                            to_inlay_hint(config, min_lines, file_id, node, closing_token, &text, Some(tr.name()?.syntax()))
                        },
                        _ => return None,
                    }
                }
            },
            ast::ItemList(list) => {
                // `node` is the item list of a `module` declaration.
                let closing_token = list.r_curly_token()?;
                let module = ast::Module::cast(list.syntax().parent()?)?;
                let text = format!("mod {}", module.name()?);
                to_inlay_hint(config, min_lines, file_id, node, closing_token, &text, Some(module.name()?.syntax()))
            },
            ast::BlockExpr(block) => {
                let closing_token = block.stmt_list()?.r_curly_token()?;

                let parent = block.syntax().parent()?;
                match_ast! {
                    match parent {
                        ast::Fn(it) => {
                            let visibility_text = it.visibility().map_or(String::new(), |it| format!("{it} "));
                            let default_text = it.default_token().map_or(String::new(), |it| format!("{it} "));
                            let const_text = it.const_token().map_or(String::new(), |it| format!("{it} "));
                            let async_text = it.async_token().map_or(String::new(), |it| format!("{it} "));
                            let unsafe_text = it.unsafe_token().map_or(String::new(), |it| format!("{it} "));
                            let safe_text = it.safe_token().map_or(String::new(), |it| format!("{it} "));
                            let abi_text = it.abi().map_or(String::new(), |it| format!("{it} "));
                            let name_text = format!(" {}", it.name()?);
                            let generic_param_text = it.generic_param_list().map_or(String::new(), |it| format!("{it}"));
                            let param_text = it.param_list().map_or(String::new(), |it| format!("{it}"));
                            let ret_type_text = it.ret_type().map_or(String::new(), |it| format!(" {it}"));

                            let text = format!("{visibility_text}{default_text}{const_text}{async_text}{unsafe_text}{safe_text}{abi_text}fn{name_text}{generic_param_text}{param_text}{ret_type_text}");
                            to_inlay_hint(config, min_lines, file_id, node, closing_token, &text, it.name().as_ref().map(|it| it.syntax()))
                        },
                        ast::Static(it) => {
                            let text = format!("static {}", it.name()?);
                            to_inlay_hint(config, min_lines, file_id, node, closing_token, &text, it.name().as_ref().map(|it| it.syntax()))
                        },
                        ast::Const(it) => {
                            let ty_text = it.ty().map_or(String::new(), |it| format!(": {it}"));

                            if it.underscore_token().is_some() {
                                let text = format!("const _{ty_text}");
                                to_inlay_hint(config, min_lines, file_id, node, closing_token, &text, None)
                            } else {
                                let text = format!("const {}{}", it.name()?, ty_text);
                                to_inlay_hint(config, min_lines, file_id, node, closing_token, &text, None)
                            }
                        },
                        ast::LoopExpr(it) => {
                            let label_text = it.label().and_then(|it| it.lifetime()).map_or(String::new(), |it| format!("{it}: "));
                            let text = format!("{label_text}loop");
                            to_inlay_hint(config, min_lines, file_id, node, closing_token, &text, None)
                        },
                        ast::WhileExpr(it) => {
                            let label_text = it.label().and_then(|it| it.lifetime()).map_or(String::new(), |it| format!("{it}: "));
                            let condition = it.condition();
                            let condition_text = condition.clone().map_or(String::new(), |it| format!(" {it}"));
                            let text = format!("{label_text}while{condition_text}");
                            to_inlay_hint(config, min_lines, file_id, node, closing_token, &text, None)
                        },
                        ast::ForExpr(it) => {
                            let label_text = it.label().and_then(|it| it.lifetime()).map_or(String::new(), |it| format!("{it}: "));
                            let pattern_text = it.pat().map_or(String::new(), |it| format!(" {it} "));
                            let iterable_text = it.iterable().map_or(String::new(), |it| format!(" {it}"));
                            let text = format!("{label_text}for{pattern_text}in{iterable_text}");
                            to_inlay_hint(config, min_lines, file_id, node, closing_token, &text, None)
                        },
                        ast::ClosureExpr(it) => {
                            let const_text = it.const_token().map_or(String::new(), |it| format!("{it} "));
                            let static_text = it.static_token().map_or(String::new(), |it| format!("{it} "));
                            let move_text = it.move_token().map_or(String::new(), |it| format!("{it} "));
                            let param_text = it.param_list().clone().map_or(String::new(), |it| format!("{it}"));
                            let ret_type_text = it.ret_type().map_or(String::new(), |it| format!(" {it}"));
                            let text = format!("{const_text}{static_text}{move_text}{param_text}{ret_type_text}");
                            to_inlay_hint(config, min_lines, file_id, node, closing_token, &text, None)
                        },
                        ast::MatchArm(it) => {
                            let pattern_text = it.pat().map_or(String::new(), |it| format!("{it}"));
                            let guard_text = it.guard().map_or(String::new(), |it| format!(" {it}"));
                            let text = format!("{pattern_text}{guard_text}");
                            to_inlay_hint(config, min_lines, file_id, node, closing_token, &text, None)
                        },
                        ast::IfExpr(parent) => {
                            // `node` is the then-branch or the else-branch of the parent if-expression.
                            let then_branch = parent.then_branch()?;
                            let then_branch = then_branch.syntax();

                            if node == *then_branch {
                                // `node` is the then-branch.
                                // Check whether `parent` is the else-branch of another parent if-expr.
                                let parent_is_else_branch = parent.syntax().parent().and_then(ast::IfExpr::cast).is_some();
                                let else_text = if parent_is_else_branch {"else "} else {""}.to_owned();

                                let condition_text = parent.condition().map_or(String::new(), |it| format!(" {it}"));
                                let if_text = format!("{else_text}if{condition_text}");
                                to_inlay_hint(config, min_lines, file_id, node, closing_token, &if_text, None)
                            } else {
                                // `node` is the else-branch.
                                match parent.else_branch()? {
                                    ast::ElseBranch::Block(_) => to_inlay_hint(config, min_lines, file_id, node, closing_token, "else", None),
                                    ast::ElseBranch::IfExpr(_) => {
                                        // Since the else branch is an if-expr, it will have its own then- and else-branches.
                                        return None;
                                    }
                                }
                            }
                        },
                        _ => {
                            // Bare block. Check for labels and modifiers.
                            let label_lifetime = block.label().and_then(|it| it.lifetime());
                            let async_token = block.async_token();
                            let move_token = block.move_token();
                            let const_token = block.const_token();
                            let unsafe_token = block.unsafe_token();
                            let try_token = block.try_token();

                            if label_lifetime.is_none() && async_token.is_none() && move_token.is_none()
                                && const_token.is_none() && unsafe_token.is_none() && try_token.is_none() {
                                    return None;
                            }

                            let label_text = label_lifetime.map_or(String::new(), |it| format!("{it}: "));
                            let async_text = async_token.map_or(String::new(), |it| format!("{it} "));
                            let move_text = move_token.map_or(String::new(), |it| format!("{it} "));
                            let const_text = const_token.map_or(String::new(), |it| format!("{it} "));
                            let unsafe_text = unsafe_token.map_or(String::new(), |it| format!("{it} "));
                            let try_text = try_token.map_or(String::new(), |it| format!("{it} "));

                            let text = format!("{label_text}{async_text}{move_text}{const_text}{unsafe_text}{try_text}");
                            let text = text.trim_end();
                            to_inlay_hint(config, min_lines, file_id, node, closing_token, text, None)
                        }
                    }
                }
            },
            ast::MacroCall(mac) => {
                let last_token = mac.syntax().last_token()?;
                if last_token.kind() != T![;] && last_token.kind() != SyntaxKind::R_CURLY {
                    return None;
                }
                let closing_token = last_token;
                let text = format!("{}!", mac.path()?);
                let target_node = mac.path().and_then(|it| it.segment());
                let target_node = target_node.as_ref().map(|it| it.syntax());

                to_inlay_hint(config, min_lines, file_id, node, closing_token, &text, target_node)
            },
            ast::MatchArmList(arm_list) => {
                // `node` is the block (`MatchArmsList`) of a match expression.
                let closing_token = arm_list.r_curly_token()?;
                let match_expr = arm_list.syntax().parent()?;
                let match_expr = ast::MatchExpr::cast(match_expr)?;

                let matched_expr_text = match_expr.expr().map_or(String::new(), |it| format!(" {it}"));
                let text = format!("match{matched_expr_text}");

                to_inlay_hint(config, min_lines, file_id, node, closing_token, &text, None)
            },
            ast::RecordFieldList(record_field_list) => {
                // `node` is one of:
                // - the record field list of a struct
                // - the record field list of an enum variant
                // - the record field list of a union
                let closing_token = record_field_list.r_curly_token()?;
                let parent = record_field_list.syntax().parent()?;

                match_ast! {
                    match parent {
                        ast::Struct(strukt) => {
                            let visibility_text = strukt.visibility().map_or(String::new(), |it| format!("{it} "));
                            let name_text = strukt.name().map_or(String::new(), |it| format!(" {it}"));
                            let generic_param_text =
                                strukt.generic_param_list().map_or(String::new(), |it| format!("{it}"));
                            let text = format!("{visibility_text}struct{name_text}{generic_param_text}");
                            to_inlay_hint(config, min_lines, file_id, node, closing_token, &text, strukt.name().as_ref().map(|it| it.syntax()))
                        },
                        ast::Variant(variant) => {
                            let visibility_text = variant.visibility().map_or(String::new(), |it| format!("{it} "));
                            let name_text = variant.name().map_or(String::new(), |it| format!("{it}"));
                            let text = format!("{visibility_text}{name_text}");

                            to_inlay_hint(config, min_lines, file_id, node, closing_token, &text, variant.name().as_ref().map(|it| it.syntax()))
                        },
                        ast::Union(parent_union) => {
                            let visibility_text = parent_union.visibility().map_or(String::new(), |it| format!("{it} "));
                            let name_text = parent_union.name().map_or(String::new(), |it| format!(" {it}"));
                            let generic_param_text =
                                parent_union.generic_param_list().map_or(String::new(), |it| format!("{it}"));
                            let text = format!("{visibility_text}union{name_text}{generic_param_text}");

                            to_inlay_hint(config, min_lines, file_id, node, closing_token, &text, parent_union.name().as_ref().map(|it| it.syntax()))
                        },
                        _ => { return None; }
                    }
                }
            },
            ast::VariantList(variant_list) => {
                // `node` is the variant list of an enum
                let closing_token = variant_list.r_curly_token()?;
                let parent_enum = variant_list.syntax().parent()?;
                let parent_enum = ast::Enum::cast(parent_enum)?;
                let visibility_text = parent_enum.visibility().map_or(String::new(), |it| format!("{it} "));
                let name_text = parent_enum.name().map_or(String::new(), |it| format!(" {it}"));
                let generic_param_text =
                    parent_enum.generic_param_list().map_or(String::new(), |it| format!("{it}"));
                let text = format!("{visibility_text}enum{name_text}{generic_param_text}");

                to_inlay_hint(config, min_lines, file_id, node, closing_token, &text, parent_enum.name().as_ref().map(|it| it.syntax()))
            },
            _ => {
                return None;
            }
        }
    }?;
    acc.push(inlay_hint);

    None
}

fn to_inlay_hint(
    config: &InlayHintsConfig,
    min_lines: usize,
    file_id: EditionedFileId,
    node: SyntaxNode,
    mut closing_token: SyntaxToken,
    label_text: &str,
    target_node: Option<&SyntaxNode>,
) -> Option<InlayHint> {
    // FIXME: To keep inlay hints from getting too long, we directly truncate them to a certain length.
    // It would probably be better to update the `HirDisplay::hir_fmt` implementations,
    //  to respect `HirDisplayWrapper` and intelligently truncate sub-tokens,
    //  which would let us use `HirDisplay::display_truncated` for the inlay hints.
    let join_and_truncate = |label: &str| -> String {
        let max_length = config.max_length;
        let mut label = label.replace('\n', " ");

        label = format!("// {label}");
        match max_length {
            Some(max_length) => {
                if label.len() > max_length {
                    label.truncate(usize::saturating_sub(max_length, 1));
                    label.push('…');
                    label
                } else {
                    label
                }
            }
            None => label.to_string(),
        }
    };

    let label_text = join_and_truncate(label_text);
    let target_range = target_node.map(|it| it.text_range());

    if let Some(mut next) = closing_token.next_token() {
        if next.kind() == T![;] || next.kind() == T![,] {
            if let Some(tok) = next.next_token() {
                closing_token = next;
                next = tok;
            }
        }
        if !(next.kind() == SyntaxKind::WHITESPACE && next.text().contains('\n')) {
            // Only display the hint if the `}` or `};` is the last token on the line
            return None;
        }
    }

    let mut lines = 1;
    node.text().for_each_chunk(|s| lines += s.matches('\n').count());
    if lines < min_lines {
        return None;
    }

    let linked_location = target_range.map(|range| FileRange { file_id: file_id.into(), range });
    InlayHint {
        range: closing_token.text_range(),
        kind: InlayKind::ClosingBrace,
        label: InlayHintLabel::simple(
            label_text,
            None,
            linked_location.map(LazyProperty::Computed),
        ),
        text_edit: None,
        position: InlayHintPosition::After,
        pad_left: true,
        pad_right: false,
        resolve_parent: Some(node.text_range()),
    }
    .into()
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

fn g() -> bool {
    false
  }
//^ // fn g() -> bool

fn h<T>(with: T, arguments: u8, ...) {
  }
//^ // fn h<T>(with: T, arguments: u8, ...)

pub fn f_pub() {
  }
//^ // pub fn f_pub()

const fn f_const() {
  }
//^ // const fn f_const()

default fn f_default() {
  }
//^ // default fn f_default()

safe fn f_safe() {
  }
//^ // safe fn f_safe()

unsafe fn f_unsafe() {
  }
//^ // unsafe fn f_unsafe()

async fn f_async() {
  }
//^ // async fn f_async()

extern "C" fn f_abi() {
  }
//^ // extern "C" fn f_abi()

trait Tr {
    fn f();
    fn g() {
    }
  //^ // fn g()
  }
//^ // trait Tr
impl Tr for () {
  }
//^ // impl Tr for ()
impl dyn Tr {
  }
//^ // impl dyn Tr

static S0: () = 0;
static S1: () = {};
static S2: () = {
 };
//^ // static S2
const _: () = {
 };
//^ // const _: ()

mod m {
  }
//^ // mod m

m! {}
m!();
m!(
 );
//^ // m!

m! {
  }
//^ // m!

fn f() {
    let v = vec![
    ];
  }
//^ // fn f()
"#,
        );
    }

    #[test]
    fn hints_closing_brace_loop_expr() {
        check_with_config(
            InlayHintsConfig { closing_brace_hints_min_lines: Some(2), ..DISABLED_CONFIG },
            r#"
fn wrap_loops() {
    'a: loop {
        loop {
            break 'a;
        }
      //^ // loop
    }
  //^ // 'a: loop
  }
//^ // fn wrap_loops()
            "#,
        );
    }

    #[test]
    fn hints_closing_brace_block_expr() {
        check_with_config(
            InlayHintsConfig { closing_brace_hints_min_lines: Some(2), ..DISABLED_CONFIG },
            r#"

    let block: ast::BlockExpr = todo!();
    let label_lifetime = block.label().map(|it| it.lifetime()).flatten();
    let async_token = block.async_token();
    let move_token = block.move_token();
    let const_token = block.const_token();
    let unsafe_token = block.unsafe_token();
    let try_token = block.try_token();
fn wrap_blocks() {
    'a: {
        {
        }
    }
  //^ // 'a:

    async {
        async move {
        }
      //^ // async move
    }
  //^ // async

    unsafe {
    }
  //^ // unsafe

    try {
    }
  //^ // try
  }
//^ // fn wrap_blocks()
            "#,
        );
    }

    #[test]
    fn hints_closing_brace_while_expr() {
        check_with_config(
            InlayHintsConfig { closing_brace_hints_min_lines: Some(2), ..DISABLED_CONFIG },
            r#"
fn while_wrapper() {
    while true {
        'a: while true {
        }
      //^ // 'a: while true
    }
  //^ // while true

    let a = false;
    while a {
        'a: while a {
        }
      //^ // 'a: while a
    }
  //^ // while a

    fn b() -> bool { true }
    while b() {
        'b: while b() {
        }
      //^ // 'b: while b()
    }
  //^ // while b()
  }
//^ // fn while_wrapper()
            "#,
        );
    }

    #[test]
    fn hints_closing_brace_while_let_expr() {
        check_with_config(
            InlayHintsConfig { closing_brace_hints_min_lines: Some(2), ..DISABLED_CONFIG },
            r#"
fn while_let_wrapper() {
    while let Some(val) = None {
    }
  //^ // while let Some(val) = None
  }
//^ // fn while_let_wrapper()
            "#,
        );
    }

    #[test]
    fn hints_closing_brace_for_in_expr() {
        check_with_config(
            InlayHintsConfig { closing_brace_hints_min_lines: Some(2), ..DISABLED_CONFIG },
            r#"
fn for_in_wrapper() {
    for _ in 0..=10 {
        'a: for _ in 0..=10 {
        }
      //^ // 'a: for _ in 0..=10
    }
  //^ // for _ in 0..=10
  }
//^ // fn for_in_wrapper()
            "#,
        );
    }

    #[test]
    fn hints_closing_brace_closure_expr() {
        check_with_config(
            InlayHintsConfig { closing_brace_hints_min_lines: Some(2), ..DISABLED_CONFIG },
            r#"
fn closure_wrapper() {
    let a = || {
        0
    };
   //^ // ||

    let b = |_: u32| {
        0
    };
   //^ // |_: u32|

   let c = |_: u64| -> u64 {
        0
   };
  //^ // |_: u64| -> u64
  }
//^ // fn closure_wrapper()
            "#,
        );
    }

    #[test]
    fn hints_closing_brace_match_expr() {
        check_with_config(
            InlayHintsConfig { closing_brace_hints_min_lines: Some(2), ..DISABLED_CONFIG },
            r#"
enum Example {
    A(u32),
    B(u32, u32),
    C(u32, u32, u32)
  }
//^ // enum Example

fn match_wrapper() {
    match Example::C(1, 2, 3) {
        Example::A(a) => {
            a
        },
       //^ // Example::A(a)
        Example::B(b1, b2) => {
            b1 + b2
        },
       //^ // Example::B(b1, b2)
        Example::C(c1, c2, c3) => {
            c1 * c2 * c3
        }
      //^ // Example::C(c1, c2, c3)
    };
   //^ // match Example::C(1, 2, 3)
  }
//^ // fn match_wrapper()
            "#,
        );
    }

    #[test]
    fn hints_closing_brace_struct() {
        check_with_config(
            InlayHintsConfig { closing_brace_hints_min_lines: Some(2), ..DISABLED_CONFIG },
            r#"
struct NoInlayHint
;

pub(in crate::foo) struct A {
  }
//^ // pub(in crate::foo) struct A

struct B {
    a: u32,
    b: u32,
  }
//^ // struct B

struct C<'a, const T_A: u32, T_B> {
    a: &'a u32,
    b: T_B,
  }
//^ // struct C<'a, const T_A: u32, T_B>

            "#,
        );
    }

    #[test]
    fn hints_closing_brace_enum() {
        check_with_config(
            InlayHintsConfig { closing_brace_hints_min_lines: Some(2), ..DISABLED_CONFIG },
            r#"
pub(in crate::foo) enum A {
  }
//^ // pub(in crate::foo) enum A

enum B {
    A,
    B,
  }
//^ // enum B

enum C<'a, const T_A: u32, T_B> {
    A(&'a u32),
    B(T_B),
  }
//^ // enum C<'a, const T_A: u32, T_B>

enum EnumStructFieldWrapper {
    A {
        a: u32,
        b: u32,
    }
  //^ // A
  }
//^ // enum EnumStructFieldWrapper
            "#,
        );
    }

    #[test]
    fn hints_closing_brace_union() {
        check_with_config(
            InlayHintsConfig { closing_brace_hints_min_lines: Some(2), ..DISABLED_CONFIG },
            r#"
pub(in crate::foo) union A {
  }
//^ // pub(in crate::foo) union A

union B {
    a: u32,
    b: u32,
  }
//^ // union B

union C<'a, const T_A: u32, T_B> {
    a: &'a u32,
    b: T_B,
  }
//^ // union C<'a, const T_A: u32, T_B>

            "#,
        );
    }

    #[test]
    fn hints_closing_brace_if() {
        check_with_config(
            InlayHintsConfig { closing_brace_hints_min_lines: Some(2), ..DISABLED_CONFIG },
            r#"
fn if_wrapper() {
    if true {
    }
  //^ // if true

    fn return_true() -> bool {
        true
    }
  //^ // fn return_true() -> bool

    if true && return_true() {
    }
  //^ // if true && return_true()

    if false {
    } else {
    }
  //^ // else

    if 0 == 1 {
    }
  //^ // if 0 == 1
    else if 0 == 2 {
    }
  //^ // else if 0 == 2
    else {
    }
  //^ // else
  }
//^ // fn if_wrapper()
            "#,
        );
    }

    #[test]
    fn hints_closing_brace_if_let() {
        check_with_config(
            InlayHintsConfig { closing_brace_hints_min_lines: Some(2), ..DISABLED_CONFIG },
            r#"
fn if_let_wrapper() {
    if let Some(0) = Some(0) {
    }
  //^ // if let Some(0) = Some(0)

    if let Some(0) = None {
    } else {
    }
  //^ // else
  }
//^ // fn if_let_wrapper()
            "#,
        );
    }

    #[test]
    fn hints_closing_brace_truncation() {
        check_with_config(
            InlayHintsConfig {
                closing_brace_hints_min_lines: Some(2),
                max_length: Some(40),
                ..DISABLED_CONFIG
            },
            r#"
// Not truncated
// 40 chars including beginning "// " and params
fn with_40_chars_including_params_() {
  }
//^ // fn with_40_chars_including_params_()

// Truncated after 'o'
// 39 chars before X including "// "
fn with_39_chars_before_X__________oXXXXXX() {
  }
//^ // fn with_39_chars_before_X__________o…

pub(in crate::foo) enum C<'a, const T_A: u32, T_B> {
  }
//^ // pub(in crate::foo) enum C<'a, const …
            "#,
        );
    }
}
