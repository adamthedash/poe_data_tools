use std::{
    fs::{create_dir_all, File},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{anyhow, bail, ensure, Context, Result};
use arrow::{
    array::{
        ArrayRef, BooleanArray, Float32Array, Float32Builder, Int16Array, Int16Builder, Int32Array,
        Int32Builder, ListBuilder, RecordBatch, StringArray, StringBuilder, UInt16Array,
        UInt16Builder, UInt32Array, UInt32Builder, UInt64Array, UInt64Builder,
    },
    csv::Writer,
    datatypes::{DataType, SchemaBuilder},
    util::display::{ArrayFormatter, FormatOptions},
};
use bytes::Bytes;
use glob::Pattern;

use crate::{
    bundle_fs::FS,
    commands::Patch,
    dat::{
        ivy_schema::{fetch_schema, ColumnSchema, DatTableSchema, SchemaCollection},
        table_view::DatTable,
    },
};

fn parse_foreignrow(bytes: &[u8]) -> u64 {
    // todo: polars doesn't support u128, so figure something out later. For now
    // just downcast
    u128::from_le_bytes(bytes.try_into().unwrap()) as u64
}

fn parse_maybe_foreignrow(bytes: &[u8]) -> Option<u64> {
    if bytes == [0xfe; 16] {
        None
    } else {
        Some(parse_foreignrow(bytes))
    }
}

fn parse_maybe_row(bytes: &[u8]) -> Option<u64> {
    if bytes == [0xfe; 8] {
        None
    } else {
        Some(parse_u64(bytes))
    }
}

fn parse_u64(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes.try_into().unwrap())
}
fn parse_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes.try_into().unwrap())
}
fn parse_i32(bytes: &[u8]) -> i32 {
    i32::from_le_bytes(bytes.try_into().unwrap())
}
fn parse_f32(bytes: &[u8]) -> f32 {
    f32::from_le_bytes(bytes.try_into().unwrap())
}

fn parse_u16(bytes: &[u8]) -> u16 {
    u16::from_le_bytes(bytes.try_into().unwrap())
}
fn parse_i16(bytes: &[u8]) -> i16 {
    i16::from_le_bytes(bytes.try_into().unwrap())
}

fn parse_bool(bytes: &[u8]) -> Result<bool> {
    assert!(bytes.len() == 1);
    ensure!(bytes[0] < 2, "Invalid boolean value: {:?}", bytes[0]);

    Ok(bytes[0] == 1)
}

