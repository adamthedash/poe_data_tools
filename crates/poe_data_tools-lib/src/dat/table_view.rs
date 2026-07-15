use std::{fmt::Display, ops::Range};

use arrow_schema::ArrowError;

use crate::{
    dat::ivy_schema::ColumnSchema,
    file_parsers::{dat::types::DatFile, error::ParseError},
};

/// Errors related to applying a specific column schema
#[derive(Debug, thiserror::Error)]
pub enum DatColumnError {
    #[error("requested column out of bounds: bytes {range:?}, row width: {width}")]
    ColumnOutOfBounds { range: Range<usize>, width: usize },

    #[error("underflow during variable data lookup")]
    PointerUnderflow,

    #[error("overflow during variable data lookup")]
    PointerOverflow,

    #[error("array out of bounds: bytes {range:?}, variable data section length: {length}")]
    ArrayOutOfBounds { range: Range<usize>, length: usize },

    #[error("string start out of bounds: byte {start}, variable data section length: {length}")]
    StringOutOfBounds { start: usize, length: usize },

    #[error("unknown array type in schema")]
    UnknownArrayType,

    #[error("unknown column type in schema: {0:?}")]
    UnknownColumnType(Box<ColumnSchema>),

    #[error("column can't be both array and interval: {0:?}")]
    ArrayInterval(Box<ColumnSchema>),

    #[error("invalid boolean value: {0}")]
    InvalidBool(u8),
}

pub(super) type ColResult<T, E = DatColumnError> = std::result::Result<T, E>;

/// Errors related to interpreting the bytes of a dat file using a schema
#[derive(Debug, thiserror::Error)]
pub enum DatError {
    /// Contextual error layer
    #[error("failed to parse column: {column:?}")]
    Column {
        column: Box<ColumnSchema>,
        source: DatColumnError,
    },

    #[error(transparent)]
    Arrow(#[from] ArrowError),

    /// Error during initial coarse parsing of dat file structure
    #[error(transparent)]
    Parse(#[from] ParseError),

    #[error("schema not found for table {0:?}")]
    SchemaNotFound(String),

    /// Error during load of bytes from filesystem
    #[error(transparent)]
    FileSystem(#[from] crate::fs::error::FSError),

    #[error("table has no data")]
    EmptyTable,
}

pub(super) type DatResult<T, E = DatError> = std::result::Result<T, E>;

// Take a null-terminated UTF-16 string
pub fn take_utf16_string(input: &[u8]) -> String {
    let u16_data = input
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c))
        .take_while(|x| *x != 0)
        .collect::<Vec<_>>();

    String::from_utf16(&u16_data).expect("Failed to parse UTF-16 string.")
}

/// Methods for reading Dat tables column-wise
impl DatFile {
    /// Number of bytes in a row
    pub fn width(&self) -> usize {
        self.rows.first().map(|row| row.len()).unwrap_or(0)
    }

    /// Returns column values as slices
    pub fn view_col(&self, offset: usize, width: usize) -> ColResult<impl Iterator<Item = &[u8]>> {
        if offset + width > self.width() {
            return Err(DatColumnError::ColumnOutOfBounds {
                range: offset..offset + width,
                width: self.width(),
            });
        }

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
    ) -> ColResult<impl Iterator<Item = ColResult<Vec<&[u8]>>>> {
        let iter = self.view_col(offset, 16)?.map(move |bytes| {
            let length = u64::from_le_bytes(bytes[..8].try_into().unwrap()) as usize;
            let pointer = u64::from_le_bytes(bytes[8..].try_into().unwrap()) as usize;

            // Check bounds
            let start = pointer
                .checked_sub(8)
                .ok_or(DatColumnError::PointerUnderflow)?;
            let end = start
                .checked_add(
                    length
                        .checked_mul(dtype_width)
                        .ok_or(DatColumnError::PointerOverflow)?,
                )
                .ok_or(DatColumnError::PointerOverflow)?;
            if end > self.variable_data.len() {
                return Err(DatColumnError::ArrayOutOfBounds {
                    range: start..end,
                    length,
                });
            }

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
    ) -> ColResult<impl Iterator<Item = ColResult<Option<String>>> + '_> {
        let iter = self.view_col(offset, 8)?.map(move |bytes| {
            let pointer = u64::from_le_bytes(bytes.try_into().unwrap()) as usize;

            let start = pointer
                .checked_sub(8)
                .ok_or(DatColumnError::PointerUnderflow)?;
            if start > self.variable_data.len() {
                return Err(DatColumnError::StringOutOfBounds {
                    start,
                    length: self.variable_data.len(),
                });
            }

            let string = take_utf16_string(&self.variable_data[start..]);
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
    ) -> ColResult<impl Iterator<Item = ColResult<Vec<T>>> + 'a> {
        let iter = self
            .view_col_as_array(offset, dtype_width)?
            .map(move |array| array.map(|array| array.into_iter().map(&parse_func).collect()));

        Ok(iter)
    }

    /// Interpret the column as an array of strings
    pub fn view_col_as_array_of_strings(
        &self,
        offset: usize,
    ) -> ColResult<impl Iterator<Item = ColResult<Vec<Option<String>>>> + '_> {
        let iter = self
            .view_col_as_array_of(offset, 8, |bytes| {
                let pointer = u64::from_le_bytes(bytes.try_into().unwrap()) as usize;

                let start = pointer
                    .checked_sub(8)
                    .ok_or(DatColumnError::PointerUnderflow)?;
                if start > self.variable_data.len() {
                    return Err(DatColumnError::StringOutOfBounds {
                        start,
                        length: self.variable_data.len(),
                    });
                }

                let string = take_utf16_string(&self.variable_data[start..]);
                let string = if string.is_empty() {
                    None
                } else {
                    Some(string)
                };

                Ok(string)
            })?
            // Pull the Result up to the item level
            .map(|x| x?.into_iter().collect::<ColResult<Vec<_>>>());

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
