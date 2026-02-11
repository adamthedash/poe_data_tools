use std::{any::type_name, collections::HashMap, path::Path};

use anyhow::{Context, Result, bail};
use arrow_array::{
    ArrowPrimitiveType, BooleanArray, GenericListArray, GenericStringArray, PrimitiveArray,
    RecordBatch,
    cast::AsArray,
    types::{Int32Type, UInt16Type, UInt64Type},
};
use itertools::izip;
use serde::Serialize;

use crate::{
    bundle_fs::FS,
    commands::{Patch, dump_tables::load_parsed_table},
    dat::ivy_schema::fetch_schema,
};

trait ColumnHelper {
    fn get_column_as<T: ArrowPrimitiveType>(&self, column: &str) -> Result<&PrimitiveArray<T>>;

    fn get_column_as_string(&self, column: &str) -> Result<&GenericStringArray<i32>>;

    fn get_column_as_list(&self, column: &str) -> Result<&GenericListArray<i32>>;

    fn get_column_as_bool(&self, column: &str) -> Result<&BooleanArray>;
}

impl ColumnHelper for RecordBatch {
    fn get_column_as<T: ArrowPrimitiveType>(&self, column: &str) -> Result<&PrimitiveArray<T>> {
        let col = self
            .column_by_name(column)
            .with_context(|| format!("Column not found: {column}"))?;

        col.as_primitive_opt::<T>().with_context(|| {
            format!(
                "Couldn't parse column {column:?} as {:?}, actual type: {:?}",
                type_name::<T>(),
                col.data_type()
            )
        })
    }

    fn get_column_as_string(&self, column: &str) -> Result<&GenericStringArray<i32>> {
        let col = self
            .column_by_name(column)
            .with_context(|| format!("Column not found: {column}"))?;

        col.as_string_opt().with_context(|| {
            format!(
                "Couldn't parse column {column:?} as string, actual type: {:?}",
                col.data_type()
            )
        })
    }

    fn get_column_as_list(&self, column: &str) -> Result<&GenericListArray<i32>> {
        let col = self
            .column_by_name(column)
            .with_context(|| format!("Column not found: {column}"))?;

        col.as_list_opt().with_context(|| {
            format!(
                "Couldn't parse column {column:?} as list, actual type: {:?}",
                col.data_type()
            )
        })
    }

