//! Machinery for hygienic macros.
//!
//! Inspired by Matthew Flatt et al., “Macros That Work Together: Compile-Time Bindings, Partial
//! Expansion, and Definition Contexts,” *Journal of Functional Programming* 22, no. 2
//! (March 1, 2012): 181–216, <https://doi.org/10.1017/S0956796812000093>.
//!
//! Also see <https://rustc-dev-guide.rust-lang.org/macro-expansion.html#hygiene-and-hierarchies>
//!
//! # The Expansion Order Hierarchy
//!
//! `ExpnData` in rustc, rust-analyzer's version is [`MacroCallLoc`]. Traversing the hierarchy
//! upwards can be achieved by walking up [`MacroCallLoc::kind`]'s contained file id, as
//! [`MacroFile`]s are interned [`MacroCallLoc`]s.
//!
//! # The Macro Definition Hierarchy
//!
//! `SyntaxContextData` in rustc and rust-analyzer. Basically the same in both.
//!
//! # The Call-site Hierarchy
//!
//! `ExpnData::call_site` in rustc, [`MacroCallLoc::call_site`] in rust-analyzer.
use std::fmt;

#[cfg(not(feature = "ra-salsa"))]
use crate::InternId;
#[cfg(feature = "ra-salsa")]
use ra_salsa::{InternId, InternValue};

use crate::{Edition, MacroCallId};

// /// Interned [`SyntaxContextData`].
// #[derive(Clone, Copy, Eq, PartialOrd, Ord)]
// pub struct SyntaxContextId(pub salsa::Id);

// impl std::hash::Hash for SyntaxContextId {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.0.hash(state);
//     }
// }

// impl PartialEq for SyntaxContextId {
//     fn eq(&self, other: &Self) -> bool {
//         let self_underlying = self.0.as_u32() & ((1 << 23) - 1);
//         let other_underlying = other.0.as_u32() & ((1 << 23) - 1);

//         self_underlying == other_underlying
//     }
// }

// impl salsa::plumbing::AsId for SyntaxContextId {
//     fn as_id(&self) -> salsa::Id {
//         self.0
//     }
// }

// impl salsa::plumbing::FromId for SyntaxContextId {
//     fn from_id(id: salsa::Id) -> Self {
//         SyntaxContextId(id)
//     }
// }

// impl fmt::Debug for SyntaxContextId {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         if f.alternate() {
//             write!(f, "{:?}", self.0)
//         } else {
//             f.debug_tuple("SyntaxContextId").field(&self.0).finish()
//         }
//     }
// }

// impl fmt::Display for SyntaxContextId {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.0.as_u32())
//     }
// }

// impl SyntaxContextId {

//     pub fn is_root(self) -> bool {
//         self == Self::ROOT
//     }

//     /// Deconstruct a `SyntaxContextId` into a raw `u32`.
//     /// This should only be used for deserialization purposes for the proc-macro server.
//     pub fn into_u32(self) -> u32 {
//         self.0.as_u32()
//     }

//     /// Constructs a `SyntaxContextId` from a raw `u32`.
//     /// This should only be used for serialization purposes for the proc-macro server.
//     pub fn from_u32(u32: u32) -> Self {
//         Self(salsa::Id::from_u32(u32))
//     }
// }

// Recursive expansion of interned macro
// ======================================

/// A syntax context describes a hierarchy tracking order of macro definitions.
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct SyntaxContext(
    salsa::Id,
    std::marker::PhantomData<&'static salsa::plumbing::interned::Value<SyntaxContext>>,
);

