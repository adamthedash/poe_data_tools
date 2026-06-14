use std::path::Path;

use anyhow::{Context, Result};
use glob::{MatchOptions, Pattern};
use poe_data_tools::{
    Patch,
    file_parsers::{FileParserExt, Parser},
    fs::{FS, FileSystem},
};

/// Extract, parse and transform files into easier to parse alternatives
pub fn translate(
    fs: &mut FS,
    patterns: &[Pattern],
    _cache_dir: &Path,
    output_folder: &Path,
    poe_version: &Patch,
) -> Result<()> {
    let filenames = fs
        .list()
        // Filter on globs
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
        // Filter out files that we can't parse
        // TODO: This might be expensive, also we might want to log skips?
        .filter(|filename| {
            Parser::from_filename(Path::new(filename), poe_version.major()).is_some()
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
            log::info!("Extracting file: {filename}");
            let parser = Parser::from_filename(Path::new(filename.as_ref()), poe_version.major())
                .expect("Already verified parser exists above");

            let out_path = output_folder
                .join(filename.as_ref())
                .with_added_extension("json");
            parser
                .parse_to_json_file(&contents, &out_path)
                .with_context(|| format!("Failed to process file: {:?}", filename))?;

            Ok(filename)
        })
        // Report results
        .for_each(|result| match result {
            Ok(filename) => log::info!("Extracted file: {}", filename),
            Err(e) => log::error!("{:?}", e),
        });

    Ok(())
}
