use std::{fs, path::Path};

use anyhow::{Context, Result, anyhow, ensure};
use glob::{MatchOptions, Pattern};

use crate::{
    arm::{parser::parse_map_str, types::Map},
    bundle_fs::FS,
};

fn parse_map(contents: &[u8]) -> Result<Map> {
    ensure!(contents[..2] == [0xff, 0xfe], ".arm magic number mismatch");

    let input =
        String::from_utf16le(contents).context("Failed to parse contents as UTF16LE string")?;

    let (remaining, map) =
        parse_map_str(&input).map_err(|e| anyhow!("Failed to parse map file: {:?}", e))?;
    // println!("{:#?}", map);

    Ok(map)
}

/// Extract files to disk matching a glob pattern
pub fn dump_maps(fs: &mut FS, patterns: &[Pattern], output_folder: &Path) -> Result<()> {
    for pattern in patterns {
        ensure!(
            pattern.as_str().ends_with(".arm"),
            "Only .arm map export is supported."
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
            let map = parse_map(&contents)
                .with_context(|| format!("Failed to parse file: {:?}", filename))
                .unwrap();

            let out_path = output_folder.join(filename).with_extension("json");
            fs::create_dir_all(out_path.parent().unwrap()).context("Failed to create folder")?;

            let f = std::fs::File::create(&out_path)
                .with_context(|| format!("Failed to create file {:?}", out_path))?;
            serde_json::to_writer_pretty(f, &map).context("Failed to serialise map")?;

            Ok(filename)
        })
        // Report results
        .for_each(|result| match result {
            Ok(filename) => eprintln!("Extracted file: {}", filename),
            Err(e) => eprintln!("Failed to extract file: {:?}", e),
        });

    Ok(())
}
