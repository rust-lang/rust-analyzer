//! Native Salsa file identifiers.
//!
//! This module defines file identification types used throughout rust-analyzer:
//!
//! - [`File`] - A file path interned via Salsa (replaces `vfs::FileId` + `PathInterner`)
//! - [`EditionedFileId`] - A file path + Rust edition, interned via Salsa
//!
//! The implementation is hand-rolled (rather than using `#[salsa::interned]`) to support:
//! - Constrained ID space (31 bits max, reserving MSB for `HirFileId` discrimination)
//! - Feature-gated Salsa support (non-Salsa builds get a stub implementation)
//!
//! This pattern follows [`SyntaxContext`](crate::SyntaxContext) in `hygiene.rs`.

use std::fmt;
use std::marker::PhantomData;

use vfs::VfsPath;

use crate::Edition;

// ============================================================================
// File - Interned file path (replaces vfs::FileId)
// ============================================================================

/// A file path interned via Salsa.
///
/// This replaces the old `vfs::FileId` + `PathInterner` combination with native
/// Salsa interning. The ID is constrained to 31 bits to reserve the MSB for
/// [`HirFileId`] discrimination.
///
/// [`HirFileId`]: crate::HirFileId
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct File(
    u32,
    #[cfg(feature = "salsa")]
    PhantomData<&'static salsa::plumbing::interned::Value<File>>,
    #[cfg(not(feature = "salsa"))]
    PhantomData<()>,
);

/// Maximum valid File ID (31 bits, MSB reserved for HirFileId discrimination)
const FILE_MAX_ID: u32 = 0x7FFF_FFFF;

#[cfg(feature = "salsa")]
const _: () = {
    use salsa::plumbing as zalsa_;
    use salsa::plumbing::interned as zalsa_struct_;

    /// The actual data stored for each interned [`File`].
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct FileData {
        pub path: VfsPath,
    }

    impl zalsa_::HasJar for File {
        type Jar = zalsa_struct_::JarImpl<File>;
        const KIND: zalsa_::JarKind = zalsa_::JarKind::Struct;
    }

    zalsa_::register_jar! {
        zalsa_::ErasedJar::erase::<File>()
    }

    impl zalsa_struct_::Configuration for File {
        const LOCATION: salsa::plumbing::Location =
            salsa::plumbing::Location { file: file!(), line: line!() };
        const DEBUG_NAME: &'static str = "File";
        const REVISIONS: std::num::NonZeroUsize = std::num::NonZeroUsize::MAX;
        const PERSIST: bool = false;

        type Fields<'a> = FileData;
        type Struct<'db> = File;

        fn serialize<S>(_: &Self::Fields<'_>, _: S) -> Result<S::Ok, S::Error>
        where
            S: zalsa_::serde::Serializer,
        {
            unimplemented!("File does not support persistence")
        }

        fn deserialize<'de, D>(_: D) -> Result<Self::Fields<'static>, D::Error>
        where
            D: zalsa_::serde::Deserializer<'de>,
        {
            unimplemented!("File does not support persistence")
        }
    }

    impl zalsa_::AsId for File {
        #[inline]
        fn as_id(&self) -> salsa::Id {
            unsafe { salsa::Id::from_index(self.0) }
        }
    }

    impl zalsa_::FromId for File {
        #[inline]
        fn from_id(id: salsa::Id) -> Self {
            debug_assert!(id.index() <= FILE_MAX_ID);
            Self(id.index(), PhantomData)
        }
    }

    unsafe impl Send for File {}
    unsafe impl Sync for File {}
    impl std::panic::UnwindSafe for File {}
    impl std::panic::RefUnwindSafe for File {}

    impl zalsa_::SalsaStructInDb for File {
        type MemoIngredientMap = salsa::plumbing::MemoIngredientSingletonIndex;

        fn lookup_ingredient_index(aux: &zalsa_::Zalsa) -> salsa::plumbing::IngredientIndices {
            aux.lookup_jar_by_type::<zalsa_struct_::JarImpl<File>>().into()
        }

        fn entries(zalsa: &zalsa_::Zalsa) -> impl Iterator<Item = zalsa_::DatabaseKeyIndex> + '_ {
            File::ingredient(zalsa).entries(zalsa).map(|e| e.key())
        }

        #[inline]
        fn cast(id: salsa::Id, type_id: std::any::TypeId) -> Option<Self> {
            if type_id == std::any::TypeId::of::<File>() {
                Some(<Self as zalsa_::FromId>::from_id(id))
            } else {
                None
            }
        }

        #[inline]
        unsafe fn memo_table(
            zalsa: &zalsa_::Zalsa,
            id: zalsa_::Id,
            current_revision: zalsa_::Revision,
        ) -> zalsa_::MemoTableWithTypes<'_> {
            unsafe { zalsa.table().memos::<zalsa_struct_::Value<File>>(id, current_revision) }
        }
    }

    unsafe impl zalsa_::Update for File {
        #[inline]
        unsafe fn maybe_update(old: *mut Self, new: Self) -> bool {
            if unsafe { *old } != new {
                unsafe { *old = new };
                true
            } else {
                false
            }
        }
    }

    impl File {
        fn ingredient(zalsa: &zalsa_::Zalsa) -> &zalsa_struct_::IngredientImpl<Self> {
            static CACHE: zalsa_::IngredientCache<zalsa_struct_::IngredientImpl<File>> =
                zalsa_::IngredientCache::new();
            unsafe {
                CACHE.get_or_create(zalsa, || {
                    zalsa.lookup_jar_by_type::<zalsa_struct_::JarImpl<File>>()
                })
            }
        }

        /// Creates a new [`File`] for the given path.
        pub fn new(db: &(impl salsa::Database + ?Sized), path: VfsPath) -> Self {
            let (zalsa, zalsa_local) = db.zalsas();
            let data = FileData { path };
            let id = Self::ingredient(zalsa).intern(zalsa, zalsa_local, data, |_, d| d);
            debug_assert!(id.0 <= FILE_MAX_ID, "File ID overflow: {} > {}", id.0, FILE_MAX_ID);
            id
        }

        /// Returns the path for this file.
        pub fn path(self, db: &dyn salsa::Database) -> &VfsPath {
            let zalsa = db.zalsa();
            &Self::ingredient(zalsa).fields(zalsa, self).path
        }
    }
};

