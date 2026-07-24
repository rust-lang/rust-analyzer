//! Rename infrastructure for rust-analyzer. It is used primarily for the
//! literal "rename" in the ide (look for tests there), but it is also available
//! as a general-purpose service. For example, it is used by the fix for the
//! "incorrect case" diagnostic.
//!
//! It leverages the [`crate::search`] functionality to find what needs to be
//! renamed. The actual renames are tricky -- field shorthands need special
//! attention, and, when renaming modules, you also want to rename files on the
//! file system.
//!
//! Another can of worms are macros:
//!
//! ```ignore
//! macro_rules! m { () => { fn f() {} } }
//! m!();
//! fn main() {
//!     f() // <- rename me
//! }
//! ```
//!
//! The correct behavior in such cases is probably to show a dialog to the user.
//! Our current behavior is ¯\_(ツ)_/¯.
use std::fmt::{self, Display};

use crate::{
    source_change::ChangeAnnotation,
    text_edit::{TextEdit, TextEditBuilder},
};
use base_db::AnchoredPathBuf;
use either::Either;
use hir::{
    FieldSource, FileRange, HasCrate, InFile, ModPath, ModuleSource, Name, PathKind, Semantics,
    sym,
};
use itertools::Itertools;
use rustc_hash::FxHashSet;
use span::{Edition, FileId, SyntaxContext};
use stdx::{TupleExt, never};
use syntax::{
    AstNode, SyntaxElement, SyntaxKind, SyntaxNode, T, TextRange, TextSize,
    ast::{self, HasAttrs, HasName, HasVisibility},
    syntax_editor::{Position, Removable, SyntaxEditor},
};

