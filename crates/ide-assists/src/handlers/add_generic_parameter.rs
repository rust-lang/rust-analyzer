use ide_db::{
    base_db::{FileId, FileLoader, FilePosition},
    defs::{Definition, NameClass, NameRefClass},
    rename,
    source_change::SourceChangeBuilder,
    FxHashMap, FxHashSet, RootDatabase,
};
use syntax::{
    ast::{self, edit_in_place::GenericParamsOwnerEdit, HasName as _},
    match_ast,
    ted::{self, Position},
    AstNode, SmolStr, SyntaxKind, SyntaxNode,
};
use text_edit::{TextRange, TextSize};

use crate::{AssistContext, AssistId, AssistKind, Assists};

// Assist: add_generic_parameter
//
// Adds a new generic parameter virally to anything that accepts one, e.g. struct, enum, union, trait, impl.
//
// ```
// struct Channel$0(u8);
// struct Color([Channel; 3]);
// struct Image(Vec<Color>);
// ```
// ->
// ```
// struct Channel<T1>(u8);
// struct Color<T1>([Channel<T1>; 3]);
// struct Image<T1>(Vec<Color<T1>>);
// ```
pub(crate) fn add_generic_parameter(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let offset = ctx.offset();

    let sema = &ctx.sema;
    let file_id = ctx.file_id();
    let position = FilePosition { file_id, offset };
    let source_file = sema.parse(file_id);
    let syntax = source_file.syntax();

    let file_text = ctx.db().file_text(file_id);
    let file_text = file_text.as_ref();

    let definitions = match find_definitions(sema, syntax, position) {
        Err(_) => {
            return None;
        }
        Ok(x) => x,
    };
    let definitions = definitions.collect::<Vec<_>>();
    // FIXME: should this also check if the namelike is a ast::Name?
    // this would make it only appear when the cursor is on the definition of a name, rather than on uses too.
    let all_definitions_can_have_generic_params =
        definitions.iter().all(|(_, x)| as_definition_with_generic_params(*x).is_some());
    if !all_definitions_can_have_generic_params {
        return None;
    }
    let definitions = definitions.into_iter();
    // FIXME: generalize this to adding and removing
    // - const generic
    // - lifetime generic
    // - function value parameters
    acc.add(
        AssistId(stringify!(add_generic_parameter), AssistKind::Generate),
        "Add generic parameter",
        /*
        see ide_db::assists::Assist.target
        This should be as large as possible because adding a generic could modify an entire crate
        in a similar way that renaming does.

        should this be just the name's text_range()? maybe the definition's text_range()?
        */
        TextRange::up_to(TextSize::of(file_text)),
        |builder| {
            let snippet_cap = ctx.config.snippet_cap;

            let config = {
                use ast::make::*;
                // FIXME: allow user to specify the name somehow? Automatically pick a name?
                Configuration {
                    param: {
                        let x = type_param(name("T1"), None);
                        ast::GenericParam::from(x)
                    },
                    arg: {
                        let x = type_arg(ext::ty_name(name("T1")));
                        ast::GenericArg::from(x)
                    },
                }
            };
            assist_impl(builder, config, snippet_cap, sema, definitions)
        },
    )
}

fn assist_impl(
    builder: &mut SourceChangeBuilder,
    config: Configuration,
    snippet_cap: Option<ide_db::SnippetCap>,
    sema: &hir::Semantics<'_, RootDatabase>,
    definitions: impl Iterator<Item = (ast::NameLike, Definition)>,
) {
    let mut state = AssistState {
        builder: SourceChangeBuilder2 { inner: builder },
        config,
        snippet_cap,
        sema,
        aux_state: Default::default(),
    };
    let mut processing_queue = ProcessingQueue::default();
    let mut acc = ChangeAccumulator::default();
    // FIXME: possibly recompute the definitions?
    // to make sure they haven't changed since the user last touched anything
    let r = (|| -> R {
        let acc = &mut acc;
        let processing_queue = &mut processing_queue;
        definitions.into_iter().try_for_each(|(name, definition)| -> R {
            state.on_definition(acc, processing_queue, name, definition)
        })?;
        'outer: loop {
            // Pop a node that off the queue that needs to have generic params added, and add them
            // This might create more things on the queue.
            // loop until the queue is empty.
            for (file_id, q) in processing_queue.files.iter_mut() {
                for (_imp, queued_nodes) in q
                    .inner_usage_waiting_to_see_if_a_surrounding_impl_would_have_generics
                    .iter_mut()
                {
                    if let Some(x) = queued_nodes.pop() {
                        let UsageWaitingOnSurroudingImpl { non_impl_generic_params_node } = x;
                        let file_id = *file_id;
                        state.on_non_impl_generic_params_ancestor(
                            acc,
                            processing_queue,
                            file_id,
                            non_impl_generic_params_node,
                        )?;
                        continue 'outer;
                    }
                }
            }
            // queue is empty since nothing got popped, so we are done.
            break;
        }

        Ok(())
    })();

    let r = (|| -> R {
        let () = r?;
        // Write, to the tree, all the changes that we accumulated.
        let () = state.finish(acc)?;
        Ok(())
    })();
    match r {
        Ok(()) => (),
        Err(_x) => {
            // FIXME: report the error better?
        }
    }
}
struct Configuration {
    param: ast::GenericParam,
    arg: ast::GenericArg,
}
struct SourceChangeBuilder2<'a> {
    inner: &'a mut SourceChangeBuilder,
}

// Small utility wrapper for ast::AstNode's
// ensures the node is mutable before doing mutable operations.
// requires some due diligence, since mutable methods are accessible on regular
// ast::AstNode's
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MutableAst<T>(T);

impl<T: ast::AstNode> From<T> for MutableAst<T> {
    fn from(x: T) -> Self {
        // owned == mutable
        use std::borrow::Cow;
        assert!(matches!(x.syntax().green(), Cow::Owned(_)));
        Self::unchecked_from(x)
    }
}

impl<T> MutableAst<T> {
    fn unchecked_from(x: T) -> Self {
        Self(x)
    }
}

impl<T> MutableAst<T> {
    fn map<U>(self, f: impl FnOnce(T) -> U) -> MutableAst<U> {
        let MutableAst(x) = self;
        let x = f(x);
        MutableAst(x)
    }
}
impl<'a> SourceChangeBuilder2<'a> {
    fn make_mut<T: ast::AstNode>(&mut self, x: T) -> MutableAst<T> {
        let x = self.inner.make_mut(x);
        MutableAst::from(x)
    }
}
struct AssistState<'b, 'sema_a, 'sema> {
    builder: SourceChangeBuilder2<'b>,
    config: Configuration,
    // FIXME: support snippets, when enabled and only one file
    #[allow(unused)]
    snippet_cap: Option<ide_db::SnippetCap>,
    sema: &'sema hir::Semantics<'sema_a, RootDatabase>,
    aux_state: AuxilliaryState,
}
#[derive(Debug, Clone, Default)]
struct AuxilliaryState {
    definitions_already_handled: FxHashSet<(Definition, SmolStr)>,
    usages_already_handled: FxHashSet<ast::NameLike>,
}
#[derive(Debug, Clone, Default)]
struct ProcessingQueue {
    // FIXME: use nohash_hasher::IntMap;
    // as of now, it is not in this crate's Cargo.toml
    files: FxHashMap<FileId, FileProcessingQueue>,
}
#[derive(Debug, Clone)]
struct UsageWaitingOnSurroudingImpl {
    non_impl_generic_params_node: AnyGenericParamsOwnerEdit,
}
#[derive(Debug, Clone, Default)]
struct FileProcessingQueue {
    inner_usage_waiting_to_see_if_a_surrounding_impl_would_have_generics:
        FxHashMap<ast::Impl, Vec<UsageWaitingOnSurroudingImpl>>,
}