impl File {
    /// Maximum valid ID (31 bits).
    pub const MAX_ID: u32 = FILE_MAX_ID;

    /// Returns the raw u32 representation of this ID.
    #[inline]
    pub const fn index(self) -> u32 {
        self.0
    }

    /// Creates a [`File`] from a raw u32 value.
    ///
    /// # Safety
    ///
    /// The raw value must be a valid interned [`File`].
    #[inline]
    pub const unsafe fn from_raw(raw: u32) -> Self {
        debug_assert!(raw <= FILE_MAX_ID);
        Self(raw, PhantomData)
    }
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "salsa")]
        {
            salsa::plumbing::with_attached_database(|db| {
                f.debug_struct("File").field("path", self.path(db)).finish()
            })
            .unwrap_or_else(|| f.debug_tuple("File").field(&self.0).finish())
        }
        #[cfg(not(feature = "salsa"))]
        {
            f.debug_tuple("File").field(&self.0).finish()
        }
    }
}

#[cfg(not(feature = "salsa"))]
impl File {
    pub fn new(_db: &dyn std::any::Any, _path: VfsPath) -> Self {
        panic!("File::new requires the 'salsa' feature")
    }

    pub fn path(self, _db: &dyn std::any::Any) -> ! {
        panic!("File::path requires the 'salsa' feature")
    }
}

// ============================================================================
// EditionedFileId - Interned file + edition
// ============================================================================

/// A file path and Rust edition, interned via Salsa.
///
/// This is the primary file identifier used throughout rust-analyzer.
/// The ID is constrained to 31 bits (max `0x7FFF_FFFF`) to reserve the MSB
/// for [`HirFileId`] discrimination between real files and macro expansions.
///
/// [`HirFileId`]: crate::HirFileId
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EditionedFileId(
    u32,
    #[cfg(feature = "salsa")]
    PhantomData<&'static salsa::plumbing::interned::Value<EditionedFileId>>,
    #[cfg(not(feature = "salsa"))]
    PhantomData<()>,
);

// Verify Salsa's max ID constant matches our expectation
#[cfg(feature = "salsa")]
const _: () = assert!(salsa::Id::MAX_U32 == u32::MAX - 0xFF);

/// Maximum valid ID (31 bits, MSB reserved for HirFileId discrimination)
const MAX_ID: u32 = 0x7FFF_FFFF;

