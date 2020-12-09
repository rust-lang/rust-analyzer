//! This module implements a reference search.
//! First, the element at the cursor position must be either an `ast::Name`
//! or `ast::NameRef`. If it's a `ast::NameRef`, at the classification step we
//! try to resolve the direct tree parent of this element, otherwise we
//! already have a definition and just need to get its HIR together with
//! some information that is needed for futher steps of searching.
//! After that, we collect files that might contain references and look
//! for text occurrences of the identifier. If there's an `ast::NameRef`
//! at the index that the match starts at and its tree parent is
//! resolved to the search element definition, we get a reference.

pub(crate) mod rename;

use hir::Semantics;
use ide_db::{
    base_db::FileId,
    defs::{Definition, NameClass, NameRefClass},
    search::Reference,
    search::{ReferenceAccess, ReferenceKind, SearchScope},
    RootDatabase,
};
use syntax::{
    algo::find_node_at_offset,
    ast::{self, GenericParamsOwner, LoopBodyOwner, NameOwner, TypeBoundsOwner},
    match_ast, AstNode, NodeOrToken, SmolStr, SyntaxKind, SyntaxNode, SyntaxToken, TextRange,
    TokenAtOffset, WalkEvent, T,
};

use crate::{display::TryToNav, FilePosition, FileRange, NavigationTarget, RangeInfo};

#[derive(Debug, Clone)]
pub struct ReferenceSearchResult {
    declaration: Declaration,
    references: Vec<Reference>,
}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub nav: NavigationTarget,
    pub kind: ReferenceKind,
    pub access: Option<ReferenceAccess>,
}

impl ReferenceSearchResult {
    pub fn declaration(&self) -> &Declaration {
        &self.declaration
    }

    pub fn decl_target(&self) -> &NavigationTarget {
        &self.declaration.nav
    }

    pub fn references(&self) -> &[Reference] {
        &self.references
    }

    /// Total number of references
    /// At least 1 since all valid references should
    /// Have a declaration
    pub fn len(&self) -> usize {
        self.references.len() + 1
    }
}

// allow turning ReferenceSearchResult into an iterator
// over References
impl IntoIterator for ReferenceSearchResult {
    type Item = Reference;
    type IntoIter = std::vec::IntoIter<Reference>;

    fn into_iter(mut self) -> Self::IntoIter {
        let mut v = Vec::with_capacity(self.len());
        v.push(Reference {
            file_range: FileRange {
                file_id: self.declaration.nav.file_id,
                range: self.declaration.nav.focus_or_full_range(),
            },
            kind: self.declaration.kind,
            access: self.declaration.access,
        });
        v.append(&mut self.references);
        v.into_iter()
    }
}

pub(crate) fn find_all_refs(
    sema: &Semantics<RootDatabase>,
    position: FilePosition,
    search_scope: Option<SearchScope>,
) -> Option<RangeInfo<ReferenceSearchResult>> {
    let _p = profile::span("find_all_refs");
    let syntax = sema.parse(position.file_id).syntax().clone();

    if let Some(res) = try_find_self_references(&syntax, position) {
        return Some(res);
    }

    if let Some(res) = try_find_lifetime_references(&syntax, position) {
        return Some(res);
    }

    let (opt_name, search_kind) = if let Some(name) =
        get_struct_def_name_for_struct_literal_search(&sema, &syntax, position)
    {
        (Some(name), ReferenceKind::StructLiteral)
    } else {
        (
            sema.find_node_at_offset_with_descend::<ast::Name>(&syntax, position.offset),
            ReferenceKind::Other,
        )
    };

    let RangeInfo { range, info: def } = find_name(&sema, &syntax, position, opt_name)?;

    let references = def
        .usages(sema)
        .set_scope(search_scope)
        .all()
        .into_iter()
        .filter(|r| search_kind == ReferenceKind::Other || search_kind == r.kind)
        .collect();

    let nav = def.try_to_nav(sema.db)?;
    let decl_range = nav.focus_or_full_range();

    let mut kind = ReferenceKind::Other;
    if let Definition::Local(local) = def {
        if let either::Either::Left(pat) = local.source(sema.db).value {
            if matches!(
                pat.syntax().parent().and_then(ast::RecordPatField::cast),
                Some(pat_field) if pat_field.name_ref().is_none()
            ) {
                kind = ReferenceKind::FieldShorthandForLocal;
            }
        }
    };

    let declaration = Declaration { nav, kind, access: decl_access(&def, &syntax, decl_range) };

    Some(RangeInfo::new(range, ReferenceSearchResult { declaration, references }))
}

