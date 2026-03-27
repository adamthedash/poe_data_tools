use std::collections::HashMap;

use serde_json::{Number, Value, json, map::Map};
use winnow::{
    Parser,
    binary::{le_f32, le_i16, le_i32, le_u8, le_u16, le_u32, le_u64, le_u128},
    combinator::{dispatch, empty, fail},
    error::ContextError,
    token::rest,
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

            "u32" => le_u32.map(|x| Value::Number(Number::from(x))),
            "i32" => le_i32.map(|x| Value::Number(Number::from(x))),
            "f32" => le_f32.map(|x| serde_json::to_value(x).unwrap()),

            "u16" => le_u16.map(|x| Value::Number(Number::from(x))),
            "i16" => le_i16.map(|x| Value::Number(Number::from(x))),

            "bool" => le_u8.map(|b|
                            if b <= 2 {
                                Some(b == 1)
                            } else {
                                eprintln!("WARN: Bad value for bool: {b}");
                                None
                            }
                        )
                        .map(|x| serde_json::to_value(x).unwrap()),

            "array" => empty.value(Value::Null),

            _ => fail,
        };

        let out = match (column.array, column.interval) {
            // Array
            (true, false) => {
                let (length, pointer) = (
                    le_u64,
                    le_u64.map(|offset| {
                        if (8..variable_section.len() as u64 + 8).contains(&offset) {
                            Some(offset)
                        } else {
                            eprintln!("WARN: Array pointer out of bounds: {offset}");
                            None
                        }
                    }),
                )
                    .parse_next(input)?;

                let Some(pointer) = pointer else {
                    return Ok(Value::Null);
                };

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
///  Return values:
///  null   // Null scalar index
///  []     // Empty array / interval
///  {
///      "TableName": "...",     // Known target table
///      "TableName": null,      // Unknown target table
///
///      Id: "...",              // Scalar with good target index, single-key target
///      Id: ["..."],            // Scalar with good target index, multi-key target
///      "RowIndex": 12345       // Scalar with bad index / no target table
///
///      Ids: [                  // Array / interval
///          "...",              // Single-key target
///          ["...", "..."],     // Multi-key target
///          {"rowIndex": 123},  // Bad index / no target keys
///          null,               // Null index (technically possible but shouldn't appear in an array)
///      ]
///
///      "RowIndices": [         // Array / interval with no target table
///         12345,
///         131235,
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

            // If the table it refers to is known
            let value = if let Some(table_name) = table_name
                // And that table exists
                && let Some(keys) = resolved_keys.get(&table_name.to_lowercase())
                // And the row it refers to exists
                && let Some(key) = keys.get(row)
                // And that row has a primary key
                && !key.is_null()
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
                    le_u64.map(|offset| {
                        if (8..variable_section.len() as u64 + 8).contains(&offset) {
                            Some(offset)
                        } else {
                            eprintln!("WARN: Array pointer out of bounds: {offset}");
                            None
                        }
                    }),
                )
                    .parse_next(input)?;

                let Some(pointer) = pointer else {
                    return Ok(Value::Null);
                };

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

        let out = match ids {
            // For scalar null refs, collapse to null
            Value::Null => Value::Null,
            Value::Array(values) => {
                if values.is_empty() {
                    // For empty array of refs, collapse to []
                    Value::Array(vec![])
                } else {
                    let ids_key = if column.is_multi() {
                        // Array
                        "Ids"
                    } else {
                        // Scalar with multi-key
                        "Id"
                    };
                    json!({
                        "TableName": table_name,
                        ids_key: values,
                    })
                }
            }
            Value::Bool(_) | Value::Number(_) | Value::String(_) => json!({
                "TableName": table_name,
                "Id": ids,
            }),
            Value::Object(_) => {
                // Scalar row index or foreign ref

                ids
            }
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
            let res = if column.is_ref() {
                ref_column(column, variable_section, resolved_keys).parse_next(input)
            } else {
                plain_column(column, variable_section).parse_next(input)
            };

            if let Ok(item) = res {
                out.insert(column_name, item);
            } else {
                // NOTE: All the item parsers should pass with null on error, so an error here should
                //  be unrecoverable. Return items parsed up until now instead of losing whole
                //  row.
                eprintln!(
                    "WARN: Error applying schema for {:?} {:?}, bytes left: {:?}",
                    schema.name,
                    column,
                    input.len(),
                );
                break;
            }
        }

        if !input.is_empty() {
            eprintln!(
                "WARN: Extra bytes left after applying schema: {:?}",
                input.len(),
            );

            let _rest = rest(input)?;
        }

        Ok(Value::Object(out))
    }
}