#[cfg(feature = "salsa")]
const _: () = {
    use salsa::plumbing as zalsa_;
    use salsa::plumbing::interned as zalsa_struct_;

    /// The actual data stored for each interned [`EditionedFileId`].
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct EditionedFileIdData {
        pub file: File,
        pub edition: Edition,
    }

    impl zalsa_::HasJar for EditionedFileId {
        type Jar = zalsa_struct_::JarImpl<EditionedFileId>;
        const KIND: zalsa_::JarKind = zalsa_::JarKind::Struct;
    }

    zalsa_::register_jar! {
        zalsa_::ErasedJar::erase::<EditionedFileId>()
    }

    impl zalsa_struct_::Configuration for EditionedFileId {
        const LOCATION: salsa::plumbing::Location =
            salsa::plumbing::Location { file: file!(), line: line!() };
        const DEBUG_NAME: &'static str = "EditionedFileId";
        // Never garbage collect file IDs - they're stable for the session
        const REVISIONS: std::num::NonZeroUsize = std::num::NonZeroUsize::MAX;
        const PERSIST: bool = false;

        type Fields<'a> = EditionedFileIdData;
        type Struct<'db> = EditionedFileId;

        fn serialize<S>(_: &Self::Fields<'_>, _: S) -> Result<S::Ok, S::Error>
        where
            S: zalsa_::serde::Serializer,
        {
            unimplemented!("EditionedFileId does not support persistence")
        }

        fn deserialize<'de, D>(_: D) -> Result<Self::Fields<'static>, D::Error>
        where
            D: zalsa_::serde::Deserializer<'de>,
        {
            unimplemented!("EditionedFileId does not support persistence")
        }
    }

    impl zalsa_::AsId for EditionedFileId {
        #[inline]
        fn as_id(&self) -> salsa::Id {
            // SAFETY: Valid EditionedFileId always contains a valid Salsa ID
            // (we verify this constraint in `new()`)
            unsafe { salsa::Id::from_index(self.0) }
        }
    }

    impl zalsa_::FromId for EditionedFileId {
        #[inline]
        fn from_id(id: salsa::Id) -> Self {
            debug_assert!(id.index() <= MAX_ID, "EditionedFileId overflow: {}", id.index());
            Self(id.index(), PhantomData)
        }
    }

    unsafe impl Send for EditionedFileId {}
    unsafe impl Sync for EditionedFileId {}

    impl zalsa_::SalsaStructInDb for EditionedFileId {
        type MemoIngredientMap = salsa::plumbing::MemoIngredientSingletonIndex;

        fn lookup_ingredient_index(aux: &zalsa_::Zalsa) -> salsa::plumbing::IngredientIndices {
            aux.lookup_jar_by_type::<zalsa_struct_::JarImpl<EditionedFileId>>().into()
        }

        fn entries(zalsa: &zalsa_::Zalsa) -> impl Iterator<Item = zalsa_::DatabaseKeyIndex> + '_ {
            EditionedFileId::ingredient(zalsa).entries(zalsa).map(|e| e.key())
        }

        #[inline]
        fn cast(id: salsa::Id, type_id: std::any::TypeId) -> Option<Self> {
            if type_id == std::any::TypeId::of::<EditionedFileId>() {
                Some(<Self as zalsa_::FromId>::from_id(id))
            } else {
                None
            }
        }

        #[inline]
        unsafe fn memo_table(
            zalsa: &zalsa_::Zalsa,
            id: zalsa_::Id,
            current_revision: zalsa_::Revision,
        ) -> zalsa_::MemoTableWithTypes<'_> {
            unsafe {
                zalsa
                    .table()
                    .memos::<zalsa_struct_::Value<EditionedFileId>>(id, current_revision)
            }
        }
    }

    unsafe impl zalsa_::Update for EditionedFileId {
        #[inline]
        unsafe fn maybe_update(old: *mut Self, new: Self) -> bool {
            if unsafe { *old } != new {
                unsafe { *old = new };
                true
            } else {
                false
            }
        }
    }

    impl EditionedFileId {
        fn ingredient(zalsa: &zalsa_::Zalsa) -> &zalsa_struct_::IngredientImpl<Self> {
            static CACHE: zalsa_::IngredientCache<zalsa_struct_::IngredientImpl<EditionedFileId>> =
                zalsa_::IngredientCache::new();
            // SAFETY: `lookup_jar_by_type` returns a valid ingredient index
            unsafe {
                CACHE.get_or_create(zalsa, || {
                    zalsa.lookup_jar_by_type::<zalsa_struct_::JarImpl<EditionedFileId>>()
                })
            }
        }

        /// Creates a new [`EditionedFileId`] for the given file and edition.
        ///
        /// If an ID for this (file, edition) pair already exists, returns the existing ID.
        pub fn new(db: &(impl salsa::Database + ?Sized), file: File, edition: Edition) -> Self {
            let (zalsa, zalsa_local) = db.zalsas();
            let data = EditionedFileIdData { file, edition };
            let id = Self::ingredient(zalsa).intern(zalsa, zalsa_local, data, |_, d| d);
            debug_assert!(
                id.0 <= MAX_ID,
                "EditionedFileId overflow: {} > {}",
                id.0,
                MAX_ID
            );
            id
        }

        /// Creates a new [`EditionedFileId`] from a path and edition.
        ///
        /// This is a convenience method that interns the path first.
        pub fn from_path(
            db: &(impl salsa::Database + ?Sized),
            path: VfsPath,
            edition: Edition,
        ) -> Self {
            let file = File::new(db, path);
            Self::new(db, file, edition)
        }

        /// Returns the [`File`] (interned path) for this file.
        pub fn file(self, db: &dyn salsa::Database) -> File {
            let zalsa = db.zalsa();
            Self::ingredient(zalsa).fields(zalsa, self).file
        }

        /// Returns the path for this file.
        pub fn path(self, db: &dyn salsa::Database) -> &VfsPath {
            self.file(db).path(db)
        }

        /// Returns the Rust edition for this file.
        pub fn edition(self, db: &dyn salsa::Database) -> Edition {
            let zalsa = db.zalsa();
            Self::ingredient(zalsa).fields(zalsa, self).edition
        }

        /// Returns both file and edition.
        pub fn unpack(self, db: &dyn salsa::Database) -> (File, Edition) {
            let zalsa = db.zalsa();
            let fields = Self::ingredient(zalsa).fields(zalsa, self);
            (fields.file, fields.edition)
        }
    }
};

