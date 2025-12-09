//! Salsa-backed file identity and the compact representation stored in spans.

use std::{fmt, marker::PhantomData};

use vfs::VfsPath;

use crate::Edition;

const SALSA_RESERVED_ID_START: u32 = u32::MAX - 0xFF;

#[cfg(feature = "salsa")]
const _: () = assert!(SALSA_RESERVED_ID_START == salsa::Id::MAX_U32);

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct File(
    u32,
    #[cfg(feature = "salsa")] PhantomData<&'static salsa::plumbing::interned::Value<File>>,
    #[cfg(not(feature = "salsa"))] PhantomData<()>,
);

// Safe because `File` is represented by a single `u32` plus zero-sized markers.
impl nohash_hasher::IsEnabled for File {}

#[cfg(feature = "salsa")]
const _: () = {
    use salsa::plumbing as zalsa_;
    use salsa::plumbing::interned as zalsa_struct_;

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

    // SAFETY: `File` interns a `VfsPath`, which owns its data and so stays valid
    // after the database lifetime is erased.
    unsafe impl zalsa_struct_::Configuration for File {
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
            assert!(*self != File::MACRO);
            unsafe { salsa::Id::from_index(self.0) }
        }
    }

    impl zalsa_::FromId for File {
        #[inline]
        fn from_id(id: salsa::Id) -> Self {
            assert!(id.index() <= File::MAX_ID);
            Self(id.index(), PhantomData)
        }
    }

    unsafe impl Send for File {}
    unsafe impl Sync for File {}
    impl std::panic::UnwindSafe for File {}
    impl std::panic::RefUnwindSafe for File {}

    impl zalsa_::SalsaStructInDb for File {
        type MemoIngredientMap = salsa::plumbing::MemoIngredientSingletonIndex;
        const LEAF_TYPE_IDS: &'static [salsa::plumbing::ConstTypeId] = &[];

        fn lookup_ingredient_index(zalsa: &zalsa_::Zalsa) -> salsa::plumbing::IngredientIndices {
            zalsa.lookup_jar_by_type::<zalsa_struct_::JarImpl<File>>().into()
        }

        fn entries(zalsa: &zalsa_::Zalsa) -> impl Iterator<Item = zalsa_::DatabaseKeyIndex> + '_ {
            File::ingredient(zalsa).entries(zalsa).map(|entry| entry.key())
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

    // SAFETY: `File` is a plain `u32` index into the interning table.
    unsafe impl zalsa_::SalsaValue for File {}

    impl File {
        fn ingredient(zalsa: &zalsa_::Zalsa) -> &zalsa_struct_::IngredientImpl<Self> {
            static CACHE: zalsa_::IngredientCache<zalsa_struct_::IngredientImpl<File>> =
                zalsa_::IngredientCache::new();
            // SAFETY: The ingredient at offset 0 in `JarImpl<File>` has type
            // `IngredientImpl<File>`.
            unsafe { CACHE.get_or_create::<zalsa_struct_::JarImpl<File>, 0>(zalsa) }
        }

        pub fn new(db: &(impl salsa::Database + ?Sized), path: VfsPath) -> Self {
            let (zalsa, zalsa_local) = db.zalsas();
            let id =
                Self::ingredient(zalsa)
                    .intern(zalsa, zalsa_local, FileData { path }, |_, data| data);
            assert!(id.0 <= Self::MAX_ID, "File ID overflow: {} > {}", id.0, Self::MAX_ID);
            id
        }

        pub fn path(self, db: &dyn salsa::Database) -> &VfsPath {
            let zalsa = db.zalsa();
            &Self::ingredient(zalsa).fields(zalsa, self).path
        }
    }
};

impl File {
    pub const MAX_ID: u32 = EditionedFileId::MAX_FILE_ID;

    /// Identifies tokens that have no source file because they originate in macro machinery.
    ///
    /// Salsa does not allocate IDs from this range, so this value cannot collide with an
    /// interned file.
    pub const MACRO: Self = Self(SALSA_RESERVED_ID_START, PhantomData);

    #[inline]
    pub const fn index(self) -> u32 {
        self.0
    }

