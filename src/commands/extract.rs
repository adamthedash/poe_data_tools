use std::{
    fs::{self},
    path::Path,
};

use anyhow::{Context, Result};
use glob::Pattern;

use crate::bundle_fs::FS;

/// Extract files to disk matching a glob pattern
pub fn extract_files(fs: &mut FS, pattern: &Pattern, output_folder: &Path) -> Result<()> {
    fs.list()
        .iter()
        .filter(|filename| pattern.matches(filename))
        .try_for_each(|filename| -> Result<(), anyhow::Error> {
            // Dump it to disk
            let contents = fs.read(filename).context("Failed to read file")?;

            let out_filename = output_folder.join(filename);
            fs::create_dir_all(out_filename.parent().unwrap())
                .context("Failed to create folder")?;

            fs::write(out_filename, &contents).context("Failed to write file")?;
            eprintln!("{}", filename);

            Ok(())
        })?;

    Ok(())
}