#[derive(Debug, Clone, Default)]
struct ChangeAccumulator {
    // FIXME: use nohash_hasher::IntMap;
    // as of now, it is not in this crate's Cargo.toml
    files: FxHashMap<FileId, FileChangeAccumulator>,
}
#[derive(Debug, Clone, Default)]
struct FileChangeAccumulator {
    // This may never need to check for duplicates, as it hasn't encountered any yet
    // so possibly change to Vec
    path_segments: FxHashSet<(ast::PathSegment, Definition)>,
    // encounters duplicates when multiple children need a parent to supply the generic param
    // e.g. trait Stuff{ fn a(x: NeedsGeneric); fn b(x: NeedsGeneric); }
    // the `trait Stuff` would be inserted twice.
    defs_with_generic_params: FxHashSet<AnyGenericParamsOwnerEdit>,
}
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
enum DefinitionWithGenericParams {
    Struct(hir::Struct),
    Union(hir::Union),
    Enum(hir::Enum),
    Trait(hir::Trait),
    TraitAlias(hir::TraitAlias),
    TypeAlias(hir::TypeAlias),
    Function(hir::Function),
}
fn as_definition_with_generic_params(
    definition: Definition,
) -> Option<DefinitionWithGenericParams> {
    let x = match definition {
        Definition::Adt(x) => match x {
            hir::Adt::Struct(x) => DefinitionWithGenericParams::Struct(x),
            hir::Adt::Union(x) => DefinitionWithGenericParams::Union(x),
            hir::Adt::Enum(x) => DefinitionWithGenericParams::Enum(x),
        },
        Definition::Trait(x) => DefinitionWithGenericParams::Trait(x),
        Definition::TraitAlias(x) => DefinitionWithGenericParams::TraitAlias(x),
        Definition::TypeAlias(x) => DefinitionWithGenericParams::TypeAlias(x),
        Definition::Function(x) => DefinitionWithGenericParams::Function(x),
        _ => return None,
    };
    Some(x)
}
impl<'b, 'sema_a, 'sema> AssistState<'b, 'sema_a, 'sema> {
    fn on_definition(
        &mut self,
        acc: &mut ChangeAccumulator,
        processing_queue: &mut ProcessingQueue,
        name: ast::NameLike,
        definition: Definition,
    ) -> R {
        let tmp_text = name.text();
        let name_str = tmp_text.as_str();
        {
            // cycle detection: (mutually) recursive types/traits/fns etc
            // We have to key on the current name too because aliases
            // have the same definition, but a different string name.

            // FIXME: don't allocate q for .contains(...)
            // this is a limitation of std::borrow::Borrow, it can't return a tuple of references
            let q = (definition, SmolStr::from(name_str));
            let was_already_handled = self.aux_state.definitions_already_handled.contains(&q);
            if was_already_handled {
                return Ok(());
            }
            self.aux_state.definitions_already_handled.insert(q);
        }

        (|| -> R {
            macro_rules! on_def_variant {
                ($ast_type: ident, $x: expr $(,)?) => {{
                    let (file_id, x) = (|| -> Option<_> {
                        use hir::HasSource;
                        let src = $x.source(self.sema.db)?;
                        let hir_file_id = src.syntax().file_id;
                        // note: HirFileId could be a macro. error out
                        // since supporting macros is hard.
                        // FIXME: support macros
                        let file_id = hir_file_id.file_id()?;
                        let name = src.value.name()?;
                        let def_node = name.syntax().ancestors().find_map(ast::$ast_type::cast)?;
                        let def_node = AnyGenericParamsOwnerEdit::cast(def_node.syntax().clone())?;
                        Some((file_id, def_node))
                    })()
                    .ok_or_else(|| Error::Str(format!("unable to find parent definition node")))?;
                    (file_id, x)
                }};
            }
            let def = as_definition_with_generic_params(definition);
            let Some(def) = def else {
                return Err(Error::Str(format!(
                    "was not a defintition that has generic params: {def:?}"
                )));
            };
            let (file_id, x) = match def {
                DefinitionWithGenericParams::Struct(x) => on_def_variant!(Struct, x),
                DefinitionWithGenericParams::Union(x) => on_def_variant!(Union, x),
                DefinitionWithGenericParams::Enum(x) => on_def_variant!(Enum, x),
                DefinitionWithGenericParams::Trait(x) => on_def_variant!(Trait, x),
                DefinitionWithGenericParams::TraitAlias(x) => on_def_variant!(TraitAlias, x),
                DefinitionWithGenericParams::TypeAlias(x) => on_def_variant!(TypeAlias, x),
                DefinitionWithGenericParams::Function(x) => on_def_variant!(Fn, x),
            };
            let acc = acc.files.entry(file_id).or_default();
            self.run_generic_params_owner_edit_without_segment(acc, x)?;
            Ok(())
        })()?;

        let usages = definition.usages(self.sema).with_override_name(name_str);
        // usages will include name, so ignore name
        let _ = name;
        usages.search(&mut |file_id, file_ref| {
            let r = self.on_usage(acc, processing_queue, file_id, file_ref.name, definition);
            match r {
                Ok(()) => false,
                Err(_x) => {
                    // FIXME: on a non-continuable error, return true (signaling to stop)
                    // also, propagate the Err through `FindUsages::search` somehow. std::ops::ControlFlow would fit perfectly.
                    false
                }
            }
        });
        Ok(())
    }
    fn on_usage(
        &mut self,
        acc: &mut ChangeAccumulator,
        processing_queue: &mut ProcessingQueue,
        file_id: FileId,
        name: ast::NameLike,
        definition: Definition,
    ) -> R {
        {
            // double visit detection: this seems to happen for type aliases in traits
            let q = &name;
            let was_already_handled = self.aux_state.usages_already_handled.contains(q);
            if was_already_handled {
                return Ok(());
            }
            self.aux_state.usages_already_handled.insert(name.clone());
        }

        let r = match name.clone() {
            // This is the usual case, when the symbol is used
            // e.g. "Vec" in "Vec<String>" is a NameRef
            ast::NameLike::NameRef(name_node) => {
                self.on_usage_name_ref(acc, processing_queue, name_node, file_id, definition)
            }
            // This is a somewhat unusual case, I have seen this when the same symbol is defined elsewhere.
            // e.g. trait HasFood{ type Food; }
            //                          ^~~~ this is a usage (ast::Name)
            // impl HasFood for Cat{ type Food = Fish; }
            //                            ^~~~ when this is a definition
            ast::NameLike::Name(name) => {
                let syn = name.syntax();
                let Some(x) = syn.ancestors().find_map(AnyGenericParamsOwnerEdit::cast) else {
                    return Ok(());
                };

                self.run_generic_params_owner_edit_without_segment(
                    acc.files.entry(file_id).or_default(),
                    x,
                )?;
                let x = match find_definitions(
                    self.sema,
                    syn,
                    FilePosition { file_id, offset: syn.text_range().start() },
                ) {
                    Ok(x) => x,
                    Err(_) => return Ok(()),
                };
                x.into_iter().try_for_each(|(name, definition)| {
                    self.on_definition(acc, processing_queue, name, definition)
                })?;
                Ok(())
            }
            // FIXME: make this work for lifetimes
            ast::NameLike::Lifetime(_lifetime) => Ok(()),
        };
        r
    }

