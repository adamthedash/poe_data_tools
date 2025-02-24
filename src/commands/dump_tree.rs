use std::{collections::HashMap, path::Path};

use anyhow::{anyhow, bail, Context, Result};
use arrow_array::{cast::AsArray, types::UInt16Type};

use super::Patch;
use crate::{
    bundle_fs::FS, commands::dump_tables::load_parsed_table, dat::ivy_schema::fetch_schema,
    tree::psg::parse_psg,
};

pub fn dump_tree(
    fs: &mut FS,
    cache_dir: &Path,
    _output_folder: &Path,
    version: &Patch,
) -> Result<()> {
    let version = match version {
        Patch::One => 1,
        Patch::Two => 2,
        _ => bail!("Only patch versions 1/2 supported for table extraction."),
    };

    // Load PSG
    let passive_tree_bytes = fs.read("metadata/passiveskillgraph.psg")?;
    let (_, _passive_tree) = parse_psg(&passive_tree_bytes)
        .map_err(|e| anyhow!("Failed to parse passive skill tree: {:?}", e))?;

    // Load DAT schemas
    let schemas = fetch_schema(cache_dir)
        .context("Failed to fetch schema file")
        .unwrap();

    // Load DAT tables
    // ==================== Passive skills ==================
    let passive_table = load_parsed_table(fs, &schemas, "data/passiveskills.datc64", version)?;
    println!("{:?}", passive_table);

    // Create LUT between passive skill ID and passive row
    let col_name = "PassiveSkillGraphId";
    let column = passive_table
        .column_by_name(col_name)
        .context(format!("Column not found in dat table: {:?}", col_name))?;
    let _passive_lut = column
        .as_primitive::<UInt16Type>()
        .into_iter()
        .enumerate()
        .fold(HashMap::new(), |mut hm, (row, passive_id)| {
            if let Some(passive_id) = passive_id {
                hm.entry(passive_id).insert_entry(row);
            };

            hm
        });

    //println!("{:#?}", passive_lut);

    // ==================== Ascendancy ==================
    let ascendancy_table = load_parsed_table(fs, &schemas, "data/ascendancy.datc64", version)?;
    println!("{:?}", ascendancy_table);

    // ==================== Stats ==================
    let stats_table = load_parsed_table(fs, &schemas, "data/stats.datc64", version)?;
    println!("{:?}", stats_table);

    Ok(())
}
