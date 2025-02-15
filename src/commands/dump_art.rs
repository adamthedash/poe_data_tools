use std::{fs, path::Path};

use anyhow::{ensure, Context, Result};
use glob::Pattern;

use crate::bundle_fs::FS;

/// Extract files to disk matching a glob pattern
pub fn extract_art(fs: &mut FS, pattern: &Pattern, output_folder: &Path) -> Result<()> {
    ensure!(
        pattern.as_str().ends_with(".dds"),
        "Only .dds art export is supported."
    );

    fs.list()
        .iter()
        .filter(|filename| pattern.matches(filename))
        .map(|filename| -> Result<_, anyhow::Error> {
            // Dump it to disk
            let contents = fs.read(filename).context("Failed to read file")?;

            let img = image::load_from_memory(&contents).context("Failed to pares DDS image")?;

            let out_filename = output_folder.join(filename).with_extension("png");
            fs::create_dir_all(out_filename.parent().unwrap())
                .context("Failed to create folder")?;

            img.save(out_filename).context("Failed to write file")?;

            Ok(filename)
        })
        .for_each(|result| match result {
            Ok(filename) => eprintln!("Extracted file: {}", filename),
            Err(e) => eprintln!("Failed to extract file: {:?}", e),
        });

    Ok(())
}