use crate::{
    RootDatabase,
    defs::Definition,
    helpers::mod_path_to_ast_with_factory,
    search::{FileReference, FileReferenceNode},
    source_change::{FileSystemEdit, SourceChange, SourceChangeBuilder},
    syntax_helpers::node_ext::expr_as_name_ref,
    traits::convert_to_def_in_trait,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenameConfig {
    pub show_conflicts: bool,
}

pub type Result<T, E = RenameError> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct RenameError(pub String);

impl fmt::Display for RenameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[macro_export]
macro_rules! _format_err {
    ($fmt:expr) => { RenameError(format!($fmt)) };
    ($fmt:expr, $($arg:tt)+) => { RenameError(format!($fmt, $($arg)+)) }
}
pub use _format_err as format_err;

#[macro_export]
macro_rules! _bail {
    ($($tokens:tt)*) => { return Err(format_err!($($tokens)*)) }
}
pub use _bail as bail;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RenameDefinition {
    Yes,
    No,
}

impl<'db> Definition<'db> {
    pub fn rename(
        &self,
        sema: &Semantics<'db, RootDatabase>,
        new_name: &str,
        rename_definition: RenameDefinition,
        config: &RenameConfig,
    ) -> Result<SourceChange> {
        // self.krate() returns None if
        // self is a built-in attr, built-in type or tool module.
        // it is not allowed for these defs to be renamed.
        // cases where self.krate() is None is handled below.
        let edition = if let Some(krate) = self.krate(sema.db) {
            // Can we not rename non-local items?
            // Then bail if non-local
            if !krate.origin(sema.db).is_local() {
                bail!("Cannot rename a non-local definition")
            }
            krate.edition(sema.db)
        } else {
            Edition::LATEST
        };

        match *self {
            Definition::Module(module) => rename_mod(sema, module, new_name),
            Definition::ToolModule(_) => {
                bail!("Cannot rename a tool module")
            }
            Definition::BuiltinType(_) => {
                bail!("Cannot rename builtin type")
            }
            Definition::BuiltinAttr(_) => {
                bail!("Cannot rename a builtin attr.")
            }
            Definition::SelfType(_) => bail!("Cannot rename `Self`"),
            Definition::Macro(mac) => rename_reference(
                sema,
                Definition::Macro(mac),
                new_name,
                rename_definition,
                edition,
                config,
            ),
            def => rename_reference(sema, def, new_name, rename_definition, edition, config),
        }
    }

    /// Textual range of the identifier which will change when renaming this
    /// `Definition`. Note that builtin types can't be
    /// renamed and extern crate names will report its range, though a rename will introduce
    /// an alias instead.
    pub fn range_for_rename(self, sema: &Semantics<'_, RootDatabase>) -> Option<FileRange> {
        let syn_ctx_is_root = |(range, ctx): (_, SyntaxContext)| ctx.is_root().then_some(range);
        let res = match self {
            Definition::Macro(mac) => {
                let src = sema.source(mac)?;
                let name = match &src.value {
                    Either::Left(it) => it.name()?,
                    Either::Right(it) => it.name()?,
                };
                src.with_value(name.syntax())
                    .original_file_range_opt(sema.db)
                    .and_then(syn_ctx_is_root)
            }
            Definition::Field(field) => {
                let src = sema.source(field)?;
                match &src.value {
                    FieldSource::Named(record_field) => {
                        let name = record_field.name()?;
                        src.with_value(name.syntax())
                            .original_file_range_opt(sema.db)
                            .and_then(syn_ctx_is_root)
                    }
                    FieldSource::Pos(_) => None,
                }
            }
            Definition::Crate(_) => None,
            Definition::Module(module) => {
                let src = module.declaration_source(sema.db)?;
                let name = src.value.name()?;
                src.with_value(name.syntax())
                    .original_file_range_opt(sema.db)
                    .and_then(syn_ctx_is_root)
            }
            Definition::Function(it) => name_range(it, sema).and_then(syn_ctx_is_root),
            Definition::Adt(adt) => match adt {
                hir::Adt::Struct(it) => name_range(it, sema).and_then(syn_ctx_is_root),
                hir::Adt::Union(it) => name_range(it, sema).and_then(syn_ctx_is_root),
                hir::Adt::Enum(it) => name_range(it, sema).and_then(syn_ctx_is_root),
            },
            Definition::EnumVariant(it) => name_range(it, sema).and_then(syn_ctx_is_root),
            Definition::Const(it) => name_range(it, sema).and_then(syn_ctx_is_root),
            Definition::Static(it) => name_range(it, sema).and_then(syn_ctx_is_root),
            Definition::Trait(it) => name_range(it, sema).and_then(syn_ctx_is_root),
            Definition::TypeAlias(it) => name_range(it, sema).and_then(syn_ctx_is_root),
            Definition::Local(it) => {
                name_range(it.primary_source(sema.db), sema).and_then(syn_ctx_is_root)
            }
            Definition::GenericParam(generic_param) => match generic_param {
                hir::GenericParam::LifetimeParam(lifetime_param) => {
                    let src = sema.source(lifetime_param)?;
                    src.with_value(src.value.lifetime()?.syntax())
                        .original_file_range_opt(sema.db)
                        .and_then(syn_ctx_is_root)
                }
                _ => {
                    let param = match generic_param {
                        hir::GenericParam::TypeParam(it) => it.merge(),
                        hir::GenericParam::ConstParam(it) => it.merge(),
                        hir::GenericParam::LifetimeParam(_) => return None,
                    };
                    let src = sema.source(param)?;
                    let name = match &src.value {
                        Either::Left(x) => x.name()?,
                        Either::Right(_) => return None,
                    };
                    src.with_value(name.syntax())
                        .original_file_range_opt(sema.db)
                        .and_then(syn_ctx_is_root)
                }
            },
            Definition::Label(label) => {
                let src = sema.source(label)?;
                let lifetime = src.value.lifetime()?;
                src.with_value(lifetime.syntax())
                    .original_file_range_opt(sema.db)
                    .and_then(syn_ctx_is_root)
            }
            Definition::ExternCrateDecl(it) => {
                let src = sema.source(it)?;
                if let Some(rename) = src.value.rename() {
                    let name = rename.name()?;
                    src.with_value(name.syntax())
                        .original_file_range_opt(sema.db)
                        .and_then(syn_ctx_is_root)
                } else {
                    let name = src.value.name_ref()?;
                    src.with_value(name.syntax())
                        .original_file_range_opt(sema.db)
                        .and_then(syn_ctx_is_root)
                }
            }
            Definition::InlineAsmOperand(it) => name_range(it, sema).and_then(syn_ctx_is_root),
            Definition::BuiltinType(_)
            | Definition::BuiltinLifetime(_)
            | Definition::BuiltinAttr(_)
            | Definition::SelfType(_)
            | Definition::ToolModule(_)
            | Definition::TupleField(_)
            | Definition::InlineAsmRegOrRegClass(_) => return None,
            // FIXME: This should be doable in theory
            Definition::DeriveHelper(_) => return None,
        };
        return res;

        fn name_range<D>(
            def: D,
            sema: &Semantics<'_, RootDatabase>,
        ) -> Option<(FileRange, SyntaxContext)>
        where
            D: hir::HasSource,
            D::Ast: ast::HasName,
        {
            let src = sema.source(def)?;
            let name = src.value.name()?;
            src.with_value(name.syntax()).original_file_range_opt(sema.db)
        }
    }
}

fn rename_mod(
    sema: &Semantics<'_, RootDatabase>,
    module: hir::Module,
    new_name: &str,
) -> Result<SourceChange> {
    let mut source_change = SourceChange::default();

    if module.is_crate_root(sema.db) {
        return Ok(source_change);
    }

    let InFile { file_id, value: def_source } = module.definition_source(sema.db);
    let edition = file_id.edition(sema.db);
    let (new_name, kind) = IdentifierKind::classify(edition, new_name)?;
    if kind != IdentifierKind::Ident {
        bail!(
            "Invalid name `{0}`: cannot rename module to {0}",
            new_name.display(sema.db, edition)
        );
    }
    if let ModuleSource::SourceFile(..) = def_source {
        let anchor = file_id.original_file(sema.db).file_id(sema.db);

        let is_mod_rs = module.is_mod_rs(sema.db);
        let has_detached_child = module.children(sema.db).any(|child| !child.is_inline(sema.db));

        // Module exists in a named file
        if !is_mod_rs {
            let path = format!("{}.rs", new_name.as_str());
            let dst = AnchoredPathBuf { anchor, path };
            source_change.push_file_system_edit(FileSystemEdit::MoveFile { src: anchor, dst })
        }

        // Rename the dir if:
        //  - Module source is in mod.rs
        //  - Module has submodules defined in separate files
        let dir_paths = match (is_mod_rs, has_detached_child, module.name(sema.db)) {
            // Go up one level since the anchor is inside the dir we're trying to rename
            (true, _, Some(mod_name)) => {
                Some((format!("../{}", mod_name.as_str()), format!("../{}", new_name.as_str())))
            }
            // The anchor is on the same level as target dir
            (false, true, Some(mod_name)) => {
                Some((mod_name.as_str().to_owned(), new_name.as_str().to_owned()))
            }
            _ => None,
        };

        if let Some((src, dst)) = dir_paths {
            let src = AnchoredPathBuf { anchor, path: src };
            let dst = AnchoredPathBuf { anchor, path: dst };
            source_change.push_file_system_edit(FileSystemEdit::MoveDir {
                src,
                src_id: anchor,
                dst,
            })
        }
    }

    if let Some(src) = module.declaration_source(sema.db) {
        let file_id = src.file_id.original_file(sema.db);
        match src.value.name() {
            Some(name) => {
                if let Some(file_range) = src
                    .with_value(name.syntax())
                    .original_file_range_opt(sema.db)
                    .map(TupleExt::head)
                {
                    let new_name = new_name.display(sema.db, edition).to_string();
                    source_change.insert_source_edit(
                        file_id.file_id(sema.db),
                        TextEdit::replace(file_range.range, new_name),
                    )
                };
            }
            _ => never!("Module source node is missing a name"),
        }
    }

    let def = Definition::Module(module);
    let usages = def.usages(sema).all();
    let ref_edits = usages.iter().map(|(file_id, references)| {
        let edition = file_id.edition(sema.db);
        (
            file_id.file_id(sema.db),
            source_edit_from_references(sema.db, references, def, &new_name, edition),
        )
    });
    source_change.extend(ref_edits);

    Ok(source_change)
}

/// Emits text edits only; the caller moves the file.
///
/// Unlike [`rename_mod`], which flips a single name token, a move changes the
/// module's whole path, so references are spliced against freshly computed
/// segments — HIR still describes the pre-move tree.
pub fn move_mod(
    sema: &Semantics<'_, RootDatabase>,
    module: hir::Module,
    new_parent: hir::Module,
    new_leaf: &str,
) -> Result<SourceChange> {
    let db = sema.db;

    if module.is_crate_root(db) {
        bail!("Cannot move the crate root");
    }

    let InFile { file_id: def_file, .. } = module.definition_source(db);
    let edition = def_file.edition(db);
    let (new_leaf_name, kind) = IdentifierKind::classify(edition, new_leaf)?;
    if kind != IdentifierKind::Ident {
        bail!("Invalid name `{new_leaf}`: cannot name a module this way");
    }

    let old_segs: Vec<Name> = module.path_segments(db).collect();
    let mut new_segs: Vec<Name> = new_parent.path_segments(db).collect();
    new_segs.push(new_leaf_name.clone());

    let decl = module
        .declaration_source(db)
        .ok_or_else(|| format_err!("module has no `mod …;` declaration"))?;
    let decl_file = decl.file_id.original_file(db).file_id(db);
    let new_parent_def = new_parent.definition_source(db);
    let new_parent_file = new_parent_def.file_id.original_file(db).file_id(db);
    if matches!(new_parent_def.value, ModuleSource::BlockExpr(_)) {
        bail!("Cannot move a module into a block");
    }

    let mut builder = SourceChangeBuilder::new(decl_file);

    let relocated = relocated_declaration(&decl.value, &new_leaf_name, db, edition);

    let decl_editor = builder.make_editor(decl.value.syntax());
    decl.value.remove(&decl_editor);
    builder.add_file_edits(decl_file, decl_editor);

    let parent_node = match &new_parent_def.value {
        ModuleSource::SourceFile(sf) => sf.syntax().clone(),
        ModuleSource::Module(m) => m
            .item_list()
            .ok_or_else(|| format_err!("new parent module has no item list"))?
            .syntax()
            .clone(),
        ModuleSource::BlockExpr(_) => unreachable!("rejected above"),
    };
    let parent_editor = builder.make_editor(&parent_node);
    parent_editor.insert_all(
        Position::last_child_of(&parent_node),
        vec![relocated.syntax().clone().into(), parent_editor.make().whitespace("\n").into()],
    );
    builder.add_file_edits(new_parent_file, parent_editor);

    let def = Definition::Module(module);
    let moved_crate = module.krate(db);
    let usages = def.usages(sema).all();
    for (file_id, references) in usages.iter() {
        let edition = file_id.edition(db);
        let prefix = crate_root_prefix_for(sema, file_id.file_id(db), moved_crate, db);
        let source_file = sema.parse(file_id);
        let editor = builder.make_editor(source_file.syntax());
        rewrite_references_for_move(references, &new_segs, prefix.as_ref(), &editor, db, edition);
        builder.add_file_edits(file_id.file_id(db), editor);
    }

    // Disjoint from the rewrites above, which target references *to* the moved
    // module — something a `super::` written inside it can never be.
    let def_src = module.definition_source(db);
    let moved_file_id = def_src.file_id.original_file(db).file_id(db);
    let moved_node = def_src.value.node();
    let super_editor = builder.make_editor(&moved_node);
    reanchor_super_in_moved_file(&moved_node, &old_segs, &super_editor, edition);
    builder.add_file_edits(moved_file_id, super_editor);

    Ok(builder.finish())
}

/// Cloning the node, rather than re-printing `mod <leaf>;`, keeps visibility,
/// attributes and doc comments.
fn relocated_declaration(
    decl: &ast::Module,
    new_leaf: &Name,
    db: &RootDatabase,
    edition: Edition,
) -> ast::Module {
    let (editor, decl) = SyntaxEditor::with_ast_node(decl);
    if let Some(name) = decl.name() {
        let renamed = editor.make().name(&new_leaf.display(db, edition).to_string());
        editor.replace(name.syntax(), renamed.syntax());
    }
    ast::Module::cast(editor.finish().new_root().clone()).unwrap()
}

/// `None` for a same-crate reference, else the extern crate name `ref_file`'s
/// crate depends on it under (which honours `package = "…"` renames).
fn crate_root_prefix_for(
    sema: &Semantics<'_, RootDatabase>,
    ref_file: FileId,
    moved_crate: hir::Crate,
    db: &RootDatabase,
) -> Option<Name> {
    let ref_crate = sema.file_to_module_def(ref_file)?.krate(db);
    if ref_crate == moved_crate {
        return None;
    }
    ref_crate.dependencies(db).into_iter().find(|dep| dep.krate == moved_crate).map(|dep| dep.name)
}

/// As spelled from a referencing file: `crate::…` inside the same crate, else
/// `<extern crate>::…`.
fn abs_mod_path(new_segs: &[Name], extern_prefix: Option<&Name>) -> ModPath {
    match extern_prefix {
        None => ModPath::from_segments(PathKind::Crate, new_segs.iter().cloned()),
        Some(krate) => ModPath::from_segments(
            PathKind::Plain,
            std::iter::once(krate.clone()).chain(new_segs.iter().cloned()),
        ),
    }
}

/// By path prefix rather than name token: a qualified reference has its leading
/// path replaced wholesale, a bare one only its leaf, and only if it changed.
fn rewrite_references_for_move(
    references: &[FileReference],
    new_segs: &[Name],
    extern_prefix: Option<&Name>,
    editor: &SyntaxEditor,
    db: &RootDatabase,
    edition: Edition,
) {
    let make = editor.make();
    // A `ModPath`, not a node: a node cannot be inserted into the tree twice, so
    // every splice site renders its own.
    let new_abs = abs_mod_path(new_segs, extern_prefix);
    // `new_segs` is a path to the moved module, so it always ends in that module's own
    // name. Defaulting to "" here would silently splice in a malformed path instead.
    let new_leaf = new_segs
        .last()
        .expect("path to the moved module cannot be empty")
        .display(db, edition)
        .to_string();

    let mut edited_ranges: Vec<TextSize> = Vec::new();
    // Bucketed by enclosing `use`: rewriting `use a::{m::X, m::Y}` member-by-member
    // would split out the first and leave the second pointing at a vanished `a::m`.
    let mut use_groups: Vec<(ast::Use, Vec<(ast::UseTree, ast::Path)>)> = Vec::new();
    for FileReference { range, name, .. } in references {
        if edited_ranges.contains(&range.start()) {
            continue;
        }
        let Some(name_ref) = name.as_name_ref() else { continue };
        // A macro-expanded reference has a differing node/range — can't splice safely.
        if name_ref.syntax().text_range() != *range {
            continue;
        }
        let Some(segment) = name_ref.syntax().parent().and_then(ast::PathSegment::cast) else {
            continue;
        };
        // Keyword-anchored references are relative and only reachable from inside
        // the moved subtree, which travels along — rewriting them would corrupt.
        if matches!(
            segment.kind(),
            Some(
                ast::PathSegmentKind::SuperKw
                    | ast::PathSegmentKind::SelfKw
                    | ast::PathSegmentKind::CrateKw
                    | ast::PathSegmentKind::SelfTypeKw
            )
        ) {
            continue;
        }
        let Some(path) = segment.syntax().parent().and_then(ast::Path::cast) else { continue };
        // In `use a::{m, n}` the member's real prefix lives on an outer `UseTree`,
        // so it can't be rewritten in place — defer to the whole-group pass below.
        let in_use_group =
            name_ref.syntax().ancestors().any(|it| ast::UseTreeList::can_cast(it.kind()));
        if in_use_group {
            if let Some(member) = name_ref.syntax().ancestors().find_map(ast::UseTree::cast)
                && let Some(use_item) = member.syntax().ancestors().find_map(ast::Use::cast)
            {
                let ur = use_item.syntax().text_range();
                match use_groups.iter_mut().find(|(u, _)| u.syntax().text_range() == ur) {
                    Some((_, members)) => members.push((member, path)),
                    None => use_groups.push((use_item, vec![(member, path)])),
                }
                edited_ranges.push(range.start());
            }
            continue;
        }
        if path.qualifier().is_some() {
            let new_path = mod_path_to_ast_with_factory(make, &new_abs, edition);
            editor.replace(path.syntax(), new_path.syntax());
            edited_ranges.push(range.start());
        } else if name_ref.text().as_str() != new_leaf {
            editor.replace(name_ref.syntax(), make.name_ref(&new_leaf).syntax());
            edited_ranges.push(range.start());
        }
    }
    for (use_item, members) in &use_groups {
        rewrite_use_group_for_move(use_item, members, &new_abs, editor, edition);
    }
}

/// Each member importing through the moved module becomes its own
/// `use <new_abs><tail>;`, with the tail copied verbatim.
fn rewrite_use_group_for_move(
    use_item: &ast::Use,
    members: &[(ast::UseTree, ast::Path)],
    new_abs: &ModPath,
    editor: &SyntaxEditor,
    edition: Edition,
) {
    let make = editor.make();

    let mut extracted: Vec<(TextSize, ast::Use)> = members
        .iter()
        .map(|(member, path_to_module)| {
            let mut path = mod_path_to_ast_with_factory(make, new_abs, edition);
            for segment in segments_after(member.path().as_ref(), path_to_module) {
                path = make.path_qualified(path, segment);
            }
            let tree = make.use_tree(
                path,
                member.use_tree_list(),
                member.rename(),
                member.star_token().is_some(),
            );
            let new_use = make.use_(use_item.attrs(), use_item.visibility(), tree);
            (member.syntax().text_range().start(), new_use)
        })
        .collect();
    extracted.sort_by_key(|(pos, _)| *pos);

    let mut elements: Vec<SyntaxElement> = Vec::new();
    for (i, (_, new_use)) in extracted.iter().enumerate() {
        if i > 0 {
            elements.push(make.whitespace("\n").into());
        }
        elements.push(new_use.syntax().clone().into());
    }

    // An all-moved list would be left as an empty `{}`, so swap the whole `use` out.
    let emptied = use_item.use_tree().and_then(|it| it.use_tree_list()).is_some_and(|top| {
        let moved_from_top = members
            .iter()
            .filter(|(member, _)| {
                member.syntax().parent().and_then(ast::UseTreeList::cast).as_ref() == Some(&top)
            })
            .count();
        moved_from_top == top.use_trees().count()
    });
    if emptied {
        editor.replace_with_many(use_item.syntax(), elements);
        return;
    }

    for (member, _) in members {
        member.remove(editor);
    }
    editor.insert_all_with_whitespace(Position::before(use_item.syntax()), elements);
}

/// Segments of `path` that follow `prefix`, in source order.
fn segments_after(path: Option<&ast::Path>, prefix: &ast::Path) -> Vec<ast::PathSegment> {
    let mut segments = Vec::new();
    let mut current = path.cloned();
    while let Some(path) = current {
        if path.syntax().text_range() == prefix.syntax().text_range() {
            break;
        }
        if let Some(segment) = path.segment() {
            segments.push(segment);
        }
        current = path.qualifier();
    }
    segments.reverse();
    segments
}

/// A run of `k` supers denotes `old_segs` minus its last `k` segments. Only the
/// leading run is rewritten, so the splice stops before the `{` of a grouped
/// `use super::{a, b}` and cannot corrupt it.
fn reanchor_super_in_moved_file(
    moved_node: &SyntaxNode,
    old_segs: &[Name],
    editor: &SyntaxEditor,
    edition: Edition,
) {
    for path in moved_node.descendants().filter_map(ast::Path::cast) {
        if path.segment().and_then(|s| s.kind()) != Some(ast::PathSegmentKind::SuperKw) {
            continue;
        }
        // Handle each run once, at its top: a `super` parent means a longer run.
        if let Some(parent) = path.syntax().parent().and_then(ast::Path::cast)
            && parent.segment().and_then(|s| s.kind()) == Some(ast::PathSegmentKind::SuperKw)
        {
            continue;
        }
        // Inside an inline module `super` is relative to it, not to the file.
        if path.syntax().ancestors().any(|n| ast::Module::can_cast(n.kind())) {
            continue;
        }
        // `pub(super)` has no `in`, so it cannot take a `crate::…` path.
        if path.syntax().ancestors().any(|n| ast::Visibility::can_cast(n.kind())) {
            continue;
        }
        let mut k = 0usize;
        let mut cur = Some(path.clone());
        while let Some(p) = cur {
            if p.segment().and_then(|s| s.kind()) == Some(ast::PathSegmentKind::SuperKw) {
                k += 1;
                cur = p.qualifier();
            } else {
                cur = None;
            }
        }
        if k == 0 || k > old_segs.len() {
            continue;
        }
        let modprefix = &old_segs[..old_segs.len() - k];
        let replacement =
            mod_path_to_ast_with_factory(editor.make(), &abs_mod_path(modprefix, None), edition);
        editor.replace(path.syntax(), replacement.syntax());
    }
}

fn rename_reference<'db>(
    sema: &Semantics<'db, RootDatabase>,
    def: Definition<'db>,
    new_name: &str,
    rename_definition: RenameDefinition,
    edition: Edition,
    config: &RenameConfig,
) -> Result<SourceChange> {
    let (mut new_name, ident_kind) = IdentifierKind::classify(edition, new_name)?;

    if matches!(
        def,
        Definition::GenericParam(hir::GenericParam::LifetimeParam(_)) | Definition::Label(_)
    ) {
        match ident_kind {
            IdentifierKind::Underscore => {
                bail!(
                    "Invalid name `{}`: not a lifetime identifier",
                    new_name.display(sema.db, edition)
                );
            }
            IdentifierKind::Ident => {
                new_name = Name::new_lifetime(&format!("'{}", new_name.as_str()))
            }
            IdentifierKind::Lifetime => (),
            IdentifierKind::LowercaseSelf => bail!(
                "Invalid name `{}`: not a lifetime identifier",
                new_name.display(sema.db, edition)
            ),
        }
    } else {
        match ident_kind {
            IdentifierKind::Lifetime => {
                cov_mark::hit!(rename_not_an_ident_ref);
                bail!("Invalid name `{}`: not an identifier", new_name.display(sema.db, edition));
            }
            IdentifierKind::Ident => cov_mark::hit!(rename_non_local),
            IdentifierKind::Underscore => (),
            IdentifierKind::LowercaseSelf => {
                bail!(
                    "Invalid name `{}`: cannot rename to `self`",
                    new_name.display(sema.db, edition)
                );
            }
        }
    }

    let def = convert_to_def_in_trait(sema.db, def);
    let usages = def.usages(sema).all();

    if !usages.is_empty() && ident_kind == IdentifierKind::Underscore {
        cov_mark::hit!(rename_underscore_multiple);
        bail!("Cannot rename reference to `_` as it is being referenced multiple times");
    }
    let mut source_change = SourceChange::default();
    source_change.extend(usages.iter().map(|(file_id, references)| {
        let edition = file_id.edition(sema.db);
        (
            file_id.file_id(sema.db),
            source_edit_from_references(sema.db, references, def, &new_name, edition),
        )
    }));

    if let Definition::Field(field) = def {
        rename_field_constructors(sema, field, &new_name, &mut source_change, config);
    }

    if rename_definition == RenameDefinition::Yes {
        // This needs to come after the references edits, because we change the annotation of existing edits
        // if a conflict is detected.
        let (file_id, edit) =
            source_edit_from_def(sema, config, def, &new_name, &mut source_change)?;
        source_change.insert_source_edit(file_id, edit);
    }
    Ok(source_change)
}

