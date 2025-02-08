//! This module implements a reference search.
//! First, the element at the cursor position must be either an `ast::Name`
//! or `ast::NameRef`. If it's an `ast::NameRef`, at the classification step we
//! try to resolve the direct tree parent of this element, otherwise we
//! already have a definition and just need to get its HIR together with
//! some information that is needed for further steps of searching.
//! After that, we collect files that might contain references and look
//! for text occurrences of the identifier. If there's an `ast::NameRef`
//! at the index that the match starts at and its tree parent is
//! resolved to the search element definition, we get a reference.
//!
//! Special handling for constructors/initializations:
//! When searching for references to a struct/enum/variant, if the cursor is positioned on:
//! - `{` after a struct/enum/variant definition
//! - `(` for tuple structs/variants
//! - `;` for unit structs
//! - The type name in a struct/enum/variant definition
//!   Then only constructor/initialization usages will be shown, filtering out other references.

use hir::{HirFileId, HirFilePosition, PathResolution, Semantics};
use ide_db::{
    FxHashMap, RootDatabase,
    defs::{Definition, NameClass, NameRefClass},
    helpers::pick_best_token,
    search::{ReferenceCategory, SearchScope, UsageSearchResult},
};
use itertools::Itertools;
use syntax::{
    AstNode,
    SyntaxKind::*,
    SyntaxNode, T, TextRange, TextSize,
    ast::{self, HasName},
    match_ast,
};

use crate::{
    HighlightedRange, TryToNav, highlight_related, navigation_target::HirNavigationTarget,
};

/// Result of a reference search operation.
#[derive(Debug, Clone)]
pub struct ReferenceSearchResult {
    /// Information about the declaration site of the searched item.
    /// For ADTs (structs/enums), this points to the type definition.
    /// May be None for primitives or items without clear declaration sites.
    pub declaration: Option<Declaration>,
    /// All references found, grouped by file.
    /// For ADTs when searching from a constructor position (e.g. on '{', '(', ';'),
    /// this only includes constructor/initialization usages.
    /// The map key is the file ID, and the value is a vector of (range, category) pairs.
    /// - range: The text range of the reference in the file
    /// - category: Metadata about how the reference is used (read/write/etc)
    pub references: FxHashMap<HirFileId, Vec<(TextRange, ReferenceCategory)>>,
}

/// Information about the declaration site of a searched item.
#[derive(Debug, Clone)]
pub struct Declaration {
    /// Navigation information to jump to the declaration
    pub nav: HirNavigationTarget,
    /// Whether the declared item is mutable (relevant for variables)
    pub is_mut: bool,
}

// Feature: Find All References
//
// Shows all references of the item at the cursor location. This includes:
// - Direct references to variables, functions, types, etc.
// - Constructor/initialization references when cursor is on struct/enum definition tokens
// - References in patterns and type contexts
// - References through dereferencing and borrowing
// - References in macro expansions
//
// Special handling for constructors:
// - When the cursor is on `{`, `(`, or `;` in a struct/enum definition
// - When the cursor is on the type name in a struct/enum definition
// These cases will show only constructor/initialization usages of the type
//
// | Editor  | Shortcut |
// |---------|----------|
// | VS Code | <kbd>Shift+Alt+F12</kbd> |
//
// ![Find All References](https://user-images.githubusercontent.com/48062697/113020670-b7c34f00-917a-11eb-8003-370ac5f2b3cb.gif)

/// Find all references to the item at the given position.
///
/// # Arguments
/// * `sema` - Semantic analysis context
/// * `position` - Position in the file where to look for the item
/// * `search_scope` - Optional scope to limit the search (e.g. current crate only)
///
/// # Returns
/// Returns `None` if no valid item is found at the position.
/// Otherwise returns a vector of `ReferenceSearchResult`, usually with one element.
/// Multiple results can occur in case of ambiguity or when searching for trait items.
///
/// # Special cases
/// - Control flow keywords (break, continue, etc): Shows all related jump points
/// - Constructor search: When on struct/enum definition tokens (`{`, `(`, `;`), shows only initialization sites
/// - Format string arguments: Shows template parameter usages
/// - Lifetime parameters: Shows lifetime constraint usages
///
/// # Constructor search
/// When the cursor is on specific tokens in a struct/enum definition:
/// - `{` after struct/enum/variant: Shows record literal initializations
/// - `(` after tuple struct/variant: Shows tuple literal initializations
/// - `;` after unit struct: Shows unit literal initializations
/// - Type name in definition: Shows all initialization usages
///   In these cases, other kinds of references (like type references) are filtered out.
pub(crate) fn find_all_refs(
    sema: &Semantics<'_, RootDatabase>,
    mut position: HirFilePosition,
    search_scope: Option<SearchScope>,
) -> Option<Vec<ReferenceSearchResult>> {
    let _p = tracing::info_span!("find_all_refs").entered();
    position.file_id = sema.adjust_edition(position.file_id);
    let syntax = sema.parse_or_expand(position.file_id);
    let make_searcher = |literal_search: bool| {
        move |def: Definition| {
            let mut usages =
                def.usages(sema).set_scope(search_scope.as_ref()).include_self_refs().all();
            if literal_search {
                retain_adt_literal_usages(&mut usages, def, sema);
            }

            let references: FxHashMap<HirFileId, Vec<(TextRange, ReferenceCategory)>> = usages
                .into_iter()
                .map(|(file_id, refs)| {
                    (
                        file_id,
                        refs.into_iter()
                            .map(|file_ref| (file_ref.range, file_ref.category))
                            .unique()
                            .collect(),
                    )
                })
                .collect();
            let declaration = match def {
                Definition::Module(module) => {
                    Some(HirNavigationTarget::from_module_to_decl(sema.db, module))
                }
                def => def.try_to_nav_hir(sema.db),
            }
            .map(|nav| Declaration {
                is_mut: matches!(def, Definition::Local(l) if l.is_mut(sema.db)),
                nav,
            });
            ReferenceSearchResult { declaration, references }
        }
    };

    // Find references for control-flow keywords.
    if let Some(res) = handle_control_flow_keywords(sema, &syntax, position) {
        return Some(vec![res]);
    }

    match name_for_constructor_search(&syntax, position) {
        Some(name) => {
            let def = match NameClass::classify(sema, &name)? {
                NameClass::Definition(it) | NameClass::ConstReference(it) => it,
                NameClass::PatFieldShorthand { local_def: _, field_ref, adt_subst: _ } => {
                    Definition::Field(field_ref)
                }
            };
            Some(vec![make_searcher(true)(def)])
        }
        None => {
            let search = make_searcher(false);
            Some(find_defs(sema, &syntax, position.offset)?.into_iter().map(search).collect())
        }
    }
}

pub(crate) fn find_defs(
    sema: &Semantics<'_, RootDatabase>,
    syntax: &SyntaxNode,
    offset: TextSize,
) -> Option<Vec<Definition>> {
    let token = syntax.token_at_offset(offset).find(|t| {
        matches!(
            t.kind(),
            IDENT
                | INT_NUMBER
                | LIFETIME_IDENT
                | STRING
                | T![self]
                | T![super]
                | T![crate]
                | T![Self]
        )
    })?;

    if let Some((.., resolution)) = sema.check_for_format_args_template(token.clone(), offset) {
        return resolution.map(Definition::from).map(|it| vec![it]);
    }

    Some(
        sema.descend_into_macros_exact(token)
            .into_iter()
            .filter_map(|it| ast::NameLike::cast(it.parent()?))
            .filter_map(move |name_like| {
                let def = match name_like {
                    ast::NameLike::NameRef(name_ref) => {
                        match NameRefClass::classify(sema, &name_ref)? {
                            NameRefClass::Definition(def, _) => def,
                            NameRefClass::FieldShorthand {
                                local_ref,
                                field_ref: _,
                                adt_subst: _,
                            } => Definition::Local(local_ref),
                            NameRefClass::ExternCrateShorthand { decl, .. } => {
                                Definition::ExternCrateDecl(decl)
                            }
                        }
                    }
                    ast::NameLike::Name(name) => match NameClass::classify(sema, &name)? {
                        NameClass::Definition(it) | NameClass::ConstReference(it) => it,
                        NameClass::PatFieldShorthand { local_def, field_ref: _, adt_subst: _ } => {
                            Definition::Local(local_def)
                        }
                    },
                    ast::NameLike::Lifetime(lifetime) => {
                        NameRefClass::classify_lifetime(sema, &lifetime)
                            .and_then(|class| match class {
                                NameRefClass::Definition(it, _) => Some(it),
                                _ => None,
                            })
                            .or_else(|| {
                                NameClass::classify_lifetime(sema, &lifetime)
                                    .and_then(NameClass::defined)
                            })?
                    }
                };
                Some(def)
            })
            .collect(),
    )
}