/// Apply a schema to a single column
fn parse_column(
    table: &DatTable,
    column: &ColumnSchema,
    cur_offset: usize,
) -> Result<(usize, Result<ArrayRef>)> {
    let (bytes_taken, series) = match (column.array, column.interval) {
        // Array
        (true, false) => {
            let series = match column.column_type.as_str() {
                // Array of "array" is used to indicate an unknown data type as far as I can tell
                "array" => Err(anyhow!("Unknown array type")),

                "string" => table
                    .view_col_as_array_of_strings(cur_offset)?
                    .collect::<Result<Vec<_>>>()
                    .map(|s| {
                        let mut builder = ListBuilder::new(StringBuilder::new());
                        for row in s {
                            for val in row {
                                builder.values().append_option(val)
                            }
                            builder.append(true);
                        }

                        builder.finish()
                    }),

                "foreignrow" => table
                    .view_col_as_array_of(cur_offset, 16, parse_foreignrow)?
                    .collect::<Result<Vec<_>>>()
                    .map(|s| {
                        let mut builder = ListBuilder::new(UInt64Builder::new());
                        for row in s {
                            for val in row {
                                builder.values().append_value(val)
                            }
                            builder.append(true);
                        }

                        builder.finish()
                    }),

                "row" => table
                    .view_col_as_array_of(cur_offset, 8, parse_maybe_row)?
                    .collect::<Result<Vec<_>>>()
                    .map(|s| {
                        let mut builder = ListBuilder::new(UInt64Builder::new());
                        for row in s {
                            for val in row {
                                builder.values().append_option(val)
                            }
                            builder.append(true);
                        }

                        builder.finish()
                    }),

                "enumrow" => table
                    .view_col_as_array_of(cur_offset, 4, parse_u32)?
                    .collect::<Result<Vec<_>>>()
                    .map(|s| {
                        let mut builder = ListBuilder::new(UInt32Builder::new());
                        for row in s {
                            for val in row {
                                builder.values().append_value(val)
                            }
                            builder.append(true);
                        }

                        builder.finish()
                    }),

                "u32" => table
                    .view_col_as_array_of(cur_offset, 4, parse_u32)?
                    .collect::<Result<Vec<_>>>()
                    .map(|s| {
                        let mut builder = ListBuilder::new(UInt32Builder::new());
                        for row in s {
                            for val in row {
                                builder.values().append_value(val)
                            }
                            builder.append(true);
                        }

                        builder.finish()
                    }),

                "f32" => table
                    .view_col_as_array_of(cur_offset, 4, parse_f32)?
                    .collect::<Result<Vec<_>>>()
                    .map(|s| {
                        let mut builder = ListBuilder::new(Float32Builder::new());
                        for row in s {
                            for val in row {
                                builder.values().append_value(val)
                            }
                            builder.append(true);
                        }

                        builder.finish()
                    }),

                "i32" => table
                    .view_col_as_array_of(cur_offset, 4, parse_i32)?
                    .collect::<Result<Vec<_>>>()
                    .map(|s| {
                        let mut builder = ListBuilder::new(Int32Builder::new());
                        for row in s {
                            for val in row {
                                builder.values().append_value(val)
                            }
                            builder.append(true);
                        }

                        builder.finish()
                    }),

                "i16" => table
                    .view_col_as_array_of(cur_offset, 2, parse_i16)?
                    .collect::<Result<Vec<_>>>()
                    .map(|s| {
                        let mut builder = ListBuilder::new(Int16Builder::new());
                        for row in s {
                            for val in row {
                                builder.values().append_value(val)
                            }
                            builder.append(true);
                        }

                        builder.finish()
                    }),

                "u16" => table
                    .view_col_as_array_of(cur_offset, 2, parse_u16)?
                    .collect::<Result<Vec<_>>>()
                    .map(|s| {
                        let mut builder = ListBuilder::new(UInt16Builder::new());
                        for row in s {
                            for val in row {
                                builder.values().append_value(val)
                            }
                            builder.append(true);
                        }

                        builder.finish()
                    }),

                _ => bail!("Unknown column type: {:?}", column),
            }
            .map(|s| Arc::new(s) as _);

            (16, series)
        }

        // Interval
        (false, true) => match column.column_type.as_str() {
            "i32" => {
                let series = table.view_col(cur_offset, 8).map(|values| {
                    let mut builder = ListBuilder::new(Int32Builder::new());
                    values.for_each(|bytes| {
                        bytes
                            .chunks_exact(4)
                            .map(parse_i32)
                            .for_each(|val| builder.values().append_value(val));
                        builder.append(true);
                    });

                    Arc::new(builder.finish()) as _
                });

                (8, series)
            }
            _ => bail!("Unknown column type: {:?}", column),
        },

        // Scalar
        (false, false) => match column.column_type.as_str() {
            "string" => {
                let series = table
                    .view_col_as_string(cur_offset)
                    .and_then(|strings| strings.collect::<Result<Vec<_>>>())
                    // .map(|s| Series::new(col_name.into(), s));
                    .map(|s| Arc::new(StringArray::from(s)) as _);
                (8, series)
            }

            "foreignrow" => {
                let series = table
                    .view_col(cur_offset, 16)
                    .map(|items| items.map(parse_maybe_foreignrow).collect::<Vec<_>>())
                    // .map(|s| Series::new(col_name.into(), s));
                    .map(|s| Arc::new(UInt64Array::from(s)) as _);
                (16, series)
            }

            "row" => {
                let series = table
                    .view_col(cur_offset, 8)
                    .map(|items| items.map(parse_maybe_row).collect::<Vec<_>>())
                    // .map(|s| Series::new(col_name.into(), s));
                    .map(|s| Arc::new(UInt64Array::from(s)) as _);
                (8, series)
            }

            "enumrow" => {
                let series = table
                    .view_col(cur_offset, 4)
                    .map(|items| items.map(parse_u32).collect::<Vec<_>>())
                    // .map(|s| Series::new(col_name.into(), s));
                    .map(|s| Arc::new(UInt32Array::from(s)) as _);
                (4, series)
            }

            "u32" => {
                let series = table
                    .view_col(cur_offset, 4)
                    .map(|items| items.map(parse_u32).collect::<Vec<_>>())
                    // .map(|s| Series::new(col_name.into(), s));
                    .map(|s| Arc::new(UInt32Array::from(s)) as _);
                (4, series)
            }

            "f32" => {
                let series = table
                    .view_col(cur_offset, 4)
                    .map(|items| items.map(parse_f32).collect::<Vec<_>>())
                    // .map(|s| Series::new(col_name.into(), s));
                    .map(|s| Arc::new(Float32Array::from(s)) as _);
                (4, series)
            }

            "i32" => {
                let series = table
                    .view_col(cur_offset, 4)
                    .map(|items| items.map(parse_i32).collect::<Vec<_>>())
                    // .map(|s| Series::new(col_name.into(), s));
                    .map(|s| Arc::new(Int32Array::from(s)) as _);
                (4, series)
            }

            "i16" => {
                let series = table
                    .view_col(cur_offset, 2)
                    .map(|items| items.map(parse_i16).collect::<Vec<_>>())
                    // .map(|s| Series::new(col_name.into(), s));
                    .map(|s| Arc::new(Int16Array::from(s)) as _);
                (2, series)
            }

            "u16" => {
                let series = table
                    .view_col(cur_offset, 2)
                    .map(|items| items.map(parse_u16).collect::<Vec<_>>())
                    // .map(|s| Series::new(col_name.into(), s));
                    .map(|s| Arc::new(UInt16Array::from(s)) as _);
                (2, series)
            }

            "bool" => {
                let series = table
                    .view_col(cur_offset, 1)
                    .and_then(|items| items.map(parse_bool).collect::<Result<Vec<_>>>())
                    // .map(|s| Series::new(col_name.into(), s));
                    .map(|s| Arc::new(BooleanArray::from(s)) as _);
                (1, series)
            }

            _ => bail!("Unknown column type: {:?}", column),
        },
        _ => bail!("Can't be both array and interval"),
    };

    Ok((bytes_taken, series))
}