fn find_name(
    sema: &Semantics<RootDatabase>,
    syntax: &SyntaxNode,
    position: FilePosition,
    opt_name: Option<ast::Name>,
) -> Option<RangeInfo<Definition>> {
    if let Some(name) = opt_name {
        let def = NameClass::classify(sema, &name)?.referenced_or_defined(sema.db);
        let range = name.syntax().text_range();
        return Some(RangeInfo::new(range, def));
    }
    let name_ref =
        sema.find_node_at_offset_with_descend::<ast::NameRef>(&syntax, position.offset)?;
    let def = NameRefClass::classify(sema, &name_ref)?.referenced(sema.db);
    let range = name_ref.syntax().text_range();
    Some(RangeInfo::new(range, def))
}

fn decl_access(def: &Definition, syntax: &SyntaxNode, range: TextRange) -> Option<ReferenceAccess> {
    match def {
        Definition::Local(_) | Definition::Field(_) => {}
        _ => return None,
    };

    let stmt = find_node_at_offset::<ast::LetStmt>(syntax, range.start())?;
    if stmt.initializer().is_some() {
        let pat = stmt.pat()?;
        if let ast::Pat::IdentPat(it) = pat {
            if it.mut_token().is_some() {
                return Some(ReferenceAccess::Write);
            }
        }
    }

    None
}

fn get_struct_def_name_for_struct_literal_search(
    sema: &Semantics<RootDatabase>,
    syntax: &SyntaxNode,
    position: FilePosition,
) -> Option<ast::Name> {
    if let TokenAtOffset::Between(ref left, ref right) = syntax.token_at_offset(position.offset) {
        if right.kind() != SyntaxKind::L_CURLY && right.kind() != SyntaxKind::L_PAREN {
            return None;
        }
        if let Some(name) =
            sema.find_node_at_offset_with_descend::<ast::Name>(&syntax, left.text_range().start())
        {
            return name.syntax().ancestors().find_map(ast::Struct::cast).and_then(|l| l.name());
        }
        if sema
            .find_node_at_offset_with_descend::<ast::GenericParamList>(
                &syntax,
                left.text_range().start(),
            )
            .is_some()
        {
            return left.ancestors().find_map(ast::Struct::cast).and_then(|l| l.name());
        }
    }
    None
}

fn try_find_lifetime_references(
    syntax: &SyntaxNode,
    position: FilePosition,
) -> Option<RangeInfo<ReferenceSearchResult>> {
    let lifetime_token =
        syntax.token_at_offset(position.offset).find(|t| t.kind() == SyntaxKind::LIFETIME)?;
    let parent = lifetime_token.parent();

    match_ast! {
        match parent {
            ast::LifetimeArg(_it) => find_all_lifetime_references(position, lifetime_token, &parent),
            ast::SelfParam(_it) => find_all_lifetime_references(position, lifetime_token, &parent),
            ast::RefType(_it) => find_all_lifetime_references(position, lifetime_token, &parent),
            ast::LifetimeParam(_it) => find_all_lifetime_references(position, lifetime_token, &parent),// decl
            ast::WherePred(_it) => find_all_lifetime_references(position, lifetime_token, &parent),
            ast::TypeBound(_it) => find_all_lifetime_references(position, lifetime_token, &parent),
            ast::Label(_it) => find_all_label_references(position, lifetime_token, &parent), // decl
            ast::BreakExpr(_it) => find_all_label_references(position, lifetime_token, &parent),
            ast::ContinueExpr(_it) => find_all_label_references(position, lifetime_token, &parent),
            _ => None,
        }
    }
}

