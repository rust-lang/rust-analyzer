//! FIXME: write short doc here

use lasso::{Key, Spur, ThreadedRodeo};
use once_cell::sync::Lazy;
use std::{
    cmp::Ordering,
    fmt,
    hash::{BuildHasherDefault, Hash, Hasher},
    num::NonZeroU32,
};

use ra_syntax::{ast, SmolStr};

pub use crate::name;
use rustc_hash::FxHasher;

type Interner = ThreadedRodeo<Spur, BuildHasherDefault<FxHasher>>;

static INTERNER: Lazy<Interner> = Lazy::new(|| {
    let cap = lasso::Capacity::for_strings(KNOWN_NAMES.len());
    let interner = Interner::with_capacity_and_hasher(cap, BuildHasherDefault::default());
    for (i, name) in KNOWN_NAMES.iter().enumerate() {
        // FIXME: Use `get_or_intern_static` once it's available.
        let spur: Spur = interner.get_or_intern(name);
        assert_eq!(unsafe { spur.into_usize() }, i);
    }
    interner
});

/// `Name` is a wrapper around string, which is used in hir for both references
/// and declarations. In theory, names should also carry hygiene info, but we are
/// not there yet!
#[derive(Clone, PartialEq, Eq)]
pub struct Name(NonZeroU32);

/// Compares names lexicographically.
impl PartialOrd for Name {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let a = self.to_string();
        let b = other.to_string();
        a.partial_cmp(&b)
    }
}

impl Ord for Name {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = self.to_string();
        let b = other.to_string();
        a.cmp(&b)
    }
}

/// `Hash` impl looks at interned string to avoid dependency on interning order.
impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.repr() {
            Repr::Interned(spur) => {
                INTERNER.resolve(&spur).hash(state);
            }
            Repr::TupleField(field) => {
                field.hash(state);
            }
        }
    }
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.repr() {
            Repr::Interned(spur) => write!(f, "Name({:?})", INTERNER.resolve(&spur)),
            Repr::TupleField(field) => write!(f, "TupleField({})", field),
        }
    }
}

enum Repr {
    Interned(Spur),
    TupleField(u32),
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.repr() {
            Repr::Interned(spur) => fmt::Display::fmt(INTERNER.resolve(&spur), f),
            Repr::TupleField(idx) => fmt::Display::fmt(&idx, f),
        }
    }
}

impl Name {
    /// Note: this is private to make creating name from random string hard.
    /// Hopefully, this should allow us to integrate hygiene cleaner in the
    /// future, and to switch to interned representation of names.
    fn new_text(text: SmolStr) -> Name {
        let spur = INTERNER.get_or_intern(text.as_str());
        Self::from_repr(Repr::Interned(spur))
    }

    pub fn new_tuple_field(idx: usize) -> Name {
        assert!(idx <= u32::MAX as usize);
        Self::from_repr(Repr::TupleField(idx as u32))
    }

    pub fn new_lifetime(lt: &ra_syntax::SyntaxToken) -> Name {
        assert!(lt.kind() == ra_syntax::SyntaxKind::LIFETIME);
        Self::new_text(lt.text().clone())
    }

    /// Resolve a name from the text of token.
    fn resolve(raw_text: &SmolStr) -> Name {
        let raw_start = "r#";
        if raw_text.as_str().starts_with(raw_start) {
            Name::new_text(SmolStr::new(&raw_text[raw_start.len()..]))
        } else {
            Name::new_text(raw_text.clone())
        }
    }

    pub fn missing() -> Name {
        Name::new_text("[missing name]".into())
    }

    fn from_repr(repr: Repr) -> Self {
        /*
        Encoding:
                 0               reserved
                 1 - 0x7fffffff  interned string
        0x80000000 - 0xffffffff  tuple field
        */

        match repr {
            Repr::Interned(spur) => {
                // Seems to be unsafe by mistake.
                let spur = unsafe { spur.into_usize() } as u32;
                Self(NonZeroU32::new(spur + 1).unwrap())
            }
            Repr::TupleField(fld) => Self(NonZeroU32::new(fld + 0x80000000).unwrap()),
        }
    }

    const fn statik(index: u32) -> Self {
        unsafe { Self(NonZeroU32::new_unchecked(index + 1)) }
    }

