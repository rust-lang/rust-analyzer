use itertools::Itertools;
use ra_syntax::{
    ast::{self, AstNode, AstToken},
    SmolStr, SyntaxKind,
    SyntaxKind::*,
    SyntaxNode, SyntaxToken, T,
};
use std::iter::successors;

pub(crate) struct SpacingDsl {

}

// #[cfg(test)]
// mod tests {

//     use std::{
//         fs,
//         path::{Path, PathBuf},
//     };

//     use crate::{
//         reformat_string,
//         rules::{indentation, spacing},
//     };

//     #[test]
//     fn indent() {
//         TestCase {
//             name: None,
//             before: "struct Test {
// x: u8,
// y: String,
// z: usize,
// }".into(),
//             after: "struct Test {
//     x: u8,
//     y: String,
//     z: usize,
// }".into(),
//         }
//         .run()
//         .map_err(|e| panic!(e))
//         .unwrap();
//     }

//     #[derive(Debug)]
//     struct TestCase {
//         name: Option<String>,
//         before: String,
//         after: String,
//     }

//     impl TestCase {
//         fn from_before_after(before: String, after: String) -> TestCase {
//             TestCase { name: None, before, after }
//         }

//         fn run(&self) -> Result<(), String> {
//             let name = self.name.as_ref().map(|it| it.as_str()).unwrap_or("");
//             let expected = &self.after;
//             let actual = &reformat_string(&self.before);
//             if expected != actual {
//                 return Err(format!(
//                     "\n\nAssertion failed: wrong formatting\
//                      \nTest: {}\n\
//                      \nBefore:\n{}\n\
//                      \nAfter:\n{}\n\
//                      \nExpected:\n{}\n",
//                     name, self.before, actual, self.after,
//                 ));
//             }
//             let second_round = &reformat_string(actual);
//             if actual != second_round {
//                 return Err(format!(
//                     "\n\nAssertion failed: formatting is not idempotent\
//                      \nTest: {}\n\
//                      \nBefore:\n{}\n\
//                      \nAfter:\n{}\n",
//                     name, actual, second_round,
//                 ));
//             }
//             Ok(())
//         }
//     }
// }
