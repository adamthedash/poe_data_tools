use std::io::{self, BufWriter, Write};

use anyhow::{Context, Result};

use crate::bundle_fs::FS;

/// Write the contents of the file to stdout
pub fn cat_file(fs: &mut FS, path: &str) -> Result<()> {
    let contents = fs.read(path).context("Failed to read file")?;

    let mut stdout = BufWriter::new(io::stdout().lock());
    stdout
        .write_all(&contents)
        .context("Failed to write to stdout")?;

    stdout.flush().context("Failed to flush stdout")
}
