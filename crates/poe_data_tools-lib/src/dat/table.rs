use std::{path::PathBuf, sync::Arc};

use arrow_array::{
    ArrayRef, BooleanArray, Float32Array, Int16Array, Int32Array, RecordBatch, StringArray,
    UInt16Array, UInt32Array, UInt64Array,
    builder::{
        Float32Builder, Int16Builder, Int32Builder, ListBuilder, StringBuilder, UInt16Builder,
        UInt32Builder, UInt64Builder,
    },
};

use super::table_view::ColResult;
use crate::{
    Patch,
    dat::{
        schema::{ColumnSchema, DatTableSchema, SchemaCollection},
        table_view::{DatColumnError, DatError, DatResult},
    },
    file_parsers::{
        FileParser,
        dat::{DatParser, types::DatFile},
    },
    fs::FileSystem,
};

fn parse_foreignrow(bytes: &[u8]) -> u64 {
    // TODO: polars doesn't support u128, so figure something out later. For now
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

fn parse_bool(bytes: &[u8]) -> ColResult<bool> {
    assert!(bytes.len() == 1);

    match bytes[0] {
        0 => Ok(false),
        1 => Ok(true),
        x => Err(DatColumnError::InvalidBool(x)),
    }
}

/// Apply a schema to a single column
fn parse_column(
    table: &DatFile,
    column: &ColumnSchema,
    cur_offset: usize,
) -> ColResult<(usize, ColResult<ArrayRef>)> {
    let (bytes_taken, series) = match (column.array, column.interval) {
        // Array
        (true, false) => {
            let series = match column.column_type.as_str() {
                // Array of "array" is used to indicate an unknown data type as far as I can tell
                "array" => Err(DatColumnError::UnknownArrayType),

                "string" => table
                    .view_col_as_array_of_strings(cur_offset)?
                    .collect::<ColResult<Vec<_>>>()
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
                    .collect::<ColResult<Vec<_>>>()
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
                    .collect::<ColResult<Vec<_>>>()
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
                    .collect::<ColResult<Vec<_>>>()
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
                    .collect::<ColResult<Vec<_>>>()
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
                    .collect::<ColResult<Vec<_>>>()
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
                    .collect::<ColResult<Vec<_>>>()
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
                    .collect::<ColResult<Vec<_>>>()
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
                    .collect::<ColResult<Vec<_>>>()
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

                _ => {
                    return Err(DatColumnError::UnknownColumnType(Box::new(
                        column.to_owned(),
                    )));
                }
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
                            .as_chunks::<4>()
                            .0
                            .iter()
                            .map(|b| parse_i32(b))
                            .for_each(|val| builder.values().append_value(val));
                        builder.append(true);
                    });

                    Arc::new(builder.finish()) as _
                });

                (8, series)
            }
            _ => {
                return Err(DatColumnError::UnknownColumnType(Box::new(
                    column.to_owned(),
                )));
            }
        },

        // Scalar
        (false, false) => match column.column_type.as_str() {
            "string" => {
                let series = table
                    .view_col_as_string(cur_offset)
                    .and_then(|strings| strings.collect::<ColResult<Vec<_>>>())
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
                    .and_then(|items| items.map(parse_bool).collect::<ColResult<Vec<_>>>())
                    // .map(|s| Series::new(col_name.into(), s));
                    .map(|s| Arc::new(BooleanArray::from(s)) as _);
                (1, series)
            }

            _ => {
                return Err(DatColumnError::UnknownColumnType(Box::new(
                    column.to_owned(),
                )));
            }
        },
        _ => return Err(DatColumnError::ArrayInterval(Box::new(column.to_owned()))),
    };

    Ok((bytes_taken, series))
}

/// Parse a table with the given schema into an Arrow RecordBatch
pub fn parse_table(table: &DatFile, schema: &DatTableSchema) -> DatResult<RecordBatch> {
    let column_names = schema.column_names().collect::<Vec<_>>();

    // Parse each of the columns
    let mut parsed_columns = vec![];
    let mut cur_offset = 0;
    for column in &schema.columns {
        // Parse column data.
        // NOTE: We return out on parse failure as it may impact the interpretation of followon columns
        // if the offset is incorrect.
        let (bytes_taken, series) =
            parse_column(table, column, cur_offset).map_err(|e| DatError::Column {
                column: Box::new(column.to_owned()),
                source: e,
            })?;

        // If we successfully parse the data, add it to the table
        match series {
            Ok(series) => {
                log::trace!(
                    "Successfully parsed column at bytes {}-{}: {:?}",
                    cur_offset,
                    cur_offset + bytes_taken,
                    column
                );
                parsed_columns.push(series);
            }
            Err(e) => {
                log::error!("Failed to parse column {:?}, skipping: {e:?}", column.name);
            }
        }
        cur_offset += bytes_taken;
    }

    // Collect em into a dataframe
    let df = RecordBatch::try_from_iter(column_names.into_iter().zip(parsed_columns))?;
    Ok(df)
}

/// Extension trait providing easier loading of dat tables
pub trait FSDatEx: FileSystem {
    /// Loads a table into an Arrow RecordBatch
    // TODO: Support for enum tables
    fn load_dat_table(
        &mut self,
        schemas: &SchemaCollection,
        path: &str,
        version: &Patch,
    ) -> DatResult<RecordBatch> {
        let version = version.major();

        // Load table schema
        // TODO: HashMap rather than vector
        let schema = schemas
            .tables
            .iter()
            // valid_for == 3 is common between both games
            .filter(|t| t.valid_for == version || t.valid_for == 3)
            .find(|t| *t.name.to_lowercase() == *PathBuf::from(&path).file_stem().unwrap())
            .ok_or_else(|| DatError::SchemaNotFound(path.to_owned()))?;

        // Load dat file & parse generic structure
        let bytes = self.read(path)?;
        let table = DatParser.parse(&bytes)?;

        if table.rows.is_empty() {
            return Err(DatError::EmptyTable);
        }

        // Apply the schema
        let df = parse_table(&table, schema)?;

        Ok(df)
    }
}

impl<T> FSDatEx for T where T: FileSystem {}
