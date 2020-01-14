use crate::dsl::{self, IndentDsl, IndentValue, SpacingDsl};
use itertools::Itertools;
use ra_syntax::{
    ast::{self, AstNode, AstToken},
    SmolStr, SyntaxKind,
    SyntaxKind::*,
    SyntaxNode, SyntaxToken, T,
};

/// Wraps expressions in a main function that must be declared in
/// a block.
#[macro_export]
macro_rules! wrap_fn {
    ( $inner:expr ) => {
        concat!("fn main() { ", $inner, " }")
    };
    ( $inner:expr, $end:expr ) => {
        concat!("fn main() { ", $inner, " }", $end)
    };
    ( $( $inner:expr )* ; $end:expr ) => {
        concat!("fn main() { ", $( $inner )*, " }", $end)
    };
}

#[rustfmt::skip]
static BIN_OPS: &[SyntaxKind] = &[
    T![+], T![+=],
    T![-], T![-=],
    T![*], T![*=],
    T![/], T![/=],
    T![==], T![!=],
    T![<], T![<=],
    T![>], T![>=],
    T![||], T![&&],
    T![&], T![&=],
    T![|], T![|=],
    T![^], T![^=],
];

#[rustfmt::skip]
pub(crate) fn spacing() -> SpacingDsl {
    let mut space_dsl = SpacingDsl::default();

    space_dsl
        .test(wrap_fn!("let x=0"), wrap_fn!("let x = 0"))
        .inside(LET_STMT).before(T![=]).single_space()
        .inside(LET_STMT).after(T![=]).single_space_or_optional_newline()

        .test(wrap_fn!("a==  b"), wrap_fn!("a == b"))
        .test(wrap_fn!("a!=  b"), wrap_fn!("a != b"))
        .test(wrap_fn!("a+  b"), wrap_fn!("a + b"))
        .test(wrap_fn!("a+=b"), wrap_fn!("a += b"))
        .test(wrap_fn!("a  -   b"), wrap_fn!("a - b"))
        .test(wrap_fn!("a*  b"), wrap_fn!("a * b"))
        .test(wrap_fn!("a/  b"), wrap_fn!("a / b"))
        .inside(BIN_EXPR).after(T![==]).single_space()
        .inside(BIN_EXPR).before(T![==]).single_space_or_optional_newline()
        .inside(BIN_EXPR).after(T![!=]).single_space()
        .inside(BIN_EXPR).before(T![!=]).single_space_or_optional_newline()
        .inside(BIN_EXPR).after(T![+]).single_space()
        .inside(BIN_EXPR).before(T![+]).single_space_or_optional_newline()
        .inside(BIN_EXPR).after(T![+=]).single_space()
        .inside(BIN_EXPR).before(T![+=]).single_space_or_optional_newline()
        .inside(BIN_EXPR).after(T![-]).single_space()
        .inside(BIN_EXPR).before(T![-]).single_space_or_optional_newline()
        .inside(BIN_EXPR).after(T![*]).single_space()
        .inside(BIN_EXPR).before(T![*]).single_space_or_optional_newline()
        .inside(BIN_EXPR).after(T![/]).single_space()
        .inside(BIN_EXPR).before(T![/]).single_space_or_optional_newline()

        .test(wrap_fn!("foo . bar . baz"), wrap_fn!("foo.bar.baz"))
        .inside(FIELD_EXPR).around(T![.]).no_space_or_newline()

        .test(wrap_fn!("if x {0}else{1};"), wrap_fn!("if x { 0 } else { 1 };"))
        .inside([IF_EXPR, BLOCK]).before(T!['{']).single_space()
        .inside([IF_EXPR, BLOCK]).after(T!['{']).single_space_or_optional_newline()
        .inside([IF_EXPR, BLOCK]).before(T!['}']).single_space_or_optional_newline()
        .inside([IF_EXPR, BLOCK]).after(T!['}']).single_space()
        .inside(EXPR_STMT).before(T![;]).no_space()

        .test(wrap_fn!("let x = [1,2,3];"), wrap_fn!("let x = [1, 2, 3];"))
        .inside(ARRAY_EXPR).after(T![,]).single_space_or_optional_newline()

        .test("struct Test{ x:usize }", "struct Test { x: usize }")
        .inside(RECORD_FIELD_DEF_LIST).before(T!['{']).single_space()
        // .inside(RECORD_FIELD_DEF_LIST).after(T!['{']).single_space_or_optional_newline()
        .inside(RECORD_FIELD_DEF).after(T![:]).single_space()
        // .inside(RECORD_FIELD_DEF_LIST).before(T!['}']).single_space_or_optional_newline()
        // .inside(RECORD_FIELD_DEF_LIST).after(T!['}']).single_space_or_optional_newline()

        .test("pub(crate)struct Test { x: u8 }", "pub(crate) struct Test { x: u8 }")
        .inside(VISIBILITY).after(T![')']).single_space()

        .test("fn main(x: usize){ let x = 0; }", "fn main(x: usize) { let x = 0; }")
        .inside(PARAM_LIST).before(T!['(']).no_space()
        .inside(PARAM_LIST).after(T!['(']).no_space_or_optional_newline()
        .inside(PARAM_LIST).before(T![')']).no_space_or_optional_newline()
        .inside(PARAM_LIST).after(T![')']).single_space()

        ;
    // ^^^ leave for testing makes messing with the rules easier
    // more rules to come

    space_dsl
}

