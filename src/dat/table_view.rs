use std::fmt::Display;

use anyhow::{ensure, Context, Result};
use nom::{
    bytes::complete::{tag, take_until},
    combinator::map_res,
    number::complete::le_u32,
    sequence::tuple,
    IResult,
};

/// Splits a byte slice into two parts around 16 consecutive 0xBB bytes.
/// Returns a tuple containing the two halves.
fn split_on_8_bb(input: &[u8]) -> IResult<&[u8], (&[u8], &[u8])> {
    // Define the delimiter: 16 consecutive 0xBB bytes
    const DELIMITER: &[u8] = &[0xBB; 8];

    // Parser to take until the delimiter and then consume it
    let parser = tuple((
        take_until(DELIMITER), // Take everything up to the delimiter
        tag(DELIMITER),        // Match the delimiter itself
    ));

    // Apply the parser and return the remaining parts
    map_res(
        parser,
        |(before, _delimiter): (&[u8], &[u8])| -> Result<(&[u8], &[u8]), ()> {
            // The rest of the input after the delimiter
            let remaining: &[u8] = &input[before.len() + DELIMITER.len()..];
            Ok((before, remaining))
        },
    )(input)
}

// Take a null-terminated UTF-16 string
fn take_utf16_string(input: &[u8]) -> String {
    let u16_data = input
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes(c.try_into().unwrap()))
        .take_while(|x| *x != 0)
        .collect::<Vec<_>>();

    String::from_utf16(&u16_data).expect("Failed to parse UTF-16 string.")
}
pub struct DatTable {
    pub rows: Vec<Vec<u8>>,
    pub variable_data: Vec<u8>,
}

impl DatTable {
    /// Parses a raw datc64 file
    pub fn from_raw_bytes(bytes: &[u8]) -> IResult<&[u8], Self> {
        let (input, num_rows) = le_u32(bytes)?;

        let (input, (fixed_data, variable_data)) = split_on_8_bb(input)?;

        let rows = if num_rows == 0 {
            vec![]
        } else {
            let bytes_per_row = fixed_data.len() / num_rows as usize;

            fixed_data
                .chunks_exact(bytes_per_row)
                .map(|row| row.to_vec())
                .collect::<Vec<_>>()
        };

        Ok((
            input,
            Self {
                rows,
                variable_data: variable_data.to_vec(),
            },
        ))
    }

    /// Number of bytes in a row
    pub fn width(&self) -> usize {
        self.rows.first().map(|row| row.len()).unwrap_or(0)
    }

    /// Returns column values as slices
    pub fn view_col(&self, offset: usize, width: usize) -> Result<impl Iterator<Item = &[u8]>> {
        ensure!(
            offset + width <= self.width(),
            "Requested column out of bounds"
        );
        let iter = self
            .rows
            .iter()
            .map(move |row| &row[offset..offset + width]);

        Ok(iter)
    }

    /// Interpret the column as an array of values, dereferencing from the variable data section
    pub fn view_col_as_array(
        &self,
        offset: usize,
        dtype_width: usize,
    ) -> Result<impl Iterator<Item = Result<Vec<&[u8]>>>> {
        let iter = self.view_col(offset, 16)?.map(move |bytes| {
            let length = u64::from_le_bytes(bytes[..8].try_into().unwrap()) as usize;
            let pointer = u64::from_le_bytes(bytes[8..].try_into().unwrap()) as usize;

            // Check bounds
            let start = pointer.checked_sub(8).context("underflow")?;
            let end = start
                .checked_add(length.checked_mul(dtype_width).context("Overflow")?)
                .context("Overflow")?;
            ensure!(end < self.variable_data.len(), "Array slice oveflow");

            let bytes = self.variable_data[start..end]
                .chunks_exact(dtype_width)
                .collect();
            Ok(bytes)
        });

        Ok(iter)
    }

    /// Interpret a column as strings, dereferencing them from the variable data section
    pub fn view_col_as_string(
        &self,
        offset: usize,
    ) -> Result<impl Iterator<Item = Result<Option<String>>> + '_> {
        let iter = self.view_col(offset, 8)?.map(move |bytes| {
            let pointer = u64::from_le_bytes(bytes.try_into().unwrap()) as usize;
            ensure!(
                pointer >= 8 && pointer < self.variable_data.len() + 8,
                "Array pointer out of bounds"
            );

            let string = take_utf16_string(&self.variable_data[pointer - 8..]);
            let string = if string.is_empty() {
                None
            } else {
                Some(string)
            };

            Ok(string)
        });

        Ok(iter)
    }

    /// Interpret the column as an array of values, dereferencing from the variable_data section
    /// and interpreting them
    pub fn view_col_as_array_of<'a, T, F: Fn(&[u8]) -> T + 'a>(
        &'a self,
        offset: usize,
        dtype_width: usize,
        parse_func: F,
    ) -> Result<impl Iterator<Item = Result<Vec<T>>> + 'a> {
        let iter = self
            .view_col_as_array(offset, dtype_width)?
            .map(move |array| array.map(|array| array.into_iter().map(&parse_func).collect()));

        Ok(iter)
    }

    /// Interpret the column as an array of strings
    pub fn view_col_as_array_of_strings(
        &self,
        offset: usize,
    ) -> Result<impl Iterator<Item = Result<Vec<Option<String>>>> + '_> {
        let iter = self
            .view_col_as_array_of(offset, 8, |bytes| {
                let pointer = u64::from_le_bytes(bytes.try_into().unwrap()) as usize;

                ensure!(pointer >= 8, "String pointer underflow");
                ensure!(
                    pointer - 8 < self.variable_data.len(),
                    "String pointer overflow"
                );

                let string = take_utf16_string(&self.variable_data[pointer - 8..]);
                let string = if string.is_empty() {
                    None
                } else {
                    Some(string)
                };

                Ok(string)
            })?
            // Pull the Result up to the item level
            .map(|x| x?.into_iter().collect::<Result<Vec<_>>>());

        Ok(iter)
    }
}

impl Display for DatTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Debug - print out the values as hex
        for row in self.rows.iter() {
            let chars = row
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");

            writeln!(f, "{}", chars)?;
        }

        Ok(())
    }
}