/// Parse a table with the given schema into an Arrow RecordBatch
pub fn parse_table(table: &DatTable, schema: &DatTableSchema) -> Result<RecordBatch> {
    // Parse each of the columns
    let mut parsed_columns = vec![];
    let mut column_names = vec![];
    let mut cur_offset = 0;
    let mut num_unknowns = 0;
    for column in &schema.columns {
        // Parse column name
        let col_name = if let Some(name) = column.name.clone() {
            name
        } else {
            num_unknowns += 1;
            format!("unknown_{}", num_unknowns - 1)
        };
        column_names.push(col_name);

        // Parse column data
        let (bytes_taken, series) = parse_column(table, column, cur_offset)
            .with_context(|| format!("Failed to parse column: {:?}", column))?;

        // If we successfully parse the data, add it to the table
        match series {
            Ok(series) => {
                parsed_columns.push(series);
            }
            Err(e) => eprintln!(
                "Failed to parse column {:?}, skipping: {:?}",
                column.name, e
            ),
        }
        cur_offset += bytes_taken;
    }

    // Collect em into a dataframe
    let df = RecordBatch::try_from_iter(column_names.into_iter().zip(parsed_columns))
        .context("Failed to create df")?;
    Ok(df)
}

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
    let (_, table) = DatTable::from_raw_bytes(bytes)
        .map_err(|e| anyhow!("Failed to parse table data: {:?}", e))?;

    ensure!(!table.rows.is_empty(), "Empty table");

    // Apply it
    let df = parse_table(&table, schema).context("Failed to apply schema to table")?;

    // Save table out as CSV todo: / JSON / SQLLite table
    save_to_csv(&df, output_path).context("Failed to write CSV")?;

    Ok(())
}

/// Loads a table into a parsed dataframe
pub fn load_parsed_table(
    fs: &mut FS,
    schemas: &SchemaCollection,
    filename: &str,
    version: u32,
) -> Result<RecordBatch> {
    // Load table schema - todo: HashMap rather than vector
    let schema = schemas
        .tables
        .iter()
        // valid_for == 3 is common between both games
        .filter(|t| t.valid_for == version || t.valid_for == 3)
        .find(|t| *t.name.to_lowercase() == *PathBuf::from(&filename).file_stem().unwrap())
        .with_context(|| format!("Couldn't find schema for {:?}", &filename))?;

    // Load dat file
    let bytes = fs.read(filename)?;
    let (_, table) = DatTable::from_raw_bytes(&bytes)
        .map_err(|e| anyhow!("Failed to parse table data: {:?}", e))?;

    ensure!(!table.rows.is_empty(), "Empty table");

    // Apply it
    let df = parse_table(&table, schema).context("Failed to apply schema to table")?;

    Ok(df)
}

/// Convert datc64 tables into CSV files
pub fn dump_tables(
    fs: &mut FS,
    pattern: &Pattern,
    cache_dir: &Path,
    output_folder: &Path,
    version: &Patch,
) -> Result<()> {
    ensure!(
        pattern.as_str().ends_with(".datc64"),
        "Only .datc64 table export is supported."
    );

    let version = match version {
        Patch::One => 1,
        Patch::Two => 2,
        _ => bail!("Only patch versions 1/2 supported for table extraction."),
    };

    // Load schema: todo: Get this from Ivy's CDN / cache it
    let schemas = fetch_schema(cache_dir).context("Failed to fetch schema file")?;

    let filenames = fs
        .list()
        .filter(|filename| pattern.matches(filename))
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
            // Load table schema - todo: HashMap rather than vector
            let schema = schemas
                .tables
                .iter()
                // valid_for == 3 is common between both games
                .filter(|t| t.valid_for == version || t.valid_for == 3)
                .find(|t| *t.name.to_lowercase() == *PathBuf::from(&filename).file_stem().unwrap())
                .with_context(|| format!("Couldn't find schema for {:?}", &filename))?;

            // Convert the data table
            let output_path = output_folder.join(filename).with_extension("csv");
            process_file(&contents, &output_path, schema)
                .with_context(|| format!("Failed to process file: {:?}", filename))?;

            Ok(filename)
        })
        // Report results
        .for_each(|result| match result {
            Ok(filename) => eprintln!("Extracted table: {}", filename),
            Err(e) => eprintln!("Failed to extract table: {:?}", e),
        });

    Ok(())
}