#[allow(warnings)]
const _: () = {
    use salsa::plumbing as zalsa_;
    use zalsa_::interned as zalsa_struct_;
    type Configuration_ = SyntaxContext;
    #[derive(Clone, Eq)]
    pub struct StructData(
        Option<MacroCallId>,
        Transparency,
        SyntaxContext,
        SyntaxContext,
        SyntaxContext,
    );

    impl PartialEq for StructData {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0 && self.1 == other.1 && self.2 == other.2
        }
    }

    impl std::hash::Hash for StructData {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.0.hash(state);
            self.1.hash(state);
            self.2.hash(state);
        }
    }
    #[doc = r" Key to use during hash lookups. Each field is some type that implements `Lookup<T>`"]
    #[doc = r" for the owned type. This permits interning with an `&str` when a `String` is required and so forth."]
    #[derive(Hash)]
    struct StructKey<'db, T0, T1, T2>(T0, T1, T2, std::marker::PhantomData<&'db ()>);

    impl<'db, T0, T1, T2> zalsa_::interned::HashEqLike<StructKey<'db, T0, T1, T2>> for StructData
    where
        Option<MacroCallId>: zalsa_::interned::HashEqLike<T0>,
        Transparency: zalsa_::interned::HashEqLike<T1>,
        SyntaxContext: zalsa_::interned::HashEqLike<T2>,
    {
        fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
            zalsa_::interned::HashEqLike::<T0>::hash(&self.0, &mut *h);
            zalsa_::interned::HashEqLike::<T1>::hash(&self.1, &mut *h);
            zalsa_::interned::HashEqLike::<T2>::hash(&self.2, &mut *h);
        }
        fn eq(&self, data: &StructKey<'db, T0, T1, T2>) -> bool {
            (zalsa_::interned::HashEqLike::<T0>::eq(&self.0, &data.0)
                && zalsa_::interned::HashEqLike::<T1>::eq(&self.1, &data.1)
                && zalsa_::interned::HashEqLike::<T2>::eq(&self.2, &data.2)
                && true)
        }
    }
    impl zalsa_struct_::Configuration for Configuration_ {
        const DEBUG_NAME: &'static str = "SyntaxContextData";
        type Data<'a> = StructData;
        type Struct<'a> = SyntaxContext;
        fn struct_from_id<'db>(id: salsa::Id) -> Self::Struct<'db> {
            SyntaxContext(id, std::marker::PhantomData)
        }
        fn deref_struct(s: Self::Struct<'_>) -> salsa::Id {
            s.0
        }
    }
    impl Configuration_ {
        pub fn ingredient<Db>(db: &Db) -> &zalsa_struct_::IngredientImpl<Self>
        where
            Db: ?Sized + zalsa_::Database,
        {
            static CACHE: zalsa_::IngredientCache<zalsa_struct_::IngredientImpl<Configuration_>> =
                zalsa_::IngredientCache::new();
            CACHE.get_or_create(db.as_dyn_database(), || {
                db.zalsa()
                    .add_or_lookup_jar_by_type(&<zalsa_struct_::JarImpl<Configuration_>>::default())
            })
        }
    }
    impl zalsa_::AsId for SyntaxContext {
        fn as_id(&self) -> salsa::Id {
            self.0
        }
    }
    impl zalsa_::FromId for SyntaxContext {
        fn from_id(id: salsa::Id) -> Self {
            Self(id, std::marker::PhantomData)
        }
    }
    unsafe impl Send for SyntaxContext {}

    unsafe impl Sync for SyntaxContext {}

    impl zalsa_::SalsaStructInDb for SyntaxContext {
        fn lookup_ingredient_index(
            aux: &dyn salsa::plumbing::JarAux,
        ) -> Option<salsa::IngredientIndex> {
            aux.lookup_jar_by_type(&zalsa_struct_::JarImpl::<Configuration_>::default())
        }
    }

    unsafe impl zalsa_::Update for SyntaxContext {
        unsafe fn maybe_update(old_pointer: *mut Self, new_value: Self) -> bool {
            if unsafe { *old_pointer } != new_value {
                unsafe { *old_pointer = new_value };
                true
            } else {
                false
            }
        }
    }
    impl<'db> SyntaxContext {
        pub fn new<
            Db_,
            T0: zalsa_::interned::Lookup<Option<MacroCallId>> + std::hash::Hash,
            T1: zalsa_::interned::Lookup<Transparency> + std::hash::Hash,
            T2: zalsa_::interned::Lookup<SyntaxContext> + std::hash::Hash,
        >(
            db: &'db Db_,
            outer_expn: T0,
            outer_transparency: T1,
            parent: T2,
            opaque: impl FnOnce(SyntaxContext) -> SyntaxContext,
            opaque_and_semitransparent: impl FnOnce(SyntaxContext) -> SyntaxContext,
        ) -> Self
        where
            Db_: ?Sized + salsa::Database,
            Option<MacroCallId>: zalsa_::interned::HashEqLike<T0>,
            Transparency: zalsa_::interned::HashEqLike<T1>,
            SyntaxContext: zalsa_::interned::HashEqLike<T2>,
        {
            let current_revision = zalsa_::current_revision(db);
            Configuration_::ingredient(db).intern(
                db.as_dyn_database(),
                StructKey::<'db>(
                    outer_expn,
                    outer_transparency,
                    parent,
                    std::marker::PhantomData::default(),
                ),
                |id, data| {
                    StructData(
                        zalsa_::interned::Lookup::into_owned(data.0),
                        zalsa_::interned::Lookup::into_owned(data.1),
                        zalsa_::interned::Lookup::into_owned(data.2),
                        opaque(zalsa_::FromId::from_id(id)),
                        opaque_and_semitransparent(zalsa_::FromId::from_id(id)),
                    )
                },
            )
        }

        /// Invariant: Only [`SyntaxContextId::ROOT`] has a [`None`] outer expansion.
        // FIXME: The None case needs to encode the context crate id. We can encode that as the MSB of
        // MacroCallId is reserved anyways so we can do bit tagging here just fine.
        // The bigger issue is that this will cause interning to now create completely separate chains
        // per crate. Though that is likely not a problem as `MacroCallId`s are already crate calling dependent.
        pub fn outer_expn<Db_>(self, db: &'db Db_) -> Option<MacroCallId>
        where
            Db_: ?Sized + zalsa_::Database,
        {
            if self.is_root() {
                return None;
            }
            let fields = Configuration_::ingredient(db).fields(db.as_dyn_database(), self);
            std::clone::Clone::clone((&fields.0))
        }
        pub fn outer_transparency<Db_>(self, db: &'db Db_) -> Transparency
        where
            Db_: ?Sized + zalsa_::Database,
        {
            if self.is_root() {
                return Transparency::Opaque;
            }
            let fields = Configuration_::ingredient(db).fields(db.as_dyn_database(), self);
            std::clone::Clone::clone((&fields.1))
        }
        pub fn parent<Db_>(self, db: &'db Db_) -> SyntaxContext
        where
            Db_: ?Sized + zalsa_::Database,
        {
            if self.is_root() {
                return self;
            }
            let fields = Configuration_::ingredient(db).fields(db.as_dyn_database(), self);
            std::clone::Clone::clone((&fields.2))
        }
        /// This context, but with all transparent and semi-transparent expansions filtered away.
        pub fn opaque<Db_>(self, db: &'db Db_) -> SyntaxContext
        where
            Db_: ?Sized + zalsa_::Database,
        {
            if self.is_root() {
                return self;
            }
            let fields = Configuration_::ingredient(db).fields(db.as_dyn_database(), self);
            std::clone::Clone::clone((&fields.3))
        }
        /// This context, but with all transparent expansions filtered away.
        pub fn opaque_and_semitransparent<Db_>(self, db: &'db Db_) -> SyntaxContext
        where
            Db_: ?Sized + zalsa_::Database,
        {
            if self.is_root() {
                return self;
            }
            let fields = Configuration_::ingredient(db).fields(db.as_dyn_database(), self);
            std::clone::Clone::clone((&fields.4))
        }
        #[doc = r" Default debug formatting for this struct (may be useful if you define your own `Debug` impl)"]
        pub fn default_debug_fmt(this: Self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            zalsa_::with_attached_database(|db| {
                let fields = Configuration_::ingredient(db).fields(db.as_dyn_database(), this);
                let mut f = f.debug_struct("SyntaxContextData");
                let f = f.field("outer_expn", &fields.0);
                let f = f.field("outer_transparency", &fields.1);
                let f = f.field("parent", &fields.2);
                let f = f.field("opaque", &fields.3);
                let f = f.field("opaque_and_semitransparent", &fields.4);
                f.finish()
            })
            .unwrap_or_else(|| {
                f.debug_tuple("SyntaxContextData")
                    .field(&zalsa_::AsId::as_id(&this))
                    .finish()
            })
        }
    }
};

impl SyntaxContext {
    pub fn is_root(self) -> bool {
        self.into_u32() <= Edition::LATEST as u32
    }
    /// The root context, which is the parent of all other contexts. All [`FileId`]s have this context.
    pub const ROOT: Self = SyntaxContext(
        salsa::Id::from_u32(salsa::Id::MAX_U32 - 1),
        std::marker::PhantomData,
    );

    pub fn into_u32(self) -> u32 {
        self.0.as_u32()
    }
    pub fn from_u32(u32: u32) -> Self {
        Self(salsa::Id::from_u32(u32), std::marker::PhantomData)
    }
}

/// A property of a macro expansion that determines how identifiers
/// produced by that expansion are resolved.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Hash, Debug)]
pub enum Transparency {
    /// Identifier produced by a transparent expansion is always resolved at call-site.
    /// Call-site spans in procedural macros, hygiene opt-out in `macro` should use this.
    Transparent,
    /// Identifier produced by a semi-transparent expansion may be resolved
    /// either at call-site or at definition-site.
    /// If it's a local variable, label or `$crate` then it's resolved at def-site.
    /// Otherwise it's resolved at call-site.
    /// `macro_rules` macros behave like this, built-in macros currently behave like this too,
    /// but that's an implementation detail.
    SemiTransparent,
    /// Identifier produced by an opaque expansion is always resolved at definition-site.
    /// Def-site spans in procedural macros, identifiers from `macro` by default use this.
    Opaque,
}

impl Transparency {
    /// Returns `true` if the transparency is [`Opaque`].
    ///
    /// [`Opaque`]: Transparency::Opaque
    pub fn is_opaque(&self) -> bool {
        matches!(self, Self::Opaque)
    }
}

impl fmt::Display for SyntaxContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.as_u32())
    }
}

impl std::fmt::Debug for SyntaxContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.0.as_u32())
        } else {
            f.debug_tuple("SyntaxContext").field(&self.0).finish()
        }
    }
}
