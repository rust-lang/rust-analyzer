//! In-memory document information.

use std::{
    hash::{Hash, Hasher},
    mem,
};

use rustc_hash::{FxHashMap, FxHasher};
use triomphe::Arc;
use vfs::VfsPath;

use crate::ClientId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DocumentEntry {
    pub(crate) vfs_author: ClientId,
    pub(crate) clients: FxHashMap<ClientId, DocumentData>,
    /// Hash of the file on disk, if we know its contents.
    ///
    /// Clients that do not have the document open refer to the text on disk.
    disk_hash: Option<u64>,
}

fn content_hash(contents: &[u8]) -> u64 {
    let mut hasher = FxHasher::default();
    contents.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RemoveDocResult {
    /// The document was completely closed (last client holding it closed it).
    CompletelyClosed,
    /// The document is still held open by surviving clients. If the removed client was the
    /// authoritative writer, `restored_content` contains the surviving client's buffer to restore to VFS.
    StillOpen { restored_content: Option<Vec<u8>> },
    /// The document was not open by this client.
    NotFound,
}

/// Holds the set of in-memory documents.
///
/// For these documents, their true contents is maintained by the client(s).
/// In multi-client mode, documents track each client's independent buffer and version
/// so that edits from one client never corrupt or panic another client's buffer, and
/// closing a file in one client does not drop it for another.
#[derive(Default, Clone)]
pub(crate) struct MemDocs {
    docs: Arc<FxHashMap<VfsPath, Arc<DocumentEntry>>>,
    added_or_removed: bool,
}

impl MemDocs {
    pub(crate) fn contains(&self, path: &VfsPath) -> bool {
        self.docs.contains_key(path)
    }

    pub(crate) fn insert(
        &mut self,
        client_id: ClientId,
        path: VfsPath,
        data: DocumentData,
    ) -> Result<(), ()> {
        self.added_or_removed = true;
        let docs = Arc::make_mut(&mut self.docs);
        if let Some(entry_arc) = docs.get_mut(&path) {
            let entry = Arc::make_mut(entry_arc);
            entry.clients.insert(client_id, data);
            entry.vfs_author = client_id;
            Err(())
        } else {
            let mut clients = FxHashMap::default();
            // A document is usually opened with the text it has on disk. The caller tells us
            // when that is not the case.
            let disk_hash = Some(content_hash(&data.data));
            clients.insert(client_id, data);
            docs.insert(
                path,
                Arc::new(DocumentEntry { vfs_author: client_id, clients, disk_hash }),
            );
            Ok(())
        }
    }

    /// Removes a client's document reference. Returns `RemoveDocResult::CompletelyClosed` if this
    /// was the last client holding the document open, `RemoveDocResult::StillOpen` if other
    /// clients still have it open (with optional restored buffer for the new author), or
    /// `RemoveDocResult::NotFound` if not open by this client.
    pub(crate) fn remove(&mut self, client_id: ClientId, path: &VfsPath) -> RemoveDocResult {
        let docs = Arc::make_mut(&mut self.docs);
        let Some(entry_arc) = docs.get_mut(path) else {
            return RemoveDocResult::NotFound;
        };

        let entry = Arc::make_mut(entry_arc);
        if entry.clients.remove(&client_id).is_none() {
            return RemoveDocResult::NotFound;
        }

        if entry.clients.is_empty() {
            docs.remove(path);
            self.added_or_removed = true;
            RemoveDocResult::CompletelyClosed
        } else {
            let restored_content = if entry.vfs_author == client_id {
                let (&survivor_id, survivor_doc) = entry.clients.iter().next().unwrap();
                entry.vfs_author = survivor_id;
                Some(survivor_doc.data.clone())
            } else {
                None
            };
            RemoveDocResult::StillOpen { restored_content }
        }
    }

    /// Removes all document references for a disconnected client. Returns:
    /// - list of paths whose last client reference was removed (completely closed)
    /// - list of (path, content) pairs where author changed and surviving content should be restored to VFS
    pub(crate) fn remove_client(
        &mut self,
        client_id: ClientId,
    ) -> (Vec<VfsPath>, Vec<(VfsPath, Vec<u8>)>) {
        let docs = Arc::make_mut(&mut self.docs);
        let mut completely_closed = Vec::new();
        let mut to_restore = Vec::new();

        for (path, entry_arc) in docs.iter_mut() {
            let entry = Arc::make_mut(entry_arc);
            if entry.clients.remove(&client_id).is_some() {
                if entry.clients.is_empty() {
                    completely_closed.push(path.clone());
                } else if entry.vfs_author == client_id {
                    let (&survivor_id, survivor_doc) = entry.clients.iter().next().unwrap();
                    entry.vfs_author = survivor_id;
                    to_restore.push((path.clone(), survivor_doc.data.clone()));
                }
            }
        }

        if !completely_closed.is_empty() {
            self.added_or_removed = true;
            for path in &completely_closed {
                docs.remove(path);
            }
        }

        (completely_closed, to_restore)
    }

    pub(crate) fn get(&self, client_id: ClientId, path: &VfsPath) -> Option<&DocumentData> {
        self.docs.get(path)?.clients.get(&client_id)
    }

    pub(crate) fn get_any(&self, path: &VfsPath) -> Option<&DocumentData> {
        let entry = self.docs.get(path)?;
        entry.clients.get(&entry.vfs_author).or_else(|| entry.clients.values().next())
    }

    pub(crate) fn get_mut(
        &mut self,
        client_id: ClientId,
        path: &VfsPath,
    ) -> Option<&mut DocumentData> {
        let entry = Arc::make_mut(Arc::make_mut(&mut self.docs).get_mut(path)?);
        entry.clients.get_mut(&client_id)
    }

    /// Whether the analyzed buffer is known to be the same as the file on disk.
    pub(crate) fn matches_disk(&self, path: &VfsPath) -> bool {
        let Some(entry) = self.docs.get(path) else {
            return false;
        };
        let Some(author_doc) = entry.clients.get(&entry.vfs_author) else {
            return false;
        };
        entry.disk_hash == Some(content_hash(&author_doc.data))
    }

    /// Records the contents of the file on disk, `None` if they are not known.
    pub(crate) fn set_disk_contents(&mut self, path: &VfsPath, contents: Option<&[u8]>) {
        let disk_hash = contents.map(content_hash);
        if let Some(entry) = Arc::make_mut(&mut self.docs).get_mut(path)
            && entry.disk_hash != disk_hash
        {
            Arc::make_mut(entry).disk_hash = disk_hash;
        }
    }

    pub(crate) fn set_author(&mut self, client_id: ClientId, path: &VfsPath) {
        if let Some(entry_arc) = Arc::make_mut(&mut self.docs).get_mut(path) {
            let entry = Arc::make_mut(entry_arc);
            entry.vfs_author = client_id;
        }
    }

    pub(crate) fn is_in_sync_with_vfs(&self, client_id: ClientId, path: &VfsPath) -> bool {
        let Some(entry) = self.docs.get(path) else {
            return false;
        };
        let Some(client_doc) = entry.clients.get(&client_id) else {
            return false;
        };
        if entry.vfs_author == client_id {
            return true;
        }
        let Some(author_doc) = entry.clients.get(&entry.vfs_author) else {
            return true;
        };
        client_doc.data == author_doc.data
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &VfsPath> {
        self.docs.keys()
    }

    pub(crate) fn take_changes(&mut self) -> bool {
        mem::replace(&mut self.added_or_removed, false)
    }
}

/// Information about a document that the Language Client
/// knows about.
/// Its lifetime is driven by the textDocument/didOpen and textDocument/didClose
/// client notifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DocumentData {
    pub(crate) version: i32,
    pub(crate) data: Vec<u8>,
}

impl DocumentData {
    pub(crate) fn new(version: i32, data: Vec<u8>) -> Self {
        DocumentData { version, data }
    }
}

#[cfg(test)]
mod tests {
    use vfs::VfsPath;

    use super::*;

    fn test_path(s: &str) -> VfsPath {
        VfsPath::new_real_path(s.to_owned())
    }

    #[test]
    fn single_client_lifecycle() {
        let mut docs = MemDocs::default();
        let path = test_path("/workspace/src/lib.rs");
        let c1 = ClientId(1);

        assert!(!docs.contains(&path));
        assert!(docs.insert(c1, path.clone(), DocumentData::new(1, b"hello".to_vec())).is_ok());
        assert!(docs.contains(&path));
        assert!(docs.take_changes());

        assert_eq!(docs.get(c1, &path).unwrap().version, 1);
        assert_eq!(docs.get(c1, &path).unwrap().data, b"hello");

        assert_eq!(docs.remove(c1, &path), RemoveDocResult::CompletelyClosed);
        assert!(!docs.contains(&path));
        assert!(docs.take_changes());
    }

    #[test]
    fn multi_client_isolated_buffers_and_lifecycle() {
        let mut docs = MemDocs::default();
        let path = test_path("/workspace/src/lib.rs");
        let c1 = ClientId(1);
        let c2 = ClientId(2);

        // c1 opens version 1
        assert!(docs.insert(c1, path.clone(), DocumentData::new(1, b"c1 text".to_vec())).is_ok());
        assert!(docs.contains(&path));

        // c2 opens version 2
        assert!(docs.insert(c2, path.clone(), DocumentData::new(2, b"c2 text".to_vec())).is_err());
        assert!(docs.contains(&path));

        // Each client has its own independent buffer and version!
        assert_eq!(docs.get(c1, &path).unwrap().version, 1);
        assert_eq!(docs.get(c1, &path).unwrap().data, b"c1 text");
        assert_eq!(docs.get(c2, &path).unwrap().version, 2);
        assert_eq!(docs.get(c2, &path).unwrap().data, b"c2 text");

        // c2 wrote last, so c2 is in sync with vfs, c1 has divergent text
        assert!(docs.is_in_sync_with_vfs(c2, &path));
        assert!(!docs.is_in_sync_with_vfs(c1, &path));

        // Mutating c1 and setting author switches author to c1
        docs.get_mut(c1, &path).unwrap().data = b"c1 edited".to_vec();
        docs.set_author(c1, &path);
        assert_eq!(docs.get(c1, &path).unwrap().data, b"c1 edited");
        assert_eq!(docs.get(c2, &path).unwrap().data, b"c2 text");
        assert!(docs.is_in_sync_with_vfs(c1, &path));
        assert!(!docs.is_in_sync_with_vfs(c2, &path));

        // c1 closes -> document still held by c2, surviving content returned!
        assert_eq!(
            docs.remove(c1, &path),
            RemoveDocResult::StillOpen { restored_content: Some(b"c2 text".to_vec()) }
        );
        assert!(docs.contains(&path));
        assert!(docs.get(c1, &path).is_none());
        assert_eq!(docs.get(c2, &path).unwrap().version, 2);
        assert!(docs.is_in_sync_with_vfs(c2, &path));

        // c2 closes -> now completely closed!
        assert_eq!(docs.remove(c2, &path), RemoveDocResult::CompletelyClosed);
        assert!(!docs.contains(&path));
    }

    #[test]
    fn matches_disk_follows_analyzed_buffer() {
        let mut docs = MemDocs::default();
        let path = test_path("/workspace/src/lib.rs");

        let (c1, c2) = (ClientId(1), ClientId(2));

        assert!(!docs.matches_disk(&path));
        docs.insert(c1, path.clone(), DocumentData::new(1, b"disk".to_vec())).unwrap();
        assert!(docs.matches_disk(&path));

        // An unsaved edit no longer matches, until some client writes the disk text again
        docs.get_mut(c1, &path).unwrap().data = b"edited".to_vec();
        assert!(!docs.matches_disk(&path));
        let _ = docs.insert(c2, path.clone(), DocumentData::new(1, b"disk".to_vec()));
        assert!(docs.matches_disk(&path));

        docs.set_disk_contents(&path, None);
        assert!(!docs.matches_disk(&path));
        docs.set_disk_contents(&path, Some(b"disk"));
        assert!(docs.matches_disk(&path));
    }

    #[test]
    fn remove_client_cleanup() {
        let mut docs = MemDocs::default();
        let path1 = test_path("/workspace/src/file1.rs");
        let path2 = test_path("/workspace/src/file2.rs");
        let c1 = ClientId(1);
        let c2 = ClientId(2);

        docs.insert(c1, path1.clone(), DocumentData::new(1, b"f1".to_vec())).unwrap();
        docs.insert(c1, path2.clone(), DocumentData::new(1, b"f2".to_vec())).unwrap();
        let _ = docs.insert(c2, path2.clone(), DocumentData::new(2, b"f2_c2".to_vec()));

        assert!(docs.contains(&path1));
        assert!(docs.contains(&path2));

        // Disconnect c2 (who is author of path2): path2 is still held by c1, so c1's text is restored
        let (closed, restored) = docs.remove_client(c2);
        assert_eq!(closed, Vec::<VfsPath>::new());
        assert_eq!(restored, vec![(path2.clone(), b"f2".to_vec())]);

        // Disconnect c1: both path1 and path2 are now completely closed
        let (closed2, restored2) = docs.remove_client(c1);
        assert_eq!(closed2, vec![path1.clone(), path2.clone()]);
        assert_eq!(restored2, Vec::<(VfsPath, Vec<u8>)>::new());
        assert!(!docs.contains(&path1));
        assert!(!docs.contains(&path2));
    }
}
