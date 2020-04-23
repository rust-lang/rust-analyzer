use std::{ffi::OsStr, fs, io, path::Path, time};

use ra_ide::Analysis;

use crate::cli::Verbosity;

pub fn integrity_suite(verbosity: Verbosity, dir: &Path, only: Option<&str>) -> io::Result<()> {
    let start = time::Instant::now();
    let mut count = 0;
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() || path.extension() != Some(OsStr::new("rs")) {
            continue;
        }
        if let Some(only) = only {
            if !path.display().to_string().contains(only) {
                continue;
            }
        }
        if verbosity.is_verbose() {
            eprintln!("{}", path.display())
        }
        count += 1;
        let text = fs::read_to_string(entry.path()).unwrap();

        let (analysis, file_id) = Analysis::from_single_file(text);
        analysis.highlight(file_id).unwrap();
    }
    eprintln!("Finished {} tests in {:?}", count, start.elapsed());
    Ok(())
}
