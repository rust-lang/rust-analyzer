#![allow(unused)]
#![recursion_limit = "512"]

macro_rules! impl_froms {
    ($e:ident: $($v:ident $(($($sv:ident),*))?),*) => {
        $(
            impl From<$v> for $e {
                fn from(it: $v) -> $e {
                    $e::$v(it)
                }
            }
            $($(
                impl From<$sv> for $e {
                    fn from(it: $sv) -> $e {
                        $e::$v($v::$sv(it))
                    }
                }
            )*)?
        )*
    }
}

mod name;
mod type_ref;
mod path;
mod attr;

mod source_id;
pub mod ids;
mod db;
mod db_ext;
mod source;
mod either;
mod diagnostics;

mod adt;
mod builtin_type;
mod nameres;

pub use crate::{
    builtin_type::{BuiltinType, FloatBitness, IntBitness, Signedness},
    either::Either,
    name::Name,
    path::{Path, PathKind},
    source::{ModuleSource, Source},
};
