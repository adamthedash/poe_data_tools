use std::fmt::Display;

use anyhow::{Context, Result, ensure};

use crate::file_parsers::dat::types::DatFile;

// Take a null-terminated UTF-16 string
fn take_utf16_string(input: &[u8]) -> String {
    let u16_data = input
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes(c.try_into().unwrap()))
        .take_while(|x| *x != 0)
        .collect::<Vec<_>>();

    String::from_utf16(&u16_data).expect("Failed to parse UTF-16 string.")
}

impl DatFile {
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
            ensure!(end <= self.variable_data.len(), "Array slice oveflow");

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

impl Display for DatFile {
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