fn rename_field_constructors(
    sema: &Semantics<'_, RootDatabase>,
    field: hir::Field,
    new_name: &Name,
    source_change: &mut SourceChange,
    config: &RenameConfig,
) {
    let db = sema.db;
    let old_name = field.name(db);
    let adt = field.parent_def(db).adt(db);
    adt.ty(db).iterate_assoc_items(db, |assoc_item| {
        let ctor = assoc_item.as_function()?;
        if ctor.has_self_param(db) {
            return None;
        }
        if ctor.ret_type(db).as_adt() != Some(adt) {
            return None;
        }

        let source = sema.source(ctor);
        let return_values = sema
            .fn_return_points(ctor)
            .into_iter()
            .filter_map(|ret| ret.value.expr())
            .chain(source.and_then(|source| source.value.body()?.tail_expr()));
        // FIXME: We could maybe skip ifs etc..

        let get_renamed_field = |mut expr| {
            while let ast::Expr::ParenExpr(e) = &expr {
                expr = e.expr()?;
            }
            let ast::Expr::RecordExpr(expr) = expr else { return None };
            if sema.type_of_expr(&expr.clone().into())?.original.as_adt()? != adt {
                return None;
            };
            expr.record_expr_field_list()?.fields().find_map(|record_field| {
                if record_field.name_ref().is_none()
                    && Name::new_root(&record_field.field_name()?.text()) == old_name
                    && let ast::Expr::PathExpr(field_name) = record_field.expr()?
                {
                    field_name.path()
                } else {
                    None
                }
            })
        };
        let renamed_fields = return_values
            .map(get_renamed_field)
            .map(|renamed_field| {
                let renamed_field = renamed_field?;
                let hir::PathResolution::Local(local) = sema.resolve_path(&renamed_field)? else {
                    return None;
                };
                let range = sema.original_range_opt(renamed_field.syntax())?.range;
                Some((range, local))
            })
            .collect::<Option<Vec<_>>>()?;

        let edition = ctor.krate(db).edition(db);
        let locals = renamed_fields.iter().map(|&(_, local)| local).collect::<FxHashSet<_>>();
        let mut all_locals_source_change = SourceChange::default();
        for local in locals {
            let mut local_source_change = Definition::Local(local)
                .rename(sema, new_name.as_str(), RenameDefinition::Yes, config)
                .ok()?;

            let (edit, _snippet) =
                local_source_change.source_file_edits.values_mut().exactly_one().ok()?;

            // The struct literal will have an edit `old_name -> old_name: new_name`, and we need to remove
            // that, as we want an overlapping edit `old_name -> new_name`.
            for &(field_range, _) in &renamed_fields {
                edit.cancel_edits_touching(field_range);
            }

            all_locals_source_change =
                std::mem::take(&mut all_locals_source_change).merge(local_source_change);
        }
        let (edit, _snippet) =
            all_locals_source_change.source_file_edits.values_mut().exactly_one().ok()?;
        for &(field_range, _) in &renamed_fields {
            edit.union(TextEdit::replace(field_range, new_name.display(db, edition).to_string()))
                .unwrap();
        }

        let file_id = *all_locals_source_change.source_file_edits.keys().exactly_one().ok()?;
        if let Some((edit, _snippet)) = source_change.source_file_edits.get_mut(&file_id) {
            for &(field_range, _) in &renamed_fields {
                edit.cancel_edits_touching(field_range);
            }
        }

        *source_change = std::mem::take(source_change).merge(all_locals_source_change);

        None::<std::convert::Infallible>
    });
}