    /// # Safety
    ///
    /// `raw` must identify either an interned file or [`File::MACRO`].
    #[inline]
    pub const unsafe fn from_raw(raw: u32) -> Self {
        assert!(raw <= Self::MAX_ID || raw == Self::MACRO.0);
        Self(raw, PhantomData)
    }
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Self::MACRO {
            return f.write_str("File::MACRO");
        }
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EditionedFileId(u32);

impl fmt::Debug for EditionedFileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("EditionedFileId").field(&self.file().index()).field(&self.edition()).finish()
    }
}

impl From<EditionedFileId> for File {
    fn from(value: EditionedFileId) -> Self {
        value.file()
    }
}

const _: () = assert!(
    EditionedFileId::RESERVED_HIGH_BITS
        + EditionedFileId::EDITION_BITS
        + EditionedFileId::FILE_ID_BITS
        == u32::BITS
);
const _: () = assert!(SALSA_RESERVED_ID_START.checked_add(Edition::LATEST as u32).is_some());
const _: () = assert!(File::MAX_ID < SALSA_RESERVED_ID_START);
const _: () = assert!(
    EditionedFileId::RESERVED_MASK ^ EditionedFileId::EDITION_MASK ^ EditionedFileId::FILE_ID_MASK
        == u32::MAX
);

impl EditionedFileId {
    pub const RESERVED_MASK: u32 = 0x8000_0000;
    pub const EDITION_MASK: u32 = 0x7F80_0000;
    pub const FILE_ID_MASK: u32 = 0x007F_FFFF;
    pub const MAX_FILE_ID: u32 = Self::FILE_ID_MASK;
    pub const RESERVED_HIGH_BITS: u32 = Self::RESERVED_MASK.count_ones();
    pub const FILE_ID_BITS: u32 = Self::FILE_ID_MASK.count_ones();
    pub const EDITION_BITS: u32 = Self::EDITION_MASK.count_ones();

    pub const fn current_edition(file: File) -> Self {
        Self::new(file, Edition::CURRENT)
    }

    pub const fn new(file: File, edition: Edition) -> Self {
        if file.0 == File::MACRO.0 {
            return Self(SALSA_RESERVED_ID_START + edition as u32);
        }
        let file = file.index();
        assert!(file <= Self::MAX_FILE_ID);
        Self(file | ((edition as u32) << Self::FILE_ID_BITS))
    }

    pub const fn from_raw(raw: u32) -> Self {
        if Self::is_macro(raw) {
            return Self(raw);
        }
        assert!(raw & Self::RESERVED_MASK == 0);
        assert!((raw & Self::EDITION_MASK) >> Self::FILE_ID_BITS <= Edition::LATEST as u32);
        Self(raw)
    }

    pub const fn as_u32(self) -> u32 {
        self.0
    }

    pub const fn file(self) -> File {
        if Self::is_macro(self.0) {
            return File::MACRO;
        }
        unsafe { File::from_raw(self.0 & Self::FILE_ID_MASK) }
    }

    pub const fn unpack(self) -> (File, Edition) {
        (self.file(), self.edition())
    }

    pub const fn edition(self) -> Edition {
        if Self::is_macro(self.0) {
            let edition = self.0 - SALSA_RESERVED_ID_START;
            return unsafe { std::mem::transmute::<u8, Edition>(edition as u8) };
        }
        let edition = (self.0 & Self::EDITION_MASK) >> Self::FILE_ID_BITS;
        assert!(edition <= Edition::LATEST as u32);
        unsafe { std::mem::transmute::<u8, Edition>(edition as u8) }
    }

    const fn is_macro(raw: u32) -> bool {
        SALSA_RESERVED_ID_START <= raw && raw <= SALSA_RESERVED_ID_START + Edition::LATEST as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macro_file_uses_salsa_reserved_ids() {
        for edition in Edition::iter() {
            let file = EditionedFileId::new(File::MACRO, edition);

            assert_eq!(file.file(), File::MACRO);
            assert_eq!(file.edition(), edition);
            assert_eq!(file.as_u32(), SALSA_RESERVED_ID_START + edition as u32);
        }
    }
}
