use std::{fs, io::BufWriter, path::Path};

use anyhow::{Context, Result, ensure};
use glob::{MatchOptions, Pattern};

use crate::{bundle_fs::FS, file_parsers::ecf::parse_ecf};

/// Extract files to disk matching a glob pattern
pub fn dump_ecf(fs: &mut FS, patterns: &[Pattern], output_folder: &Path) -> Result<()> {
    for pattern in patterns {
        ensure!(
            pattern.as_str().ends_with(".ecf"),
            "Only .ecf export is supported."
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
            println!("extracting: {:?}", filename);
            let parsed = parse_ecf(&contents)
                .with_context(|| format!("Failed to parse file: {:?}", filename))?;

            let out_path = output_folder.join(filename).with_extension("json");
            fs::create_dir_all(out_path.parent().unwrap()).context("Failed to create folder")?;

            let f = std::fs::File::create(&out_path)
                .with_context(|| format!("Failed to create file {:?}", out_path))?;
            let f = BufWriter::new(f);
            serde_json::to_writer(f, &parsed).context("Failed to serialise")?;

            Ok(filename)
        })
        // Report results
        .for_each(|result| match result {
            Ok(filename) => eprintln!("Extracted file: {}", filename),
            Err(e) => eprintln!("Failed to extract file: {:?}", e),
        });

    Ok(())
}
