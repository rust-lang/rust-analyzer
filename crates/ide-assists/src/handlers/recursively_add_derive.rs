use crate::{AssistContext, Assists};
use hir::{HasAttrs as _, HasCrate, HirFileId, ItemInNs};
use ide_db::{
    assists::{AssistId, AssistKind},
    helpers::mod_path_to_ast,
    imports::import_assets::NameToImport,
    items_locator, FxHashMap,
};
use itertools::Itertools;
use smallvec::SmallVec;
use syntax::{
    ast::{self, edit_in_place::AttrsOwnerEdit, make, HasAttrs},
    syntax_editor::Position,
    AstNode, T,
};

// Assist: recursively_add_derive
//
// Recursively add `#[derive(...)]` attributes from a struct or enum, to the type of each field.
// When the cursor is on a specific derive macro (such as `Copy`), only that derive will be added.
// If the cursor is instead on the `derive` keyword itself, all derives that the struct or enum has will be added.
//
// ```
// # //- minicore: derive, clone
// struct X(u32);
//
// struct Y(u32);
//
// #[derive(Cl$0one)]
// struct Point {
//     x: X,
//     y: Y,
// }
// ```
// ->
// ```
// #[derive(Clone)]
// struct X(u32);
//
// #[derive(Clone)]
// struct Y(u32);
//
// #[derive(Clone)]
// struct Point {
//     x: X,
//     y: Y,
// }
// ```
pub(crate) fn recursively_add_derive(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    // We're taking advantage of the fact that `#[derive(Tr$0ait)]` is expanded to `#[Trait]` when descending into the lowest `Attr` under the cursor.
    // This means that `path` will be `Trait` if the cursor is on that specific item, but `derive` if it isn't.
    let attr = ctx.find_node_at_offset_with_descend::<ast::Attr>()?;
    let path = attr.path()?;

    let adt_src = ctx.find_node_at_offset::<ast::Adt>()?;
    let adt = ctx.sema.to_def(&adt_src)?;
    let adt_ty = adt.ty(ctx.db());

    let current_module = ctx.sema.scope(adt_src.syntax())?.module();
    let current_crate = current_module.krate();
    let current_edition = current_crate.edition(ctx.db());

    let derive_items = if path.syntax().text() == "derive" {
        // The cursor is on the derive keyword, use all derive items the ADT has.
        let items: SmallVec<_> = adt
            .attrs(ctx.db())
            .by_key(&hir::sym::derive)
            .attrs()
            .filter_map(|attr| attr.parse_path_comma_token_tree(ctx.db()))
            .flatten()
            .filter_map(|(path, _)| {
                let macro_ = ctx
                    .sema
                    .resolve_mod_path(adt_src.syntax(), &path)?
                    .find_map(|item| item.as_macro())?;
                let name = path.segments().last()?;
                let name = name.as_str().to_owned();
                DeriveItem::from_path(ctx, &adt_ty, current_crate, macro_, name)
            })
            .collect();
        if items.is_empty() {
            return None;
        } else {
            items
        }
    } else {
        // Use the derive item under the cursor.
        let name = path.segments().last()?.to_string();
        let macro_ = ctx.sema.resolve_path(&path)?.as_module_def()?.as_macro()?;
        SmallVec::from([DeriveItem::from_path(ctx, &adt_ty, current_crate, macro_, name)?])
    };

    let to_derive = field_types_to_derive(ctx, &adt_src, &derive_items)?;
    if to_derive.iter().all(|(_, derives)| derives.is_empty()) {
        return None;
    }

    let formatted_items = derive_items
        .iter()
        .format_with(", ", |item, f| {
            let name = item.derive_macro.name(ctx.db());
            let name = name.display(ctx.db(), current_edition);
            f(&name)
        })
        .to_string();
    acc.add(
        AssistId("recursively_add_derive", AssistKind::Generate),
        format!("Recursively add `#[derive({formatted_items})]` to each field type"),
        adt_src.syntax().text_range(),
        |edit| {
            for (src, derives) in to_derive {
                let Some(file_id) = src.file_id.file_id() else { continue };
                let adt_src = src.value;
                let mut editor = edit.make_editor(adt_src.syntax());
                let Some(module) = ctx.sema.scope(adt_src.syntax()).map(|scope| scope.module())
                else {
                    continue;
                };

                // Create a comma-separate list containing paths to derive items.
                let derives_len = derives.len();
                let derive_paths = derives
                    .into_iter()
                    .filter_map(|item| {
                        module.find_path(ctx.db(), item, ctx.config.import_path_config())
                    })
                    .map(|path| mod_path_to_ast(&path, current_edition))
                    .enumerate()
                    .flat_map(|(i, path)| {
                        let add_comma_space = i != 0 && i < derives_len;
                        add_comma_space
                            .then(|| {
                                [make::token(T![,]).into(), make::tokens::single_space().into()]
                            })
                            .into_iter()
                            .flatten()
                            .chain(path.syntax().clone_for_update().descendants_with_tokens())
                    });

                let maybe_derive_attr = adt_src.attrs().find_map(|attr| {
                    if attr.path()?.syntax().text() == "derive" {
                        attr.token_tree()
                    } else {
                        None
                    }
                });

                if let Some(derive_attr) = maybe_derive_attr {
                    // Append to the first existing `#[derive]` attribute.
                    let pos = derive_attr
                        .right_delimiter_token()
                        .map(Position::before)
                        .unwrap_or_else(|| Position::after(derive_attr.syntax()));

                    // Check if there are no existing items, i.e. `#[derive()]`.
                    let is_empty = !derive_attr
                        .syntax()
                        .children_with_tokens()
                        .any(|x| x.kind() == syntax::SyntaxKind::IDENT);
                    if is_empty {
                        editor.insert_all(pos, derive_paths.collect());
                    } else {
                        let derive_paths =
                            [make::token(T![,]).into(), make::tokens::single_space().into()]
                                .into_iter()
                                .chain(derive_paths);
                        editor.insert_all(pos, derive_paths.collect());
                    }
                } else {
                    // Create a new `#[derive]` attribute.
                    let tt = derive_paths
                        .filter_map(|item| item.into_token())
                        .map(syntax::NodeOrToken::Token)
                        .collect();
                    let derive = make::attr_outer(make::meta_token_tree(
                        make::ext::ident_path("derive"),
                        make::token_tree(T!['('], tt).clone_for_update(),
                    ))
                    .clone_for_update();

                    // TODO: Switch to the `SyntaxEditor` equivalent of `add_attr()` once that's available.
                    let new_adt = adt_src.clone_for_update();
                    new_adt.add_attr(derive);
                    editor.replace(adt_src.syntax(), new_adt.syntax());
                }
                edit.add_file_edits(file_id, editor);
            }
        },
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct DeriveItem {
    derive_macro: hir::Macro,
    /// A guess at which trait the derive macro implements. This is used to check if a new derive needs to be added to the types of fields.
    /// If [`None`], the derive is inserted if the type does not already contain it. This means manual trait implementations are ignored.
    maybe_trait: Option<hir::Trait>,
}

impl DeriveItem {
    fn from_path(
        ctx: &AssistContext<'_>,
        ty: &hir::Type,
        current_crate: hir::Crate,
        derive_macro: hir::Macro,
        name: String,
    ) -> Option<Self> {
        if derive_macro.kind(ctx.db()) != hir::MacroKind::Derive {
            return None;
        }

        // Try to find a trait with the same name as the derive macro, which the type implements.
        let maybe_trait = items_locator::items_with_name(
            &ctx.sema,
            current_crate,
            NameToImport::exact_case_sensitive(name),
            items_locator::AssocSearchMode::Exclude,
        )
        .find_map(|item| item.as_module_def()?.as_trait())
        .filter(|trait_| ty.impls_trait(ctx.db(), *trait_, &[]));
        Some(DeriveItem { derive_macro, maybe_trait })
    }

    fn implemented_by(&self, ctx: &AssistContext<'_>, adt: &hir::Adt, src: &ast::Adt) -> bool {
        // Check if the type already has the derive attribute.
        adt.attrs(ctx.db())
            .by_key(&hir::sym::derive)
            .attrs()
            .filter_map(|attr| attr.parse_path_comma_token_tree(ctx.db()))
            .flatten()
            .filter_map(|(path, _)| ctx.sema.resolve_mod_path(src.syntax(), &path)?.find_map(|item| item.as_macro()))
            .any(|macro_| self.derive_macro == macro_)
            // If it doesn't, and there is a trait which the derive (likely) implements, check if the type has already implemented it.
            || matches!(self.maybe_trait, Some(trait_) if adt.ty(ctx.db()).impls_trait(ctx.db(), trait_, &[]))
    }
}

impl From<DeriveItem> for ItemInNs {
    fn from(value: DeriveItem) -> Self {
        value.derive_macro.into()
    }
}

fn field_list_adts<'a>(
    ctx: &'a AssistContext<'_>,
    list: ast::FieldList,
) -> impl Iterator<Item = hir::Adt> + 'a {
    // The Option indirection here is to return a single iterator type for both types of field lists.
    let mut record = None;
    let mut tuple = None;
    match list {
        ast::FieldList::RecordFieldList(list) => {
            record = Some(
                list.fields().filter_map(|field| ctx.sema.resolve_type(&field.ty()?)?.as_adt()),
            );
        }
        ast::FieldList::TupleFieldList(list) => {
            tuple = Some(
                list.fields().filter_map(|field| ctx.sema.resolve_type(&field.ty()?)?.as_adt()),
            );
        }
    }
    record.into_iter().flatten().chain(tuple.into_iter().flatten())
}

