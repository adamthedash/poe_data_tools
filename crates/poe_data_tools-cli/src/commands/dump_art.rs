use std::{fs, path::Path};

use anyhow::{Context, Result, ensure};
use glob::{MatchOptions, Pattern};
use poe_data_tools::fs::{FS, FileSystem};

use crate::VERBOSE;

/// Extract files to disk matching a glob pattern
pub fn extract_art(fs: &mut FS, patterns: &[Pattern], output_folder: &Path) -> Result<()> {
    image_extras::register();

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

    fs.batch_read(&filenames)
        // Print and filter out errors
        .filter_map(|(path, res)| match res {
            Ok(b) => Some((path, b)),
            Err(e) => {
                log::error!("Failed to extract file: {:?}: {:?}", path, e);
                None
            }
        })
        // Attempt to read file contents
        .map(|(filename, contents)| -> Result<_, anyhow::Error> {
            let img = image::load_from_memory(&contents)
                .with_context(|| format!("Failed to parse DDS image: {filename}"))?;

            let out_filename = output_folder.join(filename.as_ref()).with_extension("png");
            fs::create_dir_all(out_filename.parent().unwrap())
                .with_context(|| format!("Failed to create output folder: {out_filename:?}"))?;

            img.save(out_filename)
                .with_context(|| format!("Failed to write file: {filename}"))?;

            Ok(filename)
        })
        // Report results
        .for_each(|result| match result {
            Ok(filename) => log::info!("Extracted file: {}", filename),
            Err(e) => {
                let error_message = if *VERBOSE.get().unwrap() {
                    format!("{e:?}")
                } else {
                    format!("{e}")
                };
                log::error!("Failed to extract file: {error_message}");
            }
        });

    Ok(())
}