pub fn source_edit_from_references(
    db: &RootDatabase,
    references: &[FileReference],
    def: Definition<'_>,
    new_name: &Name,
    edition: Edition,
) -> TextEdit {
    let name_display = new_name.display(db, edition);
    let mut edit = TextEdit::builder();
    // macros can cause multiple refs to occur for the same text range, so keep track of what we have edited so far
    let mut edited_ranges = Vec::new();
    for &FileReference { range, ref name, .. } in references {
        let name_range = name.text_range();
        let has_emitted_edit = match name {
            // if the ranges differ then the node is inside a macro call, we can't really attempt
            // to make special rewrites like shorthand syntax and such, so just rename the node in
            // the macro input
            FileReferenceNode::NameRef(name_ref) if name_range == range => {
                source_edit_from_name_ref(&mut edit, name_ref, &name_display, def)
            }
            FileReferenceNode::Name(name) if name_range == range => {
                source_edit_from_name(&mut edit, name, &name_display)
            }
            _ => false,
        };
        if !has_emitted_edit && !edited_ranges.contains(&range.start()) {
            edit.replace(range, name_display.to_string());
            edited_ranges.push(range.start());
        }
    }

    edit.finish()
}

fn source_edit_from_name(
    edit: &mut TextEditBuilder,
    name: &ast::Name,
    new_name: &dyn Display,
) -> bool {
    if ast::RecordPatField::for_field_name(name).is_some()
        && let Some(ident_pat) = name.syntax().parent().and_then(ast::IdentPat::cast)
    {
        cov_mark::hit!(rename_record_pat_field_name_split);
        // Foo { ref mut field } -> Foo { new_name: ref mut field }
        //      ^ insert `new_name: `

        // FIXME: instead of splitting the shorthand, recursively trigger a rename of the
        // other name https://github.com/rust-lang/rust-analyzer/issues/6547
        edit.insert(ident_pat.syntax().text_range().start(), format!("{new_name}: "));
        return true;
    }

    false
}

