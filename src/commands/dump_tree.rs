use std::{collections::HashMap, path::Path};

use anyhow::{anyhow, bail, Context, Result};
use itertools::izip;
use polars::{frame::DataFrame, prelude::Column};

use super::Patch;
use crate::{
    bundle_fs::FS, commands::dump_tables::load_parsed_table, dat::ivy_schema::fetch_schema,
    tree::psg::parse_psg,
};

#[derive(Debug)]
struct Ascendancy<'a> {
    id: &'a str,
    name: &'a str,
    flavour_text: Option<&'a str>,
    flavour_text_colour: Option<&'a str>,
    flavour_text_rect: HashMap<&'a str, i32>,
}

#[derive(Debug)]
struct Class<'a> {
    name: &'a str,
    base_str: i32,
    base_int: i32,
    base_dex: i32,
    ascendancies: Vec<Ascendancy<'a>>,
}

#[derive(Debug)]
struct Background<'a> {
    image: &'a str,
    is_half_image: bool,
}

#[derive(Debug)]
struct Group<'a> {
    x: f32,
    y: f32,
    orbits: Vec<usize>,
    background: Background<'a>,
    nodes: Vec<u32>,
}

#[derive(Debug)]
struct Node<'a> {
    skill: u32,
    name: &'a str,
    icon: &'a str,
    ascendancy_name: &'a str,
    stats: Vec<&'a str>,
    group: usize,
    orbit: usize,
    orbit_index: usize,
    nodes_out: Vec<&'a str>,
    nodes_in: Vec<&'a str>,
}

#[derive(Debug)]
struct ExtraImage<'a> {
    x: f32,
    y: f32,
    image: &'a str,
}

#[derive(Debug)]
struct Sprite<'a> {
    filename: &'a str,
    w: usize,
    v: usize,
    coords: HashMap<&'a str, HashMap<&'a str, usize>>,
}

#[derive(Debug)]
struct Root<'a> {
    tree: &'a str,
    classes: Vec<Class<'a>>,
    groups: HashMap<&'a str, Group<'a>>,
    nodes: HashMap<&'a str, Node<'a>>,
    extra_images: HashMap<&'a str, ExtraImage<'a>>,
    jewel_slots: Vec<u32>,

    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,

    constants: HashMap<&'a str, HashMap<&'a str, usize>>,
    sprites: HashMap<&'a str, HashMap<f32, Sprite<'a>>>,
    image_zoom_levels: Vec<f32>,
    points: HashMap<&'a str, usize>,
}

pub trait DataFrameHelpers {
    fn try_get_col_by_name(&self, name: &str) -> Result<&Column>;
}

impl DataFrameHelpers for DataFrame {
    fn try_get_col_by_name(&self, name: &str) -> Result<&Column> {
        let col_index = self
            .try_get_column_index(name)
            .context(format!("Column not found in dat table: {:?}", name))?;

        Ok(&self[col_index])
    }
}

