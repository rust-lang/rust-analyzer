use std::sync::Arc;

use base_db::CrateId;
use cfg::CfgOptions;
use drop_bomb::DropBomb;
use hir_expand::{
    ast_id_map::AstIdMap, attrs::RawAttrs, hygiene::Hygiene, mod_path::ModPath, AstId, ExpandError,
    ExpandResult, HirFileId, InFile, MacroCallId, UnresolvedMacro,
};
use limit::Limit;
use once_cell::unsync::OnceCell;
use syntax::{ast, Parse, SyntaxNode};

use crate::{
    attr::Attrs, db::ItemTreeDatabase, macro_id_to_def_id, path::Path, AsMacroCall, MacroId,
    ModuleId,
};

pub struct LowerCtx<'a> {
    pub db: &'a dyn ItemTreeDatabase,
    hygiene: Hygiene,
    ast_id_map: Option<(HirFileId, OnceCell<Arc<AstIdMap>>)>,
}

impl<'a> LowerCtx<'a> {
    pub fn new(db: &'a dyn ItemTreeDatabase, hygiene: &Hygiene, file_id: HirFileId) -> Self {
        LowerCtx { db, hygiene: hygiene.clone(), ast_id_map: Some((file_id, OnceCell::new())) }
    }

    pub fn with_file_id(db: &'a dyn ItemTreeDatabase, file_id: HirFileId) -> Self {
        LowerCtx {
            db,
            hygiene: Hygiene::new(db.upcast(), file_id),
            ast_id_map: Some((file_id, OnceCell::new())),
        }
    }

    pub fn with_hygiene(db: &'a dyn ItemTreeDatabase, hygiene: &Hygiene) -> Self {
        LowerCtx { db, hygiene: hygiene.clone(), ast_id_map: None }
    }

    pub(crate) fn hygiene(&self) -> &Hygiene {
        &self.hygiene
    }

    pub(crate) fn lower_path(&self, ast: ast::Path) -> Option<Path> {
        Path::from_src(ast, self)
    }

    pub(crate) fn ast_id<N: syntax::AstNode>(&self, item: &N) -> Option<AstId<N>> {
        let &(file_id, ref ast_id_map) = self.ast_id_map.as_ref()?;
        let ast_id_map = ast_id_map.get_or_init(|| self.db.ast_id_map(file_id));
        Some(InFile::new(file_id, ast_id_map.ast_id(item)))
    }
}

/// A subset of Expander that only deals with cfg attributes. We only need it to
/// avoid cyclic queries in crate def map during enum processing.
#[derive(Debug)]
pub(crate) struct CfgExpander {
    cfg_options: CfgOptions,
    hygiene: Hygiene,
    krate: CrateId,
}

#[derive(Debug)]
pub struct Expander {
    cfg_expander: CfgExpander,
    pub(crate) current_file_id: HirFileId,
    pub(crate) module: ModuleId,
    /// `recursion_depth == usize::MAX` indicates that the recursion limit has been reached.
    recursion_depth: u32,
    recursion_limit: Limit,
}

impl CfgExpander {
    pub(crate) fn new(
        db: &dyn ItemTreeDatabase,
        current_file_id: HirFileId,
        krate: CrateId,
    ) -> CfgExpander {
        let hygiene = Hygiene::new(db.upcast(), current_file_id);
        let cfg_options = db.crate_graph()[krate].cfg_options.clone();
        CfgExpander { cfg_options, hygiene, krate }
    }

    pub(crate) fn parse_attrs(
        &self,
        db: &dyn ItemTreeDatabase,
        owner: &dyn ast::HasAttrs,
    ) -> Attrs {
        Attrs::filter(db, self.krate, RawAttrs::new(db.upcast(), owner, &self.hygiene))
    }

    pub(crate) fn is_cfg_enabled(
        &self,
        db: &dyn ItemTreeDatabase,
        owner: &dyn ast::HasAttrs,
    ) -> bool {
        let attrs = self.parse_attrs(db, owner);
        attrs.is_cfg_enabled(&self.cfg_options)
    }

    pub(crate) fn hygiene(&self) -> &Hygiene {
        &self.hygiene
    }
}

impl Expander {
    pub fn new(
        db: &dyn ItemTreeDatabase,
        current_file_id: HirFileId,
        module: ModuleId,
        recursion_limit: u32,
    ) -> Expander {
        let cfg_expander = CfgExpander::new(db, current_file_id, module.krate);
        #[cfg(not(test))]
        let recursion_limit = Limit::new(recursion_limit as usize);
        // Without this, `body::tests::your_stack_belongs_to_me` stack-overflows in debug
        #[cfg(test)]
        let recursion_limit = Limit::new(std::cmp::min(32, recursion_limit as usize));
        Expander { cfg_expander, current_file_id, module, recursion_depth: 0, recursion_limit }
    }

