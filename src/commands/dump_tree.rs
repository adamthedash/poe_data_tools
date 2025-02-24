use std::{collections::HashMap, path::Path};

use anyhow::{anyhow, bail, Context, Result};
use arrow_array::{
    cast::AsArray,
    types::{GenericStringType, Int32Type, UInt16Type, UInt64Type},
    RecordBatch,
};
use itertools::izip;

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

/// data/ascendancy.datc64
fn parse_ascendancy_table(table: &RecordBatch) -> Result<(Vec<Ascendancy>, Vec<usize>)> {
    let ids = table
        .column_by_name("Id")
        .context("Column not found: Id")?
        .as_string_opt::<i32>()
        .context("Couldn't cast column to string")?
        .into_iter()
        .map(|id| id.expect("No value for ascendancy ID"))
        .collect::<Vec<_>>();

    let names = table
        .column_by_name("Name")
        .context("Column not found: Name")?
        .as_string_opt::<i32>()
        .context("Couldn't cast column to string")?
        .into_iter()
        .map(|id| id.expect("No value for ascendancy ID"))
        .collect::<Vec<_>>();

    let flavour_text = table
        .column_by_name("FlavourText")
        .context("Column not found: FlavourText")?
        .as_string_opt::<i32>()
        .context("Couldn't cast column to string")?
        .into_iter()
        .collect::<Vec<_>>();

    // TODO: RGB to hex
    let flavour_text_colour = table
        .column_by_name("RGBFlavourTextColour")
        .context("Column not found: RGBFlavourTextColour")?
        .as_string_opt::<i32>()
        .context("Couldn't cast column to string")?
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
        .column_by_name("Characters")
        .context("Column not found: Characters")?
        .as_list_opt::<i32>()
        .context("Couldn't cast column to list")?
        .iter()
        .map(|s| -> Result<_> {
            let parent_class_id = s
                .context("No value for parent class")?
                .as_primitive_opt::<UInt64Type>()
                .context("Couldn't cast parent class list to u64")?
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
fn parse_character_table(table: &RecordBatch) -> Result<Vec<Class>> {
    let names = table
        .column_by_name("Name")
        .context("Column not found: Name")?
        .as_string_opt::<i32>()
        .context("Couldn't cast column to string")?
        .into_iter()
        .map(|x| x.expect("No value for character name"))
        .collect::<Vec<_>>();

    let base_dexs = table
        .column_by_name("BaseDexterity")
        .context("Column not found: BaseDexterity")?
        .as_primitive_opt::<Int32Type>()
        .context("Couldn't cast column to i32")?
        .into_iter()
        .map(|x| x.expect("No value for base dexterity"))
        .collect::<Vec<_>>();

    let base_strs = table
        .column_by_name("BaseStrength")
        .context("Column not found: BaseStrength")?
        .as_primitive_opt::<Int32Type>()
        .context("Couldn't cast column to i32")?
        .into_iter()
        .map(|x| x.expect("No value for base strength"))
        .collect::<Vec<_>>();

    let base_ints = table
        .column_by_name("BaseIntelligence")
        .context("Column not found: BaseIntelligence")?
        .as_primitive_opt::<Int32Type>()
        .context("Couldn't cast column to i32")?
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
fn parse_passive_table(table: &RecordBatch) -> Result<()> {
    let skill_graph_node_ids = table
        .column_by_name("PassiveSkillGraphId")
        .context("Column not found: PassiveSkillGraphId")?
        .as_primitive_opt::<UInt64Type>()
        .context("Couldn't cast column to u64")?
        .into_iter()
        .map(|x| x.expect("Unexpected null value"))
        .collect::<Vec<_>>();

    let names = table
        .column_by_name("Name")
        .context("Column not found: Name")?
        .as_string_opt::<i32>()
        .context("Couldn't cast column to string")?
        .into_iter()
        .collect::<Vec<_>>();

    let ascendancy_id = table
        .column_by_name("Ascendancy")
        .context("Column not found: Ascendancy")?
        .as_primitive_opt::<UInt64Type>()
        .context("Couldn't cast column to u64")?
        .into_iter()
        .collect::<Vec<_>>();

    let mastery_group = table
        .column_by_name("MasteryGroup")
        .context("Column not found: MasteryGroup")?
        .as_primitive_opt::<UInt64Type>()
        .context("Couldn't cast column to u64")?
        .into_iter()
        .collect::<Vec<_>>();

    // TODO: The only one that's null is the starting root node
    let icon_dds_file = table
        .column_by_name("Icon_DDSFile")
        .context("Column not found: Icon_DDSFile")?
        .as_string_opt::<i32>()
        .context("Couldn't cast column to string")?
        .into_iter()
        .collect::<Vec<_>>();

    let stats = table
        .column_by_name("Stats")
        .context("Column not found: Stats")?
        .as_list_opt::<i32>()
        .context("Couldn't cast column to list")?
        .iter()
        .map(|s| {
            let stat_ids = s
                .expect("Stats list is null")
                .as_primitive_opt::<UInt64Type>()
                .context("Couldn't cast stats list to u64")?
                .into_iter()
                .map(|x| x.expect("Stat ID value is null"))
                .collect::<Vec<_>>();

            Ok(stat_ids)
        })
        .collect::<Result<Vec<_>>>()?;

    let stat_values = (0..4)
        .map(|i| {
            let col_name = format!("Stat{}Value", i + 1);
            let stat_value = table
                .column_by_name(&col_name)
                .with_context(|| format!("Column not found: {}", col_name))?
                .as_primitive_opt::<Int32Type>()
                .context("Couldn't cast column to i32")?
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
