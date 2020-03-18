#[macro_use]
mod utils;
use utils::*;

#[test]
fn test_serialize_proc_macro() {
    let res = expand(
        "serde_derive",
        "Serialize",
        r##"
    struct Foo {}
    "##,
    );
    assert_eq_file!(&res, "fixtures/test_serialize_proc_macro.txt");
}

#[test]
fn test_serialize_proc_macro_failed() {
    let res = expand(
        "serde_derive",
        "Serialize",
        r##"
    struct {}
    "##,
    );

    assert_eq_text!(
        &res,
        r##"
SUBTREE NODELIM
  IDENT   compile_error
  PUNCH   ! [alone]
  SUBTREE {}
    LITERAL "expected identifier"
"##
    );
}
