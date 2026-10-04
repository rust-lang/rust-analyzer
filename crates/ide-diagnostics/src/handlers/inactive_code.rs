use cfg::DnfExpr;
use hir::InFile;
use ide_db::FileRange;
use stdx::format_to;
use syntax::{AstNode, SyntaxNode, SyntaxNodePtr, TextRange};

use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext, Severity};

// Diagnostic: inactive-code
//
// This diagnostic is shown for code with inactive `#[cfg]` attributes.
//
// It can be disabled selectively with `#[allow(rust_analyzer::inactive_code)]`.
pub(crate) fn inactive_code(
    ctx: &DiagnosticsContext<'_, '_>,
    d: &hir::InactiveCode,
) -> Option<Diagnostic> {
    // If there's inactive code somewhere in a macro that doesn't map to something in the call, don't propagate to the call-site.
    d.node.map(|it| it.text_range()).original_node_file_range_rooted_opt(ctx.db())?;

    let inactive = DnfExpr::new(&d.cfg).why_inactive(&d.opts);
    let mut message = "code is inactive due to #[cfg] directives".to_owned();

    if let Some(inactive) = inactive {
        let inactive_reasons = inactive.to_string();

        if inactive_reasons.is_empty() {
            format_to!(message);
        } else {
            format_to!(message, ": {}", inactive);
        }
    }
    let range = match d.node.file_id.is_macro() {
        true => range_in_macro_call(ctx, d.node)?,
        false => ctx.sema.diagnostics_display_range(d.node),
    };
    // FIXME: This shouldn't be a diagnostic
    let res = Diagnostic::new(
        DiagnosticCode::RaLint("inactive_code", Severity::WeakWarning),
        message,
        range,
    )
    .with_main_node(d.node)
    .stable()
    .with_unused(true);
    Some(res)
}

/// Finds the range in the call-site source to highlight for inactive code in a macro expansion.
///
/// A macro can use the same input tokens in several places of its output, and combine tokens
/// from several places of its input into one item (e.g. `#[wasm_bindgen]` copies the `impl`'s
/// self type onto each method). Mapping the whole inactive node up at once would then cover
/// active code too. So:
/// - only input tokens that the expansion uses *nowhere but* in the inactive node count, and
/// - of those, only the largest run that is contiguous in the source is highlighted.
fn range_in_macro_call(
    ctx: &DiagnosticsContext<'_, '_>,
    node: InFile<SyntaxNodePtr>,
) -> Option<FileRange> {
    let db = ctx.db();
    let root = ctx.sema.parse_or_expand(node.file_id);
    let inactive = node.value.to_node(&root);
    let map_up = |token: syntax::SyntaxToken| {
        if token.kind().is_trivia() {
            return None;
        }
        InFile::new(node.file_id, token.text_range()).original_node_file_range_rooted_opt(db)
    };
    let used_by_active_code = root
        .descendants_with_tokens()
        .filter_map(|it| it.into_token())
        .filter(|token| !inactive.text_range().contains_range(token.text_range()))
        .filter_map(map_up)
        .map(|it| (it.file_id, it.range))
        .collect::<ide_db::FxHashSet<_>>();
    let mut ranges = inactive
        .descendants_with_tokens()
        .filter_map(|it| it.into_token())
        .filter_map(map_up)
        .filter(|it| !used_by_active_code.contains(&(it.file_id, it.range)))
        .collect::<Vec<_>>();
    let file_id = ranges.first()?.file_id;
    ranges.retain(|it| it.file_id == file_id);
    ranges.sort_by_key(|it| it.range.start());

    let source = ctx.sema.parse(file_id);
    let mut groups: Vec<TextRange> = Vec::new();
    for range in ranges.into_iter().map(|it| it.range) {
        match groups.last_mut() {
            Some(last) if only_trivia_between(source.syntax(), last.end(), range.start()) => {
                *last = last.cover(range);
            }
            _ => groups.push(range),
        }
    }
    let range = groups.into_iter().max_by_key(|it| it.len())?;
    Some(FileRange { file_id: file_id.file_id(db), range })
}

