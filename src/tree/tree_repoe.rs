#![allow(dead_code, unused_variables)]
/// Transforming raw PSG data into the format provided by RePoE
/// TODO: This whole thing
use std::{any::type_name, collections::HashMap};

use anyhow::{Context, Result};
use arrow_array::{
    cast::AsArray,
    types::{Int32Type, UInt64Type},
    ArrowPrimitiveType, GenericListArray, GenericStringArray, PrimitiveArray, RecordBatch,
};
use itertools::izip;

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

trait ColumnHelper {
    fn get_column_as<T: ArrowPrimitiveType>(&self, column: &str) -> Result<&PrimitiveArray<T>>;

    fn get_column_as_string(&self, column: &str) -> Result<&GenericStringArray<i32>>;

    fn get_column_as_list(&self, column: &str) -> Result<&GenericListArray<i32>>;
}

impl ColumnHelper for RecordBatch {
    fn get_column_as<T: ArrowPrimitiveType>(&self, column: &str) -> Result<&PrimitiveArray<T>> {
        self.column_by_name(column)
            .with_context(|| format!("Column not found: {column}"))?
            .as_primitive_opt::<T>()
            .with_context(|| format!("Couldn't parse column {column:?} as {:?}", type_name::<T>()))
    }

    fn get_column_as_string(&self, column: &str) -> Result<&GenericStringArray<i32>> {
        self.column_by_name(column)
            .with_context(|| format!("Column not found: {column}"))?
            .as_string_opt()
            .with_context(|| format!("Couldn't parse column {column:?} as string"))
    }

    fn get_column_as_list(&self, column: &str) -> Result<&GenericListArray<i32>> {
        self.column_by_name(column)
            .with_context(|| format!("Column not found: {column}"))?
            .as_list_opt()
            .with_context(|| format!("Couldn't parse column {column:?} as list"))
    }
}

/// data/ascendancy.datc64
fn parse_ascendancy_table(table: &RecordBatch) -> Result<(Vec<Ascendancy<'_>>, Vec<usize>)> {
    let ids = table
        .get_column_as_string("Id")?
        .into_iter()
        .map(|id| id.expect("No value for ascendancy ID"))
        .collect::<Vec<_>>();

    let names = table
        .get_column_as_string("Name")?
        .into_iter()
        .map(|id| id.expect("No value for ascendancy ID"))
        .collect::<Vec<_>>();

    let flavour_text = table
        .get_column_as_string("FlavourText")?
        .into_iter()
        .collect::<Vec<_>>();

    // TODO: RGB to hex
    let flavour_text_colour = table
        .get_column_as_string("RGBFlavourTextColour")?
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
        .get_column_as::<Int32Type>("ClassNo")?
        .into_iter()
        .map(|x| x.expect("No value for parent class ID") as usize)
        .collect::<Vec<_>>();

    Ok((ascendancies, parent_classes))
}

/// data/characters.datc64
fn parse_character_table(table: &RecordBatch) -> Result<Vec<Class<'_>>> {
    let names = table
        .get_column_as_string("Name")?
        .into_iter()
        .map(|x| x.expect("No value for character name"))
        .collect::<Vec<_>>();

    let base_dexs = table
        .get_column_as::<Int32Type>("BaseDexterity")?
        .into_iter()
        .map(|x| x.expect("No value for base dexterity"))
        .collect::<Vec<_>>();

    let base_strs = table
        .get_column_as::<Int32Type>("BaseStrength")?
        .into_iter()
        .map(|x| x.expect("No value for base strength"))
        .collect::<Vec<_>>();

    let base_ints = table
        .get_column_as::<Int32Type>("BaseIntelligence")?
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
        .get_column_as::<UInt64Type>("PassiveSkillGraphId")?
        .into_iter()
        .map(|x| x.expect("Unexpected null value"))
        .collect::<Vec<_>>();

    let names = table
        .get_column_as_string("Name")?
        .into_iter()
        .collect::<Vec<_>>();

    let ascendancy_id = table
        .get_column_as::<UInt64Type>("Ascendancy")?
        .into_iter()
        .collect::<Vec<_>>();

    let mastery_group = table
        .get_column_as::<UInt64Type>("MasteryGroup")?
        .into_iter()
        .collect::<Vec<_>>();

    // TODO: The only one that's null is the starting root node
    let icon_dds_file = table
        .get_column_as_string("Icon_DDSFile")?
        .into_iter()
        .collect::<Vec<_>>();

    let stats = table
        .get_column_as_list("Stats")?
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
                .get_column_as::<Int32Type>(&col_name)?
                .into_iter()
                .map(|x| x.expect("Stat value is null"))
                .collect::<Vec<_>>();

            Ok(stat_value)
        })
        .collect::<Result<Vec<_>>>()?;

    // TODO: Look up stat formatting functions, insert values and return as strings

    unimplemented!()
}