    fn repr(&self) -> Repr {
        let i = u32::from(self.0);
        match i {
            0 => unreachable!(),
            1..=0x7fffffff => Repr::Interned(Spur::try_from_usize(i as usize - 1).unwrap()),
            0x80000000..=0xffffffff => Repr::TupleField(i - 0x80000000),
        }
    }

    pub fn as_tuple_index(&self) -> Option<usize> {
        match self.repr() {
            Repr::TupleField(idx) => Some(idx as usize),
            _ => None,
        }
    }
}

pub trait AsName {
    fn as_name(&self) -> Name;
}

impl AsName for ast::NameRef {
    fn as_name(&self) -> Name {
        match self.as_tuple_field() {
            Some(idx) => Name::new_tuple_field(idx),
            None => Name::resolve(self.text()),
        }
    }
}

impl AsName for ast::Name {
    fn as_name(&self) -> Name {
        Name::resolve(self.text())
    }
}

impl AsName for ast::NameOrNameRef {
    fn as_name(&self) -> Name {
        match self {
            ast::NameOrNameRef::Name(it) => it.as_name(),
            ast::NameOrNameRef::NameRef(it) => it.as_name(),
        }
    }
}

impl AsName for tt::Ident {
    fn as_name(&self) -> Name {
        Name::resolve(&self.text)
    }
}

impl AsName for ast::FieldKind {
    fn as_name(&self) -> Name {
        match self {
            ast::FieldKind::Name(nr) => nr.as_name(),
            ast::FieldKind::Index(idx) => {
                let idx = idx.text().parse::<usize>().unwrap_or(0);
                Name::new_tuple_field(idx)
            }
        }
    }
}

impl AsName for ra_db::Dependency {
    fn as_name(&self) -> Name {
        Name::new_text(SmolStr::new(&*self.name))
    }
}

macro_rules! known_name {
    ($ident:ident) => {
        stringify!($ident)
    };
    ($ident:ident = $string:literal) => {
        $string
    };
}

macro_rules! count {
    ( [ $current_count:expr ] [ $next_name:ident $($remaining_names:ident)* ] ) => {
        #[allow(bad_style)]
        pub const $next_name: super::Name = super::Name::statik($current_count);

        count!( [ $current_count + 1 ] [ $($remaining_names)* ] );
    };
    ( [ $current_count:expr ] [ ] ) => {};
}

macro_rules! known_names {
    ($($ident:ident $( = $string:literal )? ),* $(,)?) => {
        static KNOWN_NAMES: &[&str] = &[
            $( known_name!($ident $( = $string)?) ),*
        ];

        pub mod known {
            count!([0] [$($ident)*]);
        }
    };
}

known_names!(
    // Primitives
    isize,
    i8,
    i16,
    i32,
    i64,
    i128,
    usize,
    u8,
    u16,
    u32,
    u64,
    u128,
    f32,
    f64,
    bool,
    char,
    str,
    // Special names
    macro_rules,
    doc,
    // Components of known path (value or mod name)
    std,
    core,
    alloc,
    iter,
    ops,
    future,
    result,
    boxed,
    // Components of known path (type name)
    IntoIterator,
    Item,
    Try,
    Ok,
    Future,
    Result,
    Output,
    Target,
    Box,
    RangeFrom,
    RangeFull,
    RangeInclusive,
    RangeToInclusive,
    RangeTo,
    Range,
    Neg,
    Not,
    Index,
    // Builtin macros
    file,
    column,
    compile_error,
    line,
    assert,
    stringify,
    concat,
    include,
    include_bytes,
    include_str,
    format_args,
    format_args_nl,
    env,
    option_env,
    // Builtin derives
    Copy,
    Clone,
    Default,
    Debug,
    Hash,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    SELF_PARAM = "self",
    SELF_TYPE = "Self",
);

#[macro_export]
macro_rules! name {
    (self) => {
        $crate::name::known::SELF_PARAM
    };
    (Self) => {
        $crate::name::known::SELF_TYPE
    };
    ($ident:ident) => {
        $crate::name::known::$ident
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interner() {
        assert_eq!(name!(self).to_string(), "self");
        assert_eq!(name!(PartialEq).to_string(), "PartialEq");
    }
}