/// Whether the source between `start` and `end` contains nothing but whitespace and comments.
fn only_trivia_between(root: &SyntaxNode, start: syntax::TextSize, end: syntax::TextSize) -> bool {
    if end <= start {
        return true;
    }
    let mut token = root.token_at_offset(start).right_biased();
    while let Some(it) = token {
        if it.text_range().start() >= end {
            break;
        }
        if !it.kind().is_trivia() && it.text_range().end() > start {
            return false;
        }
        token = it.next_token();
    }
    true
}

#[cfg(test)]
mod tests {
    use ide_db::RootDatabase;
    use test_fixture::WithFixture;

    use crate::{DiagnosticCode, DiagnosticsConfig, tests::check_diagnostics_with_config};

    #[track_caller]
    pub(crate) fn check(#[rust_analyzer::rust_fixture] ra_fixture: &str) {
        let config = DiagnosticsConfig {
            disabled: std::iter::once("unlinked-file".to_owned()).collect(),
            ..DiagnosticsConfig::test_sample()
        };
        check_diagnostics_with_config(config, ra_fixture)
    }

    #[test]
    fn cfg_diagnostics() {
        check(
            r#"
fn f() {
    // The three g̶e̶n̶d̶e̶r̶s̶ statements:

    #[cfg(a)] fn f() {}  // Item statement
  //^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: a is disabled
    #[cfg(a)] {}         // Expression statement
  //^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: a is disabled
    #[cfg(a)] let x = 0; // let statement
  //^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: a is disabled

    fn abc() {}
    abc(#[cfg(a)] 0);
      //^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: a is disabled
    let x = Struct {
        #[cfg(a)] f: 0,
      //^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: a is disabled
    };
    match () {
        () => (),
        #[cfg(a)] () => (),
      //^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: a is disabled
    }

    #[cfg(a)] 0          // Trailing expression of block
  //^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: a is disabled
}
        "#,
        );
    }

    #[test]
    fn inactive_item() {
        // Additional tests in `cfg` crate. This only tests disabled cfgs.

        check(
            r#"
    #[cfg(no)] pub fn f() {}
  //^^^^^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: no is disabled

    #[cfg(no)] #[cfg(no2)] mod m;
  //^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: no is disabled

    #[cfg(all(not(a), b))] enum E {}
  //^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: b is disabled

    #[cfg(feature = "std")] use std;
  //^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: feature = "std" is disabled

    #[cfg(any())] pub fn f() {}
  //^^^^^^^^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives
"#,
        );
    }

    #[test]
    fn inactive_assoc_item() {
        check(
            r#"
struct Foo;
impl Foo {
    #[cfg(any())] pub fn f() {}
  //^^^^^^^^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives
}

trait Bar {
    #[cfg(any())] pub fn f() {}
  //^^^^^^^^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives
}
"#,
        );
    }

    /// Tests that `cfg` attributes behind `cfg_attr` is handled properly.
    #[test]
    fn inactive_via_cfg_attr() {
        check(
            r#"
    #[cfg_attr(not(never), cfg(no))] fn f() {}
  //^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: no is disabled

    #[cfg_attr(not(never), cfg(not(no)))] fn f() {}

    #[cfg_attr(never, cfg(no))] fn g() {}

    #[cfg_attr(not(never), inline, cfg(no))] fn h() {}
  //^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: no is disabled
"#,
        );
    }

    #[test]
    fn inactive_fields_and_variants() {
        check(
            r#"
enum Foo {
  #[cfg(a)] Bar,
//^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: a is disabled
  Baz {
    #[cfg(a)] baz: String,
  //^^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: a is disabled
  },
  Qux(#[cfg(a)] String),
    //^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: a is disabled
}

struct Baz {
  #[cfg(a)] baz: String,
//^^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: a is disabled
}

struct Qux(#[cfg(a)] String);
         //^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: a is disabled

union FooBar {
  #[cfg(a)] baz: u32,
//^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: a is disabled
}
"#,
        );
    }

    #[test]
    fn modules() {
        check(
            r#"
//- /main.rs
  #[cfg(outline)] mod outline;
//^^^^^^^^^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: outline is disabled

  mod outline_inner;
//^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: outline_inner is disabled

  #[cfg(inline)] mod inline {}
//^^^^^^^^^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: inline is disabled

//- /outline_inner.rs
#![cfg(outline_inner)]
//- /outline.rs
"#,
        );
    }

    #[test]
    fn cfg_true_false() {
        check(
            r#"
  #[cfg(false)] fn inactive() {}
//^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: false is disabled

  #[cfg(true)] fn active() {}

  #[cfg(any(not(true), false))] fn inactive2() {}
//^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: true is enabled and false is disabled

"#,
        );
    }

    #[test]
    fn inactive_crate() {
        let db = RootDatabase::with_files(
            r#"
#![cfg(false)]

fn foo() {}
        "#,
        );
        let file_id = db.test_crate().root_file_id(&db);
        let diagnostics = hir::attach_db(&db, || {
            crate::full_diagnostics(
                &db,
                &DiagnosticsConfig::test_sample(),
                &ide_db::assists::AssistResolveStrategy::All,
                file_id.file_id(&db),
            )
        });
        let [inactive_code] = &*diagnostics else {
            panic!("expected one inactive_code diagnostic, found {diagnostics:#?}");
        };
        assert_eq!(
            inactive_code.code,
            DiagnosticCode::RaLint("inactive_code", ide_db::Severity::WeakWarning)
        );
        assert_eq!(
            inactive_code.message,
            "code is inactive due to #[cfg] directives: false is disabled",
        );
        assert!(inactive_code.fixes.is_none());
        let full_file_range = file_id.parse(&db).syntax_node().text_range();
        assert_eq!(
            inactive_code.range,
            ide_db::FileRange { file_id: file_id.file_id(&db), range: full_file_range },
        );
    }

    #[test]
    fn cfg_in_macro_does_not_diagnose_the_whole_call() {
        check(
            r#"
macro_rules! m {
    ($e:item) => {
        #[cfg(false)]
        const _: () = ();

        $e
    };
}

m! {
    fn foo() {}
}
        "#,
        );
    }

    #[test]
    fn in_macro() {
        check(
            r#"
macro_rules! m {
    ($e:item) => {
        $e
    };
}

m! {
    #[cfg(false)] fn foo() {}
 // ^^^^^^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: false is disabled
}
        "#,
        );
        check(
            r#"
macro_rules! m {
    ($e:item) => {
        #[cfg(false)]
        $e
    };
}

m! {
    fn foo() {}
 // ^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: false is disabled
}
        "#,
        );
    }

    #[test]
    fn in_macro_with_tokens_from_elsewhere_in_the_input() {
        // `Player` ends up in the inactive item, but must not widen the
        // highlighted range over the active `kept` in between.
        check(
            r#"
macro_rules! label_last_item {
    ($label:ident $first:item $last:item) => {
        $first
        #[doc = stringify!($label)]
        $last
    };
}

label_last_item! {
    Player
    fn kept() {}
    #[cfg(feature = "off")] fn gated() {}
 // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: feature = "off" is disabled
}
        "#,
        );
    }

    #[test]
    fn in_macro_with_active_item_copied_into_inactive_one() {
        // `kept` is used by active code too, so it must not be highlighted.
        check(
            r#"
macro_rules! label_last_item {
    ($first:item $last:item) => {
        $first
        #[doc = stringify!($first)]
        $last
    };
}

label_last_item! {
    fn kept() {}
    #[cfg(feature = "off")] fn gated() {}
 // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ weak: code is inactive due to #[cfg] directives: feature = "off" is disabled
}
        "#,
        );
    }

    #[test]
    fn in_macro_inactive_glue_referencing_active_item() {
        // Like `#[derive(Facet)]` (#22984): the inactive glue mentions `Foo`,
        // but so does the active struct, so `Foo` must not be highlighted.
        check(
            r#"
macro_rules! with_glue {
    (pub struct $name:ident;) => {
        pub struct $name;
        #[cfg(false)]
        const _: Option<$name> = None;
    };
}

with_glue! {
    pub struct Foo;
}
        "#,
        );
    }

    #[test]
    fn allow() {
        check(
            r#"
macro_rules! m {
    ($e:item) => {
        #[cfg(false)]
        #[allow(rust_analyzer::inactive_code)]
        $e
    };
}

m! {
    fn foo() {}
}
        "#,
        );
    }
}