fn find_all_label_references(
    position: FilePosition,
    lifetime_token: SyntaxToken,
    syntax: &SyntaxNode,
) -> Option<RangeInfo<ReferenceSearchResult>> {
    let label_text = lifetime_token.text();
    let label = syntax.ancestors().find_map(|syn| {
        (match_ast! {
            match syn {
                ast::EffectExpr(it) => it.label(),
                ast::LoopExpr(it) => it.label(),
                ast::WhileExpr(it) => it.label(),
                ast::ForExpr(it) => it.label(),
                _ => None,
            }
        })
        .filter(|label| label.lifetime_token().as_ref().map(|lt| lt.text()) == Some(label_text))
    })?;
    let lt = label.lifetime_token()?;
    let declaration = Declaration {
        nav: NavigationTarget {
            file_id: position.file_id,
            full_range: lt.text_range(),
            focus_range: Some(lt.text_range()),
            name: label_text.clone(),
            kind: lt.kind(),
            container_name: None,
            description: None,
            docs: None,
        },
        kind: ReferenceKind::Label,
        access: None,
    };
    let label_parent = label.syntax().parent()?;
    let expr = match_ast! {
        match label_parent {
            ast::EffectExpr(it) => it.block_expr()?.syntax().clone(),
            ast::LoopExpr(it) => it.loop_body()?.syntax().clone(),
            ast::WhileExpr(it) => it.loop_body()?.syntax().clone(),
            ast::ForExpr(it) => it.loop_body()?.syntax().clone(),
            _ => return None,
        }
    };
    let references = expr
        .descendants()
        .filter_map(|syn| {
            match_ast! {
                match syn {
                    ast::BreakExpr(it) => it.lifetime_token().filter(|lt| lt.text() == label_text),
                    ast::ContinueExpr(it) => it.lifetime_token().filter(|lt| lt.text() == label_text),
                    _ => None,
                }
            }
        })
        .map(|token| Reference {
            file_range: FileRange { file_id: position.file_id, range: token.text_range() },
            kind: ReferenceKind::Label,
            access: None,
        })
        .collect();

    Some(RangeInfo::new(
        lifetime_token.text_range(),
        ReferenceSearchResult { declaration, references },
    ))
}

fn find_all_lifetime_references(
    position: FilePosition,
    lifetime_token: SyntaxToken,
    syntax: &SyntaxNode,
) -> Option<RangeInfo<ReferenceSearchResult>> {
    let lifetime_text = lifetime_token.text();
    // we need to look for something that holds a GenericParamList as this is a definition site for lifetimes
    let (lifetime_param, generic_param_list, where_clause) =
        syntax.ancestors().find_map(|syn| {
            let (gpl, where_clause) = match_ast! {
                match syn {
                    ast::Fn(it) => (it.generic_param_list()?, it.where_clause()),
                    ast::TypeAlias(it) => (it.generic_param_list()?, it.where_clause()),
                    ast::Struct(it) => (it.generic_param_list()?, it.where_clause()),
                    ast::Enum(it) => (it.generic_param_list()?, it.where_clause()),
                    ast::Union(it) => (it.generic_param_list()?, it.where_clause()),
                    ast::Trait(it) => (it.generic_param_list()?, it.where_clause()),
                    ast::Impl(it) => (it.generic_param_list()?, it.where_clause()),
                    ast::WherePred(it) => (it.generic_param_list()?, None),
                    ast::ForType(it) => (it.generic_param_list()?, None),
                    _ => return None,
                }
            };
            Some((
                gpl.lifetime_params().find(|tp| {
                    tp.lifetime_token().as_ref().map(|lt| lt.text()) == Some(lifetime_text)
                })?,
                gpl,
                where_clause,
            ))
        })?;
    let lt = lifetime_param.lifetime_token()?;
    let declaration = Declaration {
        nav: NavigationTarget {
            file_id: position.file_id,
            full_range: lt.text_range(),
            focus_range: Some(lt.text_range()),
            name: lifetime_text.clone(),
            kind: lt.kind(),
            container_name: None,
            description: None,
            docs: None,
        },
        kind: ReferenceKind::Label,
        access: None,
    };
    let gpl_parent = generic_param_list.syntax().parent()?;
    let mut references = Vec::new();

    // find references in the GenericParamList itself
    for param in generic_param_list.generic_params().filter(|gp| {
        !matches!(
            gp,
            ast::GenericParam::LifetimeParam(lp) if lp.lifetime_token().as_ref() == Some(&lt)
        )
    }) {
        find_lifetime_references_in(
            &mut references,
            lifetime_text,
            position.file_id,
            param.syntax(),
        );
    }
    // find references in the WhereClause if it exists
    if let Some(where_clause) = where_clause {
        for predicate in where_clause.predicates() {
            find_lifetime_references_in(
                &mut references,
                lifetime_text,
                position.file_id,
                predicate.syntax(),
            );
        }
    }
    // find references in the other inner nodes of whatever we are in
    find_lifetime_references(&mut references, lifetime_text, position.file_id, gpl_parent);

    Some(RangeInfo::new(
        lifetime_token.text_range(),
        ReferenceSearchResult { declaration, references },
    ))
}

