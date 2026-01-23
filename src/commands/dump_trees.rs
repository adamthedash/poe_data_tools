use std::{fs::create_dir_all, path::Path};

use anyhow::{anyhow, bail, ensure, Context, Result};
use glob::{MatchOptions, Pattern};

use super::Patch;
use crate::{
    bundle_fs::FS,
    tree::psg::{parse_psg_poe1, parse_psg_poe2},
    VERBOSE,
};

fn process_file(contents: &[u8], output_path: &Path, version: &Patch) -> Result<()> {
    // Parse the PSG file
    let (_, passive_tree) = match version {
        Patch::One => parse_psg_poe1(contents),
        Patch::Two => parse_psg_poe2(contents),
        _ => bail!("Only patch versions 1/2 supported for table extraction."),
    }
    .map_err(|e| anyhow!("Failed to parse passive skill tree: {:?}", e))?;

    // Write to file
    create_dir_all(output_path.parent().context("No parent directory")?)
        .context("Failed to create output dirs")?;
    let f = std::fs::File::create(output_path)
        .with_context(|| format!("Failed to create file {:?}", output_path))?;
    serde_json::to_writer_pretty(f, &passive_tree).context("Failed to serialise tree to JSON")
}

pub fn dump_trees(
    fs: &mut FS,
    patterns: &[Pattern],
    output_folder: &Path,
    version: &Patch,
) -> Result<()> {
    for pattern in patterns {
        ensure!(
            pattern.as_str().ends_with(".psg"),
            "Only .psg tree export is supported."
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
        .filter_map(|f| {
            f.inspect_err(|(path, e)| {
                eprintln!("Failed to extract file: {:?}: {:?}", path, e);
            })
            .ok()
        })
        // Attempt to read file contents
        .map(|(filename, contents)| -> Result<_, anyhow::Error> {
            // Convert the data table
            let output_path = output_folder.join(filename).with_extension("json");
            process_file(&contents, &output_path, version)
                .with_context(|| format!("Failed to process file: {:?}", filename))?;

            Ok(filename)
        })
        // Report results
        .for_each(|result| match result {
            Ok(filename) => eprintln!("Extracted tree: {}", filename),
            Err(e) => {
                let error_message = if *VERBOSE.get().unwrap() {
                    format!("{e:?}")
                } else {
                    format!("{e}")
                };
                eprintln!("Failed to extract tree: {error_message}");
            }
        });

    Ok(())
}
