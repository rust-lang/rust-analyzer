use ra_syntax::ast::{TokenTree, Attr};
use ra_syntax::SmolStr;
use crate::db::{DefDatabase, AstDatabase};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeriveData {
    traits: Vec<SmolStr>,
}

impl DeriveData {
    pub fn new(traits: Vec<SmolStr>) -> Self {
        DeriveData { traits }
    }

    pub fn new_from_tt(tt: TokenTree) -> Self {
        // FIXME: collect trait names separated by comma from token tree
        Self::new(vec![])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrData {
    Derive(DeriveData),
    MustUse(Option<SmolStr>),
    Unknown,
}

impl AttrData {
    pub fn new(attr: AttrKind) -> Self {
        match attr {
            AttrKind::Derive(tt) => AttrData::Derive(DeriveData::new_from_tt(tt)),
            AttrKind::MustUse(s) => AttrData::MustUse(s),
            _ => AttrData::Unknown,
        }
    }
}

pub enum AttrKind {
    Derive(TokenTree),
    MustUse(Option<SmolStr>),
    Other,
}

impl AttrKind {
    pub fn from_attr(attr: Attr) -> Self {
        if let Some((kind, tt)) = attr.as_call() {
            match kind.as_str() {
                "derive" => return AttrKind::Derive(tt),
                _ => {},
            }
        }

        if let Some(kind) = attr.as_atom() {
            match kind.as_str() {
                "must_use" => return AttrKind::MustUse(None),
                _ => {},
            }
        }

        if let Some((kind, value)) = attr.as_key_value() {
            match kind.as_str() {
                "must_use" => return AttrKind::MustUse(Some(value)),
                _ => {}
            }
        }

        AttrKind::Other
    }
}
