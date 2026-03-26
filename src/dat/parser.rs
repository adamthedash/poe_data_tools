use std::collections::HashMap;

use serde_json::{Number, Value, json, map::Map};
use winnow::{
    Parser,
    binary::{le_f32, le_i16, le_i32, le_u8, le_u16, le_u32, le_u64, le_u128},
    combinator::{dispatch, empty, fail},
    error::ContextError,
};

use crate::{
    dat::{
        ivy_schema::{ColumnSchema, DatTableSchema},
        table_view::take_utf16_string,
    },
    file_parsers::shared::winnow::WinnowParser,
};

/// Read an index and read a string out of the variable section starting at the index
/// Returns null instead of erroring if a bad offset is found
fn string<'a>(variable_section: &[u8]) -> impl WinnowParser<&'a [u8], Option<String>> {
    let vs_len = variable_section.len();
    le_u64.map(move |offset| {
        if (8..vs_len as u64 + 8).contains(&offset) {
            let string = take_utf16_string(&variable_section[offset as usize - 8..]);
            Some(string)
        } else {
            eprintln!("WARN: Bad offset for string {offset}");
            None
        }
    })
}

/// Basic data types & arrays of them
fn plain_column<'a>(
    column: &ColumnSchema,
    variable_section: &'static [u8],
) -> impl WinnowParser<&'a [u8], Value> {
    move |input: &mut &[u8]| -> winnow::Result<_> {
        let mut item_parser = dispatch! {
            empty.value(column.column_type.as_str());

            "string" => string(variable_section).map(|x| serde_json::to_value(x).unwrap()),

            "u32" => le_u32.map(|x| Value::Number(Number::from(x)) ),
            "i32" => le_i32.map(|x| Value::Number(Number::from(x)) ),
            "f32" => le_f32.map(|x| serde_json::to_value(x).unwrap() ),

            "u16" => le_u16.map(|x| Value::Number(Number::from(x)) ),
            "i16" => le_i16.map(|x| Value::Number(Number::from(x)) ),

            "bool" => le_u8
                .verify_map(|b| (b <= 2).then_some(b == 1))
                .map(Value::Bool),

            _ => fail,
        };

        let out = match (column.array, column.interval) {
            // Array
            (true, false) => {
                let (length, pointer) = (
                    le_u64,
                    le_u64.verify(|offset| (8..variable_section.len() as u64 + 8).contains(offset)),
                )
                    .parse_next(input)?;

                let mut input = &variable_section[pointer as usize - 8..];

                let items = std::iter::repeat_with(|| item_parser.parse_next(&mut input))
                    .take(length as usize)
                    .collect::<winnow::Result<Vec<_>>>()?;

                Value::Array(items)
            }
            // Interval
            (false, true) => Value::Array(vec![
                item_parser.parse_next(input)?,
                item_parser.parse_next(input)?,
            ]),
            // Scalar
            (false, false) => item_parser.parse_next(input)?,
            (true, true) => unreachable!(),
        };

        Ok(out)
    }
}

/// Foreign/self references, enums, and arrays of them
///
///  {
///      "TableName": "...",
///      Id: "...",              // Scalar
///      Ids: [                  // Array / interval
///          "...",              // Single-key target
///          ["...", "..."],     // Multi-key target
///          {"rowIndex": 123},  // Bad index / no target keys
///          null,               // Null index (should only happen for scalars)
///      ]
///  }
///
fn ref_column<'a>(
    column: &ColumnSchema,
    variable_section: &'static [u8],
    resolved_keys: &HashMap<String, Vec<Value>>,
) -> impl WinnowParser<&'a [u8], Value> {
    move |input: &mut &[u8]| -> winnow::Result<_> {
        let table_name = column.references.as_ref().map(|r| &r.table);

        let mut item_parser = |input: &mut &[u8]| -> winnow::Result<_> {
            let row = dispatch! {
                empty.value(column.column_type.as_str());
                "foreignrow" => le_u128
                    .map(|r| (r != 0xfefefefe_fefefefe_fefefefe_fefefefe).then_some(r as usize)),
                "row" => le_u64.map(|r| (r != 0xfefefefe_fefefefe).then_some(r as usize)),
                // Enums are non-nullable
                "enumrow" => le_u32.map(|r| Some(r as usize)),
                _ => fail,
            }
            .parse_next(input)?;

            let Some(row) = row else {
                return Ok(Value::Null);
            };

            let value = if let Some(table_name) = table_name
                && let Some(keys) = resolved_keys.get(&table_name.to_lowercase())
                && let Some(key) = keys.get(row)
            {
                key.clone()
            } else {
                json!({"RowIndex": row})
            };

            Ok(value)
        };

        let ids = match (column.array, column.interval) {
            // Array
            (true, false) => {
                let (length, pointer) = (
                    le_u64,
                    le_u64.verify(|offset| (8..variable_section.len() as u64 + 8).contains(offset)),
                )
                    .parse_next(input)?;

                let mut input = &variable_section[pointer as usize - 8..];

                let items = std::iter::repeat_with(|| item_parser.parse_next(&mut input))
                    .take(length as usize)
                    .collect::<winnow::Result<Vec<_>>>()?;

                Value::Array(items)
            }
            // Interval
            (false, true) => Value::Array(vec![
                item_parser.parse_next(input)?,
                item_parser.parse_next(input)?,
            ]),
            // Scalar
            (false, false) => item_parser.parse_next(input)?,
            (true, true) => unreachable!(),
        };

        let out = if ids.is_null() {
            Value::Null
        } else {
            let id_key = if ids.is_array() { "Ids" } else { "Id" };

            json!({
                "TableName": table_name,
                id_key: ids,
            })
        };

        Ok(out)
    }
}

/// Creates a winnow parser from a schema which can then be applied to the bytes of the dat table
pub fn create_parser<'a>(
    resolved_keys: &HashMap<String, Vec<Value>>,
    variable_section: &'static [u8],
    schema: &DatTableSchema,
) -> impl Parser<&'a [u8], Value, ContextError> {
    move |input: &mut &[u8]| {
        let mut out = Map::new();

        for (column, column_name) in schema.columns.iter().zip(schema.column_names()) {
            let column_name = &column_name;
            let item = dispatch! {
                empty.value(column.column_type.as_str());
                "row" | "enumrow" | "foreignrow" => ref_column(column, variable_section, resolved_keys),
                "string" | "u32" | "i32" | "f32" | "u16" | "i16" | "bool" => plain_column(column, variable_section),
                _ => fail,
            }
            .parse_next(input)?;
            out.insert(column_name.to_owned(), item);
        }

        Ok(Value::Object(out))
    }
}