#[rustfmt::skip]
pub(crate) fn indentation() -> IndentDsl {
    let mut indent_dsl = IndentDsl::default();
    indent_dsl
        .anchor(FN_DEF)
        .rule("Indent fn def")
            .inside(BLOCK)
            .matching([EXPR_STMT, LET_STMT])
            .set(IndentValue::Indent)
            .test(
"fn main () {let x=0;\nlet y=1;\n}",
r#"fn main() {
    let x = 0;
    let y = 1;
}"#)
        .rule("Indent struct fields def")
            .inside(RECORD_FIELD_DEF_LIST)
            .matching(RECORD_FIELD_DEF)
            .set(IndentValue::Indent)
            .test(
r#"struct Test {
x: String,
}"#, 
r#"struct Test {
    x: String,
}"#)

        .anchor([RECORD_LIT, RECORD_FIELD])
        .rule("Indent struct fields lit")
            .inside(RECORD_FIELD_LIST)
            .matching(RECORD_FIELD)
            .set(IndentValue::Indent)
            .test(wrap_fn!(
r#"let t = Test {
x: String,
};"#), 
wrap_fn!(r#"let t = Test {
    x: String,
};"#))

        .anchor(LET_STMT)
        .rule("Indent chained method calls")
            .inside(METHOD_CALL_EXPR)
            .matching(T![.])
            .set(IndentValue::Indent)
            .test(
wrap_fn!(r#"let a = foo()
.bar()
.baz();"#),
wrap_fn!(r#"let a = foo()
    .bar()
    .baz();"#))
    
    ;
    
    // more rules to come

    indent_dsl
}

#[cfg(test)]
mod tests {

    use crate::{
        edit_tree::EditTree, //diff_view::DiffView,
        engine::format_str,
        rules::{indentation, spacing},
    };
    use ra_syntax::SourceFile;
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    #[test]
    fn test_sys() {
        TestCase {
            name: None,
            // TODO add to this test as we go
            before: "struct Test  { x:u8 }".into(),
            after: "struct Test { x: u8 }\n".into(),
        }
        .run()
        .map_err(|e| panic!(e))
        .unwrap();
    }

    #[test]
    fn test_inline_spacing() {
        let rules = spacing();
        let tests: Vec<TestCase> = rules
            .tests
            .iter()
            .map(|&(before, after)| {
                let before = before.to_string();
                let after = format!("{}\n", after);
                TestCase::from_before_after(before, after)
            })
            .collect();
        run(&tests)
    }

    #[test]
    fn test_inline_indent() {
        let rules = indentation();
        let tests: Vec<TestCase> = rules
            .tests
            .iter()
            .map(|&(before, after)| {
                let before = before.to_string();
                let after = format!("{}\n", after);
                TestCase::from_before_after(before, after)
            })
            .collect();
        run(&tests)
    }

    #[derive(Debug)]
    struct TestCase {
        name: Option<String>,
        before: String,
        after: String,
    }

    impl TestCase {
        fn from_before_after(before: String, after: String) -> TestCase {
            TestCase { name: None, before, after }
        }

        fn run(&self) -> Result<(), String> {
            let name = self.name.as_ref().map(|it| it.as_str()).unwrap_or("");
            let expected = &self.after;
            let actual = &format_str(&self.before).unwrap();
            if expected != actual {
                return Err(format!(
                    "\n\nAssertion failed: wrong formatting\
                     \nTest: {}\n\
                     \nBefore:\n{:?}\n\
                     \nAfter:\n{:?}\n\
                     \nExpected:\n{:?}\n",
                    name, self.before, actual, self.after,
                ));
            }
            let second_round = &format_str(actual).unwrap();
            if actual != second_round {
                return Err(format!(
                    "\n\nAssertion failed: formatting is not idempotent\
                     \nTest: {}\n\
                     \nBefore:\n{:?}\n\
                     \nAfter:\n{:?}\n",
                    name, actual, second_round,
                ));
            }
            Ok(())
        }
    }

    fn run(tests: &[TestCase]) {
        let mut n_failed = 0;
        for test in tests {
            if let Err(msg) = test.run() {
                n_failed += 1;
                eprintln!("{}", msg)
            }
        }
        if n_failed > 0 {
            panic!("{} failed test cases out of {} total", n_failed, tests.len())
        }
    }
}
