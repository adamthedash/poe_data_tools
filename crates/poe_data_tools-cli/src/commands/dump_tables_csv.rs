use std::{
    fs::{File, create_dir_all},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result, bail, ensure};
use arrow_array::{RecordBatch, StringArray};
use arrow_cast::display::{ArrayFormatter, FormatOptions};
use arrow_csv::Writer;
use arrow_schema::{DataType, SchemaBuilder};
use bytes::Bytes;
use glob::{MatchOptions, Pattern};
use poe_data_tools::{
    Patch,
    dat::{
        schema::{DatTableSchema, fetch_schema, load_schema},
        table::parse_table,
    },
    file_parsers::{FileParser, dat::DatParser},
    fs::{FS, FileSystem},
};

use crate::VERBOSE;

/// Save the dataframe to a table, handling list columns
fn save_to_csv(table: &RecordBatch, path: &Path) -> Result<()> {
    let (schema, mut columns, _) = table.clone().into_parts();
    let mut schema_builder = SchemaBuilder::from(&*schema);

    // Stringify list columns
    columns
        .iter_mut()
        .enumerate()
        .filter(|(_, c)| c.data_type().is_nested())
        .for_each(|(i, c)| {
            // Use arrow's formatter to format the sub-array
            let stringy_vals = {
                let options = FormatOptions::default();
                let formatter =
                    ArrayFormatter::try_new(c, &options).expect("Failed to create table formatter");

                (0..c.len())
                    .map(|i| format!("{}", formatter.value(i)))
                    .collect::<Vec<_>>()
            };

            // Update the table data / schema
            *c = Arc::new(StringArray::from(stringy_vals)) as _;

            let field = (**schema_builder.field(i))
                .clone()
                .with_data_type(DataType::Utf8);

            *schema_builder.field_mut(i) = Arc::new(field);
        });

    let schema = Arc::new(schema_builder.finish());
    let table = RecordBatch::try_new(schema, columns).context("Failed to re-create table")?;

    create_dir_all(path.parent().context("No parent directory")?)
        .context("Failed to create output dirs")?;

    Writer::new(File::create(path).context("Failed to create output file")?)
        .write(&table)
        .context("Failed to write DF to file")
}

fn process_file(bytes: &Bytes, output_path: &Path, schema: &DatTableSchema) -> Result<()> {
    // Load dat file
    let table = DatParser
        .parse(bytes)
        .context("Failed to parse table data")?;

    ensure!(!table.rows.is_empty(), "Empty table");

    // Apply it
    let df = parse_table(&table, schema).context("Failed to apply schema to table")?;

    // Save table out as CSV todo: / JSON / SQLLite table
    save_to_csv(&df, output_path).context("Failed to write CSV")?;

    Ok(())
}

/// Convert datc64 tables into CSV files
pub fn dump_tables(
    fs: &mut FS,
    patterns: &[Pattern],
    cache_dir: &Path,
    output_folder: &Path,
    version: &Patch,
    schema: Option<impl AsRef<Path>>,
) -> Result<()> {
    for pattern in patterns {
        ensure!(
            pattern.as_str().ends_with(".datc64"),
            "Only .datc64 table export is supported."
        );
    }

    let version = match version {
        Patch::One => 1,
        Patch::Two => 2,
        _ => bail!("Only patch versions 1/2 supported for table extraction."),
    };

    // Load schema: todo: Get this from Ivy's CDN / cache it
    let schemas = if let Some(path) = schema {
        load_schema(path.as_ref()).context("Failed to load schema file")?
    } else {
        fetch_schema(cache_dir).context("Failed to fetch schema file")?
    };

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
            // Load table schema - TODO: HashMap rather than vector
            let schema = schemas
                .tables
                .iter()
                // valid_for == 3 is common between both games
                .filter(|t| t.valid_for == version || t.valid_for == 3)
                .find(|t| {
                    *t.name.to_lowercase() == *PathBuf::from(filename.as_ref()).file_stem().unwrap()
                })
                .with_context(|| format!("Couldn't find schema for {:?}", filename))?;

            // Convert the data table
            let output_path = output_folder.join(filename.as_ref()).with_extension("csv");
            process_file(&contents, &output_path, schema)
                .with_context(|| format!("Failed to process file: {:?}", filename))?;

            Ok(filename)
        })
        // Report results
        .for_each(|result| match result {
            Ok(filename) => log::info!("Extracted table: {}", filename),
            Err(e) => {
                let error_message = if *VERBOSE.get().unwrap() {
                    format!("{e:?}")
                } else {
                    format!("{e}")
                };
                log::error!("Failed to extract table: {error_message}");
            }
        });

    Ok(())
}