    fn on_usage_name_ref(
        &mut self,
        acc: &mut ChangeAccumulator,
        processing_queue: &mut ProcessingQueue,
        name_node: ast::NameRef,
        file_id: FileId,
        definition: Definition,
    ) -> R {
        let name_node_syntax = name_node.syntax();

        let mut ancestors = name_node_syntax.ancestors();
        let x = ancestors.next();
        // self == ancestors[0]
        stdx::always!(x.is_some_and(|x| x == *name_node_syntax));
        let x = ancestors.next();
        // usages always are a path segment, usually in a single segment path
        // e.g. "Vec" in "Vec<String>", multi segment: "std::vec::Vec<String>"
        let Some(parent) = x else {
            return Err(Error::Str(format!("node did not have a parent, node = {name_node:?}")));
        };

        let Some(segment) = ast::PathSegment::cast(parent.clone()) else {
            return Err(Error::Str(format!(
                "node's parent was not a path segment, node = {name_node:?}, parent = {parent:?}"
            )));
        };

        #[derive(Debug, Clone)]
        enum Either4<A, B, C, D> {
            A(A),
            B(B),
            C(C),
            D(D),
        }
        let x = ancestors.find_map(|x| {
            // find an ancestor that is:
            let x = match_ast! {
                match x {
                    // e.g. impl items, fn (params, return, bounds), struct members, enum variant members
                    // impl Stuff{ fn do_stuff(x: Vec<String>); }
                    // struct Stuff{ x: Vec<String> }
                    AnyGenericParamsOwnerEdit(x) => Either4::A(x),
                    // usually a path expression, usually inferred, e.g. "Vec" in "Vec::new()"
                    // also patterns and struct literals
                    // Here I just assume it is always inferred, so I ignore this path segment.
                    ast::Expr(x) => Either4::B(x),
                    ast::Pat(x) => Either4::D(x),
                    // use Vec as MyVec;
                    ast::UseTree(x) =>Either4::C(x),
                    _ => return None,
                }
            };
            Some(x)
        });
        let Some(meaningful_parent) = x else {
            // broken ast, or possible top level ast that I haven't accounted for
            return Ok(());
        };
        let ancestor_with_generic_params = match meaningful_parent {
            Either4::B(x) => {
                let _ = x;
                let _ = segment;
                return Ok(());
            }
            Either4::D(x) => {
                let _ = x;
                let _ = segment;
                return Ok(());
            }
            Either4::C(x) => {
                // a use alias needs to add generics to its usages
                let Some(name) = (|| -> Option<_> {
                    let x = x.rename()?;
                    let x = x.name()?;
                    Some(x)
                })() else {
                    // no right ident
                    // either means broken ast or it's an _
                    // we could try to see which case it is, but it doesn't matter
                    return Ok(());
                };
                let defs = find_definitions(
                    self.sema,
                    name.syntax(),
                    FilePosition { file_id, offset: name.syntax().text_range().start() },
                );

                let defs = defs
                    .map_err(|_| Error::Str(format!("no definitions found for generic thing")))?;

                let r = defs.into_iter().try_for_each(|(name, definition)| -> R {
                    self.on_definition(acc, processing_queue, name, definition)
                });
                return r;
            }
            Either4::A(x) => x,
        };
        let outer_acc = acc;
        self.add_segment(outer_acc.files.entry(file_id).or_default(), segment, definition)?;
        /*
           We want to use an outer parameter if there is one already.
           However, we cannot access generic parameters across certain boundaries.
           The rules that I have figured out are probably incomplete.
           I took some information from rustc_resolve, specifically RibKind, which specifies how upvars
           can be accessed.
           upvar just means e.g.
           impl<T> X<T>{ type A = T; }
                                  ^upvar, has to access outer("up") impl's scope
           I also just tried various combinations to see which failed to compile.
           Notably:
           - anything can access upvar of impl, function signature included
           - as soon as you see an expr or non-alias HasGenericParams ancestor, stop
        */

        let acc = outer_acc.files.entry(file_id).or_default();

        let ancestor_with_generic_params = 'thing: {
            let mut current_generic_params_node = ancestor_with_generic_params.clone();
            let mut candidate_generic_params_node = current_generic_params_node;
            if ast::Impl::can_cast(candidate_generic_params_node.syntax().kind()) {
                break 'thing candidate_generic_params_node;
            }
            let x = ancestor_with_generic_params.syntax();
            for x in x.ancestors() {
                // this takes care of both function bodies and const generics
                if let Some(_) = ast::Expr::cast(x.clone()) {
                    break;
                }
                let Some(x) = AnyGenericParamsOwnerEdit::cast(x.clone()) else {
                    continue;
                };
                current_generic_params_node = x.clone();

                // impl's are not allowed to use an outer generic param.
                // This doesn't seem like it's necessary for rust to restrict like this
                // but that's how it is right now.
                let Some(x) = ast::Impl::cast(x.syntax().clone()) else {
                    // all other (non-Impl) HasGenericParams can freely define generic params.
                    candidate_generic_params_node = current_generic_params_node.clone();
                    continue;
                };
                // Only use this impl's generic param if it already would get one
                // This is because doing the following is not allowed:

                #[rustfmt::skip] #[cfg(FALSE)] const _: () = {
                    struct A;
                    impl<T> A { fn make_vec() -> Vec<T> { vec![] } }
                    //   ^ error: unconstrained type parameter
                };
                // note: We only know if it would already get one in some cases, if the impl
                // has already been visited.
                // We can't easily control visit order, so we have to wait until that impl has been processed.

                if acc.defs_with_generic_params.contains(&current_generic_params_node) {
                    // we worked on this impl already, so we're done
                    return Ok(());
                }
                // haven't seen this impl yet, so wait to see if it needs generics, so we can just use those
                // instead of making our own
                let queue = processing_queue.files.entry(file_id).or_default();
                let work = queue
                    .inner_usage_waiting_to_see_if_a_surrounding_impl_would_have_generics
                    .entry(x)
                    .or_default();
                work.push(UsageWaitingOnSurroudingImpl {
                    non_impl_generic_params_node: candidate_generic_params_node,
                });
                return Ok(());
            }
            candidate_generic_params_node
        };
        if let Some(x) = ast::Impl::cast(ancestor_with_generic_params.syntax().clone()) {
            let queue = processing_queue.files.entry(file_id).or_default();
            // This impl does have surrounding generics, so we can tell all things waiting
            // that they don't need to add any generics at their inner scope.
            let _ = queue
                .inner_usage_waiting_to_see_if_a_surrounding_impl_would_have_generics
                .remove(&x);
        }

        self.on_non_impl_generic_params_ancestor(
            outer_acc,
            processing_queue,
            file_id,
            ancestor_with_generic_params,
        )?;
        Ok(())
    }

    fn on_non_impl_generic_params_ancestor(
        &mut self,
        acc: &mut ChangeAccumulator,
        processing_queue: &mut ProcessingQueue,
        file_id: FileId,
        ancestor_with_generic_params: AnyGenericParamsOwnerEdit,
    ) -> R {
        /*
        this 'block thing functions as
        if let ...
        && let ...
        since `&& let` (let_chains) isn't on stable.
        */
        'block: {
            let Some(ancestor_with_generic_params) =
                ast::AnyHasName::cast(ancestor_with_generic_params.syntax().clone())
            else {
                // no name, should only be taken for "impl<T> ..."
                // so this should probably never be reached.
                break 'block self.run_generic_params_owner_edit_without_segment(
                    acc.files.entry(file_id).or_default(),
                    ancestor_with_generic_params,
                );
            };
            let Some(name_of_ancestor_with_generic_params) = ancestor_with_generic_params.name()
            else {
                break 'block Ok(());
            };
            let defs = find_definitions(
                self.sema,
                ancestor_with_generic_params.syntax(),
                FilePosition {
                    file_id,
                    offset: name_of_ancestor_with_generic_params.syntax().text_range().start(),
                },
            );

            let defs =
                defs.map_err(|_| Error::Str(format!("no definitions found for generic thing")))?;

            defs.into_iter().try_for_each(|(name, definition)| -> R {
                self.on_definition(acc, processing_queue, name, definition)
            })
        }
    }
}

