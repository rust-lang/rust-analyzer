use rustc_hash::FxHashSet;
use sortedvec::def_sorted_vec;
use relative_path::RelativePathBuf;

use crate::FileId;

fn key_deriv(v: &(FileId, RelativePathBuf)) -> &str {
    v.1.as_relative_path().as_str()
}

def_sorted_vec! {
    #[derive(Debug, Clone)]
    pub struct FileMap: (FileId, RelativePathBuf) => &str, key_deriv
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