    fn get_column_as_bool(&self, column: &str) -> Result<&BooleanArray> {
        let col = self
            .column_by_name(column)
            .with_context(|| format!("Column not found: {column}"))?;

        col.as_boolean_opt().with_context(|| {
            format!(
                "Couldn't parse column {column:?} as bool, actual type: {:?}",
                col.data_type()
            )
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PassiveSkillInfo {
    pub flavour_text: Option<String>,
    pub graph_passive_id: u16,
    pub icon: Option<String>,
    pub passive_id: String,
    pub name: Option<String>,
    pub is_ascendancy_starting_node: bool,
    pub is_icon_only: bool,
    pub is_jewel_socket: bool,
    pub is_keystone: bool,
    pub is_multiple_choice: bool,
    pub is_multiple_choice_option: bool,
    pub is_notable: bool,
    pub skill_points: u32,
    pub reminder_text: Vec<String>,
    pub stats: HashMap<String, i32>,
}

/// Load passive skill info as in the "passives" section of RePoE
pub fn load_passive_info(
    fs: &mut FS,
    version: &Patch,
    cache_dir: &Path,
) -> Result<Vec<PassiveSkillInfo>> {
    let schemas = fetch_schema(cache_dir).context("Failed to load schema")?;

    let version_number = match version {
        Patch::One => 1,
        Patch::Two => 2,
        Patch::Specific(v) if v.starts_with("3") => 1,
        Patch::Specific(v) if v.starts_with("4") => 2,
        _ => bail!("Unsupported version: {:?}", version),
    };

    let get_filename = |f| match version_number {
        1 => format!("data/{f}.datc64"),
        2 => format!("data/balance/{f}.datc64"),
        _ => unreachable!(),
    };

    let passive_table =
        load_parsed_table(fs, &schemas, &get_filename("passiveskills"), version_number)?;

    let flavour_text = passive_table
        .get_column_as_string("FlavourText")?
        .into_iter()
        .map(|x| x.map(|x| x.to_string()));
    let graph_passive_id = passive_table
        .get_column_as::<UInt16Type>("PassiveSkillGraphId")?
        .into_iter()
        .map(Option::unwrap);
    let icon = passive_table
        .get_column_as_string("Icon_DDSFile")?
        .into_iter()
        .map(|x| x.map(|x| x.to_string()));
    let id = passive_table
        .get_column_as_string("Id")?
        .into_iter()
        .map(|s| s.unwrap().to_string());
    let is_ascendancy_starting_node = passive_table
        .get_column_as_bool("IsAscendancyStartingNode")?
        .into_iter()
        .map(Option::unwrap);
    let is_icon_only = passive_table
        .get_column_as_bool("IsJustIcon")?
        .into_iter()
        .map(Option::unwrap);
    let is_jewel_socket = passive_table
        .get_column_as_bool("IsJewelSocket")?
        .into_iter()
        .map(Option::unwrap);
    let is_keystone = passive_table
        .get_column_as_bool("IsKeystone")?
        .into_iter()
        .map(Option::unwrap);
    let is_multiple_choice = passive_table
        .get_column_as_bool("IsMultipleChoice")?
        .into_iter()
        .map(Option::unwrap);
    let is_multiple_choice_option = passive_table
        .get_column_as_bool("IsMultipleChoiceOption")?
        .into_iter()
        .map(Option::unwrap);
    let is_notable = passive_table
        .get_column_as_bool("IsNotable")?
        .into_iter()
        .map(Option::unwrap);
    let name = passive_table
        .get_column_as_string("Name")?
        .into_iter()
        .map(|x| x.map(|x| x.to_string()));
    let skill_points = passive_table
        .get_column_as::<Int32Type>("SkillPointsGranted")?
        .into_iter()
        .map(Option::unwrap);

    // Stats

    let stat_ids = passive_table.get_column_as_list("Stats")?.iter().map(|s| {
        s.expect("Stats list is null")
            .as_primitive_opt::<UInt64Type>()
            .expect("Couldn't cast stats list to u64")
            .into_iter()
            .map(|x| x.expect("Stat ID value is null"))
            .collect::<Vec<_>>()
    });

    let stat_values = (1..=5)
        .map(|i| -> Result<_> {
            let values = passive_table
                .get_column_as::<Int32Type>(&format!("Stat{i}Value"))?
                .into_iter()
                .map(Option::unwrap);

            Ok(values)
        })
        .collect::<Result<Vec<_>>>()?;
    let stat_values = Zip { iters: stat_values };

    let stat_table = load_parsed_table(fs, &schemas, &get_filename("stats"), version_number)?;

    let stats = stat_table
        .get_column_as_string("Id")?
        .into_iter()
        .map(Option::unwrap)
        .collect::<Vec<_>>();

    let stat_maps = stat_ids.zip(stat_values).map(|(ids, values)| {
        ids.into_iter()
            .zip(values)
            .map(|(id, value)| {
                (
                    stats
                        .get(id as usize)
                        .with_context(|| format!("Stat index out of bounds: {id}"))
                        .unwrap()
                        .to_string(),
                    value,
                )
            })
            .collect::<HashMap<_, _>>()
    });

    // Reminder texts

    let reminder_text_table =
        load_parsed_table(fs, &schemas, &get_filename("remindertext"), version_number)?;
    let reminder_texts = reminder_text_table
        .get_column_as_string("Text")?
        .into_iter()
        .map(Option::unwrap)
        .collect::<Vec<_>>();

    let reminder_text = passive_table
        .get_column_as_list("ReminderStrings")?
        .iter()
        .map(|s| {
            s.expect("Reminder text list is null")
                .as_primitive_opt::<UInt64Type>()
                .expect("Couldn't cast stats list to u64")
                .into_iter()
                .map(|x| x.expect("Reminder text value is null"))
                .map(|i| {
                    reminder_texts
                        .get(i as usize)
                        .with_context(|| format!("Reminder index out of bounds: {i}"))
                        .unwrap()
                        .to_string()
                })
                .collect::<Vec<_>>()
        });

    let passive_skills = izip!(
        flavour_text,
        graph_passive_id,
        icon,
        id,
        is_ascendancy_starting_node,
        is_icon_only,
        is_jewel_socket,
        is_keystone,
        is_multiple_choice,
        is_multiple_choice_option,
        is_notable,
        name,
        skill_points,
        stat_maps,
        reminder_text,
    )
    .map(
        |(
            flavour_text,
            graph_passive_id,
            icon,
            id,
            is_ascendancy_starting_node,
            is_icon_only,
            is_jewel_socket,
            is_keystone,
            is_multiple_choice,
            is_multiple_choice_option,
            is_notable,
            name,
            skill_points,
            stat_maps,
            reminder_text,
        )| {
            PassiveSkillInfo {
                flavour_text,
                graph_passive_id,
                icon,
                passive_id: id,
                is_ascendancy_starting_node,
                is_icon_only,
                is_jewel_socket,
                is_keystone,
                is_multiple_choice,
                is_multiple_choice_option,
                is_notable,
                skill_points: skill_points as u32,
                reminder_text,
                stats: stat_maps,
                name,
            }
        },
    )
    .collect();

    Ok(passive_skills)
}

struct Zip<I> {
    iters: Vec<I>,
}

impl<I: Iterator> Iterator for Zip<I> {
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iters
            .iter_mut()
            .map(|iter| iter.next())
            .collect::<Option<_>>()
    }
}