#[derive(Debug, Clone)]
enum Error {
    Str(String),
}
type MyResult<T> = Result<T, Error>;
type R = MyResult<()>;

impl Configuration {
    fn add_generics_to_path_segment(&mut self, parent: MutableAst<ast::PathSegment>) -> R {
        let generic_arg_list = parent.map(|x| x.get_or_create_generic_arg_list());
        add_generic_arg(generic_arg_list.0.clone(), self.arg.clone_subtree().clone_for_update());
        Ok(())
    }

    fn run_generic_params_owner_edit_without_segment_impl(
        &mut self,
        parent: MutableAst<AnyGenericParamsOwnerEdit>,
    ) -> R {
        let gpl = parent.clone().map(|x| x.get_or_create_generic_param_list());
        // FIXME: this seems inefficient having to clone twice
        gpl.0.add_generic_param(self.param.clone_subtree().clone_for_update());
        Ok(())
    }
}

impl<'b, 'sema_a, 'sema> AssistState<'b, 'sema_a, 'sema> {
    fn add_segment(
        &mut self,
        acc: &mut FileChangeAccumulator,
        segment: ast::PathSegment,
        definition: Definition,
    ) -> R {
        let x = (segment, definition);
        acc.path_segments.insert(x);
        Ok(())
    }

    fn run_generic_params_owner_edit_without_segment<T>(
        &mut self,
        acc: &mut FileChangeAccumulator,
        parent: T,
    ) -> R
    where
        T: GenericParamsOwnerEdit,
        AnyGenericParamsOwnerEdit: From<T>,
    {
        let x = parent.into();
        acc.defs_with_generic_params.insert(x);
        Ok(())
    }

    fn finish(self, acc: ChangeAccumulator) -> R {
        let Self { mut builder, snippet_cap: _, config: mut thingy, sema: _, aux_state: _ } = self;

        let ChangeAccumulator { files } = acc;

        /*
            FIXME: add a way to check for duplicate generic parameter names
            e.g. for:
            struct Thing$0<T1>(T1);
            current implementation:
                -> struct Thing<T1, T1>(T1);
                                    ^^
            desired implementation:
                -> struct Thing<T1, T2>(T1);
                                    ^^
            It should also not duplicate names of upvars.
            e.g.
            trait DoStuff{ fn stuff<T1>(hello: Thing$0, other: T1); }
            trait DoStuff<T1>{ fn stuff<T1>(hello: Thing<T1>, other: T1); }
                   added: ^^                             ^^
                accidentally refered to ^^ when looking at^
            desired:
            trait DoStuff<T2>{ fn stuff<T1>(hello: Thing<T2>, other: T1); }
        */
        for (file_id, acc) in files {
            let FileChangeAccumulator { path_segments, defs_with_generic_params } = acc;
            builder.inner.edit_file(file_id);
            /*
               note: make sure to make_mut for both of these before modifying any of them
            */
            let path_segments = path_segments
                .into_iter()
                .map(|(x, def)| (builder.make_mut(x), def))
                .collect::<Vec<_>>();

            let defs_with_generic_params = defs_with_generic_params
                .into_iter()
                .map(|x| builder.make_mut(x))
                .collect::<Vec<_>>();
            path_segments.into_iter().try_for_each(|(x, _def)| -> R {
                thingy.add_generics_to_path_segment(x)?;
                Ok(())
            })?;
            defs_with_generic_params.into_iter().try_for_each(|x| -> R {
                thingy.run_generic_params_owner_edit_without_segment_impl(x)?;
                Ok(())
            })?;
        }
        Ok(())
    }
}