    pub fn enter_expand<T: ast::AstNode>(
        &mut self,
        db: &dyn ItemTreeDatabase,
        macro_call: ast::MacroCall,
        resolver: impl Fn(ModPath) -> Option<MacroId>,
    ) -> Result<ExpandResult<Option<(Mark, Parse<T>)>>, UnresolvedMacro> {
        // FIXME: within_limit should support this, instead of us having to extract the error
        let mut unresolved_macro_err = None;

        let result = self.within_limit(db, |this| {
            let macro_call = InFile::new(this.current_file_id, &macro_call);
            match macro_call.as_call_id_with_errors(db.upcast(), this.module.krate(), |path| {
                resolver(path).map(|it| macro_id_to_def_id(db, it))
            }) {
                Ok(call_id) => call_id,
                Err(resolve_err) => {
                    unresolved_macro_err = Some(resolve_err);
                    ExpandResult { value: None, err: None }
                }
            }
        });

        if let Some(err) = unresolved_macro_err {
            Err(err)
        } else {
            Ok(result)
        }
    }

    pub fn enter_expand_id<T: ast::AstNode>(
        &mut self,
        db: &dyn ItemTreeDatabase,
        call_id: MacroCallId,
    ) -> ExpandResult<Option<(Mark, Parse<T>)>> {
        self.within_limit(db, |_this| ExpandResult::ok(Some(call_id)))
    }

    fn enter_expand_inner(
        db: &dyn ItemTreeDatabase,
        call_id: MacroCallId,
        error: Option<ExpandError>,
    ) -> ExpandResult<Option<InFile<Parse<SyntaxNode>>>> {
        let file_id = call_id.as_file();
        let ExpandResult { value, err } = db.parse_or_expand_with_err(file_id);

        ExpandResult { value: Some(InFile::new(file_id, value)), err: error.or(err) }
    }

    pub fn exit(&mut self, db: &dyn ItemTreeDatabase, mut mark: Mark) {
        self.cfg_expander.hygiene = Hygiene::new(db.upcast(), mark.file_id);
        self.current_file_id = mark.file_id;
        if self.recursion_depth == u32::MAX {
            // Recursion limit has been reached somewhere in the macro expansion tree. Reset the
            // depth only when we get out of the tree.
            if !self.current_file_id.is_macro() {
                self.recursion_depth = 0;
            }
        } else {
            self.recursion_depth -= 1;
        }
        mark.bomb.defuse();
    }

    pub fn ctx<'a>(&self, db: &'a dyn ItemTreeDatabase) -> LowerCtx<'a> {
        LowerCtx::new(db, &self.cfg_expander.hygiene, self.current_file_id)
    }

    pub(crate) fn to_source<T>(&self, value: T) -> InFile<T> {
        InFile { file_id: self.current_file_id, value }
    }

    pub(crate) fn parse_attrs(
        &self,
        db: &dyn ItemTreeDatabase,
        owner: &dyn ast::HasAttrs,
    ) -> Attrs {
        self.cfg_expander.parse_attrs(db, owner)
    }

    pub(crate) fn cfg_options(&self) -> &CfgOptions {
        &self.cfg_expander.cfg_options
    }

    pub fn current_file_id(&self) -> HirFileId {
        self.current_file_id
    }

    pub(crate) fn parse_path(
        &mut self,
        db: &dyn ItemTreeDatabase,
        path: ast::Path,
    ) -> Option<Path> {
        let ctx = LowerCtx::with_hygiene(db, &self.cfg_expander.hygiene);
        Path::from_src(path, &ctx)
    }

    fn within_limit<F, T: ast::AstNode>(
        &mut self,
        db: &dyn ItemTreeDatabase,
        op: F,
    ) -> ExpandResult<Option<(Mark, Parse<T>)>>
    where
        F: FnOnce(&mut Self) -> ExpandResult<Option<MacroCallId>>,
    {
        if self.recursion_depth == u32::MAX {
            // Recursion limit has been reached somewhere in the macro expansion tree. We should
            // stop expanding other macro calls in this tree, or else this may result in
            // exponential number of macro expansions, leading to a hang.
            //
            // The overflow error should have been reported when it occurred (see the next branch),
            // so don't return overflow error here to avoid diagnostics duplication.
            cov_mark::hit!(overflow_but_not_me);
            return ExpandResult::only_err(ExpandError::RecursionOverflowPoisoned);
        } else if self.recursion_limit.check(self.recursion_depth as usize + 1).is_err() {
            self.recursion_depth = u32::MAX;
            cov_mark::hit!(your_stack_belongs_to_me);
            return ExpandResult::only_err(ExpandError::Other(
                "reached recursion limit during macro expansion".into(),
            ));
        }

        let ExpandResult { value, err } = op(self);
        let Some(call_id) = value else {
            return ExpandResult { value: None, err };
        };

        Self::enter_expand_inner(db, call_id, err).map(|value| {
            value.and_then(|InFile { file_id, value }| {
                let parse = value.cast::<T>()?;

                self.recursion_depth += 1;
                self.cfg_expander.hygiene = Hygiene::new(db.upcast(), file_id);
                let old_file_id = std::mem::replace(&mut self.current_file_id, file_id);
                let mark =
                    Mark { file_id: old_file_id, bomb: DropBomb::new("expansion mark dropped") };
                Some((mark, parse))
            })
        })
    }
}

#[derive(Debug)]
pub struct Mark {
    file_id: HirFileId,
    bomb: DropBomb,
}
