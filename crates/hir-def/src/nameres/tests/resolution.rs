use base_db::SourceDatabase;
use hir_expand::{InFile, files::FilePosition, mod_path::ModPath};
use intern::Interned;
use span::{Edition, SyntaxContext};
use syntax::{AstNode, algo::find_node_at_offset, ast};
use test_fixture::WithFixture;
use test_utils::extract_annotations;

use crate::{
    DefWithBodyId,
    expr_store::{Body, HygieneId, path::Path, scope::ExprScopes},
    hir::{Expr, Pat},
    resolver::{Resolver, TypeNs, ValueNs, resolver_for_scope},
    test_db::TestDB,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Resolution {
    owner: DefWithBodyId,
    types: Option<TypeNs>,
    values: Option<ValueNs>,
}

impl Resolution {
    fn resolve(
        db: &TestDB,
        owner: DefWithBodyId,
        resolver: &Resolver<'_>,
        path: &Path,
        hygiene: HygieneId,
    ) -> Self {
        Self {
            owner,
            types: resolver.resolve_path_in_type_ns_fully(db, path),
            values: resolver.resolve_path_in_value_ns_fully(db, path, hygiene),
        }
    }
}

enum ResolutionExpectation {
    ResolvesTo { path: Path, text: String },
    ResolvesToDefinition(Resolution),
    Unresolved,
}

impl ResolutionExpectation {
    fn parse(db: &TestDB, definitions: &[(String, Resolution)], annotation: &str) -> Self {
        if annotation == "unresolved" {
            return Self::Unresolved;
        }
        let Some(target) = annotation.strip_prefix("resolves-to: ") else {
            panic!("unknown resolution annotation `{annotation}`");
        };
        if !target.contains("::") {
            let Some((_, resolution)) = definitions.iter().find(|(label, _)| label == target)
            else {
                panic!("unknown definition anchor `{target}`");
            };
            return Self::ResolvesToDefinition(*resolution);
        }
        let text = target.to_owned();
        let path = ast::make::path_from_text(target);
        let path = ModPath::from_src(db, path, &mut |_| SyntaxContext::root(Edition::CURRENT))
            .unwrap_or_else(|| panic!("failed to parse resolution path `{annotation}`"));
        Self::ResolvesTo { path: Path::BarePath(Interned::new(path)), text }
    }
}

fn check(#[rust_analyzer::rust_fixture] ra_fixture: &str) {
    let (db, files) = TestDB::with_many_files(ra_fixture);
    let mut annotations = Vec::new();

    for file_id in files {
        let text = db.file_text(file_id.file_id(&db));
        for (range, annotation) in extract_annotations(text.text(&db)) {
            annotations.push((file_id, range, annotation));
        }
    }
    assert!(!annotations.is_empty(), "no resolution annotations found");
    assert!(
        annotations.iter().any(|(_, _, annotation)| !annotation.starts_with("defines: ")),
        "no resolution assertions found"
    );

    let mut definitions = Vec::new();
    for (file_id, range, annotation) in &annotations {
        let Some(label) = annotation.strip_prefix("defines: ") else { continue };
        assert!(
            definitions.iter().all(|(existing, _)| existing != label),
            "duplicate definition anchor `{label}`"
        );
        let position = FilePosition { file_id: *file_id, offset: range.start() };
        let function = DefWithBodyId::from(db.function_at_position(position));
        let (body, source_map) = Body::with_source_map(&db, function);
        let syntax = file_id.parse(&db).syntax_node();
        let name = find_node_at_offset::<ast::Name>(&syntax, range.start())
            .unwrap_or_else(|| panic!("no binding name at {range:?}"));
        assert_eq!(name.syntax().text_range(), *range);
        let pat = name
            .syntax()
            .ancestors()
            .find_map(ast::Pat::cast)
            .unwrap_or_else(|| panic!("no binding pattern at {range:?}"));
        let pat = source_map
            .node_pat(InFile::new((*file_id).into(), &pat))
            .and_then(|pat| pat.as_pat())
            .unwrap_or_else(|| panic!("binding pattern at {range:?} was not lowered"));
        let Pat::Bind { id, subpat: _ } = body[pat] else {
            panic!("pattern at {range:?} is not a binding");
        };
        definitions.push((
            label.to_owned(),
            Resolution { owner: function, types: None, values: Some(ValueNs::LocalBinding(id)) },
        ));
    }

    for (file_id, range, annotation) in annotations {
        if annotation.starts_with("defines: ") {
            continue;
        }
        let expectation = ResolutionExpectation::parse(&db, &definitions, &annotation);
        let position = FilePosition { file_id, offset: range.start() };
        let function = DefWithBodyId::from(db.function_at_position(position));
        let (body, source_map) = Body::with_source_map(&db, function);
        let syntax = file_id.parse(&db).syntax_node();
        let path_expr = find_node_at_offset::<ast::PathExpr>(&syntax, range.start())
            .unwrap_or_else(|| panic!("no path expression at {range:?}"));
        assert_eq!(path_expr.syntax().text_range(), range);
        let expr = ast::Expr::from(path_expr);
        let expr = source_map
            .node_expr(InFile::new(file_id.into(), &expr))
            .and_then(|expr| expr.as_expr())
            .unwrap_or_else(|| panic!("path expression at {range:?} was not lowered"));
        let Expr::Path(path) = &body[expr] else {
            panic!("expression at {range:?} did not lower to a path");
        };
        let scopes = ExprScopes::of(&db, function);
        let resolver = resolver_for_scope(&db, function, scopes.scope_for(expr));
        let actual =
            Resolution::resolve(&db, function, &resolver, path, body.expr_path_hygiene(expr));
        let expected = match expectation {
            ResolutionExpectation::ResolvesTo { path, text } => {
                let resolution =
                    Resolution::resolve(&db, function, &resolver, &path, HygieneId::ROOT);
                let Resolution { owner: _, types, values } = &resolution;
                assert!(
                    types.is_some() || values.is_some(),
                    "expected path `{text}` does not resolve"
                );
                resolution
            }
            ResolutionExpectation::ResolvesToDefinition(resolution) => resolution,
            ResolutionExpectation::Unresolved => {
                Resolution { owner: function, types: None, values: None }
            }
        };
        assert_eq!(actual, expected, "resolution differs at {range:?}");
    }
}

#[test]
fn resolves_path_expression_to_item() {
    check(
        r#"
fn target() {}

fn main() {
    target();
  //^^^^^^ resolves-to: crate::target
}
"#,
    );
}

#[test]
fn path_expression_is_unresolved() {
    check(
        r#"
fn main() {
    missing();
  //^^^^^^^ unresolved
}
"#,
    );
}

#[test]
fn resolves_import_alias_across_files() {
    check(
        r#"
//- /main.rs
mod defs;
use defs::target as alias;

fn main() {
    alias();
  //^^^^^ resolves-to: crate::defs::target
}

//- /defs.rs
pub fn target() {}
"#,
    );
}

#[test]
fn absolute_path_ignores_local_module() {
    check(
        r#"
//- /main.rs crate:main deps:dependency
mod dependency {}
use ::dependency::target as imported;

fn main() {
    imported();
  //^^^^^^^^ resolves-to: ::dependency::target
}

//- /lib.rs crate:dependency
pub fn target() {}
"#,
    );
}

#[test]
fn resolves_raw_identifier_alias() {
    check(
        r#"
fn r#type() {}
use crate::r#type as imported;

fn main() {
    imported();
  //^^^^^^^^ resolves-to: crate::r#type
}
"#,
    );
}

#[test]
fn resolves_item_in_type_and_value_namespaces() {
    check(
        r#"
struct Target;

fn main() {
    Target;
  //^^^^^^ resolves-to: crate::Target
}
"#,
    );
}

#[test]
fn resolves_parameter_to_definition_anchor() {
    check(
        r#"
fn main(parameter: i32) {
      //^^^^^^^^^ defines: parameter
    {
        let ignored = parameter;
    }
    {
        parameter;
      //^^^^^^^^^ resolves-to: parameter
    }
}
"#,
    );
}

#[test]
fn resolves_shadowing_binding_to_nearest_definition() {
    check(
        r#"
fn main(binding: String) {
      //^^^^^^^ defines: parameter
    let binding = &binding;
      //^^^^^^^ defines: local
    binding;
  //^^^^^^^ resolves-to: local
}
"#,
    );
}

#[test]
fn binding_initializer_resolves_before_shadowing() {
    check(
        r#"
fn main(binding: String) {
      //^^^^^^^ defines: parameter
    let binding: &str = &binding;
                       //^^^^^^^ resolves-to: parameter
}
"#,
    );
}

#[test]
fn reference_pattern_contributes_binding() {
    check(
        r#"
fn main() {
    if let Some(&binding) = value() {
               //^^^^^^^ defines: binding
        binding;
      //^^^^^^^ resolves-to: binding
    }
}
"#,
    );
}

#[test]
fn while_let_contributes_binding() {
    check(
        r#"
fn main(value: Option<f32>) {
    while let Some(binding) = value {
                 //^^^^^^^ defines: binding
        binding;
      //^^^^^^^ resolves-to: binding
    }
}
"#,
    );
}

#[test]
fn chained_while_let_contributes_binding() {
    check(
        r#"
fn main(value: Option<f32>) {
    while (((let Some(_) = value)))
        && let Some(binding) = value
                  //^^^^^^^ defines: binding
    {
        binding;
      //^^^^^^^ resolves-to: binding
    }
}
"#,
    );
}

#[test]
fn match_guard_let_contributes_binding() {
    check(
        r#"
fn main(value: Option<f32>) {
    match value {
        _ if let Some(binding) = value => {
                    //^^^^^^^ defines: binding
            binding
          //^^^^^^^ resolves-to: binding
        }
        _ => {}
    }
}
"#,
    );
}

#[test]
fn let_chain_resolves_each_binding_in_its_scope() {
    check(
        r#"
fn main(value: Option<i32>) {
    if let Some(binding) = value
              //^^^^^^^ defines: first
        && binding > 1
         //^^^^^^^ resolves-to: first
        && let Some(binding) = value
                  //^^^^^^^ defines: second
        && binding > 1
         //^^^^^^^ resolves-to: second
    {}
}
"#,
    );
}

#[test]
#[should_panic(expected = "resolution differs")]
fn definition_anchor_identity_includes_owning_body() {
    check(
        r#"
fn first(binding: i32) {
       //^^^^^^^ defines: first_binding
}

fn second(binding: i32) {
    binding;
  //^^^^^^^ resolves-to: first_binding
}
"#,
    );
}

#[test]
#[should_panic(expected = "expected path `crate::missing` does not resolve")]
fn expected_resolution_path_must_resolve() {
    check(
        r#"
fn main() {
    missing();
  //^^^^^^^ resolves-to: crate::missing
}
"#,
    );
}

#[test]
#[should_panic(expected = "no resolution annotations found")]
fn fixture_must_contain_resolution_annotations() {
    check("fn main() {}");
}

#[test]
#[should_panic(expected = "no resolution assertions found")]
fn fixture_must_contain_resolution_assertions() {
    check(
        r#"
fn main(binding: i32) {}
      //^^^^^^^ defines: binding
"#,
    );
}