fn find_lifetime_references(
    references: &mut Vec<Reference>,
    lifetime_text: &SmolStr,
    file_id: FileId,
    gpl_parent: SyntaxNode,
) {
    match_ast! {
        match gpl_parent {
            ast::Fn(it) => {
                if let Some(param_list) = it.param_list() {
                    find_lifetime_references_in(references, lifetime_text, file_id, param_list.syntax());
                }
                if let Some(ret_type) = it.ret_type() {
                    find_lifetime_references_in(references, lifetime_text, file_id, ret_type.syntax());
                }
                if let Some(body) = it.body() {
                    find_lifetime_references_in_fn(references, lifetime_text, file_id, &body);
                }
            },
            ast::TypeAlias(it) => {
                if let Some(type_bound_list) = it.type_bound_list() {
                    find_lifetime_references_in(references, lifetime_text, file_id, type_bound_list.syntax());
                }
                if let Some(ty) = it.ty() {
                    find_lifetime_references_in(references, lifetime_text, file_id, ty.syntax());
                }
            },
            ast::Struct(it) => if let Some(field_list) = it.field_list() {
                find_lifetime_references_in(references, lifetime_text, file_id, field_list.syntax());
            },
            ast::Enum(it) => if let Some(variant_list) = it.variant_list() {
                find_lifetime_references_in(references, lifetime_text, file_id, variant_list.syntax());
            },
            ast::Union(it) => if let Some(record_field_list) = it.record_field_list() {
                find_lifetime_references_in(references, lifetime_text, file_id, record_field_list.syntax());
            },
            ast::Trait(it) => {
                if let Some(type_bound_list) = it.type_bound_list() {
                    find_lifetime_references_in(references, lifetime_text, file_id, type_bound_list.syntax());
                }
                if let Some(assoc_item_list) = it.assoc_item_list() {
                    find_lifetime_references_in_assoc_list(references, lifetime_text, file_id, &assoc_item_list);
                }
            },
            ast::Impl(it) => {
                if let Some(trait_) = it.trait_() {
                    find_lifetime_references_in(references, lifetime_text, file_id, trait_.syntax());
                }
                if let Some(self_ty) = it.self_ty() {
                    find_lifetime_references_in(references, lifetime_text, file_id, self_ty.syntax());
                }
                if let Some(assoc_item_list) = it.assoc_item_list() {
                    find_lifetime_references_in_assoc_list(references, lifetime_text, file_id, &assoc_item_list);
                }
            },
            ast::WherePred(it) => if let Some(ty) = it.ty() {
                find_lifetime_references_in(references, lifetime_text, file_id, ty.syntax());
            },
            ast::ForType(it) => if let Some(ty) = it.ty() {
                find_lifetime_references_in(references, lifetime_text, file_id, ty.syntax());
            },
            _ => (),
        }
    };
}

fn find_lifetime_references_in(
    references: &mut Vec<Reference>,
    lifetime_text: &SmolStr,
    file_id: FileId,
    syntax: &SyntaxNode,
) {
    references.extend(syntax.descendants_with_tokens().filter_map(|ele| match ele {
        NodeOrToken::Token(token) if token.text() == lifetime_text => Some(Reference {
            file_range: FileRange { file_id, range: token.text_range() },
            kind: ReferenceKind::Label,
            access: None,
        }),
        _ => None,
    }));
}

