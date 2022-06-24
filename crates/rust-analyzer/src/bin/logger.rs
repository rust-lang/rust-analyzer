//! Simple logger that logs either to stderr or to a file, using `tracing_subscriber`
//! filter syntax and `tracing_appender` for non blocking output.

use std::{
    fs::File,
    io::{self, Stderr},
    sync::Arc,
};

use rust_analyzer::Result;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    fmt::{writer::BoxMakeWriter, MakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};
use tracing_tree::HierarchicalLayer;

pub(crate) struct Logger {
    filter: EnvFilter,
    file: Option<File>,
}

struct MakeWriterStderr;

impl<'a> MakeWriter<'a> for MakeWriterStderr {
    type Writer = Stderr;

    fn make_writer(&'a self) -> Self::Writer {
        io::stderr()
    }
}

impl Logger {
    pub(crate) fn new(file: Option<File>, filter: Option<&str>) -> Logger {
        let filter = filter.map_or(EnvFilter::default(), EnvFilter::new);

        Logger { filter, file }
    }

    pub(crate) fn install(self) -> Result<()> {
        // The meaning of CHALK_DEBUG I suspected is to tell chalk crates
        // (i.e. chalk-solve, chalk-ir, chalk-recursive) how to filter tracing
        // logs. But now we can only have just one filter, which means we have to
        // merge chalk filter to our main filter (from RA_LOG env).
        //
        // The acceptable syntax of CHALK_DEBUG is `target[span{field=value}]=level`.
        // As the value should only affect chalk crates, we'd better manually
        // specify the target. And for simplicity, CHALK_DEBUG only accept the value
        // that specify level.
        let chalk_level_dir = std::env::var("CHALK_DEBUG")
            .map(|val| {
                val.parse::<LevelFilter>().expect(
                    "invalid CHALK_DEBUG value, expect right log level (like debug or trace)",
                )
            })
            .ok();

        let writer = match self.file {
            Some(file) => BoxMakeWriter::new(Arc::new(file)),
            None => BoxMakeWriter::new(io::stderr),
        };

        let ra_fmt_layer = HierarchicalLayer::default()
            .with_indent_lines(true)
            .with_ansi(false)
            .with_indent_amount(2)
            .with_targets(true)
            .with_writer(writer);

        let mut filter = self.filter;
        if let Some(val) = chalk_level_dir {
            filter = filter
                .add_directive(format!("chalk_solve={}", val).parse()?)
                .add_directive(format!("chalk_ir={}", val).parse()?)
                .add_directive(format!("chalk_recursive={}", val).parse()?);
        }
        Registry::default().with(filter).with(ra_fmt_layer).init();

        Ok(())
    }
}
