use std::{fs, path::PathBuf};

use clap::{command, Parser};
use nom::{
    bytes::complete::{tag, take_until},
    combinator::{map, map_res},
    number::complete::le_u32,
    sequence::tuple,
    IResult,
};
use poe_game_data_parser::{schema_parsing::infer_types, table_view::DatTable};

/// A simple CLI tool that parses table data from a datc64 file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the .datc64 file on disk
    dat64_path: PathBuf,
}

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

fn take_utf16_string(input: &[u8]) -> String {
    let u16_data = input
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes(c.try_into().unwrap()))
        .take_while(|x| *x != 0)
        .collect::<Vec<_>>();

    String::from_utf16(&u16_data).expect("Failed to parse UTF-16 string.")
}

fn parse_table(input: &[u8]) -> IResult<&[u8], ()> {
    let (input, num_rows) = le_u32(input)?;
    println!("rows: {}", num_rows);

    let (input, (fixed_data, variable_data)) = split_on_8_bb(input)?;

    let bytes_per_row = fixed_data.len() / num_rows as usize;
    println!(
        "numeric data: {:?} bytes, per row: {} bytes",
        fixed_data.len(),
        bytes_per_row
    );
    println!("variable data: {:?} bytes", variable_data.len());

    let rows = fixed_data
        .chunks_exact(bytes_per_row)
        .map(|row| row.to_vec())
        .collect::<Vec<_>>();

    let table = DatTable {
        rows,
        variable_data: variable_data.to_vec(),
    };
    println!("{}", table);

    infer_types(&table);

    unimplemented!()
}

fn main() {
    let args = Cli::parse();

    println!("{:?}", args);

    let bytes = fs::read(args.dat64_path).expect("Failed to read table file");

    let (input, _) = parse_table(&bytes).expect("Failed to parse table");
}