// copied (and modified) from ide_db::rename (it was a private function)
// FIXME: share code with the previously mentioned module
// note: in practice, the return value seems to always be length 1
// [(node_of_name_reference, e.g. NAME_REF @ 7..10, hir of definition of that name, e.g. StructId(0))]
fn find_definitions(
    sema: &hir::Semantics<'_, RootDatabase>,
    syntax: &SyntaxNode,
    position: FilePosition,
) -> rename::Result<impl Iterator<Item = (ast::NameLike, Definition)>> {
    use ide_db::rename::{bail, format_err, RenameError};
    let symbols = sema.find_nodes_at_offset_with_descend::<ast::NameLike>(syntax, position.offset);
    let symbols = symbols.map(|name_like| {
        let res = match &name_like {
            ast::NameLike::Name(name)
                if name.syntax().parent().map_or(false, |it| ast::Rename::can_cast(it.kind()))
                // FIXME: uncomment this once we resolve to usages to extern crate declarations
                // && name
                //     .syntax()
                //     .ancestors()
                //     .nth(2)
                //     .map_or(true, |it| !ast::ExternCrate::can_cast(it.kind()))
                 =>
            {
                let x = NameClass::classify(sema, name);
                let name_class = x.ok_or_else(|| format_err!("No references found at position"))?;
                let def = match name_class {
                    NameClass::Definition(it) => it,
                    NameClass::ConstReference(it) => it,
                    NameClass::PatFieldShorthand { local_def, field_ref: _ } => {
                        Definition::Local(local_def)
                    }
                };
                Ok((name_like, def))
            }
            ast::NameLike::Name(name) => NameClass::classify(sema, name)
                .map(|class| match class {
                    NameClass::Definition(it) | NameClass::ConstReference(it) => it,
                    NameClass::PatFieldShorthand { local_def, field_ref: _ } => {
                        Definition::Local(local_def)
                    }
                })
                .map(|def| (name_like.clone(), def))
                .ok_or_else(|| format_err!("No references found at position")),
            ast::NameLike::NameRef(name_ref) => {
                NameRefClass::classify(sema, name_ref)
                    .map(|class| match class {
                        NameRefClass::Definition(def) => def,
                        NameRefClass::FieldShorthand { local_ref, field_ref: _ } => {
                            Definition::Local(local_ref)
                        }
                        NameRefClass::ExternCrateShorthand { decl, .. } => {
                            Definition::ExternCrateDecl(decl)
                        }
                    })
                    // FIXME: uncomment this once we resolve to usages to extern crate declarations
                    .filter(|def| !matches!(def, Definition::ExternCrateDecl(..)))
                    .ok_or_else(|| format_err!("No references found at position"))
                    .and_then(|def| {
                        // if the name differs from the definitions name it has to be an alias
                        // if def
                        //     .name(sema.db)
                        //     .map_or(false, |it| it.to_smol_str() != name_ref.text().as_str())
                        // {
                        //     return Err(format_err!("Renaming aliases is currently unsupported"))
                        // }

                        // I do support adding generic parameters to aliases.
                        Ok((name_like.clone(), def))
                    })
            }
            ast::NameLike::Lifetime(lifetime) => NameRefClass::classify_lifetime(sema, lifetime)
                .and_then(|class| match class {
                    NameRefClass::Definition(def) => Some(def),
                    _ => None,
                })
                .or_else(|| {
                    NameClass::classify_lifetime(sema, lifetime).and_then(|it| match it {
                        NameClass::Definition(it) => Some(it),
                        _ => None,
                    })
                })
                .map(|def| (name_like, def))
                .ok_or_else(|| format_err!("No references found at position")),
        };

        res
    });

    let res: rename::Result<Vec<_>> = symbols.collect();
    match res {
        Ok(v) => {
            if v.is_empty() {
                // FIXME: some semantic duplication between "empty vec" and "Err()"
                bail!("No references found at position")
            } else {
                // remove duplicates, comparing `Definition`s
                use itertools::Itertools;
                Ok(v.into_iter().unique_by(|t| t.1))
            }
        }
        Err(e) => Err(e),
    }
}
// FIXME: add this to crates/syntax/src/ast/node_ext.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AnyGenericParamsOwnerEdit {
    Fn(ast::Fn),
    Impl(ast::Impl),
    Trait(ast::Trait),
    Struct(ast::Struct),
    Enum(ast::Enum),
    Union(ast::Union),
    TypeAlias(ast::TypeAlias),
    TraitAlias(ast::TraitAlias),
}

const _: () = {
    macro_rules! for_each_member {
        ($next: ident) => {
            $next!(
                (FN, Fn),
                (IMPL, Impl),
                (TRAIT, Trait),
                (STRUCT, Struct),
                (ENUM, Enum),
                (UNION, Union),
                (TYPE_ALIAS, TypeAlias),
                (TRAIT_ALIAS, TraitAlias),
            )
        };
    }
    macro_rules! impl_from {
        ($(($kind: ident, $variant: ident),)*) => { $(
            impl From<ast::$variant> for AnyGenericParamsOwnerEdit {
                fn from(x: ast::$variant) -> Self {
                    Self::$variant(x)
                }
            }
        )* };
    }
    for_each_member!(impl_from);
    impl AstNode for AnyGenericParamsOwnerEdit {
        fn can_cast(kind: SyntaxKind) -> bool {
            macro_rules! next {
                ($(($kind: ident, $variant: ident),)*) => {
                    match kind {
                        $(SyntaxKind::$kind => true,)*
                        _ => false,
                    }
                };
            }
            for_each_member!(next)
        }

        fn cast(syntax: SyntaxNode) -> Option<Self> {
            macro_rules! next {
                ($(($kind: ident, $variant: ident),)*) => {
                    match syntax.kind() {
                        $(SyntaxKind::$kind => Some(Self::$variant(ast::$variant::cast(syntax)?)),)*
                        _ => return None,
                    }
                };
            }
            for_each_member!(next)
        }

        fn syntax(&self) -> &SyntaxNode {
            macro_rules! next {
                ($(($kind: ident, $variant: ident),)*) => {
                    match self {
                        $(Self::$variant(x) => x.syntax(),)*
                    }
                };
            }
            for_each_member!(next)
        }
    }
    impl ast::HasGenericParams for AnyGenericParamsOwnerEdit {}
    impl GenericParamsOwnerEdit for AnyGenericParamsOwnerEdit {
        fn get_or_create_generic_param_list(&self) -> ast::GenericParamList {
            macro_rules! next {
                ($(($kind: ident, $variant: ident),)*) => {
                    match self {
                        $(Self::$variant(x) => x.get_or_create_generic_param_list(),)*
                    }
                };
            }
            for_each_member!(next)
        }
        fn get_or_create_where_clause(&self) -> ast::WhereClause {
            macro_rules! next {
                ($(($kind: ident, $variant: ident),)*) => {
                    match self {
                        $(Self::$variant(x) => x.get_or_create_where_clause(),)*
                    }
                };
            }
            for_each_member!(next)
        }
    }
};

