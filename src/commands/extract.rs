use std::{
    fs::{self},
    path::Path,
};

use anyhow::{Context, Result};
use glob::{MatchOptions, Pattern};

use crate::fs::{FS, FileSystem};

/// Extract files to disk matching a glob pattern
pub fn extract_files(fs: &mut FS, patterns: &[Pattern], output_folder: &Path) -> Result<()> {
    let filenames = fs
        .list()
        .filter(|filename| {
            patterns.iter().any(|pattern| {
                pattern.matches_with(
                    filename,
                    MatchOptions {
                        require_literal_separator: true,
                        ..Default::default()
                    },
                )
            })
        })
        .collect::<Vec<_>>();

    fs.batch_read(&filenames)
        // Print and filter out errors
        .filter_map(|f| match f {
            Ok(x) => Some(x),
            Err((path, e)) => {
                log::error!("Failed to extract file: {:?}: {:?}", path, e);
                None
            }
        })
        // Attempt to read file contents
        .map(|(filename, contents)| -> Result<_, anyhow::Error> {
            let out_filename = output_folder.join(filename.as_ref());
            fs::create_dir_all(out_filename.parent().unwrap()).with_context(|| {
                format!(
                    "Failed to create folder: {:?}",
                    out_filename.parent().unwrap()
                )
            })?;

            fs::write(&out_filename, &contents)
                .with_context(|| format!("Failed to write file: {:?}", out_filename))?;

            Ok(filename)
        })
        // Report results
        .for_each(|result| match result {
            Ok(filename) => log::info!("Extracted file: {}", filename),
            Err(e) => log::error!("Failed to extract file: {:?}", e),
        });

    Ok(())
}