fn source_edit_from_name_ref(
    edit: &mut TextEditBuilder,
    name_ref: &ast::NameRef,
    new_name: &dyn Display,
    def: Definition<'_>,
) -> bool {
    if name_ref.super_token().is_some() {
        return true;
    }

    if let Some(record_field) = ast::RecordExprField::for_name_ref(name_ref) {
        let rcf_name_ref = record_field.name_ref();
        let rcf_expr = record_field.expr();
        match &(rcf_name_ref, rcf_expr.and_then(|it| expr_as_name_ref(&it))) {
            // field: init-expr, check if we can use a field init shorthand
            (Some(field_name), Some(init)) => {
                let new_name = new_name.to_string();
                if field_name == name_ref {
                    if init.text() == new_name {
                        cov_mark::hit!(test_rename_field_put_init_shorthand);
                        // Foo { field: local } -> Foo { local }
                        //       ^^^^^^^ delete this

                        // same names, we can use a shorthand here instead.
                        // we do not want to erase attributes hence this range start
                        let s = field_name.syntax().text_range().start();
                        let e = init.syntax().text_range().start();
                        edit.delete(TextRange::new(s, e));
                        return true;
                    }
                } else if init == name_ref && field_name.text() == new_name {
                    cov_mark::hit!(test_rename_local_put_init_shorthand);
                    // Foo { field: local } -> Foo { field }
                    //            ^^^^^^^ delete this

                    // same names, we can use a shorthand here instead.
                    // we do not want to erase attributes hence this range start
                    let s = field_name.syntax().text_range().end();
                    let e = init.syntax().text_range().end();
                    edit.delete(TextRange::new(s, e));
                    return true;
                }
            }
            // init shorthand
            (None, Some(_)) if matches!(def, Definition::Field(_)) => {
                cov_mark::hit!(test_rename_field_in_field_shorthand);
                // Foo { field } -> Foo { new_name: field }
                //       ^ insert `new_name: `
                let offset = name_ref.syntax().text_range().start();
                edit.insert(offset, format!("{new_name}: "));
                return true;
            }
            (None, Some(_)) if matches!(def, Definition::Local(_)) => {
                cov_mark::hit!(test_rename_local_in_field_shorthand);
                // Foo { field } -> Foo { field: new_name }
                //            ^ insert `: new_name`
                let offset = name_ref.syntax().text_range().end();
                edit.insert(offset, format!(": {new_name}"));
                return true;
            }
            _ => (),
        }
    } else if let Some(record_field) = ast::RecordPatField::for_field_name_ref(name_ref) {
        let rcf_name_ref = record_field.name_ref();
        let rcf_pat = record_field.pat();
        match (rcf_name_ref, rcf_pat) {
            // field: rename
            (Some(field_name), Some(ast::Pat::IdentPat(pat)))
                if field_name == *name_ref && pat.at_token().is_none() =>
            {
                // field name is being renamed
                if let Some(name) = pat.name() {
                    let new_name = new_name.to_string();
                    if name.text() == new_name {
                        cov_mark::hit!(test_rename_field_put_init_shorthand_pat);
                        // Foo { field: ref mut local } -> Foo { ref mut field }
                        //       ^^^^^^^ delete this
                        //                      ^^^^^ replace this with `field`

                        // same names, we can use a shorthand here instead/
                        // we do not want to erase attributes hence this range start
                        let s = field_name.syntax().text_range().start();
                        let e = pat.syntax().text_range().start();
                        edit.delete(TextRange::new(s, e));
                        edit.replace(name.syntax().text_range(), new_name);
                        return true;
                    }
                }
            }
            _ => (),
        }
    }
    false
}

