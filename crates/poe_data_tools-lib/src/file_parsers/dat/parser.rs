use winnow::{
    Parser,
    binary::le_u32,
    combinator::terminated,
    token::{rest, take_until},
};

use super::types::*;
use crate::file_parsers::shared::winnow::WinnowParser;

fn table_section<'a>() -> impl WinnowParser<&'a [u8], &'a [u8]> {
    terminated(
        take_until(0.., [0xBB; 8].as_slice()), //
        [0xBB; 8].as_slice(),
    )
}

pub fn dat<'a>() -> impl WinnowParser<&'a [u8], DatFile> {
    (
        le_u32, //
        table_section(),
        rest,
    )
        .map(|(num_rows, fixed_data, variable_data)| {
            let rows = if num_rows == 0 {
                vec![]
            } else {
                let bytes_per_row = fixed_data.len() / num_rows as usize;

                fixed_data
                    .chunks_exact(bytes_per_row)
                    .map(|row| row.to_vec())
                    .collect::<Vec<_>>()
            };

            DatFile {
                rows,
                variable_data: variable_data.to_vec(),
            }
        })
}
