use ra_syntax::Parse;
use lru_cache::LruCache;
use parking_lot::Mutex;

use crate::FileId;

const TREE_CACHE_SIZE: usize = 256;

#[derive(Debug)]
pub struct TreeCache {
    cache: Mutex<LruCache<FileId, Parse>>,
}

impl Default for TreeCache {
    fn default() -> TreeCache {
        TreeCache { cache: Mutex::new(LruCache::new(TREE_CACHE_SIZE)) }
    }
}

impl TreeCache {
    pub(crate) fn get(&self, file_id: FileId) -> Option<Parse> {
        let mut cache = self.cache.lock();
        cache.get_mut(&file_id).map(|it| it.clone())
    }
    pub(crate) fn insert(&self, file_id: FileId, parse: Parse) {
        let mut cache = self.cache.lock();
        cache.insert(file_id, parse);
    }
}
