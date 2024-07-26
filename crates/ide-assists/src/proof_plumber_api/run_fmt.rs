//! ProofPlumber API for formatting
//!
//! Uses verusfmt at <https://github.com/verus-lang/verusfmt>
//!
//! Since TOST node (VST node) abstracts away whitespace, indentation, newline and stuff
//! for easier manipulation, we need to restore those
//! before we send the proof code back to the IDE user
//!
//! It creates a temporary file for formatting at $TMPDIR
//!

use crate::AssistContext;
use core::ops::Range;
use syntax::{ast, AstNode};

/*
verus! {

proof fn f() {
    /*marker fmt start*/
    assert(x == 3) by {
        assert(x == 3);
    }/*marker fmt end*/
    ;
}

} // verus!

*/
impl<'a> AssistContext<'a> {
    /// Format a function
    /// Internally does the following
    /// 1) print the function into a temporary file (ugly, but syntactically correct one)
    /// 2) run verusfmt on the temporary file
    /// 3) return the formatted function as a string
    pub fn fmt<N: AstNode>(
        &self,
        sth_to_remove: N,        // old
        text_to_replace: String, // new
    ) -> Option<String> {
        let func: ast::Fn = self.find_node_at_offset::<ast::Fn>()?.clone();
        self.run_fmt_replacing(&func, sth_to_remove, text_to_replace)
    }

    fn run_fmt_replacing<N: AstNode>(
        &self,
        func: &ast::Fn,          // original
        sth_to_remove: N,        // old
        text_to_replace: String, // new
    ) -> Option<String> {
        let fn_range = func.syntax().text_range();
        let expr_range = sth_to_remove.syntax().text_range();
        let expr_range_in_fn = expr_range.checked_sub(fn_range.start())?;
        let range: core::ops::Range<usize> = expr_range_in_fn.into();
        let string_result = self.try_fmt(func.to_string(), range, text_to_replace)?;
        let joined_string = string_result.join("\n");
        let result = joined_string.trim_start(); // note the indentation at the inserted location
        Some(String::from(result))
    }

    // for now, format only a function
    // 1) print the function into a temporary file (ugly, but syntactically correct one)
    // 2) run verusfmt on the temporary file
    // 3) return the formatted function as a string
    //
    fn try_fmt(
        &self,
        mut fn_as_text: String,
        range_to_remove: Range<usize>,
        mut text_to_replace: String, // from vst
    ) -> Option<Vec<String>> {
        let start_marker = "/*marker fmt start*/";
        let end_marker = "/*marker fmt end*/";

        text_to_replace.insert_str(0, "\n/*marker fmt start*/\n");

        text_to_replace.push_str("\n/*marker fmt end*/");

        fn_as_text.replace_range::<Range<usize>>(range_to_remove, &text_to_replace);

        fn_as_text.insert_str(0, "verus!{\n");
        fn_as_text.push_str("\n}");

        let verusfmt_options = verusfmt::RunOptions { file_name: None, run_rustfmt: false, rustfmt_config: Default::default() };
        let fmt_result = verusfmt::run(&fn_as_text, verusfmt_options);
        match fmt_result {
            Ok(formatted) => {
                let mut result = Vec::new();
                let mut is_line_target = false;
                for line in formatted.lines() {
                    if line.contains(start_marker) {
                        is_line_target = true;
                        continue;
                    }
                    if line.contains(end_marker) {
                        // trailing comment
                        let mut new_line = String::from(line);
                        new_line = new_line.replace(end_marker, "");
                        if new_line.len() > 0 {
                            result.push(new_line.to_string());
                        }
                        break;
                    }
                    if is_line_target {
                        result.push(line.to_string())
                    }
                }
                return Some(result);
            }
            Err(_) => return None,
        }
    }
}
