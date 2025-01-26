use std::fmt::Display;

pub struct DatTable {
    pub rows: Vec<Vec<u8>>,
    pub variable_data: Vec<u8>,
}

impl DatTable {
    /// Number of bytes in a row
    pub fn width(&self) -> usize {
        self.rows[0].len()
    }

    /// Returns column values as slices
    pub fn view_col(&self, offset: usize, width: usize) -> Vec<&[u8]> {
        assert!(offset + width <= self.width());
        self.rows
            .iter()
            .map(|row| &row[offset..offset + width])
            .collect()
    }

    /// Iterate over a column interpreted as the given type
    pub fn view_col_as<T, F: Fn(&[u8]) -> T>(
        &self,
        offset: usize,
        width: usize,
        parse_func: F,
    ) -> Vec<T> {
        self.view_col(offset, width)
            .into_iter()
            .map(parse_func)
            .collect()
    }

    /// Prints the table interpreted as integers of various sizes
    pub fn print_with_schema(&self, byte_sizes: &[usize]) {
        self.rows.iter().for_each(|row| {
            let mut offset = 0;
            for size in byte_sizes {
                let parsed = match size {
                    16 => u128::from_le_bytes(row[offset..offset + size].try_into().unwrap()),
                    8 => u64::from_le_bytes(row[offset..offset + size].try_into().unwrap()) as u128,
                    4 => u32::from_le_bytes(row[offset..offset + size].try_into().unwrap()) as u128,
                    2 => u16::from_le_bytes(row[offset..offset + size].try_into().unwrap()) as u128,
                    1 => row[offset] as u128,
                    _ => panic!(),
                };
                print!("{:4} ", parsed);
                offset += size;
            }
            println!();
        });
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