fn find_lifetime_references_in_assoc_list(
    references: &mut Vec<Reference>,
    lifetime_text: &SmolStr,
    file_id: FileId,
    assoc_items: &ast::AssocItemList,
) {
    for assoc_item in assoc_items.assoc_items() {
        // assoc items can't shadow lifetime variables
        match assoc_item {
            ast::AssocItem::Fn(it) => {
                if let Some(it) = it.generic_param_list() {
                    find_lifetime_references_in(references, lifetime_text, file_id, it.syntax());
                }
                if let Some(it) = it.param_list() {
                    find_lifetime_references_in(references, lifetime_text, file_id, it.syntax());
                }
                if let Some(it) = it.ret_type() {
                    find_lifetime_references_in(references, lifetime_text, file_id, it.syntax());
                }
                if let Some(it) = it.where_clause() {
                    find_lifetime_references_in(references, lifetime_text, file_id, it.syntax());
                }
                if let Some(body) = it.body() {
                    find_lifetime_references_in_fn(references, lifetime_text, file_id, &body);
                }
            }
            _ => {
                find_lifetime_references_in(references, lifetime_text, file_id, assoc_item.syntax())
            }
        }
    }
}

fn find_lifetime_references_in_fn(
    references: &mut Vec<Reference>,
    lifetime_text: &SmolStr,
    file_id: FileId,
    body: &ast::BlockExpr,
) {
    // skip inner items inside this function as they may redeclare a lifetime with the same name
    // as the one we are looking for
    let items = [
        SyntaxKind::STRUCT,
        SyntaxKind::ENUM,
        SyntaxKind::FN,
        SyntaxKind::UNION,
        SyntaxKind::TYPE_ALIAS,
        SyntaxKind::IMPL,
        SyntaxKind::TRAIT,
    ];
    let mut skip_until = None;
    for event in body.syntax().preorder_with_tokens() {
        if let Some(kind) = skip_until {
            if matches!(
                event,
                WalkEvent::Leave(NodeOrToken::Node(node)) if node.kind() == kind
            ) {
                skip_until = None
            }
        } else {
            match event {
                WalkEvent::Enter(NodeOrToken::Node(node)) => {
                    if items.contains(&node.kind()) {
                        skip_until = Some(node.kind());
                    }
                }
                WalkEvent::Enter(NodeOrToken::Token(token))
                    if token.kind() == T![lifetime] && token.text() == lifetime_text =>
                {
                    references.push(Reference {
                        file_range: FileRange { file_id, range: token.text_range() },
                        kind: ReferenceKind::Label,
                        access: None,
                    });
                }
                _ => (),
            }
        }
    }
}

