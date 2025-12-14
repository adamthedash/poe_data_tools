use std::io::{self, BufWriter, Write};

use anyhow::{Context, Result};
use glob::{MatchOptions, Pattern};

use crate::bundle_fs::FS;

/// List filenames matching a glob pattern
pub fn list_files(file_system: &FS, patterns: &[Pattern]) -> Result<()> {
    // Use a buffered writer since we're dumping a lot of data
    let mut stdout = BufWriter::new(io::stdout().lock());

    file_system
        .list()
        .filter(|path| {
            patterns.iter().any(|pattern| {
                pattern.matches_with(
                    path,
                    MatchOptions {
                        require_literal_separator: true,
                        ..Default::default()
                    },
                )
            })
        })
        .try_for_each(|p| writeln!(stdout, "{}", p).context("Failed to write to stdout"))?;

    stdout.flush().context("Failed to flush stdout")
}
