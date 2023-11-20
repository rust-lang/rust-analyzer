use std::{process::Command, collections::hash_map::DefaultHasher, time::Instant, env, path::Path, hash::{Hasher, Hash}, fs::File, io::Write};

use crate::{AssistContext, verus_error::*, tests::CHANHEE_VERUS_PATH};
use hir::Semantics;
use syntax::{
    ast::{self, vst, HasModuleItem, HasName},
    AstNode, SyntaxToken, SyntaxKind,
};

impl<'a> AssistContext<'a> {
    pub fn vst_path_from_text(&self, text: &str) -> Option<vst::Path> {
        let path = ast::make::path_from_text(text);
        let vst_path = vst::Path::try_from(path).ok()?;
        return Some(vst_path);
    }
}