/// data/ascendancy.datc64
fn parse_ascendancy_table(table: &DataFrame) -> Result<(Vec<Ascendancy>, Vec<usize>)> {
    let ids = table
        .try_get_col_by_name("Id")?
        .str()?
        .into_iter()
        .map(|id| id.expect("No value for ascendancy ID"))
        .collect::<Vec<_>>();

    let names = table
        .try_get_col_by_name("Name")?
        .str()?
        .into_iter()
        .map(|id| id.expect("No value for ascendancy ID"))
        .collect::<Vec<_>>();

    let flavour_text = table
        .try_get_col_by_name("FlavourText")?
        .str()?
        .into_iter()
        .collect::<Vec<_>>();

    // TODO: RGB to hex
    let flavour_text_colour = table
        .try_get_col_by_name("RGBFlavourTextColour")?
        .str()?
        .into_iter()
        .collect::<Vec<_>>();

    // TODO
    let flavour_text_rect = (0..ids.len()).map(|_| HashMap::new()).collect::<Vec<_>>();

    let ascendancies = izip!(
        ids,
        names,
        flavour_text,
        flavour_text_colour,
        flavour_text_rect
    )
    .map(
        |(id, name, flavour_text, flavour_text_colour, flavour_text_rect)| Ascendancy {
            id,
            name,
            flavour_text,
            flavour_text_colour,
            flavour_text_rect,
        },
    )
    .collect::<Vec<_>>();

    let parent_classes = table
        .try_get_col_by_name("Characters")?
        .list()?
        .into_iter()
        .map(|s| -> Result<_> {
            let parent_class_id = s
                .context("No value for parent class")?
                .u64()?
                .into_iter()
                .next()
                .context("Parent class list is empty")?
                .expect("Null value for parent class");

            Ok(parent_class_id as usize)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((ascendancies, parent_classes))
}

/// data/characters.datc64
fn parse_character_table(table: &DataFrame) -> Result<Vec<Class>> {
    let names = table
        .try_get_col_by_name("Name")?
        .str()?
        .into_iter()
        .map(|x| x.expect("No value for character name"))
        .collect::<Vec<_>>();

    let base_dexs = table
        .try_get_col_by_name("BaseDexterity")?
        .i32()?
        .into_iter()
        .map(|x| x.expect("No value for base dexterity"))
        .collect::<Vec<_>>();

    let base_strs = table
        .try_get_col_by_name("BaseStrength")?
        .i32()?
        .into_iter()
        .map(|x| x.expect("No value for base strength"))
        .collect::<Vec<_>>();

    let base_ints = table
        .try_get_col_by_name("BaseIntelligence")?
        .i32()?
        .into_iter()
        .map(|x| x.expect("No value for base intelligence"))
        .collect::<Vec<_>>();

    let classes = izip!(names, base_dexs, base_ints, base_strs)
        .map(|(name, base_dex, base_int, base_str)| Class {
            name,
            base_str,
            base_int,
            base_dex,
            ascendancies: vec![],
        })
        .collect::<Vec<_>>();

    Ok(classes)
}

/// data/passiveskills.datc64
fn parse_passive_table(table: &DataFrame) -> Result<()> {
    let skill_graph_node_ids = table
        .try_get_col_by_name("PassiveSkillGraphId")?
        .u64()?
        .into_iter()
        .map(|x| x.expect("Unexpected null value"))
        .collect::<Vec<_>>();

    let names = table
        .try_get_col_by_name("Name")?
        .str()?
        .into_iter()
        .collect::<Vec<_>>();

    let ascendancy_id = table
        .try_get_col_by_name("Ascendancy")?
        .u64()?
        .into_iter()
        .collect::<Vec<_>>();

    let mastery_group = table
        .try_get_col_by_name("MasteryGroup")?
        .u64()?
        .into_iter()
        .collect::<Vec<_>>();

    // TODO: The only one that's null is the starting root node
    let icon_dds_file = table
        .try_get_col_by_name("Icon_DDSFile")?
        .str()?
        .into_iter()
        .collect::<Vec<_>>();

    let stats = table
        .try_get_col_by_name("Stats")?
        .list()?
        .into_iter()
        .map(|s| {
            let stat_ids = s
                .expect("Stats list is null")
                .u64()?
                .into_iter()
                .map(|x| x.expect("Stat ID value is null"))
                .collect::<Vec<_>>();

            Ok(stat_ids)
        })
        .collect::<Result<Vec<_>>>()?;

    let stat_values = (0..4)
        .map(|i| {
            let stat_value = table
                .try_get_col_by_name(&format!("Stat{}Value", i + 1))?
                .i32()?
                .into_iter()
                .map(|x| x.expect("Stat value is null"))
                .collect::<Vec<_>>();

            Ok(stat_value)
        })
        .collect::<Result<Vec<_>>>()?;

    // TODO: Look up stat formatting functions, insert values and return as strings

    unimplemented!()
}

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
    let (_, passive_tree) = parse_psg(&passive_tree_bytes)
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
    let _passive_lut = passive_table
        .try_get_col_by_name("PassiveSkillGraphId")?
        .u16()?
        .into_iter()
        .enumerate()
        .fold(HashMap::new(), |mut hm, (row, passive_id)| {
            if let Some(passive_id) = passive_id {
                hm.entry(passive_id).insert_entry(row);
            };

            hm
        });

    //println!("{:#?}", passive_lut);

    // ==================== Characters (Base class) ==================
    let character_table = load_parsed_table(fs, &schemas, "data/characters.datc64", version)?;
    println!("{:?}", character_table);
    let mut characters = parse_character_table(&character_table)
        .context("Failed to collect required information from the characters table")?;

    // ==================== Ascendancy ==================
    let ascendancy_table = load_parsed_table(fs, &schemas, "data/ascendancy.datc64", version)?;
    println!("{:?}", ascendancy_table);
    let (ascendancies, parent_class_ids) = parse_ascendancy_table(&ascendancy_table)
        .context("Failed to collect required information from the ascendancies table")?;
    //println!("{:#?}", ascendancies);

    // Assign ascendancies to their parent class
    parent_class_ids
        .into_iter()
        .zip(ascendancies)
        .for_each(|(i, a)| {
            characters[i].ascendancies.push(a);
        });

    //println!("{:#?}", characters);

    // ==================== Groups ==================
    // TODO: Where do we infer group number from? Natural ordering of PSG file?
    let groups = passive_tree
        .groups
        .iter()
        .map(|g| {
            let nodes = g.passives.iter().map(|p| p.id).collect::<Vec<_>>();

            Group {
                x: g.x,
                y: g.y,
                // TODO
                orbits: vec![],
                // TODO
                background: Background {
                    image: "TODO",
                    is_half_image: false,
                },
                nodes,
            }
        })
        .collect::<Vec<_>>();

    //println!("{:#?}", groups);
    //println!("num groups: {:?}", groups.len());

    // ==================== Stats ==================
    let stats_table = load_parsed_table(fs, &schemas, "data/stats.datc64", version)?;
    println!("{:?}", stats_table);

    Ok(())
}
