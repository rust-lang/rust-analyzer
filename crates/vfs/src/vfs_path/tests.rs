use super::*;

#[test]
fn virtual_path_starts_with_is_component_based() {
    let path = |path: &str| VfsPath::new_virtual_path(path.to_owned());

    assert!(!path("/foobar").starts_with(&path("/foo")));
    assert!(path("/foo/bar").starts_with(&path("/foo")));
}

#[test]
fn virtual_path_join_from_root() {
    let path = VfsPath::new_virtual_path("/".to_owned());

    assert_eq!(path.join("foo").unwrap(), VfsPath::new_virtual_path("/foo".to_owned()));
}

#[test]
fn virtual_path_join_normalizes_relative_paths() {
    let cases = [
        ("/foo", "", "/foo"),
        ("/foo", ".", "/foo"),
        ("/foo", "..", "/"),
        ("/foo", "a/../b", "/foo/b"),
        ("/foo/bar", "./../x", "/foo/x"),
        ("/foo/", "bar", "/foo/bar"),
        ("/foo", "bar/", "/foo/bar"),
        ("/foo", "a//b", "/foo/a/b"),
    ];

    for (base, path, expected) in cases {
        let base = VfsPath::new_virtual_path(base.to_owned());
        assert_eq!(base.join(path).unwrap(), VfsPath::new_virtual_path(expected.to_owned()));
    }
}

#[test]
fn virtual_path_join_replaces_base_with_absolute_path() {
    let path = VfsPath::new_virtual_path("/foo/bar".to_owned());

    assert_eq!(path.join("/baz/../quux").unwrap(), VfsPath::new_virtual_path("/quux".to_owned()));
    assert_eq!(path.join("/../../quux").unwrap(), VfsPath::new_virtual_path("/quux".to_owned()));
}

#[test]
fn virtual_path_join_rejects_relative_escape_from_root() {
    let path = VfsPath::new_virtual_path("/foo".to_owned());

    assert_eq!(path.join("../../bar"), None);
}

#[test]
fn virtual_path_extensions() {
    assert_eq!(VirtualPath("/".to_owned()).name_and_extension(), None);
    assert_eq!(
        VirtualPath("/directory".to_owned()).name_and_extension(),
        Some(("directory", None))
    );
    assert_eq!(
        VirtualPath("/directory/".to_owned()).name_and_extension(),
        Some(("directory", None))
    );
    assert_eq!(
        VirtualPath("/directory/file".to_owned()).name_and_extension(),
        Some(("file", None))
    );
    assert_eq!(
        VirtualPath("/directory/.file".to_owned()).name_and_extension(),
        Some((".file", None))
    );
    assert_eq!(
        VirtualPath("/directory/.file.rs".to_owned()).name_and_extension(),
        Some((".file", Some("rs")))
    );
    assert_eq!(
        VirtualPath("/directory/file.rs".to_owned()).name_and_extension(),
        Some(("file", Some("rs")))
    );
}
