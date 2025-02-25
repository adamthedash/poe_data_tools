use std::{
    fs::{create_dir_all, File},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, ensure, Context, Result};
use bytes::Bytes;
use glob::Pattern;
use polars::{
    frame::DataFrame,
    io::SerWriter,
    prelude::{Column, CsvWriter, DataType, NamedFrom},
    series::Series,
};

use crate::{
    bundle_fs::FS,
    commands::Patch,
    dat::{
        ivy_schema::{fetch_schema, ColumnSchema, DatTableSchema},
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
    cur_unknown: usize,
) -> Result<(bool, usize, Result<Series>)> {
    // Parse column name
    let (is_unknown, col_name) = if let Some(name) = column.name.clone() {
        (false, name)
    } else {
        (true, format!("unknown_{}", cur_unknown))
    };

    let (bytes_taken, series) = match (column.array, column.interval) {
        // Array
        (true, false) => {
            let series = match column.column_type.as_str() {
                "string" => table
                    .view_col_as_array_of_strings(cur_offset)?
                    .map(|row| row.map(|x| Series::new("".into(), &x)))
                    .collect::<Result<Vec<_>>>(),

                "foreignrow" => table
                    .view_col_as_array_of(cur_offset, 16, parse_foreignrow)?
                    .map(|row| row.map(|x| Series::new("".into(), &x)))
                    .collect::<Result<Vec<_>>>(),

                "row" => table
                    .view_col_as_array_of(cur_offset, 16, parse_foreignrow)?
                    .map(|row| row.map(|x| Series::new("".into(), &x)))
                    .collect::<Result<Vec<_>>>(),

                "enumrow" => table
                    .view_col_as_array_of(cur_offset, 4, parse_u32)?
                    .map(|row| row.map(|x| Series::new("".into(), &x)))
                    .collect::<Result<Vec<_>>>(),

                "f32" => table
                    .view_col_as_array_of(cur_offset, 4, parse_f32)?
                    .map(|row| row.map(|x| Series::new("".into(), &x)))
                    .collect::<Result<Vec<_>>>(),

                "i32" => table
                    .view_col_as_array_of(cur_offset, 4, parse_i32)?
                    .map(|row| row.map(|x| Series::new("".into(), &x)))
                    .collect::<Result<Vec<_>>>(),

                "i16" => table
                    .view_col_as_array_of(cur_offset, 2, parse_i16)?
                    .map(|row| row.map(|x| Series::new("".into(), &x)))
                    .collect::<Result<Vec<_>>>(),

                "u16" => table
                    .view_col_as_array_of(cur_offset, 2, parse_u16)?
                    .map(|row| row.map(|x| Series::new("".into(), &x)))
                    .collect::<Result<Vec<_>>>(),

                _ => bail!("Unknown column type: {:?}", column),
            }
            .map(|s| Series::new(col_name.into(), s));

            (16, series)
        }

        // Interval
        (false, true) => match column.column_type.as_str() {
            "i32" => {
                let series = table
                    .view_col(cur_offset, 8)
                    .map(|values| {
                        values
                            .map(|bytes| bytes.chunks_exact(4).map(parse_i32).collect::<Vec<_>>())
                            .map(|x| Series::new("".into(), &x))
                            .collect::<Vec<_>>()
                    })
                    .map(|s| Series::new(col_name.into(), s));

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
                    .map(|s| Series::new(col_name.into(), s));
                (8, series)
            }

            "foreignrow" => {
                let series = table
                    .view_col(cur_offset, 16)
                    .map(|items| items.map(parse_maybe_foreignrow).collect::<Vec<_>>())
                    .map(|s| Series::new(col_name.into(), s));
                (16, series)
            }

            "row" => {
                let series = table
                    .view_col(cur_offset, 16)
                    .map(|items| items.map(parse_maybe_foreignrow).collect::<Vec<_>>())
                    .map(|s| Series::new(col_name.into(), s));
                (16, series)
            }

            "enumrow" => {
                let series = table
                    .view_col(cur_offset, 4)
                    .map(|items| items.map(parse_u32).collect::<Vec<_>>())
                    .map(|s| Series::new(col_name.into(), s));
                (4, series)
            }

            "f32" => {
                let series = table
                    .view_col(cur_offset, 4)
                    .map(|items| items.map(parse_f32).collect::<Vec<_>>())
                    .map(|s| Series::new(col_name.into(), s));
                (4, series)
            }

            "i32" => {
                let series = table
                    .view_col(cur_offset, 4)
                    .map(|items| items.map(parse_i32).collect::<Vec<_>>())
                    .map(|s| Series::new(col_name.into(), s));
                (4, series)
            }

            "i16" => {
                let series = table
                    .view_col(cur_offset, 2)
                    .map(|items| items.map(parse_i16).collect::<Vec<_>>())
                    .map(|s| Series::new(col_name.into(), s));
                (2, series)
            }

            "u16" => {
                let series = table
                    .view_col(cur_offset, 2)
                    .map(|items| items.map(parse_u16).collect::<Vec<_>>())
                    .map(|s| Series::new(col_name.into(), s));
                (2, series)
            }

            "bool" => {
                let series = table
                    .view_col(cur_offset, 1)
                    .and_then(|items| items.map(parse_bool).collect::<Result<Vec<_>>>())
                    .map(|s| Series::new(col_name.into(), s));
                (1, series)
            }

            _ => bail!("Unknown column type: {:?}", column),
        },
        _ => bail!("Can't be both array and interval"),
    };

    Ok((is_unknown, bytes_taken, series))
}

/// Parse a table with the given schema into a Polars DataFrame
fn parse_table(table: &DatTable, schema: &DatTableSchema) -> Result<DataFrame> {
    // Parse each of the columns
    let mut parsed_columns = vec![];
    let mut cur_offset = 0;
    let mut num_unknowns = 0;
    for column in &schema.columns {
        let (is_unknown, bytes_taken, series) =
            parse_column(table, column, cur_offset, num_unknowns)
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
        if is_unknown {
            num_unknowns += 1;
        }
    }

    // Collect em into a dataframe
    let df = DataFrame::new(parsed_columns.into_iter().map(Column::from).collect())
        .context("Failed to create df")?;
    Ok(df)
}

/// Stringify the Series into an escaped list
fn series_to_string(series: &Series) -> String {
    let string_func = match series.dtype() {
        // If it's a string, escape it
        DataType::String => |x| format!("{:?}", x),
        _ => |x| format!("{}", x),
        //_ => panic!("Invalid dtype"),
    };

    let stringy_vals = series.iter().map(string_func).collect::<Vec<_>>();
    format!("[{}]", stringy_vals.join(","))
}

/// Save the dataframe to a table, handling list columns
fn save_to_csv(table: &mut DataFrame, path: &Path) -> Result<()> {
    // Stringify list columns
    let new_cols = table
        .get_columns()
        .iter()
        .filter(|col| matches!(col.dtype(), DataType::List(..)))
        .map(|col| {
            // Stringify the column
            let string_col = col
                .list()
                .unwrap()
                .into_iter()
                .map(|row_vals| series_to_string(&row_vals.unwrap()))
                .collect::<Vec<_>>();
            Column::new(col.name().clone(), string_col)
        })
        .collect::<Vec<_>>();

    // Put them into the dataframe
    let mut table = table.clone();
    new_cols.into_iter().for_each(|col| {
        table.with_column(col).unwrap();
    });

    create_dir_all(path.parent().context("No parent directory")?)
        .context("Failed to create output dirs")?;

    CsvWriter::new(File::create(path).context("Failed to create output file")?)
        .finish(&mut table)
        .context("Failed to write DF to file")
}

fn process_file(bytes: &Bytes, output_path: &Path, schema: &DatTableSchema) -> Result<()> {
    // Load dat file
    let (_, table) = DatTable::from_raw_bytes(bytes)
        .map_err(|e| anyhow!("Failed to parse table data: {:?}", e))?;

    ensure!(!table.rows.is_empty(), "Empty table");

    // Apply it
    let mut df = parse_table(&table, schema).context("Failed to apply schema to table")?;

    // Save table out as CSV todo: / JSON / SQLLite table
    save_to_csv(&mut df, output_path).context("Failed to write CSV")?;

    Ok(())
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
