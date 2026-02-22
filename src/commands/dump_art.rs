use std::{fs, path::Path};

use anyhow::{Context, Result, ensure};
use glob::{MatchOptions, Pattern};

use crate::bundle_fs::FS;

/// Extract files to disk matching a glob pattern
pub fn extract_art(fs: &mut FS, patterns: &[Pattern], output_folder: &Path) -> Result<()> {
    for pattern in patterns {
        ensure!(
            pattern.as_str().ends_with(".dds"),
            "Only .dds art export is supported."
        );
    }

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
    let filenames = filenames.iter().map(|f| f.as_str()).collect::<Vec<_>>();

    fs.batch_read(&filenames)
        // Print and filter out errors
        .filter_map(|f| match f {
            Ok(x) => Some(x),
            Err((path, e)) => {
                eprintln!("Failed to extract file: {:?}: {:?}", path, e);
                None
            }
        })
        // Attempt to read file contents
        .map(|(filename, contents)| -> Result<_, anyhow::Error> {
            let img = image::load_from_memory(&contents).context("Failed to pares DDS image")?;

            let out_filename = output_folder.join(filename).with_extension("png");
            fs::create_dir_all(out_filename.parent().unwrap())
                .context("Failed to create folder")?;

            img.save(out_filename).context("Failed to write file")?;

            Ok(filename)
        })
        // Report results
        .for_each(|result| match result {
            Ok(filename) => eprintln!("Extracted file: {}", filename),
            Err(e) => eprintln!("Failed to extract file: {:?}", e),
        });

    Ok(())
}