// Public API available in both salsa and non-salsa builds
impl EditionedFileId {
    /// Maximum valid ID (31 bits).
    ///
    /// The MSB is reserved for [`HirFileId`] to discriminate between
    /// real files and macro expansion files.
    pub const MAX_ID: u32 = MAX_ID;

    /// Returns the raw u32 representation of this ID.
    ///
    /// This is useful for packing into other structures like spans.
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Creates an [`EditionedFileId`] from a raw u32 value.
    ///
    /// # Safety
    ///
    /// The raw value must be a valid interned [`EditionedFileId`] that was
    /// previously created via [`EditionedFileId::new`].
    #[inline]
    pub const unsafe fn from_u32(raw: u32) -> Self {
        debug_assert!(raw <= MAX_ID);
        Self(raw, PhantomData)
    }

    /// Alias for [`from_u32`](Self::from_u32) for backwards compatibility.
    ///
    /// # Safety
    ///
    /// See [`from_u32`](Self::from_u32).
    #[inline]
    pub const unsafe fn from_raw(raw: u32) -> Self {
        // SAFETY: Caller guarantees the raw value is valid.
        unsafe { Self::from_u32(raw) }
    }

    /// Creates a dummy [`EditionedFileId`] for testing purposes.
    ///
    /// The returned ID is not valid for database lookups - it's only useful
    /// for creating dummy spans in tests.
    #[inline]
    pub const fn dummy(edition: Edition) -> Self {
        // Use a recognizable magic number (0xe4e4e) for the file ID
        // and pack it with the edition. Since this isn't a real interned ID,
        // any lookups will panic.
        const DUMMY_FILE_ID: u32 = 0xe4e4e;
        // Pack dummy file ID with edition marker
        let raw = DUMMY_FILE_ID | ((edition as u8 as u32) << 24);
        debug_assert!(raw <= MAX_ID);
        Self(raw, PhantomData)
    }
}

impl fmt::Debug for EditionedFileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "salsa")]
        {
            salsa::plumbing::with_attached_database(|db| {
                let (file, edition) = self.unpack(db);
                f.debug_struct("EditionedFileId")
                    .field("file", &file)
                    .field("edition", &edition)
                    .finish()
            })
            .unwrap_or_else(|| f.debug_tuple("EditionedFileId").field(&self.0).finish())
        }
        #[cfg(not(feature = "salsa"))]
        {
            f.debug_tuple("EditionedFileId").field(&self.0).finish()
        }
    }
}

// Non-salsa stub implementation
#[cfg(not(feature = "salsa"))]
impl EditionedFileId {
    /// Stub for non-salsa builds. Panics if called.
    pub fn new(_db: &dyn std::any::Any, _file: File, _edition: Edition) -> Self {
        panic!("EditionedFileId::new requires the 'salsa' feature")
    }

    /// Stub for non-salsa builds. Panics if called.
    pub fn from_path(_db: &dyn std::any::Any, _path: VfsPath, _edition: Edition) -> Self {
        panic!("EditionedFileId::from_path requires the 'salsa' feature")
    }

    /// Stub for non-salsa builds. Panics if called.
    pub fn file(self, _db: &dyn std::any::Any) -> ! {
        panic!("EditionedFileId::file requires the 'salsa' feature")
    }

    /// Stub for non-salsa builds. Panics if called.
    pub fn path(self, _db: &dyn std::any::Any) -> ! {
        panic!("EditionedFileId::path requires the 'salsa' feature")
    }

    /// Stub for non-salsa builds. Panics if called.
    pub fn edition(self, _db: &dyn std::any::Any) -> ! {
        panic!("EditionedFileId::edition requires the 'salsa' feature")
    }

    /// Stub for non-salsa builds. Panics if called.
    pub fn unpack(self, _db: &dyn std::any::Any) -> ! {
        panic!("EditionedFileId::unpack requires the 'salsa' feature")
    }
}
