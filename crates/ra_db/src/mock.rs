use rustc_hash::FxHashSet;
use sortedvec::sortedvec;
use relative_path::RelativePathBuf;

use crate::FileId;

sortedvec! {
    #[derive(Debug, Clone)]
    pub struct FileMap {
        fn key_deriv(v: &(FileId, RelativePathBuf)) -> &str {
            v.1.as_relative_path().as_str()
        }
    }
}

impl FileMap {
    pub fn add(&mut self, path: RelativePathBuf) -> FileId {
        let file_id = FileId((self.len() + 1) as u32);
        self.insert((file_id, path));
        file_id
    }

    pub fn files(&self) -> FxHashSet<FileId> {
        self.iter().map(|&(id, _)| id).collect()
    }

    pub fn file_id(&self, path: &str) -> FileId {
        assert!(path.starts_with('/'));
        self.find(&&path[1..]).unwrap().0
    }
}