fn source_edit_from_def<'db>(
    sema: &Semantics<'db, RootDatabase>,
    config: &RenameConfig,
    def: Definition<'db>,
    new_name: &Name,
    source_change: &mut SourceChange,
) -> Result<(FileId, TextEdit)> {
    let mut edit = TextEdit::builder();
    if let Definition::Local(local) = def {
        let mut file_id = None;

        let conflict_annotation =
            if config.show_conflicts && !sema.rename_conflicts(&local, new_name).is_empty() {
                Some(
                    source_change.insert_annotation(ChangeAnnotation {
                        label: "This rename will change the program's meaning".to_owned(),
                        needs_confirmation: true,
                        description: Some(
                            "Some variable(s) will shadow the renamed variable \
                        or be shadowed by it if the rename is performed"
                                .to_owned(),
                        ),
                    }),
                )
            } else {
                None
            };

        for source in local.sources(sema.db) {
            let source = match source.source.clone().original_ast_node_rooted(sema.db) {
                Some(source) => source,
                None => {
                    match source
                        .as_ident_pat()
                        .and_then(|x| x.name())
                        .and_then(|x| sema.original_range_opt(x.syntax()))
                        .or_else(|| {
                            source
                                .source
                                .syntax()
                                .original_file_range_opt(sema.db)
                                .map(TupleExt::head)
                        }) {
                        Some(FileRange { file_id: file_id2, range }) => {
                            file_id = Some(file_id2);
                            edit.replace(
                                range,
                                new_name.display(sema.db, file_id2.edition(sema.db)).to_string(),
                            );
                            continue;
                        }
                        None => {
                            bail!("Can't rename local that is defined in a macro declaration")
                        }
                    }
                }
            };
            file_id = Some(source.file_id);
            if let Either::Left(pat) = source.value {
                let name_range = pat.name().unwrap().syntax().text_range();

                // special cases required for renaming fields/locals in Record patterns
                if let Some(pat_field) = pat.syntax().parent().and_then(ast::RecordPatField::cast) {
                    if let Some(name_ref) = pat_field.name_ref() {
                        if new_name.as_str() == name_ref.text().as_str().trim_start_matches("r#")
                            && pat.at_token().is_none()
                        {
                            // Foo { field: ref mut local } -> Foo { ref mut field }
                            //       ^^^^^^ delete this
                            //                      ^^^^^ replace this with `field`
                            cov_mark::hit!(test_rename_local_put_init_shorthand_pat);
                            edit.delete(
                                name_ref
                                    .syntax()
                                    .text_range()
                                    .cover_offset(pat.syntax().text_range().start()),
                            );
                            edit.replace(name_range, name_ref.text().to_string());
                        } else {
                            // Foo { field: ref mut local @ local 2} -> Foo { field: ref mut new_name @ local2 }
                            // Foo { field: ref mut local } -> Foo { field: ref mut new_name }
                            //                      ^^^^^ replace this with `new_name`
                            edit.replace(
                                name_range,
                                new_name
                                    .display(sema.db, source.file_id.edition(sema.db))
                                    .to_string(),
                            );
                        }
                    } else {
                        // Foo { ref mut field } -> Foo { field: ref mut new_name }
                        //   original_ast_node_rootedd: `
                        //               ^^^^^ replace this with `new_name`
                        edit.insert(
                            pat.syntax().text_range().start(),
                            format!("{}: ", pat_field.field_name().unwrap()),
                        );
                        edit.replace(
                            name_range,
                            new_name.display(sema.db, source.file_id.edition(sema.db)).to_string(),
                        );
                    }
                } else {
                    edit.replace(
                        name_range,
                        new_name.display(sema.db, source.file_id.edition(sema.db)).to_string(),
                    );
                }
            }
        }
        let mut edit = edit.finish();

        for (edit, _) in source_change.source_file_edits.values_mut() {
            edit.set_annotation(conflict_annotation);
        }
        edit.set_annotation(conflict_annotation);

        let Some(file_id) = file_id else { bail!("No file available to rename") };
        return Ok((file_id.file_id(sema.db), edit));
    }
    let FileRange { file_id, range } = def
        .range_for_rename(sema)
        .ok_or_else(|| format_err!("No identifier available to rename"))?;
    let (range, new_name) = match def {
        Definition::ExternCrateDecl(decl) if decl.alias(sema.db).is_none() => (
            TextRange::empty(range.end()),
            format!(" as {}", new_name.display(sema.db, file_id.edition(sema.db)),),
        ),
        _ => (range, new_name.display(sema.db, file_id.edition(sema.db)).to_string()),
    };
    edit.replace(range, new_name);
    Ok((file_id.file_id(sema.db), edit.finish()))
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IdentifierKind {
    Ident,
    Lifetime,
    Underscore,
    LowercaseSelf,
}

impl IdentifierKind {
    pub fn classify(edition: Edition, new_name: &str) -> Result<(Name, IdentifierKind)> {
        match parser::LexedStr::single_token(edition, new_name) {
            Some(res) => match res {
                (SyntaxKind::IDENT, _) => Ok((Name::new_root(new_name), IdentifierKind::Ident)),
                (T![_], _) => {
                    Ok((Name::new_symbol_root(sym::underscore), IdentifierKind::Underscore))
                }
                (SyntaxKind::LIFETIME_IDENT, _) if new_name != "'static" && new_name != "'_" => {
                    Ok((Name::new_lifetime(new_name), IdentifierKind::Lifetime))
                }
                _ if SyntaxKind::from_keyword(new_name, edition).is_some() => match new_name {
                    "self" => Ok((Name::new_root(new_name), IdentifierKind::LowercaseSelf)),
                    "crate" | "super" | "Self" => {
                        bail!("Invalid name `{}`: cannot rename to a keyword", new_name)
                    }
                    _ => Ok((Name::new_root(new_name), IdentifierKind::Ident)),
                },
                (_, Some(syntax_error)) => bail!("Invalid name `{}`: {}", new_name, syntax_error),
                (_, None) => bail!("Invalid name `{}`: not an identifier", new_name),
            },
            None => bail!("Invalid name `{}`: not an identifier", new_name),
        }
    }
}
