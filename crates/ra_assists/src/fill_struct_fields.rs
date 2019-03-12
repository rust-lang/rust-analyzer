use std::fmt::Write;

use hir::{
    AdtDef, Ty, FieldSource, source_binder,
    db::HirDatabase,
};
use ra_syntax::ast::{self, AstNode};

use crate::{AssistCtx, Assist, AssistId};

pub(crate) fn fill_match_arms(mut ctx: AssistCtx<impl HirDatabase>) -> Option<Assist> {
    unimplmented!()
}
