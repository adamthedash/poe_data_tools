use std::collections::HashMap;

use serde_json::map::Map;
use winnow::{
    Parser,
    binary::{le_f32, le_i16, le_i32, le_u8, le_u16, le_u32, le_u64, le_u128},
    combinator::{dispatch, empty, fail},
    error::ContextError,
};

use crate::{
    dat::{ivy_schema::DatTableSchema, table_view::take_utf16_string},
    file_parsers::shared::winnow::WinnowParser,
};

/// Read an index and read a string out of the variable section starting at the index
fn string<'a>(variable_section: &[u8]) -> impl WinnowParser<&'a [u8], String> {
    let vs_len = variable_section.len();
    le_u64
        .verify(move |offset| (8..vs_len as u64 + 8).contains(offset))
        .map(move |offset| take_utf16_string(&variable_section[offset as usize - 8..]))
}

/// Creates a winnow parser from a schema which can then be applied to the bytes of the dat table
pub fn create_parser<'a>(
    resolved_keys: &HashMap<String, Vec<serde_json::Value>>,
    variable_section: &'static [u8],
    schema: &DatTableSchema,
) -> impl Parser<&'a [u8], serde_json::Value, ContextError> {
    move |input: &mut &[u8]| {
        let mut out = Map::new();

        for (column, column_name) in schema.columns.iter().zip(schema.column_names()) {
            let table_name = if let Some(table_name) = &column.references {
                table_name.table.as_str()
            } else {
                ""
            };

            let mut item_parser = dispatch! {
                empty.value(column.column_type.as_str());

                "foreignrow" => le_u128.map(move |row| {
                    if row == u128::from_le_bytes([0xfe; 16]) {
                        serde_json::Value::Null
                    } else {
                        let mut value = Map::new();
                        value.insert("TableName".to_owned(), serde_json::to_value(table_name).unwrap());
                        value.insert("RowIndex".to_owned(), serde_json::to_value(row).unwrap());

                        if let Some(keys) = resolved_keys.get(&table_name.to_lowercase())
                        && let Some(key) = keys.get(row as usize) {
                            value.insert("Key".to_owned(), key.clone());
                        }

                        serde_json::Value::Object(value)
                    }
                }),

                // NOTE: Tables with self references need to be resolved twice. Once to initialise
                // non-reference columns, second to resolve self-references
                "row" => le_u64.map(move |row| {
                    if row == u64::from_le_bytes([0xfe; 8]) {
                        serde_json::Value::Null
                    } else {
                        let mut value = Map::new();
                        value.insert("TableName".to_owned(), serde_json::to_value(table_name).unwrap());
                        value.insert("RowIndex".to_owned(), serde_json::to_value(row).unwrap());

                        if let Some(keys) = resolved_keys.get(&table_name.to_lowercase())
                        && let Some(key) = keys.get(row as usize) {
                            value.insert("Key".to_owned(), key.clone());
                        }

                        serde_json::Value::Object(value)
                    }
                }),

                "enumrow" => le_u32.map(move |row| {
                    if let Some(table) = resolved_keys.get(&table_name.to_lowercase())
                        && let Some(e) = table.get(row as usize)
                    {
                        e.clone()
                    } else {
                        let mut value = Map::new();
                        value.insert("TableName".to_owned(), serde_json::to_value(table_name).unwrap());
                        value.insert("RowIndex".to_owned(), serde_json::to_value(row).unwrap());
                        serde_json::Value::Object(value)
                    }
                }),

                "string" => string(variable_section).map(serde_json::Value::String),

                "u32" => le_u32.map(|x| serde_json::to_value(x).unwrap() ),
                "i32" => le_i32.map(|x| serde_json::to_value(x).unwrap() ),
                "f32" => le_f32.map(|x| serde_json::to_value(x).unwrap() ),

                "u16" => le_u16.map(|x| serde_json::to_value(x).unwrap() ),
                "i16" => le_i16.map(|x| serde_json::to_value(x).unwrap() ),

                "bool" => le_u8
                    .verify_map(|b| (b <= 2).then_some(b == 1))
                    .map(serde_json::Value::Bool),

                _ => fail,

            };

            match (column.array, column.interval) {
                // Array
                (true, false) => {
                    let (length, pointer) = (
                        le_u64,
                        le_u64.verify(|offset| {
                            (8..variable_section.len() as u64 + 8).contains(offset)
                        }),
                    )
                        .parse_next(input)?;

                    let mut input = &variable_section[pointer as usize - 8..];

                    let mut items = vec![];
                    for _ in 0..length {
                        let item = item_parser.parse_next(&mut input)?;
                        items.push(item);
                    }

                    out.insert(column_name, serde_json::to_value(items).unwrap());
                }
                // Interval
                (false, true) => {
                    let values = vec![
                        item_parser.parse_next(input)?,
                        item_parser.parse_next(input)?,
                    ];

                    out.insert(column_name, serde_json::to_value(values).unwrap());
                }
                // Scalar
                (false, false) => {
                    let value = item_parser.parse_next(input)?;
                    out.insert(column_name, serde_json::to_value(value).unwrap());
                }
                (true, true) => unreachable!(),
            }
        }

        Ok(serde_json::Value::Object(out))
    }
}
