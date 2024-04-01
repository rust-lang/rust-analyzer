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


use std::{process::Command, collections::hash_map::DefaultHasher, time::Instant, env, path::Path, hash::{Hasher, Hash}, fs::{read_to_string, File}, io::Write};
use core::ops::Range;
use crate::AssistContext;
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
        // let source_file = &self.source_file;
        // let fmt_path = &self.config.fmt_path;
        let fmt_path = std::env::var("VERUS_FMT_BINARY_PATH").expect("please set VERUS_FMT_BINARY_PATH environment variable");


        let start_marker = "/*marker fmt start*/";
        let end_marker = "/*marker fmt end*/";

        text_to_replace.insert_str(0, "\n/*marker fmt start*/\n");

        text_to_replace.push_str("\n/*marker fmt end*/");

        fn_as_text.replace_range::<Range<usize>>(range_to_remove, &text_to_replace);

        fn_as_text.insert_str(0, "verus!{\n");
        fn_as_text.push_str("\n}");
        // dbg!("{}", &fn_as_text);
        

        // #[cfg(test)] // We get verus path from config of editor. In test, we use a hardcoded path
        // let fmt_path = HARDCODED_VERUS_FMT_PATH_FOR_TEST.to_string(); // TODO: maybe move this to test config


        dbg!(&fmt_path);
        if fmt_path.len() == 0 {
            dbg!("verusfmt path not set");
        }
        
        // REIVEW: instead of writing to a file in the tmp directory, consider using `memfd_create` for an anonymous file
        // refer to `man memfd_create` or `dev/shm`
        let mut hasher = DefaultHasher::new();
        let now = Instant::now();
        now.hash(&mut hasher);
        // in linux, set env TMPDIR to set the tmp directory. Otherwise, it fails
        let tmp_dir = env::temp_dir();
        let tmp_name = format!("{}/_verus_assert_comment_{:?}_.rs", tmp_dir.display(), hasher.finish());
        dbg!(&tmp_name);
        let path = Path::new(&tmp_name);
        let display = path.display();

        // Open a file in write-only mode, returns `io::Result<File>`
        let mut file = match File::create(&path) {
            Err(why) => {
                dbg!("couldn't create {}: {}", display, why);
                return None;
            }
            Ok(file) => file,
        };

        // Write the modified verus program to `file`, returns `io::Result<()>`
        match file.write_all(fn_as_text.as_bytes()) {
            Err(why) => {
                dbg!("couldn't write to {}: {}", display, why);
                return None;
            }
            Ok(_) => dbg!("successfully wrote to {}", display),
        };

        let output = 
            Command::new(fmt_path)
                .arg(path)
                .output();

        let output = output.ok()?;
        dbg!(&output);
        if output.status.success() {
            // let contents = std::fs::read_to_string(path).expect("Should have been able to read the file");

            let mut result = Vec::new();
            let mut is_line_target = false;
            for line in read_to_string(path).expect("Should have been able to read the file").lines() {
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
        } else {
            // disambiguate verification failure     VS    compile error etc
            // match std::str::from_utf8(&output.stdout) {
            //     Ok(out) => {
            //         if out.contains("verification results:: verified: 0 errors: 0") {
            //             // failure from other errors. (e.g. compile error)
            //             return None;
            //         } else {
            //             // verification failure
            //             return Some(false);
            //         }
            //     }
            //     Err(_) => return None,
            // }
            return None;
        }
    }
}
