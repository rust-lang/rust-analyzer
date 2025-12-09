use super::*;

#[test]
fn path_prefix() {
    let mut config = PathClassifier::builder();
    config.add_root(vec![VfsPath::new_virtual_path("/foo".into())]);
    config.add_root(vec![VfsPath::new_virtual_path("/foo/bar/baz".into())]);
    let config = config.build();

    let paths = [
        VfsPath::new_virtual_path("/foo/src/lib.rs".into()),
        VfsPath::new_virtual_path("/foo/src/bar/baz/lib.rs".into()),
        VfsPath::new_virtual_path("/foo/bar/baz/lib.rs".into()),
        VfsPath::new_virtual_path("/quux/lib.rs".into()),
    ];

    let roots = paths.iter().map(|path| config.classify(path)).collect::<Vec<_>>();
    assert_eq!(roots, vec![0, 0, 1, 2]);
}

#[test]
fn name_prefix() {
    let mut config = PathClassifier::builder();
    config.add_root(vec![VfsPath::new_virtual_path("/foo".into())]);
    config.add_root(vec![VfsPath::new_virtual_path("/foo-things".into())]);
    let config = config.build();

    let paths = [
        VfsPath::new_virtual_path("/foo/src/lib.rs".into()),
        VfsPath::new_virtual_path("/foo-things/src/lib.rs".into()),
        VfsPath::new_virtual_path("/other/src/lib.rs".into()),
    ];

    let roots = paths.iter().map(|path| config.classify(path)).collect::<Vec<_>>();
    assert_eq!(roots, vec![0, 1, 2]);
}

#[test]
fn classify_configured() {
    let mut config = PathClassifier::builder();
    config.add_root(vec![VfsPath::new_virtual_path("/foo".into())]);
    config.add_root(vec![VfsPath::new_virtual_path("/foo/bar/baz".into())]);
    let config = config.build();

    let classify = |path: &str| config.classify_configured(&VfsPath::new_virtual_path(path.into()));
    assert_eq!(classify("/foo/src/lib.rs"), Some(0));
    assert_eq!(classify("/foo/bar/baz/lib.rs"), Some(1));
    assert_eq!(classify("/quux/lib.rs"), None);
}

#[test]
fn classify_configured_default_config() {
    let config = PathClassifier::default();
    let path = VfsPath::new_virtual_path("/foo/lib.rs".into());
    assert_eq!(config.classify_configured(&path), None);
}

/// Ensure that we don't consider `/foo/bar_baz.rs` to be in the
/// `/foo/bar/` root.
#[test]
fn name_prefix_partially_matches() {
    let mut config = PathClassifier::builder();
    config.add_root(vec![VfsPath::new_virtual_path("/foo".into())]);
    config.add_root(vec![VfsPath::new_virtual_path("/foo/bar".into())]);
    let config = config.build();

    let paths = [
        VfsPath::new_virtual_path("/foo/lib.rs".into()),
        VfsPath::new_virtual_path("/foo/bar_baz.rs".into()),
        VfsPath::new_virtual_path("/foo/bar/biz.rs".into()),
    ];

    let roots = paths.iter().map(|path| config.classify(path)).collect::<Vec<_>>();
    assert_eq!(roots, vec![0, 0, 1]);
}