/// Filter out all non-literal usages for adt-defs
fn retain_adt_literal_usages(
    usages: &mut UsageSearchResult,
    def: Definition,
    sema: &Semantics<'_, RootDatabase>,
) {
    let refs = usages.references.values_mut();
    match def {
        Definition::Adt(hir::Adt::Enum(enum_)) => {
            refs.for_each(|it| {
                it.retain(|reference| {
                    reference
                        .name
                        .as_name_ref()
                        .is_some_and(|name_ref| is_enum_lit_name_ref(sema, enum_, name_ref))
                })
            });
            usages.references.retain(|_, it| !it.is_empty());
        }
        Definition::Adt(_) | Definition::Variant(_) => {
            refs.for_each(|it| {
                it.retain(|reference| reference.name.as_name_ref().is_some_and(is_lit_name_ref))
            });
            usages.references.retain(|_, it| !it.is_empty());
        }
        _ => {}
    }
}

/// Returns `Some` if the cursor is at a position where we should search for constructor/initialization usages.
/// This is used to implement the special constructor search behavior when the cursor is on specific tokens
/// in a struct/enum/variant definition.
///
/// # Returns
/// - `Some(name)` if the cursor is on:
///   - `{` after a struct/enum/variant definition
///   - `(` for tuple structs/variants
///   - `;` for unit structs
///   - The type name in a struct/enum/variant definition
/// - `None` otherwise
///
/// The returned name is the name of the type whose constructor usages should be searched for.
fn name_for_constructor_search(
    syntax: &SyntaxNode,
    position: HirFilePosition,
) -> Option<ast::Name> {
    let token = syntax.token_at_offset(position.offset).right_biased()?;
    let token_parent = token.parent()?;
    let kind = token.kind();
    if kind == T![;] {
        ast::Struct::cast(token_parent)
            .filter(|struct_| struct_.field_list().is_none())
            .and_then(|struct_| struct_.name())
    } else if kind == T!['{'] {
        match_ast! {
            match token_parent {
                ast::RecordFieldList(rfl) => match_ast! {
                    match (rfl.syntax().parent()?) {
                        ast::Variant(it) => it.name(),
                        ast::Struct(it) => it.name(),
                        ast::Union(it) => it.name(),
                        _ => None,
                    }
                },
                ast::VariantList(vl) => ast::Enum::cast(vl.syntax().parent()?)?.name(),
                _ => None,
            }
        }
    } else if kind == T!['('] {
        let tfl = ast::TupleFieldList::cast(token_parent)?;
        match_ast! {
            match (tfl.syntax().parent()?) {
                ast::Variant(it) => it.name(),
                ast::Struct(it) => it.name(),
                _ => None,
            }
        }
    } else {
        None
    }
}

/// Checks if a name reference is part of an enum variant literal expression.
/// Used to filter references when searching for enum variant constructors.
///
/// # Arguments
/// * `sema` - Semantic analysis context
/// * `enum_` - The enum type to check against
/// * `name_ref` - The name reference to check
///
/// # Returns
/// `true` if the name reference is used as part of constructing a variant of the given enum.
fn is_enum_lit_name_ref(
    sema: &Semantics<'_, RootDatabase>,
    enum_: hir::Enum,
    name_ref: &ast::NameRef,
) -> bool {
    let path_is_variant_of_enum = |path: ast::Path| {
        matches!(
            sema.resolve_path(&path),
            Some(PathResolution::Def(hir::ModuleDef::Variant(variant)))
                if variant.parent_enum(sema.db) == enum_
        )
    };
    name_ref
        .syntax()
        .ancestors()
        .find_map(|ancestor| {
            match_ast! {
                match ancestor {
                    ast::PathExpr(path_expr) => path_expr.path().map(path_is_variant_of_enum),
                    ast::RecordExpr(record_expr) => record_expr.path().map(path_is_variant_of_enum),
                    _ => None,
                }
            }
        })
        .unwrap_or(false)
}

/// Checks if a path ends with the given name reference.
/// Helper function for checking constructor usage patterns.
fn path_ends_with(path: Option<ast::Path>, name_ref: &ast::NameRef) -> bool {
    path.and_then(|path| path.segment())
        .and_then(|segment| segment.name_ref())
        .map_or(false, |segment| segment == *name_ref)
}

/// Checks if a name reference is used in a literal (constructor) context.
/// Used to filter references when searching for struct/variant constructors.
///
/// # Returns
/// `true` if the name reference is used as part of a struct/variant literal expression.
fn is_lit_name_ref(name_ref: &ast::NameRef) -> bool {
    name_ref.syntax().ancestors().find_map(|ancestor| {
        match_ast! {
            match ancestor {
                ast::PathExpr(path_expr) => Some(path_ends_with(path_expr.path(), name_ref)),
                ast::RecordExpr(record_expr) => Some(path_ends_with(record_expr.path(), name_ref)),
                _ => None,
            }
        }
    }).unwrap_or(false)
}

fn handle_control_flow_keywords(
    sema: &Semantics<'_, RootDatabase>,
    syntax: &SyntaxNode,
    HirFilePosition { file_id, offset }: HirFilePosition,
) -> Option<ReferenceSearchResult> {
    let edition = file_id.edition(sema.db);
    let token = pick_best_token(syntax.token_at_offset(offset), |kind| match kind {
        _ if kind.is_keyword(edition) => 4,
        T![=>] => 3,
        _ => 1,
    })?;

    let references = match token.kind() {
        T![fn] | T![return] | T![try] => highlight_related::highlight_exit_points(sema, token),
        T![async] => highlight_related::highlight_yield_points(sema, token),
        T![loop] | T![while] | T![break] | T![continue] => {
            highlight_related::highlight_break_points(sema, token)
        }
        T![for] if token.parent().and_then(ast::ForExpr::cast).is_some() => {
            highlight_related::highlight_break_points(sema, token)
        }
        T![if] | T![=>] | T![match] => highlight_related::highlight_branch_exit_points(sema, token),
        _ => return None,
    }
    .into_iter()
    .map(|(file_id, ranges)| {
        let ranges = ranges
            .into_iter()
            .map(|HighlightedRange { range, category }| (range, category))
            .collect();
        (file_id, ranges)
    })
    .collect();

    Some(ReferenceSearchResult { declaration: None, references })
}

#[cfg(test)]
mod tests {
    use expect_test::{Expect, expect};
    use hir::EditionedFileId;
    use ide_db::{FileId, RootDatabase};
    use stdx::format_to;

    use crate::{SearchScope, fixture};

