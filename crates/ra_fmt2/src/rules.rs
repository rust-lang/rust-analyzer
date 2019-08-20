use crate::dsl::{self, SpacingDsl, IndentDsl, IndentValue};
use itertools::Itertools;
use ra_syntax::{
    ast::{self, AstNode, AstToken},
    SmolStr, SyntaxKind,
    SyntaxKind::*,
    SyntaxNode, SyntaxToken, T,
};

pub(crate) fn spacing() -> SpacingDsl {
    let mut space_dsl = SpacingDsl::default();

    space_dsl
        .test("fn main() { let x = [1,2,3]; }", "fn main() { let x = [1, 2, 3]; }")
        .inside(ARRAY_EXPR).after(T![,]).single_space()

        .test("struct Test{x:usize    }", "struct Test { x: usize }")
        .inside(NAMED_FIELD_DEF_LIST).before(T!['{']).single_space()
        .inside(NAMED_FIELD_DEF_LIST).after(T!['{']).single_space_or_optional_newline()
        .inside(NAMED_FIELD_DEF_LIST).before(T!['}']).single_space_or_optional_newline()
        .inside(NAMED_FIELD_DEF_LIST).after(T!['}']).single_space_or_optional_newline()
        .inside(NAMED_FIELD_DEF).after(T![:]).single_space()

        .test("pub(crate)struct Test { x: u8 }", "pub(crate) struct Test { x: u8 }")
        .inside(VISIBILITY).after(T![')']).single_space()


        // must be done in engine so as not to disturb precidence or keeping track of Syntax Blocks "\n"
        // .rule(dsl::SpacingRule {
        //     pattern: SOURCE_FILE.into(),
        //     space: dsl::Space { loc: dsl::SpaceLoc::After, value: dsl::SpaceValue::Newline }
        // });
    // more rules to come

    space_dsl
}

pub(crate) fn indentation() -> IndentDsl {
    let mut indent_dsl = IndentDsl::default();

    indent_dsl
        .rule("Indent struct fields")
            .inside(NAMED_FIELD_DEF_LIST)
            .matching(NAMED_FIELD_DEF)
            .set(IndentValue::Indent)
            .test(r#"
                struct Test {
                x: String,
                }"#, r#"
                struct Test {
                    x: String,
                }"#);

    // more rules to come

    indent_dsl
}

#[cfg(test)]
mod tests {
 
    use crate::{
        edit_tree::EditTree, //diff_view::DiffView,
        engine::format_str, rules::{spacing},
    };
    use ra_syntax::SourceFile;
    use std::{ fs, path::{Path, PathBuf},};

    #[test]
    fn test_sys() {
        TestCase {
            name: None,
            // TODO add to this test as we go
            before: "struct Test  {x: u8  }".into(),
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

    #[derive(Debug)]
    struct TestCase {
        name: Option<String>,
        before: String,
        after: String,
    }

        fn run(&self) -> Result<(), String> {
            let name = self.name.as_ref().map(|it| it.as_str()).unwrap_or("");
            let expected = &self.after;
            let actual = &format_str(&self.before).unwrap().to_string();
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
            let second_round = &format_str(actual).unwrap().to_string();
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

        fn run(&self) -> Result<(), String> {
            let name = self.name.as_ref().map(|it| it.as_str()).unwrap_or("");
            let expected = &self.after;
            let actual = &format_str(&self.before).unwrap().to_string();
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
            let second_round = &format_str(actual).unwrap().to_string();
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