fn derive_targets(
    ctx: &AssistContext<'_>,
    derives: &[DeriveItem],
    adt: &hir::Adt,
) -> Option<(hir::InFileWrapper<HirFileId, ast::Adt>, Vec<DeriveItem>)> {
    if !adt.krate(ctx.db()).origin(ctx.db()).is_local() {
        // Only allow edits to crates that are members of the same workspace.
        return None;
    }

    let src = ctx.sema.source(*adt)?;
    let derives: Vec<_> = derives
        .iter()
        .filter(|derive| !derive.implemented_by(ctx, adt, &src.value))
        .cloned()
        .collect();

    if derives.is_empty() {
        None
    } else {
        Some((src, derives))
    }
}

fn field_types_to_derive(
    ctx: &AssistContext<'_>,
    ty: &ast::Adt,
    derives: &[DeriveItem],
) -> Option<FxHashMap<hir::InFileWrapper<HirFileId, ast::Adt>, Vec<DeriveItem>>> {
    let mut res: FxHashMap<_, _> = FxHashMap::default();
    let mut worklist: Vec<_> = match &ty {
        ast::Adt::Enum(enum_) => enum_
            .variant_list()?
            .variants()
            .filter_map(|variant| variant.field_list())
            .flat_map(|list| field_list_adts(ctx, list))
            .filter_map(|adt| Some((adt, derive_targets(ctx, derives, &adt)?)))
            .collect(),
        ast::Adt::Struct(struct_) => field_list_adts(ctx, struct_.field_list()?)
            .filter_map(|adt| Some((adt, derive_targets(ctx, derives, &adt)?)))
            .collect(),
        ast::Adt::Union(union_) => field_list_adts(ctx, union_.record_field_list()?.into())
            .filter_map(|adt| Some((adt, derive_targets(ctx, derives, &adt)?)))
            .collect(),
    };

    while let Some((adt, (src, derives))) = worklist.pop() {
        match &adt {
            hir::Adt::Enum(enum_) => {
                let fields = enum_.variants(ctx.db()).into_iter().flat_map(|variant| {
                    variant
                        .fields(ctx.db())
                        .into_iter()
                        .filter_map(|field| field.ty(ctx.db()).as_adt())
                        .filter_map(|adt| Some((adt, derive_targets(ctx, &derives, &adt)?)))
                        .filter(|(_, (field_src, _))| !res.contains_key(field_src))
                });
                worklist.extend(fields);
            }
            hir::Adt::Struct(struct_) => {
                let fields = struct_
                    .fields(ctx.db())
                    .into_iter()
                    .filter_map(|field| field.ty(ctx.db()).as_adt())
                    .filter_map(|adt| Some((adt, derive_targets(ctx, &derives, &adt)?)))
                    .filter(|(_, (field_src, _))| !res.contains_key(field_src));
                worklist.extend(fields);
            }
            hir::Adt::Union(union_) => {
                let fields = union_
                    .fields(ctx.db())
                    .into_iter()
                    .filter_map(|field| field.ty(ctx.db()).as_adt())
                    .filter_map(|adt| Some((adt, derive_targets(ctx, &derives, &adt)?)))
                    .filter(|(_, (field_src, _))| !res.contains_key(field_src));
                worklist.extend(fields);
            }
        }
        res.insert(src, derives);
    }
    Some(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{check_assist, check_assist_not_applicable};

    #[test]
    fn simple() {
        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                struct Foo(u32);

                #[derive(Cl$0one)]
                struct Bar(Foo)
            "#,
            r#"
                #[derive(Clone)]
                struct Foo(u32);

                #[derive(Clone)]
                struct Bar(Foo)
            "#,
        );
    }

    #[test]
    fn recursive() {
        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                struct Foo(u32);

                struct Bar(Foo)

                #[derive(Cl$0one)]
                struct Baz(Bar)
            "#,
            r#"
                #[derive(Clone)]
                struct Foo(u32);

                #[derive(Clone)]
                struct Bar(Foo)

                #[derive(Clone)]
                struct Baz(Bar)
            "#,
        );
    }

    #[test]
    fn append_to_existing_derive_attr() {
        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                #[derive(Debug)]
                struct Foo(u32);

                #[derive(Cl$0one)]
                struct Bar(Foo)
            "#,
            r#"
                #[derive(Debug, Clone)]
                struct Foo(u32);

                #[derive(Clone)]
                struct Bar(Foo)
            "#,
        );

        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                #[derive()]
                struct Foo(u32);

                #[derive(Cl$0one)]
                struct Bar(Foo)
            "#,
            r#"
                #[derive(Clone)]
                struct Foo(u32);

                #[derive(Clone)]
                struct Bar(Foo)
            "#,
        );

        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                #[derive(Debug)]
                #[derive(PartialEq)]
                struct Foo(u32);

                #[derive(Cl$0one)]
                struct Bar(Foo)
            "#,
            r#"
                #[derive(Debug, Clone)]
                #[derive(PartialEq)]
                struct Foo(u32);

                #[derive(Clone)]
                struct Bar(Foo)
            "#,
        );
    }

    #[test]
    fn trait_path_already_manually_implemented() {
        check_assist_not_applicable(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                struct Foo;

                impl Clone for Foo {
                    fn clone(&self) -> Self {
                        Self
                    }
                }

                #[derive(Cl$0one)]
                struct Bar(Foo)
            "#,
        );
    }

    #[test]
    fn trait_path_some_already_manually_implemented() {
        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                struct Foo;

                impl Clone for Foo {
                    fn clone(&self) -> Self {
                        Self
                    }
                }

                struct Bar;

                #[derive(Cl$0one)]
                struct Baz(Foo, Bar)
            "#,
            r#"
                struct Foo;

                impl Clone for Foo {
                    fn clone(&self) -> Self {
                        Self
                    }
                }

                #[derive(Clone)]
                struct Bar;

                #[derive(Clone)]
                struct Baz(Foo, Bar)
            "#,
        )
    }

    #[test]
    fn macro_path_already_manually_implemented() {
        check_assist_not_applicable(
            recursively_add_derive,
            r#"
                //- minicore: derive, hash
                struct Foo;

                impl core::hash::Hash for Foo {
                    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                        todo!()
                    }
                }

                #[derive(Ha$0sh)]
                struct Bar(Foo)
            "#,
        );
    }

    #[test]
    fn macro_path_some_already_manually_implemented() {
        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, hash
                struct Foo;

                impl core::hash::Hash for Foo {
                    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                        todo!()
                    }
                }

                struct Bar;

                #[derive(Ha$0sh)]
                struct Baz(Foo, Bar)
            "#,
            r#"
                struct Foo;

                impl core::hash::Hash for Foo {
                    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                        todo!()
                    }
                }

                #[derive(Hash)]
                struct Bar;

                #[derive(Hash)]
                struct Baz(Foo, Bar)
            "#,
        )
    }

    #[test]
    fn enum_fields() {
        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                struct Foo;

                struct Bar;

                #[derive(Cl$0one)]
                enum Baz {
                    Foo(Foo),
                    Bar { bar: Bar },
                }
            "#,
            r#"
                #[derive(Clone)]
                struct Foo;

                #[derive(Clone)]
                struct Bar;

                #[derive(Clone)]
                enum Baz {
                    Foo(Foo),
                    Bar { bar: Bar },
                }
            "#,
        );

        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                struct Foo;

                enum Bar {
                    Foo(Foo),
                }

                #[derive(Clo$0ne)]
                struct Baz {
                    bar: Bar,
                }
            "#,
            r#"
                #[derive(Clone)]
                struct Foo;

                #[derive(Clone)]
                enum Bar {
                    Foo(Foo),
                }

                #[derive(Clone)]
                struct Baz {
                    bar: Bar,
                }
            "#,
        );
    }

    #[test]
    fn proc_macro_derive_item() {
        // Note that only the derive macro Hash is in the prelude, not the trait itself.
        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, hash
                struct Foo;

                #[derive(Ha$0sh)]
                struct Bar(Foo)
            "#,
            r#"
                #[derive(Hash)]
                struct Foo;

                #[derive(Hash)]
                struct Bar(Foo)
            "#,
        );
    }

    #[test]
    fn reexported_path() {
        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                use core::clone::Clone as Clone2;

                #[derive(Clone2)]
                struct Foo;

                struct Bar;

                #[derive(Cl$0one)]
                struct Baz(Foo, Bar)
            "#,
            r#"
                use core::clone::Clone as Clone2;

                #[derive(Clone2)]
                struct Foo;

                #[derive(Clone2)]
                struct Bar;

                #[derive(Clone)]
                struct Baz(Foo, Bar)
            "#,
        );

        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                mod x {
                    use core::clone::Clone as Clone2;

                    #[derive(Clone2)]
                    struct Foo;
                }

                struct Bar;

                #[derive(Cl$0one)]
                struct Baz(x::Foo, Bar)
            "#,
            r#"
                mod x {
                    use core::clone::Clone as Clone2;

                    #[derive(Clone2)]
                    struct Foo;
                }

                #[derive(Clone)]
                struct Bar;

                #[derive(Clone)]
                struct Baz(x::Foo, Bar)
            "#,
        );

        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                mod x {
                    use core::clone::Clone as Clone2;

                    struct Foo;

                    impl Clone2 for Foo {
                        fn clone2(&self) -> Self {
                            Foo
                        }
                    }
                }

                struct Bar;

                #[derive(Cl$0one)]
                struct Baz(x::Foo, Bar)
            "#,
            r#"
                mod x {
                    use core::clone::Clone as Clone2;

                    struct Foo;

                    impl Clone2 for Foo {
                        fn clone2(&self) -> Self {
                            Foo
                        }
                    }
                }

                #[derive(Clone)]
                struct Bar;

                #[derive(Clone)]
                struct Baz(x::Foo, Bar)
            "#,
        );
    }

    #[test]
    fn multi_file() {
        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                //- /main.rs
                mod foo;
                mod bar;

                #[derive(Cl$0one)]
                struct Baz(foo::Foo, bar::Bar);

                //- /foo.rs
                pub struct Foo;

                //- /bar.rs
                pub struct Bar;
            "#,
            r#"
                //- /foo.rs
                #[derive(Clone)]
                pub struct Foo;

                //- /bar.rs
                #[derive(Clone)]
                pub struct Bar;
            "#,
        );

        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                //- /main.rs
                mod foo;

                #[derive(Cl$0one)]
                struct Baz(foo::Foo);

                //- /foo.rs
                mod bar;
                pub struct Foo(bar::Bar);

                //- /foo/bar.rs
                pub struct Bar(Baz);
                pub struct Baz;
            "#,
            r#"
                //- /foo.rs
                mod bar;
                #[derive(Clone)]
                pub struct Foo(bar::Bar);

                //- /foo/bar.rs
                #[derive(Clone)]
                pub struct Bar(Baz);
                #[derive(Clone)]
                pub struct Baz;
            "#,
        );
    }

    #[test]
    fn only_changes_current_workspace() {
        check_assist_not_applicable(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                //- /main.rs crate:a deps:foo
                #[derive(Cl$0one)]
                struct Bar(foo::Foo);

                //- /lib.rs library crate:foo
                pub struct Foo;
            "#,
        );

        check_assist_not_applicable(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                //- /main.rs crate:a deps:baz
                #[derive(Cl$0one)]
                struct Foo(Bar, baz::Baz);

                struct Bar;
                impl Clone for Bar {
                    fn clone(&self) -> Self {
                        Bar
                    }
                }

                //- /lib.rs library crate:baz
                pub struct Baz;
            "#,
        );

        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                //- /main.rs crate:a deps:foo,bar
                #[derive(Cl$0one)]
                struct Baz(foo::Foo, bar::Bar);
                //- /lib.rs crate:foo
                pub struct Foo;
                //- /lib.rs library crate:bar
                pub struct Bar;
            "#,
            r#"
                #[derive(Clone)]
                pub struct Foo;
            "#,
        );
    }

    #[test]
    fn derive_multiple() {
        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone, hash
                struct Bar;

                #[de$0rive(Clone, Hash)]
                struct Foo(Bar);
            "#,
            r#"
                #[derive(Clone, Hash)]
                struct Bar;

                #[derive(Clone, Hash)]
                struct Foo(Bar);
            "#,
        );
    }

    #[test]
    fn derive_multiple_recursive() {
        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone, hash
                struct Foo;

                struct Bar(Foo);

                impl core::hash::Hash for Bar {
                    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                        todo!()
                    }
                }

                struct Baz;

                #[de$0rive(Clone, Hash)]
                struct Taz(Bar, Baz);
            "#,
            r#"
                #[derive(Clone)]
                struct Foo;

                #[derive(Clone)]
                struct Bar(Foo);

                impl core::hash::Hash for Bar {
                    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                        todo!()
                    }
                }

                #[derive(Clone, Hash)]
                struct Baz;

                #[derive(Clone, Hash)]
                struct Taz(Bar, Baz);
            "#,
        );
    }

    #[test]
    fn derive_multiple_append_to_existing() {
        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone, hash
                #[derive(PartialEq)]
                struct Foo;

                #[de$0rive(Clone, Hash)]
                struct Bar(Foo);
            "#,
            r#"
                #[derive(PartialEq, Clone, Hash)]
                struct Foo;

                #[derive(Clone, Hash)]
                struct Bar(Foo);
            "#,
        );
    }

    #[test]
    fn absolute_path() {
        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, hash
                struct Foo;
                #[derive(core::ha$0sh::Hash)]
                struct Bar(Foo);
            "#,
            r#"
                #[derive(Hash)]
                struct Foo;
                #[derive(core::hash::Hash)]
                struct Bar(Foo);
            "#,
        );

        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                struct Foo;
                #[derive(core::clone::Clo$0ne)]
                struct Bar(Foo);
            "#,
            r#"
                #[derive(Clone)]
                struct Foo;
                #[derive(core::clone::Clone)]
                struct Bar(Foo);
            "#,
        );
    }

    #[test]
    fn multiple_fields_of_same_type() {
        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                struct Foo;
                struct Bar(Foo);
                #[derive(Clo$0ne)]
                struct Baz(Foo, Foo, Bar);
            "#,
            r#"
                #[derive(Clone)]
                struct Foo;
                #[derive(Clone)]
                struct Bar(Foo);
                #[derive(Clone)]
                struct Baz(Foo, Foo, Bar);
            "#,
        );
    }

    #[test]
    fn other_attr() {
        check_assist_not_applicable(
            recursively_add_derive,
            r#"
                struct Foo(u32);

                #[cf$0g]
                struct Bar(Foo);
            "#,
        );

        check_assist_not_applicable(
            recursively_add_derive,
            r#"
                struct Foo(u32);

                #[cf$0g()]
                struct Bar(Foo);
            "#,
        );

        check_assist_not_applicable(
            recursively_add_derive,
            r#"
                struct Foo(u32);

                #[c$0fg(Bar, Baz)]
                struct Bar(Foo);
            "#,
        );

        check_assist_not_applicable(
            recursively_add_derive,
            r#"
                struct Foo(u32);

                #[cfg(Ba$0r)]
                struct Bar(Foo);
            "#,
        );
    }

    #[test]
    fn derive_single_multiple_available() {
        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                struct Foo(u32);

                #[derive(Hash, Cl$0one, Debug)]
                struct Bar(Foo)
            "#,
            r#"
                #[derive(Clone)]
                struct Foo(u32);

                #[derive(Hash, Clone, Debug)]
                struct Bar(Foo)
            "#,
        );
    }

    #[test]
    fn new_derive_attrs_indent() {
        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                mod x {
                    struct Foo(u32);

                    #[derive(Cl$0one)]
                    struct Bar(Foo)
                }
            "#,
            r#"
                mod x {
                    #[derive(Clone)]
                    struct Foo(u32);

                    #[derive(Clone)]
                    struct Bar(Foo)
                }
            "#,
        );
    }

    #[test]
    fn union_type() {
        check_assist(
            recursively_add_derive,
            r#"
                //- minicore: derive, clone
                pub struct Foo(u32);

                #[derive(Cl$0one)]
                pub union Bar {
                    foo: Foo,
                    baz: u32,
                }
            "#,
            r#"
                #[derive(Clone)]
                pub struct Foo(u32);

                #[derive(Clone)]
                pub union Bar {
                    foo: Foo,
                    baz: u32,
                }
            "#,
        );
    }
}
