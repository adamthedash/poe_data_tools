use std::{collections::HashMap, fs::create_dir_all, path::Path};

use anyhow::{Context, Result, anyhow, bail, ensure};
use glob::{MatchOptions, Pattern};

use super::Patch;
use crate::{
    VERBOSE,
    bundle_fs::FS,
    tree::{
        passive_info::{PassiveSkillInfo, load_passive_info},
        psg::PassiveSkillGraph,
    },
};

fn process_file(
    contents: &[u8],
    output_path: &Path,
    version: &Patch,
    passive_info: &HashMap<u16, PassiveSkillInfo>,
) -> Result<()> {
    // Parse the PSG file
    let (_, mut passive_tree) = match version {
        Patch::One => PassiveSkillGraph::parse_poe1(contents),
        Patch::Two => PassiveSkillGraph::parse_poe2(contents),
        _ => bail!("Only patch versions 1/2 supported for table extraction."),
    }
    .map_err(|e| anyhow!("Failed to parse passive skill tree: {:?}", e))?;

    // Add passive info - only nodes that are in the graph
    let passive_info = {
        let ids = passive_tree
            .groups
            .iter()
            .flat_map(|g| g.passives.iter().map(|p| p.id as u16));
        ids.map(|id| (id, passive_info[&id].clone())).collect()
    };
    passive_tree.passive_info = Some(passive_info);

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
    cache_dir: &Path,
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

    let passive_info = load_passive_info(fs, version, cache_dir)?
        .into_iter()
        .map(|p| (p.graph_passive_id, p))
        .collect::<HashMap<_, _>>();

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
            process_file(&contents, &output_path, version, &passive_info)
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