// I copied this from edit_in_place.rs::add_generic_param
// I'm not sure why this didnt exist already
// FIXME: add to crates/syntax/src/ast/edit_in_place.rs
fn add_generic_arg(self_: ast::GenericArgList, generic_arg: ast::GenericArg) {
    match self_.generic_args().last() {
        Some(last) => {
            let position = Position::after(last.syntax());
            use ast::make;
            let elements = vec![
                make::token(syntax::T![,]).into(),
                make::tokens::single_space().into(),
                generic_arg.syntax().clone().into(),
            ];
            ted::insert_all(position, elements);
        }
        None => {
            let after_l_angle = Position::after(self_.l_angle_token().unwrap());
            ted::insert(after_l_angle, generic_arg.syntax());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;
    // FIXME: add tests to make sure it doesn't panic with strange input
    // FIXME: add cov_mark::{hit, check} in some tests
    /*
    note: Tests should cover all of
    AnyGenericParamsOwnerEdit's variants.
    */

    #[test]
    fn tuple_struct_basic() {
        check_assist(
            add_generic_parameter,
            r#"struct Color$0<T2>(u32);"#,
            r#"struct Color<T2, T1>(u32);"#,
        );
    }
    #[test]
    fn unit_struct_basic() {
        check_assist(add_generic_parameter, r#"struct Color$0;"#, r#"struct Color<T1>;"#);
        check_assist(add_generic_parameter, r#"struct Color$0<T2>;"#, r#"struct Color<T2, T1>;"#);
    }
    #[test]
    fn struct_basic() {
        check_assist(
            add_generic_parameter,
            r#"struct Color$0 { x: u32 }"#,
            r#"struct Color<T1> { x: u32 }"#,
        );
        check_assist(
            add_generic_parameter,
            r#"struct Color$0<T2> { x: u32 }"#,
            r#"struct Color<T2, T1> { x: u32 }"#,
        );
    }
    #[test]
    fn union_basic() {
        check_assist(
            add_generic_parameter,
            r#"union Color$0 { x: u32 }"#,
            r#"union Color<T1> { x: u32 }"#,
        );
        check_assist(
            add_generic_parameter,
            r#"union Color$0<T2> { x: u32 }"#,
            r#"union Color<T2, T1> { x: u32 }"#,
        );
    }

    #[test]
    fn enum_basic() {
        check_assist(
            add_generic_parameter,
            r#"enum Color$0 { A{x: u32} }"#,
            r#"enum Color<T1> { A{x: u32} }"#,
        );
        check_assist(
            add_generic_parameter,
            r#"enum Color$0<T2> { A{x: u32} }"#,
            r#"enum Color<T2, T1> { A{x: u32} }"#,
        );
    }
    #[test]
    fn nest_struct_direct() {
        check_assist(
            add_generic_parameter,
            r#"
struct Channel$0 { x: u8 }
struct Color{ channel: Channel }
"#,
            r#"
struct Channel<T1> { x: u8 }
struct Color<T1>{ channel: Channel<T1> }
"#,
        );
    }
    #[test]
    fn nest_tuple_struct_direct() {
        check_assist(
            add_generic_parameter,
            r#"
struct Channel$0 { x: u8 }
struct Color(Channel);
"#,
            r#"
struct Channel<T1> { x: u8 }
struct Color<T1>(Channel<T1>);
"#,
        );
    }
    #[test]
    fn nest_tuple_struct_direct_multi() {
        check_assist(
            add_generic_parameter,
            r#"
struct Channel$0 { x: u8 }
struct Color1(Channel);
struct Color2(Channel);
"#,
            r#"
struct Channel<T1> { x: u8 }
struct Color1<T1>(Channel<T1>);
struct Color2<T1>(Channel<T1>);
"#,
        );
    }
    #[test]
    fn nest_enum_tuple_struct_direct() {
        check_assist(
            add_generic_parameter,
            r#"
struct Channel$0 { x: u8 }
enum Color{ A(Channel) }
"#,
            r#"
struct Channel<T1> { x: u8 }
enum Color<T1>{ A(Channel<T1>) }
"#,
        );
    }
    #[test]
    fn nest_enum_record_struct_direct() {
        check_assist(
            add_generic_parameter,
            r#"
struct Channel$0 { x: u8 }
enum Color{ A{c: Channel} }
"#,
            r#"
struct Channel<T1> { x: u8 }
enum Color<T1>{ A{c: Channel<T1>} }
"#,
        );
    }

    #[test]
    fn multi_layered_struct() {
        check_assist(
            add_generic_parameter,
            r#"
struct Channel$0(u8);
struct Color([Channel; 3]);
struct Image(Vec<Color>);
"#,
            r#"
struct Channel<T1>(u8);
struct Color<T1>([Channel<T1>; 3]);
struct Image<T1>(Vec<Color<T1>>);
"#,
        );
    }
    #[test]
    fn cyclic_structs() {
        check_assist(
            add_generic_parameter,
            r#"
struct Cons$0(i32, List);
struct List(Option<Box<Cons>>);
"#,
            r#"
struct Cons<T1>(i32, List<T1>);
struct List<T1>(Option<Box<Cons<T1>>>);
"#,
        );
    }
    #[test]
    fn cyclic_structs_impl() {
        check_assist(
            add_generic_parameter,
            r#"
struct Cons$0(i32, List);
impl Cons{ fn len(&self){42} }
struct List(Option<Box<Cons>>);
impl List{ fn len(&self){42} }
"#,
            r#"
struct Cons<T1>(i32, List<T1>);
impl<T1> Cons<T1>{ fn len(&self){42} }
struct List<T1>(Option<Box<Cons<T1>>>);
impl<T1> List<T1>{ fn len(&self){42} }
"#,
        );
    }
    #[test]
    fn type_alias_basic() {
        check_assist(
            add_generic_parameter,
            r#"
type Color$0 = u32;
"#,
            r#"
type Color<T1> = u32;
"#,
        );
    }
    #[test]
    fn type_alias_into_struct() {
        check_assist(
            add_generic_parameter,
            r#"
struct Channel$0(u8);
type Color = [Channel; 3];
struct Image(Vec<Color>);
"#,
            r#"
struct Channel<T1>(u8);
type Color<T1> = [Channel<T1>; 3];
struct Image<T1>(Vec<Color<T1>>);
"#,
        );
    }

    #[test]
    fn trait_basic() {
        check_assist(
            add_generic_parameter,
            r#"
trait HasFood$0 {}
"#,
            r#"
trait HasFood<T1> {}
"#,
        );
    }
    #[test]
    fn trait_into_trait_bound() {
        check_assist(
            add_generic_parameter,
            r#"
trait HasFood$0 {}
trait HasSupplies: HasFood {}
"#,
            r#"
trait HasFood<T1> {}
trait HasSupplies<T1>: HasFood<T1> {}
"#,
        );
    }

    #[test]
    fn trait_into_struct() {
        check_assist(
            add_generic_parameter,
            r#"
trait HasFood$0 {}
struct Supplies(Box<dyn HasFood>);
"#,
            r#"
trait HasFood<T1> {}
struct Supplies<T1>(Box<dyn HasFood<T1>>);
"#,
        );
    }
    /*
    this is where some ambiguity comes in:
    it is unclear where the <T1> should go in this case.
    it may go after make: make<T1>
    or after CanMakeFood: CanMakeFood<T1>

    one thing reason for doing make<T1> is that one can just use this assist
    on CanMakeFood to add <T1> to it.
    another alternative is to temporarily add a bound ":HasFood"
    to CanMakeFood so that it adds <T1> to CanMakeFood
    */
    #[test]
    fn trait_into_trait_fn() {
        check_assist(
            add_generic_parameter,
            r#"
trait HasFood$0 {}
trait CanMakeFood{ fn make() -> impl HasFood; }
"#,
            r#"
trait HasFood<T1> {}
trait CanMakeFood<T1>{ fn make() -> impl HasFood<T1>; }
"#,
        );
    }

    // here, T1 looks like it is being used before it is defined
    // but rust allows it, so no need to special case this
    #[test]
    fn trait_into_fn_bound() {
        check_assist(
            add_generic_parameter,
            r#"
trait HasFood$0 {}
fn make<HF: HasFood>(){}
"#,
            r#"
trait HasFood<T1> {}
fn make<HF: HasFood<T1>, T1>(){}
"#,
        );
    }
    #[test]
    fn trait_into_trait_fn_bound() {
        check_assist(
            add_generic_parameter,
            r#"
trait HasFood$0 {}
trait CanMakeFood{ fn make<HF: HasFood>(); }
"#,
            r#"
trait HasFood<T1> {}
trait CanMakeFood<T1>{ fn make<HF: HasFood<T1>>(); }
"#,
        );
    }

    #[test]
    fn struct_into_trait_fn_into_impl() {
        check_assist(
            add_generic_parameter,
            r#"
struct Food$0;
trait MakeFood{ fn make() -> Food; }
impl MakeFood for Farm{ fn make() -> Food { Food }}
"#,
            r#"
struct Food<T1>;
trait MakeFood<T1>{ fn make() -> Food<T1>; }
impl<T1> MakeFood<T1> for Farm{ fn make() -> Food<T1> { Food }}
"#,
        );
    }

    #[test]
    fn trait_alias_basic() {
        check_assist(
            add_generic_parameter,
            r#"
trait Debug2$0 = Debug;
"#,
            r#"
trait Debug2<T1> = Debug;
"#,
        );
    }
    #[test]
    fn trait_alias_into_trait_bound() {
        check_assist(
            add_generic_parameter,
            r#"
trait Debug2$0 = Debug;
trait SuperDebug: Debug2 {}
"#,
            r#"
trait Debug2<T1> = Debug;
trait SuperDebug<T1>: Debug2<T1> {}
"#,
        );
    }

    // not sure if dyn TraitAlias is allowed, but it works here
    #[test]
    fn trait_alias_into_struct() {
        check_assist(
            add_generic_parameter,
            r#"
trait Debug2$0 = Debug;
struct Debug2Thing(Box<dyn Debug2>);
"#,
            r#"
trait Debug2<T1> = Debug;
struct Debug2Thing<T1>(Box<dyn Debug2<T1>>);
"#,
        );
    }

    #[test]
    fn struct_into_use_into_struct() {
        check_assist(
            add_generic_parameter,
            r#"
struct ColorImpl$0(u8);
use ColorImpl as Color;
struct Image(Vec<Color>);
"#,
            r#"
struct ColorImpl<T1>(u8);
use ColorImpl as Color;
struct Image<T1>(Vec<Color<T1>>);
"#,
        );
    }

    #[test]
    fn struct_into_impl_with_self_type() {
        check_assist(
            add_generic_parameter,
            r#"
struct Color$0(u8);
impl Color { fn new() -> Self { Color(0) }}
impl Color { fn replace1(self: &Self) -> Self { Color(1) }}
"#,
            r#"
struct Color<T1>(u8);
impl<T1> Color<T1> { fn new() -> Self { Color(0) }}
impl<T1> Color<T1> { fn replace1(self: &Self) -> Self { Color(1) }}
"#,
        );
    }

    /*
    Traits don't play super nicely
    There are a few options here:
    1. <T1> used only in Color's impl
        trait HasChannel{ type ChannelTy; fn channel(&self) -> &Self::ChannelTy; }
    2. <T1> used in both fn, type alias
        trait HasChannel{ type ChannelTy<T1>; fn channel<T1>(&self) -> &Self::ChannelTy<T1>; }
    3. <T1> used in both trait, type alias
        trait HasChannel<T1>{ type ChannelTy<T1>; fn channel(&self) -> &Self::ChannelTy<T1>; }

    For the non-trait analog, we prefer adding <T1> to the inner HasGenericParams.
    In this case, this analogs as preferring 2. over 3.

    1. seems to be the most likely case because adt's use their parts inside
    of impls and trait impls, and if their part adds a generic,
    then that generic gets added inside of the trait body, which means
    that it can access the outer trait generic param.

    Maybe when usages are detected, it should try to see if there is a surrounding
    definition to take the generic params from.
    */
    #[allow(unused)]
    fn struct_into_trait_impl_with_type_alias_option_1() {
        check_assist(
            add_generic_parameter,
            r#"
struct Channel$0(u8);
struct Color(Channel);
trait HasChannel{ type ChannelTy; fn channel(&self) -> &Self::ChannelTy; }
impl HasChannel for Color { type ChannelTy = Channel;
    fn channel(&self) -> &Self::ChannelTy { &self.0 } }
"#,
            r#"
struct Channel<T1>(u8);
struct Color<T1>(Channel<T1>);
trait HasChannel{ type ChannelTy<T1>; fn channel<T1>(&self) -> &Self::ChannelTy<T1>; }
impl<T1> HasChannel for Color<T1> { type ChannelTy<T1> = Channel<T1>;
    fn channel<T1>(&self) -> &Self::ChannelTy<T1> { &self.0 } }
"#,
        );
    }
    #[test]
    fn struct_into_trait_impl_with_type_alias_option_2() {
        check_assist(
            add_generic_parameter,
            r#"
struct Channel$0(u8);
struct Color(Channel);
trait HasChannel{ type ChannelTy; fn channel(&self) -> &Self::ChannelTy; }
impl HasChannel for Color { type ChannelTy = Channel;
    fn channel(&self) -> &Self::ChannelTy { &self.0 } }
"#,
            r#"
struct Channel<T1>(u8);
struct Color<T1>(Channel<T1>);
trait HasChannel{ type ChannelTy; fn channel(&self) -> &Self::ChannelTy; }
impl<T1> HasChannel for Color<T1> { type ChannelTy = Channel<T1>;
    fn channel(&self) -> &Self::ChannelTy { &self.0 } }
"#,
        );
    }

    #[test]
    fn function_basic() {
        check_assist(add_generic_parameter, r#"fn func$0(){}"#, r#"fn func<T1>(){}"#);
        check_assist(add_generic_parameter, r#"fn func$0<T2>(){}"#, r#"fn func<T2, T1>(){}"#);
    }

    #[test]
    fn struct_into_function_return() {
        check_assist(
            add_generic_parameter,
            r#"
struct Color$0 {}
fn func() -> Color {42}
"#,
            r#"
struct Color<T1> {}
fn func<T1>() -> Color<T1> {42}
"#,
        );
    }
    #[test]
    fn struct_into_function_param() {
        check_assist(
            add_generic_parameter,
            r#"
struct Color$0 {}
fn func(c: Color) {42}
"#,
            r#"
struct Color<T1> {}
fn func<T1>(c: Color<T1>) {42}
"#,
        );
    }

    /*
    Inside an expression, do not add any generic parameters.
    This is because they are usually inferred.
    This is helped along if the type is mentioned in the function signature
    so that type inference can work from the params/return type.

    Maybe in the future this could be changed.
    */
    #[test]
    fn struct_in_expr() {
        check_assist(
            add_generic_parameter,
            r#"
struct Color$0 { x: u32 }
fn func() { Color{x: 42} }
fn func() { let Color{x: y} = 42; }
fn func() { Color::red(42) }
fn func() { const_func::<{Color{x: 9}}> }
"#,
            r#"
struct Color<T1> { x: u32 }
fn func() { Color{x: 42} }
fn func() { let Color{x: y} = 42; }
fn func() { Color::red(42) }
fn func() { const_func::<{Color{x: 9}}> }
"#,
        );
    }

    #[test]
    fn struct_in_const_type() {
        check_assist(
            add_generic_parameter,
            r#"
struct Color$0 { x: u32 }
fn func<const C: Color>() {}
"#,
            r#"
struct Color<T1> { x: u32 }
fn func<const C: Color<T1>, T1>() {}
"#,
        );
    }

    // FIXME: make these type ascription cases work, possibly adding <_> as a generic param
    #[test]
    fn type_ascription_in_fn() {
        check_assist(
            add_generic_parameter,
            r#"
struct Color$0 { x: u32 }
fn func() { let c: Color; }
"#,
            r#"
struct Color<T1> { x: u32 }
fn func() { let c: Color; }
"#,
        );
    }
    #[test]
    fn type_ascription_in_fn_in_trait() {
        check_assist(
            add_generic_parameter,
            r#"
struct Color$0 { x: u32 }
trait Thing{ fn func() { let c: Color; } }
"#,
            r#"
struct Color<T1> { x: u32 }
trait Thing{ fn func() { let c: Color; } }
"#,
        );
    }
    #[test]
    fn duplicate_parameter_name() {
        // FIXME: Make this use a name other than T1 when there exists a parameter with the same name already.
        check_assist(
            add_generic_parameter,
            r#"
struct Color$0<T1> {}
"#,
            r#"
struct Color<T1, T1> {}
"#,
        );
    }

    #[test]
    fn duplicate_parameter_name_upvar() {
        // FIXME: Make this use a name other than T1 when there exists a (possibly upvar) parameter with the same name already.
        check_assist(
            add_generic_parameter,
            r#"
struct Thing;
trait DoStuff{ fn stuff<T1>(hello: Thing$0); }
"#,
            r#"
struct Thing<T1>;
trait DoStuff<T1>{ fn stuff<T1>(hello: Thing<T1>); }
"#,
        );
    }
    #[test]
    fn use_alias_in_different_modules() {
        check_assist(
            add_generic_parameter,
            r#"
struct ColorImpl$0(u8);
use ColorImpl as Color;
struct Image(Vec<Color>);
mod tmp{ use super::ColorImpl as Color2; struct Image2(Vec<Color2>); }
"#,
            r#"
struct ColorImpl<T1>(u8);
use ColorImpl as Color;
struct Image<T1>(Vec<Color<T1>>);
mod tmp{ use super::ColorImpl as Color2; struct Image2<T1>(Vec<Color2<T1>>); }
"#,
        );
    }
    #[test]
    fn non_generic_impl_has_fn_returns_generic() {
        check_assist(
            add_generic_parameter,
            r#"
struct Color$0(u8);
struct MakeColor;
impl MakeColor { fn make() -> Color { Color(0) }}
"#,
            r#"
struct Color<T1>(u8);
struct MakeColor;
impl MakeColor { fn make<T1>() -> Color<T1> { Color(0) }}
"#,
        );
    }
    #[test]
    fn generic_impl_has_fn_returns_generic() {
        check_assist(
            add_generic_parameter,
            r#"
struct Color$0(u8);
struct MakeColor2(Color);
impl MakeColor2 { fn make() -> Color { Color(0) }}
"#,
            r#"
struct Color<T1>(u8);
struct MakeColor2<T1>(Color<T1>);
impl<T1> MakeColor2<T1> { fn make() -> Color<T1> { Color(0) }}
"#,
        );
    }
    mod realer_cases {
        use super::*;
        #[test]

        fn impl_from() {
            check_assist(
                add_generic_parameter,
                r#"
struct Color { channels: [Channel; 3] }
struct Channel$0(u8);
impl From<Channel> for Color {
    fn from(x: Channel){ Color{ channels: [x.clone(); 3] } } }
"#,
                r#"
struct Color<T1> { channels: [Channel<T1>; 3] }
struct Channel<T1>(u8);
impl<T1> From<Channel<T1>> for Color<T1> {
    fn from(x: Channel<T1>){ Color{ channels: [x.clone(); 3] } } }
"#,
            );
        }
        #[test]
        fn impl_deref() {
            // example from crates/hir/src/semantics.rs
            check_assist(
                add_generic_parameter,
                r#"
struct TraitId$0;
pub struct VisibleTraits(pub FxHashSet<TraitId>);
impl ops::Deref for VisibleTraits {
    type Target = FxHashSet<TraitId>;
    fn deref(&self) -> &Self::Target { &self.0 } }
"#,
                r#"
struct TraitId<T1>;
pub struct VisibleTraits<T1>(pub FxHashSet<TraitId<T1>>);
impl<T1> ops::Deref for VisibleTraits<T1> {
    type Target = FxHashSet<TraitId<T1>>;
    fn deref(&self) -> &Self::Target { &self.0 } }
"#,
            );
        }
        /*
        This tests specifically against an implementation that orders adding the generics
        by the depth of the ast. This is one implementation that I tried.
        Box<Expr> is nested deep enough that it would get visited after
        fn new(x: Expr)

        which means that new(x: Expr) does not see the surrounding impl with generics
        so it adds generics to the `new` function.

        A correct implementation would order checks by waiting for the impl to possibly be analyzed.
        */
        #[test]
        fn differently_nested_things() {
            check_assist(
                add_generic_parameter,
                r#"
struct Expr$0;
impl ExprParen { fn new(x: Expr){} }
struct ExprParen { x: Box<Expr> }
"#,
                r#"
struct Expr<T1>;
impl<T1> ExprParen<T1> { fn new(x: Expr<T1>){} }
struct ExprParen<T1> { x: Box<Expr<T1>> }
"#,
            );
        }
        /*
           with the above in mind, is it possible to create a cyclic dependency?
           I wasn't able to, but these should hopefully be weird cases
        */

        #[test]
        fn strange_impl_dependencies() {
            check_assist(
                add_generic_parameter,
                r#"
struct Thing$0;
impl HasFood for Wrap { type Food = Thing; }
impl Lunch { fn new(_: Thing){} }
struct Lunch(<Wrap as HasFood>::Food);
struct Wrap(Thing);
"#,
                r#"
struct Thing<T1>;
impl<T1> HasFood for Wrap<T1> { type Food = Thing<T1>; }
impl<T1> Lunch<T1> { fn new(_: Thing<T1>){} }
struct Lunch<T1>(<Wrap<T1> as HasFood>::Food);
struct Wrap<T1>(Thing<T1>);
"#,
            );
        }
        #[test]
        fn strange_impl_dependencies2() {
            check_assist(
                add_generic_parameter,
                r#"
struct Thing$0;
impl HasFood for Wrap { type Food = Thing; }
impl Lunch { fn new(_: Thing){} }
struct Lunch;
struct Wrap(Thing);
"#,
                r#"
struct Thing<T1>;
impl<T1> HasFood for Wrap<T1> { type Food = Thing<T1>; }
impl Lunch { fn new<T1>(_: Thing<T1>){} }
struct Lunch;
struct Wrap<T1>(Thing<T1>);
"#,
            );
        }

        #[test]
        fn strange_impl_dependencies3() {
            check_assist(
                add_generic_parameter,
                r#"
struct Blue$0;
trait HasColor{ type Color; }
struct Sky; impl HasColor for Sky{
    type Color = Blue; }
struct Ocean; impl HasColor for Ocean {
    type Color = <Sky as HasColor>::Color; }
"#,
                r#"
struct Blue<T1>;
trait HasColor{ type Color<T1>; }
struct Sky; impl HasColor for Sky{
    type Color<T1> = Blue<T1>; }
struct Ocean; impl HasColor for Ocean {
    type Color<T1> = <Sky as HasColor>::Color<T1>; }
"#,
            );
        }
    }
    // FIXME: add more not_applicable cases
    #[test]
    fn na_fn_part_0() {
        check_assist_not_applicable(add_generic_parameter, r#"fn f(){$0}"#);
    }
    #[test]
    fn na_fn_part_1() {
        check_assist_not_applicable(add_generic_parameter, r#"fn f()$0{}"#);
    }
    #[test]
    fn na_fn_part_2() {
        check_assist_not_applicable(add_generic_parameter, r#"fn f($0){}"#);
    }
    #[test]
    fn na_fn_part_3() {
        check_assist_not_applicable(add_generic_parameter, r#"fn$0 f(){}"#);
    }
    #[test]
    fn na_fn_part_4() {
        check_assist_not_applicable(add_generic_parameter, r#"f$0n f(){}"#);
    }
    #[test]
    fn na_fn_part_5() {
        check_assist_not_applicable(add_generic_parameter, r#"$0fn f(){}"#);
    }
}
