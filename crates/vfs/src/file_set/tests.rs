use super::*;
use crate::loader::*;
use crate::{AbsPath, AbsPathBuf};

#[derive(Debug)]
struct DummyLoader;

const NO_WRONG: fn() -> ! = || panic!("dummy loader should not be used");
impl Handle for DummyLoader {
    fn spawn(_: Sender) -> Self {
        NO_WRONG()
    }
    fn set_config(&mut self, _: Config) {
        NO_WRONG()
    }
    fn invalidate(&mut self, _: AbsPathBuf) {
        NO_WRONG()
    }
    fn load_sync(&mut self, _: &AbsPath) -> Option<Vec<u8>> {
        NO_WRONG()
    }
}

#[test]
fn path_prefix() {
    let mut file_set = FileSetConfig::builder();
    file_set.add_file_set(vec![VfsPath::new_virtual_path("/foo".into())]);
    file_set.add_file_set(vec![VfsPath::new_virtual_path("/foo/bar/baz".into())]);
    let file_set = file_set.build();

    let mut vfs = Vfs::new(Box::new(DummyLoader));
    vfs.set_file_contents(VfsPath::new_virtual_path("/foo/src/lib.rs".into()), Some(Vec::new()));
    vfs.set_file_contents(
        VfsPath::new_virtual_path("/foo/src/bar/baz/lib.rs".into()),
        Some(Vec::new()),
    );
    vfs.set_file_contents(
        VfsPath::new_virtual_path("/foo/bar/baz/lib.rs".into()),
        Some(Vec::new()),
    );
    vfs.set_file_contents(VfsPath::new_virtual_path("/quux/lib.rs".into()), Some(Vec::new()));

    let partition = file_set.partition(&vfs).into_iter().map(|it| it.len()).collect::<Vec<_>>();
    assert_eq!(partition, vec![2, 1, 1]);
}

#[test]
fn name_prefix() {
    let mut file_set = FileSetConfig::builder();
    file_set.add_file_set(vec![VfsPath::new_virtual_path("/foo".into())]);
    file_set.add_file_set(vec![VfsPath::new_virtual_path("/foo-things".into())]);
    let file_set = file_set.build();

    let mut vfs = Vfs::new(Box::new(DummyLoader));
    vfs.set_file_contents(VfsPath::new_virtual_path("/foo/src/lib.rs".into()), Some(Vec::new()));
    vfs.set_file_contents(
        VfsPath::new_virtual_path("/foo-things/src/lib.rs".into()),
        Some(Vec::new()),
    );

    let partition = file_set.partition(&vfs).into_iter().map(|it| it.len()).collect::<Vec<_>>();
    assert_eq!(partition, vec![1, 1, 0]);
}