    #[test]
    fn exclude_tests() {
        check(
            r#"
fn test_func() {}

fn func() {
    test_func$0();
}

#[test]
fn test() {
    test_func();
}
"#,
            expect![[r#"
                test_func Function FileId(EditionedFileId(Id(2000))) 0..17 3..12

                FileId(EditionedFileId(Id(2000))) 35..44
                FileId(EditionedFileId(Id(2000))) 75..84 test
            "#]],
        );

        check(
            r#"
fn test_func() {}

fn func() {
    test_func$0();
}

#[::core::prelude::v1::test]
fn test() {
    test_func();
}
"#,
            expect![[r#"
                test_func Function FileId(EditionedFileId(Id(2000))) 0..17 3..12

                FileId(EditionedFileId(Id(2000))) 35..44
                FileId(EditionedFileId(Id(2000))) 96..105 test
            "#]],
        );
    }

    #[test]
    fn test_access() {
        check(
            r#"
struct S { f$0: u32 }

#[test]
fn test() {
    let mut x = S { f: 92 };
    x.f = 92;
}
"#,
            expect![[r#"
                f Field FileId(EditionedFileId(Id(2000))) 11..17 11..12

                FileId(EditionedFileId(Id(2000))) 61..62 read test
                FileId(EditionedFileId(Id(2000))) 76..77 write test
            "#]],
        );
    }

    #[test]
    fn test_struct_literal_after_space() {
        check(
            r#"
struct Foo $0{
    a: i32,
}
impl Foo {
    fn f() -> i32 { 42 }
}
fn main() {
    let f: Foo;
    f = Foo {a: Foo::f()};
}
"#,
            expect![[r#"
                Foo Struct FileId(EditionedFileId(Id(2000))) 0..26 7..10

                FileId(EditionedFileId(Id(2000))) 101..104
            "#]],
        );
    }

    #[test]
    fn test_struct_literal_before_space() {
        check(
            r#"
struct Foo$0 {}
    fn main() {
    let f: Foo;
    f = Foo {};
}
"#,
            expect![[r#"
                Foo Struct FileId(EditionedFileId(Id(2000))) 0..13 7..10

                FileId(EditionedFileId(Id(2000))) 41..44
                FileId(EditionedFileId(Id(2000))) 54..57
            "#]],
        );
    }

    #[test]
    fn test_struct_literal_with_generic_type() {
        check(
            r#"
struct Foo<T> $0{}
    fn main() {
    let f: Foo::<i32>;
    f = Foo {};
}
"#,
            expect![[r#"
                Foo Struct FileId(EditionedFileId(Id(2000))) 0..16 7..10

                FileId(EditionedFileId(Id(2000))) 64..67
            "#]],
        );
    }

    #[test]
    fn test_struct_literal_for_tuple() {
        check(
            r#"
struct Foo$0(i32);

fn main() {
    let f: Foo;
    f = Foo(1);
}
"#,
            expect![[r#"
                Foo Struct FileId(EditionedFileId(Id(2000))) 0..16 7..10

                FileId(EditionedFileId(Id(2000))) 54..57
            "#]],
        );
    }

    #[test]
    fn test_struct_literal_for_union() {
        check(
            r#"
union Foo $0{
    x: u32
}

fn main() {
    let f: Foo;
    f = Foo { x: 1 };
}
"#,
            expect![[r#"
                Foo Union FileId(EditionedFileId(Id(2000))) 0..24 6..9

                FileId(EditionedFileId(Id(2000))) 62..65
            "#]],
        );
    }

    #[test]
    fn test_enum_after_space() {
        check(
            r#"
enum Foo $0{
    A,
    B(),
    C{},
}
fn main() {
    let f: Foo;
    f = Foo::A;
    f = Foo::B();
    f = Foo::C{};
}
"#,
            expect![[r#"
                Foo Enum FileId(EditionedFileId(Id(2000))) 0..37 5..8

                FileId(EditionedFileId(Id(2000))) 74..77
                FileId(EditionedFileId(Id(2000))) 90..93
                FileId(EditionedFileId(Id(2000))) 108..111
            "#]],
        );
    }

    #[test]
    fn test_variant_record_after_space() {
        check(
            r#"
enum Foo {
    A $0{ n: i32 },
    B,
}
fn main() {
    let f: Foo;
    f = Foo::B;
    f = Foo::A { n: 92 };
}
"#,
            expect![[r#"
                A Variant FileId(EditionedFileId(Id(2000))) 15..27 15..16

                FileId(EditionedFileId(Id(2000))) 95..96
            "#]],
        );
    }

    #[test]
    fn test_variant_tuple_before_paren() {
        check(
            r#"
enum Foo {
    A$0(i32),
    B,
}
fn main() {
    let f: Foo;
    f = Foo::B;
    f = Foo::A(92);
}
"#,
            expect![[r#"
                A Variant FileId(EditionedFileId(Id(2000))) 15..21 15..16

                FileId(EditionedFileId(Id(2000))) 89..90
            "#]],
        );
    }

    #[test]
    fn test_enum_before_space() {
        check(
            r#"
enum Foo$0 {
    A,
    B,
}
fn main() {
    let f: Foo;
    f = Foo::A;
}
"#,
            expect![[r#"
                Foo Enum FileId(EditionedFileId(Id(2000))) 0..26 5..8

                FileId(EditionedFileId(Id(2000))) 50..53
                FileId(EditionedFileId(Id(2000))) 63..66
            "#]],
        );
    }

    #[test]
    fn test_enum_with_generic_type() {
        check(
            r#"
enum Foo<T> $0{
    A(T),
    B,
}
fn main() {
    let f: Foo<i8>;
    f = Foo::A(1);
}
"#,
            expect![[r#"
                Foo Enum FileId(EditionedFileId(Id(2000))) 0..32 5..8

                FileId(EditionedFileId(Id(2000))) 73..76
            "#]],
        );
    }

    #[test]
    fn test_enum_for_tuple() {
        check(
            r#"
enum Foo$0{
    A(i8),
    B(i8),
}
fn main() {
    let f: Foo;
    f = Foo::A(1);
}
"#,
            expect![[r#"
                Foo Enum FileId(EditionedFileId(Id(2000))) 0..33 5..8

                FileId(EditionedFileId(Id(2000))) 70..73
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_for_local() {
        check(
            r#"
fn main() {
    let mut i = 1;
    let j = 1;
    i = i$0 + j;

    {
        i = 0;
    }

    i = 5;
}"#,
            expect![[r#"
                i Local FileId(EditionedFileId(Id(2000))) 20..25 24..25 write

                FileId(EditionedFileId(Id(2000))) 50..51 write
                FileId(EditionedFileId(Id(2000))) 54..55 read
                FileId(EditionedFileId(Id(2000))) 76..77 write
                FileId(EditionedFileId(Id(2000))) 94..95 write
            "#]],
        );
    }

    #[test]
    fn search_filters_by_range() {
        check(
            r#"
fn foo() {
    let spam$0 = 92;
    spam + spam
}
fn bar() {
    let spam = 92;
    spam + spam
}
"#,
            expect![[r#"
                spam Local FileId(EditionedFileId(Id(2000))) 19..23 19..23

                FileId(EditionedFileId(Id(2000))) 34..38 read
                FileId(EditionedFileId(Id(2000))) 41..45 read
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_for_param_inside() {
        check(
            r#"
fn foo(i : u32) -> u32 { i$0 }
"#,
            expect![[r#"
                i ValueParam FileId(EditionedFileId(Id(2000))) 7..8 7..8

                FileId(EditionedFileId(Id(2000))) 25..26 read
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_for_fn_param() {
        check(
            r#"
fn foo(i$0 : u32) -> u32 { i }
"#,
            expect![[r#"
                i ValueParam FileId(EditionedFileId(Id(2000))) 7..8 7..8

                FileId(EditionedFileId(Id(2000))) 25..26 read
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_field_name() {
        check(
            r#"
//- /lib.rs
struct Foo {
    pub spam$0: u32,
}

fn main(s: Foo) {
    let f = s.spam;
}
"#,
            expect![[r#"
                spam Field FileId(EditionedFileId(Id(2000))) 17..30 21..25

                FileId(EditionedFileId(Id(2000))) 67..71 read
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_impl_item_name() {
        check(
            r#"
struct Foo;
impl Foo {
    fn f$0(&self) {  }
}
"#,
            expect![[r#"
                f Function FileId(EditionedFileId(Id(2000))) 27..43 30..31

                (no references)
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_enum_var_name() {
        check(
            r#"
enum Foo {
    A,
    B$0,
    C,
}
"#,
            expect![[r#"
                B Variant FileId(EditionedFileId(Id(2000))) 22..23 22..23

                (no references)
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_enum_var_field() {
        check(
            r#"
enum Foo {
    A,
    B { field$0: u8 },
    C,
}
"#,
            expect![[r#"
                field Field FileId(EditionedFileId(Id(2000))) 26..35 26..31

                (no references)
            "#]],
        );
    }

    #[test]
    fn test_self() {
        check(
            r#"
struct S$0<T> {
    t: PhantomData<T>,
}

impl<T> S<T> {
    fn new() -> Self {
        Self {
            t: Default::default(),
        }
    }
}
"#,
            expect![[r#"
                S Struct FileId(EditionedFileId(Id(2000))) 0..38 7..8

                FileId(EditionedFileId(Id(2000))) 48..49
                FileId(EditionedFileId(Id(2000))) 71..75
                FileId(EditionedFileId(Id(2000))) 86..90
            "#]],
        )
    }

    #[test]
    fn test_self_inside_not_adt_impl() {
        check(
            r#"
pub trait TestTrait {
    type Assoc;
    fn stuff() -> Self;
}
impl TestTrait for () {
    type Assoc$0 = u8;
    fn stuff() -> Self {
        let me: Self = ();
        me
    }
}
"#,
            expect![[r#"
                Assoc TypeAlias FileId(EditionedFileId(Id(2000))) 92..108 97..102

                FileId(EditionedFileId(Id(2000))) 31..36
            "#]],
        )
    }

    #[test]
    fn test_find_all_refs_two_modules() {
        check(
            r#"
//- /lib.rs
pub mod foo;
pub mod bar;

fn f() {
    let i = foo::Foo { n: 5 };
}

//- /foo.rs
use crate::bar;

pub struct Foo {
    pub n: u32,
}

fn f() {
    let i = bar::Bar { n: 5 };
}

//- /bar.rs
use crate::foo;

pub struct Bar {
    pub n: u32,
}

fn f() {
    let i = foo::Foo$0 { n: 5 };
}
"#,
            expect![[r#"
                Foo Struct FileId(EditionedFileId(Id(2001))) 17..51 28..31 foo

                FileId(EditionedFileId(Id(2000))) 53..56
                FileId(EditionedFileId(Id(2002))) 79..82
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_decl_module() {
        check(
            r#"
//- /lib.rs
mod foo$0;

use foo::Foo;

fn f() {
    let i = Foo { n: 5 };
}

//- /foo.rs
pub struct Foo {
    pub n: u32,
}
"#,
            expect![[r#"
                foo Module FileId(EditionedFileId(Id(2000))) 0..8 4..7

                FileId(EditionedFileId(Id(2000))) 14..17 import
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_decl_module_on_self() {
        check(
            r#"
//- /lib.rs
mod foo;

//- /foo.rs
use self$0;
"#,
            expect![[r#"
                foo Module FileId(EditionedFileId(Id(2000))) 0..8 4..7

                FileId(EditionedFileId(Id(2001))) 4..8 import
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_decl_module_on_self_crate_root() {
        check(
            r#"
//- /lib.rs
use self$0;
"#,
            expect![[r#"
                Module FileId(EditionedFileId(Id(2000))) 0..10

                FileId(EditionedFileId(Id(2000))) 4..8 import
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_super_mod_vis() {
        check(
            r#"
//- /lib.rs
mod foo;

//- /foo.rs
mod some;
use some::Foo;

fn f() {
    let i = Foo { n: 5 };
}

//- /foo/some.rs
pub(super) struct Foo$0 {
    pub n: u32,
}
"#,
            expect![[r#"
                Foo Struct FileId(EditionedFileId(Id(2002))) 0..41 18..21 some

                FileId(EditionedFileId(Id(2001))) 20..23 import
                FileId(EditionedFileId(Id(2001))) 47..50
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_with_scope() {
        let code = r#"
            //- /lib.rs
            mod foo;
            mod bar;

            pub fn quux$0() {}

            //- /foo.rs
            fn f() { super::quux(); }

            //- /bar.rs
            fn f() { super::quux(); }
        "#;

        check_with_scope(
            code,
            None,
            expect![[r#"
                quux Function FileId(EditionedFileId(Id(2000))) 19..35 26..30

                FileId(EditionedFileId(Id(2001))) 16..20
                FileId(EditionedFileId(Id(2002))) 16..20
            "#]],
        );

        check_with_scope(
            code,
            Some(&mut |db| {
                SearchScope::single_file(EditionedFileId::current_edition(db, FileId::from_raw(2)))
            }),
            expect![[r#"
                quux Function FileId(EditionedFileId(Id(2000))) 19..35 26..30

                FileId(EditionedFileId(Id(2002))) 16..20
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_macro_def() {
        check(
            r#"
#[macro_export]
macro_rules! m1$0 { () => (()) }

fn foo() {
    m1();
    m1();
}
"#,
            expect![[r#"
                m1 Macro FileId(EditionedFileId(Id(2000))) 0..46 29..31

                FileId(EditionedFileId(Id(2000))) 63..65
                FileId(EditionedFileId(Id(2000))) 73..75
            "#]],
        );
    }

    #[test]
    fn test_basic_highlight_read_write() {
        check(
            r#"
fn foo() {
    let mut i$0 = 0;
    i = i + 1;
}
"#,
            expect![[r#"
                i Local FileId(EditionedFileId(Id(2000))) 19..24 23..24 write

                FileId(EditionedFileId(Id(2000))) 34..35 write
                FileId(EditionedFileId(Id(2000))) 38..39 read
            "#]],
        );
    }

    #[test]
    fn test_basic_highlight_field_read_write() {
        check(
            r#"
struct S {
    f: u32,
}

fn foo() {
    let mut s = S{f: 0};
    s.f$0 = 0;
}
"#,
            expect![[r#"
                f Field FileId(EditionedFileId(Id(2000))) 15..21 15..16

                FileId(EditionedFileId(Id(2000))) 55..56 read
                FileId(EditionedFileId(Id(2000))) 68..69 write
            "#]],
        );
    }

    #[test]
    fn test_basic_highlight_decl_no_write() {
        check(
            r#"
fn foo() {
    let i$0;
    i = 1;
}
"#,
            expect![[r#"
                i Local FileId(EditionedFileId(Id(2000))) 19..20 19..20

                FileId(EditionedFileId(Id(2000))) 26..27 write
            "#]],
        );
    }

    #[test]
    fn test_find_struct_function_refs_outside_module() {
        check(
            r#"
mod foo {
    pub struct Foo;

    impl Foo {
        pub fn new$0() -> Foo { Foo }
    }
}

fn main() {
    let _f = foo::Foo::new();
}
"#,
            expect![[r#"
                new Function FileId(EditionedFileId(Id(2000))) 54..81 61..64

                FileId(EditionedFileId(Id(2000))) 126..129
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_nested_module() {
        check(
            r#"
//- /lib.rs
mod foo { mod bar; }

fn f$0() {}

//- /foo/bar.rs
use crate::f;

fn g() { f(); }
"#,
            expect![[r#"
                f Function FileId(EditionedFileId(Id(2000))) 22..31 25..26

                FileId(EditionedFileId(Id(2001))) 11..12 import
                FileId(EditionedFileId(Id(2001))) 24..25
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_struct_pat() {
        check(
            r#"
struct S {
    field$0: u8,
}

fn f(s: S) {
    match s {
        S { field } => {}
    }
}
"#,
            expect![[r#"
                field Field FileId(EditionedFileId(Id(2000))) 15..24 15..20

                FileId(EditionedFileId(Id(2000))) 68..73 read
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_enum_var_pat() {
        check(
            r#"
enum En {
    Variant {
        field$0: u8,
    }
}

fn f(e: En) {
    match e {
        En::Variant { field } => {}
    }
}
"#,
            expect![[r#"
                field Field FileId(EditionedFileId(Id(2000))) 32..41 32..37

                FileId(EditionedFileId(Id(2000))) 102..107 read
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_enum_var_privacy() {
        check(
            r#"
mod m {
    pub enum En {
        Variant {
            field$0: u8,
        }
    }
}

fn f() -> m::En {
    m::En::Variant { field: 0 }
}
"#,
            expect![[r#"
                field Field FileId(EditionedFileId(Id(2000))) 56..65 56..61

                FileId(EditionedFileId(Id(2000))) 125..130 read
            "#]],
        );
    }

    #[test]
    fn test_find_self_refs() {
        check(
            r#"
struct Foo { bar: i32 }

impl Foo {
    fn foo(self) {
        let x = self$0.bar;
        if true {
            let _ = match () {
                () => self,
            };
        }
    }
}
"#,
            expect![[r#"
                self SelfParam FileId(EditionedFileId(Id(2000))) 47..51 47..51

                FileId(EditionedFileId(Id(2000))) 71..75 read
                FileId(EditionedFileId(Id(2000))) 152..156 read
            "#]],
        );
    }

    #[test]
    fn test_find_self_refs_decl() {
        check(
            r#"
struct Foo { bar: i32 }

impl Foo {
    fn foo(self$0) {
        self;
    }
}
"#,
            expect![[r#"
                self SelfParam FileId(EditionedFileId(Id(2000))) 47..51 47..51

                FileId(EditionedFileId(Id(2000))) 63..67 read
            "#]],
        );
    }

    #[test]
    fn test_highlight_if_branches() {
        check(
            r#"
fn main() {
    let x = if$0 true {
        1
    } else if false {
        2
    } else {
        3
    };

    println!("x: {}", x);
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 24..26
                FileId(EditionedFileId(Id(2000))) 42..43
                FileId(EditionedFileId(Id(2000))) 55..57
                FileId(EditionedFileId(Id(2000))) 74..75
                FileId(EditionedFileId(Id(2000))) 97..98
            "#]],
        );
    }

    #[test]
    fn test_highlight_match_branches() {
        check(
            r#"
fn main() {
    $0match Some(42) {
        Some(x) if x > 0 => println!("positive"),
        Some(0) => println!("zero"),
        Some(_) => println!("negative"),
        None => println!("none"),
    };
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 16..21
                FileId(EditionedFileId(Id(2000))) 61..81
                FileId(EditionedFileId(Id(2000))) 102..118
                FileId(EditionedFileId(Id(2000))) 139..159
                FileId(EditionedFileId(Id(2000))) 177..193
            "#]],
        );
    }

    #[test]
    fn test_highlight_match_arm_arrow() {
        check(
            r#"
fn main() {
    match Some(42) {
        Some(x) if x > 0 $0=> println!("positive"),
        Some(0) => println!("zero"),
        Some(_) => println!("negative"),
        None => println!("none"),
    }
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 58..60
                FileId(EditionedFileId(Id(2000))) 61..81
            "#]],
        );
    }

    #[test]
    fn test_highlight_nested_branches() {
        check(
            r#"
fn main() {
    let x = $0if true {
        if false {
            1
        } else {
            match Some(42) {
                Some(_) => 2,
                None => 3,
            }
        }
    } else {
        4
    };

    println!("x: {}", x);
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 24..26
                FileId(EditionedFileId(Id(2000))) 65..66
                FileId(EditionedFileId(Id(2000))) 140..141
                FileId(EditionedFileId(Id(2000))) 167..168
                FileId(EditionedFileId(Id(2000))) 215..216
            "#]],
        );
    }

    #[test]
    fn test_highlight_match_with_complex_guards() {
        check(
            r#"
fn main() {
    let x = $0match (x, y) {
        (a, b) if a > b && a % 2 == 0 => 1,
        (a, b) if a < b || b % 2 == 1 => 2,
        (a, _) if a > 40 => 3,
        _ => 4,
    };

    println!("x: {}", x);
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 24..29
                FileId(EditionedFileId(Id(2000))) 80..81
                FileId(EditionedFileId(Id(2000))) 124..125
                FileId(EditionedFileId(Id(2000))) 155..156
                FileId(EditionedFileId(Id(2000))) 171..172
            "#]],
        );
    }

    #[test]
    fn test_highlight_mixed_if_match_expressions() {
        check(
            r#"
fn main() {
    let x = $0if let Some(x) = Some(42) {
        1
    } else if let None = None {
        2
    } else {
        match 42 {
            0 => 3,
            _ => 4,
        }
    };
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 24..26
                FileId(EditionedFileId(Id(2000))) 60..61
                FileId(EditionedFileId(Id(2000))) 73..75
                FileId(EditionedFileId(Id(2000))) 102..103
                FileId(EditionedFileId(Id(2000))) 153..154
                FileId(EditionedFileId(Id(2000))) 173..174
            "#]],
        );
    }

    fn check(#[rust_analyzer::rust_fixture] ra_fixture: &str, expect: Expect) {
        check_with_scope(ra_fixture, None, expect)
    }

    fn check_with_scope(
        #[rust_analyzer::rust_fixture] ra_fixture: &str,
        search_scope: Option<&mut dyn FnMut(&RootDatabase) -> SearchScope>,
        expect: Expect,
    ) {
        let (analysis, pos) = fixture::position(ra_fixture);
        let refs = analysis
            .find_all_refs(pos.into(), search_scope.map(|it| it(&analysis.db)))
            .unwrap()
            .unwrap();

        let mut actual = String::new();
        for mut refs in refs {
            actual += "\n\n";

            if let Some(decl) = refs.declaration {
                format_to!(actual, "{}", decl.nav.debug_render());
                if decl.is_mut {
                    format_to!(actual, " write",)
                }
                actual += "\n\n";
            }

            for (file_id, references) in &mut refs.references {
                references.sort_by_key(|(range, _)| range.start());
                for (range, category) in references {
                    format_to!(actual, "{:?} {:?}", file_id, range);
                    for (name, _flag) in category.iter_names() {
                        format_to!(actual, " {}", name.to_lowercase());
                    }
                    actual += "\n";
                }
            }

            if refs.references.is_empty() {
                actual += "(no references)\n";
            }
        }
        expect.assert_eq(actual.trim_start())
    }

    #[test]
    fn test_find_lifetimes_function() {
        check(
            r#"
trait Foo<'a> {}
impl<'a> Foo<'a> for &'a () {}
fn foo<'a, 'b: 'a>(x: &'a$0 ()) -> &'a () where &'a (): Foo<'a> {
    fn bar<'a>(_: &'a ()) {}
    x
}
"#,
            expect![[r#"
                'a LifetimeParam FileId(EditionedFileId(Id(2000))) 55..57 55..57

                FileId(EditionedFileId(Id(2000))) 63..65
                FileId(EditionedFileId(Id(2000))) 71..73
                FileId(EditionedFileId(Id(2000))) 82..84
                FileId(EditionedFileId(Id(2000))) 95..97
                FileId(EditionedFileId(Id(2000))) 106..108
            "#]],
        );
    }

    #[test]
    fn test_find_lifetimes_type_alias() {
        check(
            r#"
type Foo<'a, T> where T: 'a$0 = &'a T;
"#,
            expect![[r#"
                'a LifetimeParam FileId(EditionedFileId(Id(2000))) 9..11 9..11

                FileId(EditionedFileId(Id(2000))) 25..27
                FileId(EditionedFileId(Id(2000))) 31..33
            "#]],
        );
    }

    #[test]
    fn test_find_lifetimes_trait_impl() {
        check(
            r#"
trait Foo<'a> {
    fn foo() -> &'a ();
}
impl<'a> Foo<'a> for &'a () {
    fn foo() -> &'a$0 () {
        unimplemented!()
    }
}
"#,
            expect![[r#"
                'a LifetimeParam FileId(EditionedFileId(Id(2000))) 47..49 47..49

                FileId(EditionedFileId(Id(2000))) 55..57
                FileId(EditionedFileId(Id(2000))) 64..66
                FileId(EditionedFileId(Id(2000))) 89..91
            "#]],
        );
    }

    #[test]
    fn test_map_range_to_original() {
        check(
            r#"
macro_rules! foo {($i:ident) => {$i} }
fn main() {
    let a$0 = "test";
    foo!(a);
}
"#,
            expect![[r#"
                a Local FileId(EditionedFileId(Id(2000))) 59..60 59..60

                MacroFile(MacroCallId(Id(4c00))) 0..1 read
            "#]],
        );
    }

    #[test]
    fn test_map_range_to_original_ref() {
        check(
            r#"
macro_rules! foo {($i:ident) => {$i} }
fn main() {
    let a = "test";
    foo!(a$0);
}
"#,
            expect![[r#"
                a Local FileId(EditionedFileId(Id(2000))) 59..60 59..60

                MacroFile(MacroCallId(Id(4c00))) 0..1 read
            "#]],
        );
    }

    #[test]
    fn test_find_labels() {
        check(
            r#"
fn foo<'a>() -> &'a () {
    'a: loop {
        'b: loop {
            continue 'a$0;
        }
        break 'a;
    }
}
"#,
            expect![[r#"
                'a Label FileId(EditionedFileId(Id(2000))) 29..32 29..31

                FileId(EditionedFileId(Id(2000))) 80..82
                FileId(EditionedFileId(Id(2000))) 108..110
            "#]],
        );
    }

    #[test]
    fn test_find_const_param() {
        check(
            r#"
fn foo<const FOO$0: usize>() -> usize {
    FOO
}
"#,
            expect![[r#"
                FOO ConstParam FileId(EditionedFileId(Id(2000))) 7..23 13..16

                FileId(EditionedFileId(Id(2000))) 42..45
            "#]],
        );
    }

    #[test]
    fn test_trait() {
        check(
            r#"
trait Foo$0 where Self: {}

impl Foo for () {}
"#,
            expect![[r#"
                Foo Trait FileId(EditionedFileId(Id(2000))) 0..24 6..9

                FileId(EditionedFileId(Id(2000))) 31..34
            "#]],
        );
    }

    #[test]
    fn test_trait_self() {
        check(
            r#"
trait Foo where Self$0 {
    fn f() -> Self;
}

impl Foo for () {}
"#,
            expect![[r#"
                Self TypeParam FileId(EditionedFileId(Id(2000))) 0..44 6..9

                FileId(EditionedFileId(Id(2000))) 16..20
                FileId(EditionedFileId(Id(2000))) 37..41
            "#]],
        );
    }

    #[test]
    fn test_self_ty() {
        check(
            r#"
        struct $0Foo;

        impl Foo where Self: {
            fn f() -> Self;
        }
        "#,
            expect![[r#"
                Foo Struct FileId(EditionedFileId(Id(2000))) 0..11 7..10

                FileId(EditionedFileId(Id(2000))) 18..21
                FileId(EditionedFileId(Id(2000))) 28..32
                FileId(EditionedFileId(Id(2000))) 50..54
            "#]],
        );
        check(
            r#"
struct Foo;

impl Foo where Self: {
    fn f() -> Self$0;
}
"#,
            expect![[r#"
                impl Impl FileId(EditionedFileId(Id(2000))) 13..57 18..21

                FileId(EditionedFileId(Id(2000))) 18..21
                FileId(EditionedFileId(Id(2000))) 28..32
                FileId(EditionedFileId(Id(2000))) 50..54
            "#]],
        );
    }
    #[test]
    fn test_self_variant_with_payload() {
        check(
            r#"
enum Foo { Bar() }

impl Foo {
    fn foo(self) {
        match self {
            Self::Bar$0() => (),
        }
    }
}

"#,
            expect![[r#"
                Bar Variant FileId(EditionedFileId(Id(2000))) 11..16 11..14

                FileId(EditionedFileId(Id(2000))) 89..92
            "#]],
        );
    }

    #[test]
    fn test_trait_alias() {
        check(
            r#"
trait Foo {}
trait Bar$0 = Foo where Self: ;
fn foo<T: Bar>(_: impl Bar, _: &dyn Bar) {}
"#,
            expect![[r#"
                Bar TraitAlias FileId(EditionedFileId(Id(2000))) 13..42 19..22

                FileId(EditionedFileId(Id(2000))) 53..56
                FileId(EditionedFileId(Id(2000))) 66..69
                FileId(EditionedFileId(Id(2000))) 79..82
            "#]],
        );
    }

    #[test]
    fn test_trait_alias_self() {
        check(
            r#"
trait Foo = where Self$0: ;
"#,
            expect![[r#"
                Self TypeParam FileId(EditionedFileId(Id(2000))) 0..25 6..9

                FileId(EditionedFileId(Id(2000))) 18..22
            "#]],
        );
    }

    #[test]
    fn test_attr_differs_from_fn_with_same_name() {
        check(
            r#"
#[test]
fn test$0() {
    test();
}
"#,
            expect![[r#"
                test Function FileId(EditionedFileId(Id(2000))) 0..33 11..15

                FileId(EditionedFileId(Id(2000))) 24..28 test
            "#]],
        );
    }

    #[test]
    fn test_const_in_pattern() {
        check(
            r#"
const A$0: i32 = 42;

fn main() {
    match A {
        A => (),
        _ => (),
    }
    if let A = A {}
}
"#,
            expect![[r#"
                A Const FileId(EditionedFileId(Id(2000))) 0..18 6..7

                FileId(EditionedFileId(Id(2000))) 42..43
                FileId(EditionedFileId(Id(2000))) 54..55
                FileId(EditionedFileId(Id(2000))) 97..98
                FileId(EditionedFileId(Id(2000))) 101..102
            "#]],
        );
    }

    #[test]
    fn test_primitives() {
        check(
            r#"
fn foo(_: bool) -> bo$0ol { true }
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 10..14
                FileId(EditionedFileId(Id(2000))) 19..23
            "#]],
        );
    }

    #[test]
    fn test_transitive() {
        check(
            r#"
//- /level3.rs new_source_root:local crate:level3
pub struct Fo$0o;
//- /level2.rs new_source_root:local crate:level2 deps:level3
pub use level3::Foo;
//- /level1.rs new_source_root:local crate:level1 deps:level2
pub use level2::Foo;
//- /level0.rs new_source_root:local crate:level0 deps:level1
pub use level1::Foo;
"#,
            expect![[r#"
                Foo Struct FileId(EditionedFileId(Id(2000))) 0..15 11..14

                FileId(EditionedFileId(Id(2002))) 16..19 import
                FileId(EditionedFileId(Id(2001))) 16..19 import
                FileId(EditionedFileId(Id(2003))) 16..19 import
            "#]],
        );
    }

    #[test]
    fn test_decl_macro_references() {
        check(
            r#"
//- /lib.rs crate:lib
#[macro_use]
mod qux;
mod bar;

pub use self::foo;
//- /qux.rs
#[macro_export]
macro_rules! foo$0 {
    () => {struct Foo;};
}
//- /bar.rs
foo!();
//- /other.rs crate:other deps:lib new_source_root:local
lib::foo!();
"#,
            expect![[r#"
                foo Macro FileId(EditionedFileId(Id(2001))) 0..61 29..32

                FileId(EditionedFileId(Id(2000))) 46..49 import
                FileId(EditionedFileId(Id(2002))) 0..3
                FileId(EditionedFileId(Id(2003))) 5..8
            "#]],
        );
    }

    #[test]
    fn macro_doesnt_reference_attribute_on_call() {
        check(
            r#"
macro_rules! m {
    () => {};
}

#[proc_macro_test::attr_noop]
m$0!();

"#,
            expect![[r#"
                m Macro FileId(EditionedFileId(Id(2000))) 0..32 13..14

                FileId(EditionedFileId(Id(2000))) 64..65
            "#]],
        );
    }

    #[test]
    fn multi_def() {
        check(
            r#"
macro_rules! m {
    ($name:ident) => {
        mod module {
            pub fn $name() {}
        }

        pub fn $name() {}
    }
}

m!(func$0);

fn f() {
    func();
    module::func();
}
            "#,
            expect![[r#"
                func Function MacroFile(MacroCallId(Id(3c00))) 10..23 15..19 module

                FileId(EditionedFileId(Id(2000))) 181..185


                func Function MacroFile(MacroCallId(Id(3c00))) 24..37 29..33

                FileId(EditionedFileId(Id(2000))) 161..165
            "#]],
        )
    }

    #[test]
    fn attr_expanded() {
        check(
            r#"
//- proc_macros: identity
#[proc_macros::identity]
fn func$0() {
    func();
}
"#,
            expect![[r#"
                func Function MacroFile(MacroCallId(Id(4400))) 0..17 2..6

                MacroFile(MacroCallId(Id(4400))) 9..13
            "#]],
        )
    }

    #[test]
    fn attr_assoc_item() {
        check(
            r#"
//- proc_macros: identity

trait Trait {
    #[proc_macros::identity]
    fn func() {
        Self::func$0();
    }
}
"#,
            expect![[r#"
                func Function MacroFile(MacroCallId(Id(4800))) 0..23 2..6 Trait

                MacroFile(MacroCallId(Id(4800))) 15..19
            "#]],
        )
    }

    // FIXME: import is classified as function
    #[test]
    fn attr() {
        check(
            r#"
//- proc_macros: identity
use proc_macros::identity;

#[proc_macros::$0identity]
fn func() {}
"#,
            expect![[r#"
                identity Attribute FileId(EditionedFileId(Id(2001))) 1..107 32..40

                FileId(EditionedFileId(Id(2000))) 43..51
            "#]],
        );
        check(
            r#"
#![crate_type="proc-macro"]
#[proc_macro_attribute]
fn func$0() {}
"#,
            expect![[r#"
                func Attribute FileId(EditionedFileId(Id(2000))) 28..64 55..59

                (no references)
            "#]],
        );
    }

    // FIXME: import is classified as function
    #[test]
    fn proc_macro() {
        check(
            r#"
//- proc_macros: mirror
use proc_macros::mirror;

mirror$0! {}
"#,
            expect![[r#"
                mirror ProcMacro FileId(EditionedFileId(Id(2001))) 1..77 22..28

                FileId(EditionedFileId(Id(2000))) 26..32
            "#]],
        )
    }

    #[test]
    fn derive() {
        check(
            r#"
//- proc_macros: derive_identity
//- minicore: derive
use proc_macros::DeriveIdentity;

#[derive(proc_macros::DeriveIdentity$0)]
struct Foo;
"#,
            expect![[r#"
                derive_identity Derive FileId(EditionedFileId(Id(2002))) 1..107 45..60

                FileId(EditionedFileId(Id(2000))) 17..31 import
                MacroFile(MacroCallId(Id(4c00))) 17..31
            "#]],
        );
        check(
            r#"
#![crate_type="proc-macro"]
#[proc_macro_derive(Derive, attributes(x))]
pub fn deri$0ve(_stream: TokenStream) -> TokenStream {}
"#,
            expect![[r#"
                derive Derive FileId(EditionedFileId(Id(2000))) 28..125 79..85

                (no references)
            "#]],
        );
    }

    #[test]
    fn assoc_items_trait_def() {
        check(
            r#"
trait Trait {
    const CONST$0: usize;
}

impl Trait for () {
    const CONST: usize = 0;
}

impl Trait for ((),) {
    const CONST: usize = 0;
}

fn f<T: Trait>() {
    let _ = <()>::CONST;

    let _ = T::CONST;
}
"#,
            expect![[r#"
                CONST Const FileId(EditionedFileId(Id(2000))) 18..37 24..29 Trait

                FileId(EditionedFileId(Id(2000))) 71..76
                FileId(EditionedFileId(Id(2000))) 125..130
                FileId(EditionedFileId(Id(2000))) 183..188
                FileId(EditionedFileId(Id(2000))) 206..211
            "#]],
        );
        check(
            r#"
trait Trait {
    type TypeAlias$0;
}

impl Trait for () {
    type TypeAlias = ();
}

impl Trait for ((),) {
    type TypeAlias = ();
}

fn f<T: Trait>() {
    let _: <() as Trait>::TypeAlias;

    let _: T::TypeAlias;
}
"#,
            expect![[r#"
                TypeAlias TypeAlias FileId(EditionedFileId(Id(2000))) 18..33 23..32 Trait

                FileId(EditionedFileId(Id(2000))) 66..75
                FileId(EditionedFileId(Id(2000))) 117..126
                FileId(EditionedFileId(Id(2000))) 181..190
                FileId(EditionedFileId(Id(2000))) 207..216
            "#]],
        );
        check(
            r#"
trait Trait {
    fn function$0() {}
}

impl Trait for () {
    fn function() {}
}

impl Trait for ((),) {
    fn function() {}
}

fn f<T: Trait>() {
    let _ = <()>::function;

    let _ = T::function;
}
"#,
            expect![[r#"
                function Function FileId(EditionedFileId(Id(2000))) 18..34 21..29 Trait

                FileId(EditionedFileId(Id(2000))) 65..73
                FileId(EditionedFileId(Id(2000))) 112..120
                FileId(EditionedFileId(Id(2000))) 166..174
                FileId(EditionedFileId(Id(2000))) 192..200
            "#]],
        );
    }

    #[test]
    fn assoc_items_trait_impl_def() {
        check(
            r#"
trait Trait {
    const CONST: usize;
}

impl Trait for () {
    const CONST$0: usize = 0;
}

impl Trait for ((),) {
    const CONST: usize = 0;
}

fn f<T: Trait>() {
    let _ = <()>::CONST;

    let _ = T::CONST;
}
"#,
            expect![[r#"
                CONST Const FileId(EditionedFileId(Id(2000))) 65..88 71..76

                FileId(EditionedFileId(Id(2000))) 183..188
            "#]],
        );
        check(
            r#"
trait Trait {
    type TypeAlias;
}

impl Trait for () {
    type TypeAlias$0 = ();
}

impl Trait for ((),) {
    type TypeAlias = ();
}

fn f<T: Trait>() {
    let _: <() as Trait>::TypeAlias;

    let _: T::TypeAlias;
}
"#,
            expect![[r#"
                TypeAlias TypeAlias FileId(EditionedFileId(Id(2000))) 61..81 66..75

                FileId(EditionedFileId(Id(2000))) 23..32
                FileId(EditionedFileId(Id(2000))) 117..126
                FileId(EditionedFileId(Id(2000))) 181..190
                FileId(EditionedFileId(Id(2000))) 207..216
            "#]],
        );
        check(
            r#"
trait Trait {
    fn function() {}
}

impl Trait for () {
    fn function$0() {}
}

impl Trait for ((),) {
    fn function() {}
}

fn f<T: Trait>() {
    let _ = <()>::function;

    let _ = T::function;
}
"#,
            expect![[r#"
                function Function FileId(EditionedFileId(Id(2000))) 62..78 65..73

                FileId(EditionedFileId(Id(2000))) 166..174
            "#]],
        );
    }

    #[test]
    fn assoc_items_ref() {
        check(
            r#"
trait Trait {
    const CONST: usize;
}

impl Trait for () {
    const CONST: usize = 0;
}

impl Trait for ((),) {
    const CONST: usize = 0;
}

fn f<T: Trait>() {
    let _ = <()>::CONST$0;

    let _ = T::CONST;
}
"#,
            expect![[r#"
                CONST Const FileId(EditionedFileId(Id(2000))) 65..88 71..76

                FileId(EditionedFileId(Id(2000))) 183..188
            "#]],
        );
        check(
            r#"
trait Trait {
    type TypeAlias;
}

impl Trait for () {
    type TypeAlias = ();
}

impl Trait for ((),) {
    type TypeAlias = ();
}

fn f<T: Trait>() {
    let _: <() as Trait>::TypeAlias$0;

    let _: T::TypeAlias;
}
"#,
            expect![[r#"
                TypeAlias TypeAlias FileId(EditionedFileId(Id(2000))) 18..33 23..32 Trait

                FileId(EditionedFileId(Id(2000))) 66..75
                FileId(EditionedFileId(Id(2000))) 117..126
                FileId(EditionedFileId(Id(2000))) 181..190
                FileId(EditionedFileId(Id(2000))) 207..216
            "#]],
        );
        check(
            r#"
trait Trait {
    fn function() {}
}

impl Trait for () {
    fn function() {}
}

impl Trait for ((),) {
    fn function() {}
}

fn f<T: Trait>() {
    let _ = <()>::function$0;

    let _ = T::function;
}
"#,
            expect![[r#"
                function Function FileId(EditionedFileId(Id(2000))) 62..78 65..73

                FileId(EditionedFileId(Id(2000))) 166..174
            "#]],
        );
    }

    #[test]
    fn name_clashes() {
        check(
            r#"
trait Foo {
    fn method$0(&self) -> u8;
}

struct Bar {
    method: u8,
}

impl Foo for Bar {
    fn method(&self) -> u8 {
        self.method
    }
}
fn method() {}
"#,
            expect![[r#"
                method Function FileId(EditionedFileId(Id(2000))) 16..39 19..25 Foo

                FileId(EditionedFileId(Id(2000))) 101..107
            "#]],
        );
        check(
            r#"
trait Foo {
    fn method(&self) -> u8;
}

struct Bar {
    method$0: u8,
}

impl Foo for Bar {
    fn method(&self) -> u8 {
        self.method
    }
}
fn method() {}
"#,
            expect![[r#"
                method Field FileId(EditionedFileId(Id(2000))) 60..70 60..66

                FileId(EditionedFileId(Id(2000))) 136..142 read
            "#]],
        );
        check(
            r#"
trait Foo {
    fn method(&self) -> u8;
}

struct Bar {
    method: u8,
}

impl Foo for Bar {
    fn method$0(&self) -> u8 {
        self.method
    }
}
fn method() {}
"#,
            expect![[r#"
                method Function FileId(EditionedFileId(Id(2000))) 98..148 101..107

                (no references)
            "#]],
        );
        check(
            r#"
trait Foo {
    fn method(&self) -> u8;
}

struct Bar {
    method: u8,
}

impl Foo for Bar {
    fn method(&self) -> u8 {
        self.method$0
    }
}
fn method() {}
"#,
            expect![[r#"
                method Field FileId(EditionedFileId(Id(2000))) 60..70 60..66

                FileId(EditionedFileId(Id(2000))) 136..142 read
            "#]],
        );
        check(
            r#"
trait Foo {
    fn method(&self) -> u8;
}

struct Bar {
    method: u8,
}

impl Foo for Bar {
    fn method(&self) -> u8 {
        self.method
    }
}
fn method$0() {}
"#,
            expect![[r#"
                method Function FileId(EditionedFileId(Id(2000))) 151..165 154..160

                (no references)
            "#]],
        );
    }

    #[test]
    fn raw_identifier() {
        check(
            r#"
fn r#fn$0() {}
fn main() { r#fn(); }
"#,
            expect![[r#"
                r#fn Function FileId(EditionedFileId(Id(2000))) 0..12 3..7

                FileId(EditionedFileId(Id(2000))) 25..29
            "#]],
        );
    }

    #[test]
    fn implicit_format_args() {
        check(
            r#"
//- minicore: fmt
fn test() {
    let a = "foo";
    format_args!("hello {a} {a$0} {}", a);
                      // ^
                          // ^
                                   // ^
}
"#,
            expect![[r#"
                a Local FileId(EditionedFileId(Id(2000))) 20..21 20..21

                MacroFile(MacroCallId(Id(4807))) 28..29 read
                MacroFile(MacroCallId(Id(4807))) 32..33 read
                MacroFile(MacroCallId(Id(4807))) 39..40 read
            "#]],
        );
    }

    #[test]
    fn goto_ref_fn_kw() {
        check(
            r#"
macro_rules! N {
    ($i:ident, $x:expr, $blk:expr) => {
        for $i in 0..$x {
            $blk
        }
    };
}

fn main() {
    $0fn f() {
        N!(i, 5, {
            println!("{}", i);
            return;
        });

        for i in 1..5 {
            return;
        }

       (|| {
            return;
        })();
    }
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 136..138
                FileId(EditionedFileId(Id(2000))) 207..213
                FileId(EditionedFileId(Id(2000))) 264..270
            "#]],
        )
    }

    #[test]
    fn goto_ref_exit_points() {
        check(
            r#"
fn$0 foo() -> u32 {
    if true {
        return 0;
    }

    0?;
    0xDEAD_BEEF
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 0..2
                FileId(EditionedFileId(Id(2000))) 40..46
                FileId(EditionedFileId(Id(2000))) 62..63
                FileId(EditionedFileId(Id(2000))) 69..80
            "#]],
        );
    }

    #[test]
    fn test_ref_yield_points() {
        check(
            r#"
pub async$0 fn foo() {
    let x = foo()
        .await
        .await;
    || { 0.await };
    (async { 0.await }).await
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 4..9
                FileId(EditionedFileId(Id(2000))) 48..53
                FileId(EditionedFileId(Id(2000))) 63..68
                FileId(EditionedFileId(Id(2000))) 114..119
            "#]],
        );
    }

    #[test]
    fn goto_ref_for_kw() {
        check(
            r#"
fn main() {
    $0for i in 1..5 {
        break;
        continue;
    }
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 16..19
                FileId(EditionedFileId(Id(2000))) 40..45
                FileId(EditionedFileId(Id(2000))) 55..63
            "#]],
        )
    }

    #[test]
    fn goto_ref_on_break_kw() {
        check(
            r#"
fn main() {
    for i in 1..5 {
        $0break;
        continue;
    }
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 16..19
                FileId(EditionedFileId(Id(2000))) 40..45
            "#]],
        )
    }

    #[test]
    fn goto_ref_on_break_kw_for_block() {
        check(
            r#"
fn main() {
    'a:{
        $0break 'a;
    }
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 16..19
                FileId(EditionedFileId(Id(2000))) 29..37
            "#]],
        )
    }

    #[test]
    fn goto_ref_on_break_with_label() {
        check(
            r#"
fn foo() {
    'outer: loop {
         break;
         'inner: loop {
            'innermost: loop {
            }
            $0break 'outer;
            break;
        }
        break;
    }
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 15..27
                FileId(EditionedFileId(Id(2000))) 39..44
                FileId(EditionedFileId(Id(2000))) 127..139
                FileId(EditionedFileId(Id(2000))) 178..183
            "#]],
        );
    }

    #[test]
    fn goto_ref_on_return_in_try() {
        check(
            r#"
fn main() {
    fn f() {
        try {
            $0return;
        }

        return;
    }
    return;
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 16..18
                FileId(EditionedFileId(Id(2000))) 51..57
                FileId(EditionedFileId(Id(2000))) 78..84
            "#]],
        )
    }

    #[test]
    fn goto_ref_on_break_in_try() {
        check(
            r#"
fn main() {
    for i in 1..100 {
        let x: Result<(), ()> = try {
            $0break;
        };
    }
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 16..19
                FileId(EditionedFileId(Id(2000))) 84..89
            "#]],
        )
    }

    #[test]
    fn goto_ref_on_return_in_async_block() {
        check(
            r#"
fn main() {
    $0async {
        return;
    }
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 16..21
                FileId(EditionedFileId(Id(2000))) 32..38
            "#]],
        )
    }

    #[test]
    fn goto_ref_on_return_in_macro_call() {
        check(
            r#"
//- minicore:include
//- /lib.rs
macro_rules! M {
    ($blk:expr) => {
        fn f() {
            $blk
        }

        $blk
    };
}

fn main() {
    M!({
        return$0;
    });

    f();
    include!("a.rs")
}

//- /a.rs
{
    return;
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 46..48
                FileId(EditionedFileId(Id(2000))) 106..108
                FileId(EditionedFileId(Id(2000))) 122..149
                FileId(EditionedFileId(Id(2000))) 135..141
                FileId(EditionedFileId(Id(2000))) 165..181
                FileId(EditionedFileId(Id(2001))) 6..12
            "#]],
        )
    }

    // The following are tests for short_associated_function_fast_search() in crates/ide-db/src/search.rs, because find all references
    // use `FindUsages` and I found it easy to test it here.

    #[test]
    fn goto_ref_on_short_associated_function() {
        cov_mark::check!(short_associated_function_fast_search);
        check(
            r#"
struct Foo;
impl Foo {
    fn new$0() {}
}

fn bar() {
    Foo::new();
}
fn baz() {
    Foo::new;
}
        "#,
            expect![[r#"
                new Function FileId(EditionedFileId(Id(2000))) 27..38 30..33

                FileId(EditionedFileId(Id(2000))) 62..65
                FileId(EditionedFileId(Id(2000))) 91..94
            "#]],
        );
    }

    #[test]
    fn goto_ref_on_short_associated_function_with_aliases() {
        cov_mark::check!(short_associated_function_fast_search);
        cov_mark::check!(container_use_rename);
        cov_mark::check!(container_type_alias);
        check(
            r#"
//- /lib.rs
mod a;
mod b;

struct Foo;
impl Foo {
    fn new$0() {}
}

fn bar() {
    b::c::Baz::new();
}

//- /a.rs
use crate::Foo as Bar;

fn baz() { Bar::new(); }
fn quux() { <super::b::Other as super::b::Trait>::Assoc::new(); }

//- /b.rs
pub(crate) mod c;

pub(crate) struct Other;
pub(crate) trait Trait {
    type Assoc;
}
impl Trait for Other {
    type Assoc = super::Foo;
}

//- /b/c.rs
type Itself<T> = T;
pub(in super::super) type Baz = Itself<crate::Foo>;
        "#,
            expect![[r#"
                new Function FileId(EditionedFileId(Id(2000))) 42..53 45..48

                FileId(EditionedFileId(Id(2000))) 83..86
                FileId(EditionedFileId(Id(2001))) 40..43
                FileId(EditionedFileId(Id(2001))) 106..109
            "#]],
        );
    }

    #[test]
    fn goto_ref_on_short_associated_function_self_works() {
        cov_mark::check!(short_associated_function_fast_search);
        cov_mark::check!(self_type_alias);
        check(
            r#"
//- /lib.rs
mod module;

struct Foo;
impl Foo {
    fn new$0() {}
    fn bar() { Self::new(); }
}
trait Trait {
    type Assoc;
    fn baz();
}
impl Trait for Foo {
    type Assoc = Self;
    fn baz() { Self::new(); }
}

//- /module.rs
impl super::Foo {
    fn quux() { Self::new(); }
}
fn foo() { <super::Foo as super::Trait>::Assoc::new(); }
                "#,
            expect![[r#"
                new Function FileId(EditionedFileId(Id(2000))) 40..51 43..46

                FileId(EditionedFileId(Id(2000))) 73..76
                FileId(EditionedFileId(Id(2000))) 195..198
                FileId(EditionedFileId(Id(2001))) 40..43
                FileId(EditionedFileId(Id(2001))) 99..102
            "#]],
        );
    }

    #[test]
    fn goto_ref_on_short_associated_function_overlapping_self_ranges() {
        check(
            r#"
struct Foo;
impl Foo {
    fn new$0() {}
    fn bar() {
        Self::new();
        impl Foo {
            fn baz() { Self::new(); }
        }
    }
}
            "#,
            expect![[r#"
                new Function FileId(EditionedFileId(Id(2000))) 27..38 30..33

                FileId(EditionedFileId(Id(2000))) 68..71
                FileId(EditionedFileId(Id(2000))) 123..126
            "#]],
        );
    }

    #[test]
    fn goto_ref_on_short_associated_function_no_direct_self_but_path_contains_self() {
        cov_mark::check!(short_associated_function_fast_search);
        check(
            r#"
struct Foo;
impl Foo {
    fn new$0() {}
}
trait Trait {
    type Assoc;
}
impl<A, B> Trait for (A, B) {
    type Assoc = B;
}
impl Foo {
    fn bar() {
        <((), Foo) as Trait>::Assoc::new();
        <((), Self) as Trait>::Assoc::new();
    }
}
            "#,
            expect![[r#"
                new Function FileId(EditionedFileId(Id(2000))) 27..38 30..33

                FileId(EditionedFileId(Id(2000))) 188..191
                FileId(EditionedFileId(Id(2000))) 233..236
            "#]],
        );
    }

    // Checks that we can circumvent our fast path logic using complicated type level functions.
    // This mainly exists as a documentation. I don't believe it is fixable.
    // Usages search is not 100% accurate anyway; we miss macros.
    #[test]
    fn goto_ref_on_short_associated_function_complicated_type_magic_can_confuse_our_logic() {
        cov_mark::check!(short_associated_function_fast_search);
        cov_mark::check!(same_name_different_def_type_alias);
        check(
            r#"
struct Foo;
impl Foo {
    fn new$0() {}
}

struct ChoiceA;
struct ChoiceB;
trait Choice {
    type Choose<A, B>;
}
impl Choice for ChoiceA {
    type Choose<A, B> = A;
}
impl Choice for ChoiceB {
    type Choose<A, B> = B;
}
type Choose<A, C> = <C as Choice>::Choose<A, Foo>;

fn bar() {
    Choose::<(), ChoiceB>::new();
}
                "#,
            expect![[r#"
                new Function FileId(EditionedFileId(Id(2000))) 27..38 30..33

                (no references)
            "#]],
        );
    }

    #[test]
    fn goto_ref_on_short_associated_function_same_path_mention_alias_and_self() {
        cov_mark::check!(short_associated_function_fast_search);
        check(
            r#"
struct Foo;
impl Foo {
    fn new$0() {}
}

type IgnoreFirst<A, B> = B;

impl Foo {
    fn bar() {
        <IgnoreFirst<Foo, Self>>::new();
    }
}
                "#,
            expect![[r#"
                new Function FileId(EditionedFileId(Id(2000))) 27..38 30..33

                FileId(EditionedFileId(Id(2000))) 131..134
            "#]],
        );
    }

    #[test]
    fn goto_ref_on_included_file() {
        check(
            r#"
//- minicore:include
//- /lib.rs
include!("foo.rs");
fn howdy() {
    let _ = FOO;
}
//- /foo.rs
const FOO$0: i32 = 0;
"#,
            expect![[r#"
                FOO Const MacroFile(MacroCallId(Id(4401))) 0..15 5..8

                FileId(EditionedFileId(Id(2000))) 45..48
            "#]],
        );
    }

    #[test]
    fn test_highlight_if_let_match_combined() {
        check(
            r#"
enum MyEnum { A(i32), B(String), C }

fn main() {
    let val = MyEnum::A(42);

    let x = $0if let MyEnum::A(x) = val {
        1
    } else if let MyEnum::B(s) = val {
        2
    } else {
        match val {
            MyEnum::C => 3,
            _ => 4,
        }
    };
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 92..94
                FileId(EditionedFileId(Id(2000))) 128..129
                FileId(EditionedFileId(Id(2000))) 141..143
                FileId(EditionedFileId(Id(2000))) 177..178
                FileId(EditionedFileId(Id(2000))) 237..238
                FileId(EditionedFileId(Id(2000))) 257..258
            "#]],
        );
    }

    #[test]
    fn test_highlight_nested_match_expressions() {
        check(
            r#"
enum Outer { A(Inner), B }
enum Inner { X, Y(i32) }

fn main() {
    let val = Outer::A(Inner::Y(42));

    $0match val {
        Outer::A(inner) => match inner {
            Inner::X => println!("Inner::X"),
            Inner::Y(n) if n > 0 => println!("Inner::Y positive: {}", n),
            Inner::Y(_) => println!("Inner::Y non-positive"),
        },
        Outer::B => println!("Outer::B"),
    }
}
"#,
            expect![[r#"
                FileId(EditionedFileId(Id(2000))) 108..113
                FileId(EditionedFileId(Id(2000))) 185..205
                FileId(EditionedFileId(Id(2000))) 243..279
                FileId(EditionedFileId(Id(2000))) 308..341
                FileId(EditionedFileId(Id(2000))) 374..394
            "#]],
        );
    }
}
