use crate::{
    commands::Patch,
    dat::{
        ivy_schema::{ColumnSchema, DatTableSchema, SchemaCollection},
        table_view::DatTable,
    },
};
use anyhow::{anyhow, bail, ensure, Context};
use glob::glob;
use std::{
    fs::{self, create_dir_all, File},
    path::Path,
};

use anyhow::Result;
use polars::{
    frame::DataFrame,
    io::SerWriter,
    prelude::{Column, CsvWriter, DataType, NamedFrom},
    series::Series,
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
) -> Result<(bool, usize, Series)> {
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
                    .collect::<Result<Vec<_>>>()?,

                "foreignrow" => table
                    .view_col_as_array_of(cur_offset, 16, parse_foreignrow)?
                    .map(|row| row.map(|x| Series::new("".into(), &x)))
                    .collect::<Result<Vec<_>>>()?,

                "row" => table
                    .view_col_as_array_of(cur_offset, 16, parse_foreignrow)?
                    .map(|row| row.map(|x| Series::new("".into(), &x)))
                    .collect::<Result<Vec<_>>>()?,

                "enumrow" => table
                    .view_col_as_array_of(cur_offset, 4, parse_u32)?
                    .map(|row| row.map(|x| Series::new("".into(), &x)))
                    .collect::<Result<Vec<_>>>()?,

                "f32" => table
                    .view_col_as_array_of(cur_offset, 4, parse_f32)?
                    .map(|row| row.map(|x| Series::new("".into(), &x)))
                    .collect::<Result<Vec<_>>>()?,

                "i32" => table
                    .view_col_as_array_of(cur_offset, 4, parse_i32)?
                    .map(|row| row.map(|x| Series::new("".into(), &x)))
                    .collect::<Result<Vec<_>>>()?,

                "i16" => table
                    .view_col_as_array_of(cur_offset, 2, parse_i16)?
                    .map(|row| row.map(|x| Series::new("".into(), &x)))
                    .collect::<Result<Vec<_>>>()?,

                "u16" => table
                    .view_col_as_array_of(cur_offset, 2, parse_u16)?
                    .map(|row| row.map(|x| Series::new("".into(), &x)))
                    .collect::<Result<Vec<_>>>()?,

                _ => bail!("Unknown column type: {:?}", column),
            };

            let series = Series::new(col_name.into(), series);
            (16, series)
        }

        // Interval
        (false, true) => match column.column_type.as_str() {
            "i32" => {
                let values = table
                    .view_col(cur_offset, 8)?
                    .map(|bytes| bytes.chunks_exact(4).map(parse_i32).collect::<Vec<_>>())
                    .map(|x| Series::new("".into(), &x))
                    .collect::<Vec<_>>();

                let series = Series::new(col_name.into(), values);
                (8, series)
            }
            _ => bail!("Unknown column type: {:?}", column),
        },

        // Scalar
        (false, false) => match column.column_type.as_str() {
            "string" => {
                let values = table
                    .view_col_as_string(cur_offset)?
                    .collect::<Result<Vec<_>>>()?;
                let series = Series::new(col_name.into(), values);
                (8, series)
            }

            "foreignrow" => {
                let values = table
                    .view_col(cur_offset, 16)?
                    .map(parse_maybe_foreignrow)
                    .collect::<Vec<_>>();
                let series = Series::new(col_name.into(), values);
                (16, series)
            }

            "row" => {
                let values = table
                    .view_col(cur_offset, 16)?
                    .map(parse_maybe_foreignrow)
                    .collect::<Vec<_>>();
                let series = Series::new(col_name.into(), values);
                (16, series)
            }

            "enumrow" => {
                let values = table
                    .view_col(cur_offset, 4)?
                    .map(parse_u32)
                    .collect::<Vec<_>>();
                let series = Series::new(col_name.into(), values);
                (4, series)
            }

            "f32" => {
                let values = table
                    .view_col(cur_offset, 4)?
                    .map(parse_f32)
                    .collect::<Vec<_>>();
                let series = Series::new(col_name.into(), values);
                (4, series)
            }

            "i32" => {
                let values = table
                    .view_col(cur_offset, 4)?
                    .map(parse_i32)
                    .collect::<Vec<_>>();
                let series = Series::new(col_name.into(), values);
                (4, series)
            }

            "i16" => {
                let values = table
                    .view_col(cur_offset, 2)?
                    .map(parse_i16)
                    .collect::<Vec<_>>();
                let series = Series::new(col_name.into(), values);
                (2, series)
            }

            "u16" => {
                let values = table
                    .view_col(cur_offset, 2)?
                    .map(parse_u16)
                    .collect::<Vec<_>>();
                let series = Series::new(col_name.into(), values);
                (2, series)
            }

            "bool" => {
                let values = table
                    .view_col(cur_offset, 1)?
                    .map(parse_bool)
                    .collect::<Result<Vec<_>>>()?;
                let series = Series::new(col_name.into(), values);
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

        parsed_columns.push(series);
        cur_offset += bytes_taken;
        if is_unknown {
            num_unknowns += 1;
        }
    }

    // Collect em into a dataframe
    let df = DataFrame::new(parsed_columns.into_iter().map(Column::from).collect())
        .expect("Failed to create df");
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
fn save_to_csv(table: &mut DataFrame, path: &Path) {
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

    create_dir_all(path.parent().expect("No parent directory"))
        .expect("Failed to create output dirs");

    CsvWriter::new(File::create(path).expect("Failed to create output file"))
        .finish(&mut table)
        .expect("Failed to write DF to file");
}

fn process_file(dat_path: &Path, output_path: &Path, schema: &DatTableSchema) -> Result<()> {
    // Load dat file
    let bytes = fs::read(dat_path).context("Failed to read table file")?;
    let (_, table) = DatTable::from_raw_bytes(&bytes)
        .map_err(|e| anyhow!("Failed to parse table data: {:?}: {:?}", dat_path, e))?;

    ensure!(!table.rows.is_empty(), "Empty table");

    // Apply it
    let mut df = parse_table(&table, schema).context("Failed to apply schema to table")?;

    // Save table out as CSV todo: / JSON / SQLLite table
    save_to_csv(&mut df, output_path);

    Ok(())
}

/// Convert datc64 tables into CSV files
pub fn dump_tables(
    datc64_root: &Path,
    schema_path: &Path,
    output_folder: &Path,
    version: &Patch,
) -> Result<()> {
    let version = match version {
        Patch::One => 1,
        Patch::Two => 2,
        _ => bail!("Only patch versions 1/2 supported for table extraction."),
    };

    // Load schema: todo: Get this from Ivy's CDN / cache it
    let schemas: SchemaCollection = serde_json::from_str(
        &fs::read_to_string(schema_path).context("Failed to read schema file")?,
    )
    .context("Failed to parse schema file")?;

    // Find the dat files
    let glob_pattern = datc64_root
        .join("**/*.datc64")
        .to_string_lossy()
        .to_string();
    let files = glob(&glob_pattern)
        .context("Failed to glob datc files")?
        .filter_map(Result::ok);

    for dat_path in files {
        // Load table schema - todo: HashMap rather than vector
        let schema = schemas
            .tables
            .iter()
            // valid_for == 3 is common between both games
            .filter(|t| t.valid_for == version || t.valid_for == 3)
            .find(|t| *t.name.to_lowercase() == *dat_path.file_stem().unwrap());

        let schema = if let Some(schema) = schema {
            schema
        } else {
            eprintln!("Couldn't find schema for {:?}", dat_path.file_name());
            continue;
        };

        // Convert the data table
        let output_path = output_folder
            .join(dat_path.strip_prefix(datc64_root).unwrap())
            .with_extension("csv");

        let res = process_file(&dat_path, &output_path, schema)
            .with_context(|| format!("Failed to process file: {:?}", dat_path.file_name()));

        if let Err(e) = res {
            eprintln!("{:?}", e);
        }
    }

    Ok(())
}