fn try_find_self_references(
    syntax: &SyntaxNode,
    position: FilePosition,
) -> Option<RangeInfo<ReferenceSearchResult>> {
    let self_token =
        syntax.token_at_offset(position.offset).find(|t| t.kind() == SyntaxKind::SELF_KW)?;
    let parent = self_token.parent();
    match_ast! {
        match parent {
            ast::SelfParam(it) => (),
            ast::PathSegment(segment) => {
                segment.self_token()?;
                let path = segment.parent_path();
                if path.qualifier().is_some() && !ast::PathExpr::can_cast(path.syntax().parent()?.kind()) {
                    return None;
                }
            },
            _ => return None,
        }
    };
    let function = parent.ancestors().find_map(ast::Fn::cast)?;
    let self_param = function.param_list()?.self_param()?;
    let param_self_token = self_param.self_token()?;

    let declaration = Declaration {
        nav: NavigationTarget {
            file_id: position.file_id,
            full_range: self_param.syntax().text_range(),
            focus_range: Some(param_self_token.text_range()),
            name: param_self_token.text().clone(),
            kind: param_self_token.kind(),
            container_name: None,
            description: None,
            docs: None,
        },
        kind: ReferenceKind::SelfKw,
        access: Some(if self_param.mut_token().is_some() {
            ReferenceAccess::Write
        } else {
            ReferenceAccess::Read
        }),
    };
    let references = function
        .body()
        .map(|body| {
            body.syntax()
                .descendants()
                .filter_map(ast::PathExpr::cast)
                .filter_map(|expr| {
                    let path = expr.path()?;
                    if path.qualifier().is_none() {
                        path.segment()?.self_token()
                    } else {
                        None
                    }
                })
                .map(|token| Reference {
                    file_range: FileRange { file_id: position.file_id, range: token.text_range() },
                    kind: ReferenceKind::SelfKw,
                    access: declaration.access, // FIXME: properly check access kind here instead of copying it from the declaration
                })
                .collect()
        })
        .unwrap_or_default();

    Some(RangeInfo::new(
        param_self_token.text_range(),
        ReferenceSearchResult { declaration, references },
    ))
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};
    use ide_db::base_db::FileId;
    use stdx::format_to;

    use crate::{fixture, SearchScope};

    #[test]
    fn test_struct_literal_after_space() {
        check(
            r#"
struct Foo <|>{
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
                Foo STRUCT FileId(0) 0..26 7..10 Other

                FileId(0) 101..104 StructLiteral
            "#]],
        );
    }

    #[test]
    fn test_struct_literal_before_space() {
        check(
            r#"
struct Foo<|> {}
    fn main() {
    let f: Foo;
    f = Foo {};
}
"#,
            expect![[r#"
                Foo STRUCT FileId(0) 0..13 7..10 Other

                FileId(0) 41..44 Other
                FileId(0) 54..57 StructLiteral
            "#]],
        );
    }

    #[test]
    fn test_struct_literal_with_generic_type() {
        check(
            r#"
struct Foo<T> <|>{}
    fn main() {
    let f: Foo::<i32>;
    f = Foo {};
}
"#,
            expect![[r#"
                Foo STRUCT FileId(0) 0..16 7..10 Other

                FileId(0) 64..67 StructLiteral
            "#]],
        );
    }

    #[test]
    fn test_struct_literal_for_tuple() {
        check(
            r#"
struct Foo<|>(i32);

fn main() {
    let f: Foo;
    f = Foo(1);
}
"#,
            expect![[r#"
                Foo STRUCT FileId(0) 0..16 7..10 Other

                FileId(0) 54..57 StructLiteral
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
    i = i<|> + j;

    {
        i = 0;
    }

    i = 5;
}"#,
            expect![[r#"
                i IDENT_PAT FileId(0) 24..25 Other Write

                FileId(0) 50..51 Other Write
                FileId(0) 54..55 Other Read
                FileId(0) 76..77 Other Write
                FileId(0) 94..95 Other Write
            "#]],
        );
    }

    #[test]
    fn search_filters_by_range() {
        check(
            r#"
fn foo() {
    let spam<|> = 92;
    spam + spam
}
fn bar() {
    let spam = 92;
    spam + spam
}
"#,
            expect![[r#"
                spam IDENT_PAT FileId(0) 19..23 Other

                FileId(0) 34..38 Other Read
                FileId(0) 41..45 Other Read
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_for_param_inside() {
        check(
            r#"
fn foo(i : u32) -> u32 { i<|> }
"#,
            expect![[r#"
                i IDENT_PAT FileId(0) 7..8 Other

                FileId(0) 25..26 Other Read
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_for_fn_param() {
        check(
            r#"
fn foo(i<|> : u32) -> u32 { i }
"#,
            expect![[r#"
                i IDENT_PAT FileId(0) 7..8 Other

                FileId(0) 25..26 Other Read
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_field_name() {
        check(
            r#"
//- /lib.rs
struct Foo {
    pub spam<|>: u32,
}

fn main(s: Foo) {
    let f = s.spam;
}
"#,
            expect![[r#"
                spam RECORD_FIELD FileId(0) 17..30 21..25 Other

                FileId(0) 67..71 Other Read
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_impl_item_name() {
        check(
            r#"
struct Foo;
impl Foo {
    fn f<|>(&self) {  }
}
"#,
            expect![[r#"
                f FN FileId(0) 27..43 30..31 Other

            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_enum_var_name() {
        check(
            r#"
enum Foo {
    A,
    B<|>,
    C,
}
"#,
            expect![[r#"
                B VARIANT FileId(0) 22..23 22..23 Other

            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_enum_var_field() {
        check(
            r#"
enum Foo {
    A,
    B { field<|>: u8 },
    C,
}
"#,
            expect![[r#"
                field RECORD_FIELD FileId(0) 26..35 26..31 Other

            "#]],
        );
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
    let i = foo::Foo<|> { n: 5 };
}
"#,
            expect![[r#"
                Foo STRUCT FileId(1) 17..51 28..31 Other

                FileId(0) 53..56 StructLiteral
                FileId(2) 79..82 StructLiteral
            "#]],
        );
    }

    // `mod foo;` is not in the results because `foo` is an `ast::Name`.
    // So, there are two references: the first one is a definition of the `foo` module,
    // which is the whole `foo.rs`, and the second one is in `use foo::Foo`.
    #[test]
    fn test_find_all_refs_decl_module() {
        check(
            r#"
//- /lib.rs
mod foo<|>;

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
                foo SOURCE_FILE FileId(1) 0..35 Other

                FileId(0) 14..17 Other
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
pub(super) struct Foo<|> {
    pub n: u32,
}
"#,
            expect![[r#"
                Foo STRUCT FileId(2) 0..41 18..21 Other

                FileId(1) 20..23 Other
                FileId(1) 47..50 StructLiteral
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_with_scope() {
        let code = r#"
            //- /lib.rs
            mod foo;
            mod bar;

            pub fn quux<|>() {}

            //- /foo.rs
            fn f() { super::quux(); }

            //- /bar.rs
            fn f() { super::quux(); }
        "#;

        check_with_scope(
            code,
            None,
            expect![[r#"
                quux FN FileId(0) 19..35 26..30 Other

                FileId(1) 16..20 StructLiteral
                FileId(2) 16..20 StructLiteral
            "#]],
        );

        check_with_scope(
            code,
            Some(SearchScope::single_file(FileId(2))),
            expect![[r#"
                quux FN FileId(0) 19..35 26..30 Other

                FileId(2) 16..20 StructLiteral
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_macro_def() {
        check(
            r#"
#[macro_export]
macro_rules! m1<|> { () => (()) }

fn foo() {
    m1();
    m1();
}
"#,
            expect![[r#"
                m1 MACRO_CALL FileId(0) 0..46 29..31 Other

                FileId(0) 63..65 StructLiteral
                FileId(0) 73..75 StructLiteral
            "#]],
        );
    }

    #[test]
    fn test_basic_highlight_read_write() {
        check(
            r#"
fn foo() {
    let mut i<|> = 0;
    i = i + 1;
}
"#,
            expect![[r#"
                i IDENT_PAT FileId(0) 23..24 Other Write

                FileId(0) 34..35 Other Write
                FileId(0) 38..39 Other Read
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
    s.f<|> = 0;
}
"#,
            expect![[r#"
                f RECORD_FIELD FileId(0) 15..21 15..16 Other

                FileId(0) 55..56 RecordFieldExprOrPat Read
                FileId(0) 68..69 Other Write
            "#]],
        );
    }

    #[test]
    fn test_basic_highlight_decl_no_write() {
        check(
            r#"
fn foo() {
    let i<|>;
    i = 1;
}
"#,
            expect![[r#"
                i IDENT_PAT FileId(0) 19..20 Other

                FileId(0) 26..27 Other Write
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
        pub fn new<|>() -> Foo { Foo }
    }
}

fn main() {
    let _f = foo::Foo::new();
}
"#,
            expect![[r#"
                new FN FileId(0) 54..81 61..64 Other

                FileId(0) 126..129 StructLiteral
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_nested_module() {
        check(
            r#"
//- /lib.rs
mod foo { mod bar; }

fn f<|>() {}

//- /foo/bar.rs
use crate::f;

fn g() { f(); }
"#,
            expect![[r#"
                f FN FileId(0) 22..31 25..26 Other

                FileId(1) 11..12 Other
                FileId(1) 24..25 StructLiteral
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_struct_pat() {
        check(
            r#"
struct S {
    field<|>: u8,
}

fn f(s: S) {
    match s {
        S { field } => {}
    }
}
"#,
            expect![[r#"
                field RECORD_FIELD FileId(0) 15..24 15..20 Other

                FileId(0) 68..73 FieldShorthandForField Read
            "#]],
        );
    }

    #[test]
    fn test_find_all_refs_enum_var_pat() {
        check(
            r#"
enum En {
    Variant {
        field<|>: u8,
    }
}

fn f(e: En) {
    match e {
        En::Variant { field } => {}
    }
}
"#,
            expect![[r#"
                field RECORD_FIELD FileId(0) 32..41 32..37 Other

                FileId(0) 102..107 FieldShorthandForField Read
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
            field<|>: u8,
        }
    }
}

fn f() -> m::En {
    m::En::Variant { field: 0 }
}
"#,
            expect![[r#"
                field RECORD_FIELD FileId(0) 56..65 56..61 Other

                FileId(0) 125..130 RecordFieldExprOrPat Read
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
        let x = self<|>.bar;
        if true {
            let _ = match () {
                () => self,
            };
        }
    }
}
"#,
            expect![[r#"
                self SELF_KW FileId(0) 47..51 47..51 SelfKw Read

                FileId(0) 71..75 SelfKw Read
                FileId(0) 152..156 SelfKw Read
            "#]],
        );
    }

    #[test]
    fn test_find_labels() {
        check(
            r#"
fn main() {
    'foo: loop {
        'bar: while true {
            continue 'foo;
        }
        break 'foo<|>;
    }
}
"#,
            expect![[r#"
                'foo LIFETIME FileId(0) 16..20 16..20 Label

                FileId(0) 77..81 Label
                FileId(0) 107..111 Label
            "#]],
        );
    }

    #[test]
    fn test_find_labels_decl() {
        check(
            r#"
fn main() {
    'foo<|>: loop {
        'bar: while true {
            continue 'foo;
        }
        break 'foo;
    }
}
"#,
            expect![[r#"
                'foo LIFETIME FileId(0) 16..20 16..20 Label

                FileId(0) 77..81 Label
                FileId(0) 107..111 Label
            "#]],
        );
    }

    #[test]
    fn test_find_lifetimes_function() {
        check(
            r#"
trait Foo<'a> {}
impl<'a> Foo<'a> for &'a () {}

fn foo<'a, 'b: 'a>(x: &'a<|> ()) -> &'a () where &'a (): Foo<'a>  {
    fn bar<'a>(_: &'a ()) {}
    x
}
"#,
            expect![[r#"
                'a LIFETIME FileId(0) 56..58 56..58 Label

                FileId(0) 64..66 Label
                FileId(0) 96..98 Label
                FileId(0) 107..109 Label
                FileId(0) 72..74 Label
                FileId(0) 83..85 Label
            "#]],
        );
    }

    #[test]
    fn test_find_type_alias() {
        check(
            r#"
type Foo<'a, T> where T: 'a<|> = &'a T;
"#,
            expect![[r#"
                'a LIFETIME FileId(0) 9..11 9..11 Label

                FileId(0) 25..27 Label
                FileId(0) 31..33 Label
            "#]],
        );
    }

    #[test]
    fn test_find_trait_impl() {
        check(
            r#"
trait Foo<'a> {
    fn foo() -> &'a ();
}

impl<'a> Foo<'a> for &'a () {
    fn foo() -> &'a<|> () {
        unimplemented!()
    }
}
"#,
            expect![[r#"
                'a LIFETIME FileId(0) 48..50 48..50 Label

                FileId(0) 56..58 Label
                FileId(0) 65..67 Label
                FileId(0) 90..92 Label
            "#]],
        );
    }

    fn check(ra_fixture: &str, expect: Expect) {
        check_with_scope(ra_fixture, None, expect)
    }

    fn check_with_scope(ra_fixture: &str, search_scope: Option<SearchScope>, expect: Expect) {
        let (analysis, pos) = fixture::position(ra_fixture);
        let refs = analysis.find_all_refs(pos, search_scope).unwrap().unwrap();

        let mut actual = String::new();
        {
            let decl = refs.declaration;
            format_to!(actual, "{} {:?}", decl.nav.debug_render(), decl.kind);
            if let Some(access) = decl.access {
                format_to!(actual, " {:?}", access)
            }
            actual += "\n\n";
        }

        for r in &refs.references {
            format_to!(actual, "{:?} {:?} {:?}", r.file_range.file_id, r.file_range.range, r.kind);
            if let Some(access) = r.access {
                format_to!(actual, " {:?}", access);
            }
            actual += "\n";
        }
        expect.assert_eq(&actual)
    }
}
