use std::{fs, path::PathBuf};

use clap::{command, Parser};
use nom::{
    bytes::complete::take_until, combinator::map, number::complete::le_u32, sequence::tuple,
    IResult,
};

/// A simple CLI tool that parses table data from a dat64 file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the .dat64 file on disk
    dat64_path: PathBuf,
}

fn take_until_string_separator(input: &[u8]) -> IResult<&[u8], &[u8]> {
    // Define the pattern for 8 consecutive 0xBB bytes
    const PATTERN: &[u8] = &[0xBB; 8];

    // Use `take_until` to consume the stream until we find the pattern
    let parser = tuple((take_until(PATTERN), take_until(PATTERN)));
    map(parser, |(data, _)| data)(input)
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

    let (input, fixed_data) = take_until_string_separator(input)?;

    let bytes_per_row = fixed_data.len() / num_rows as usize;
    println!(
        "numeric data: {:?} bytes, per row: {} bytes",
        fixed_data.len(),
        bytes_per_row
    );

    let rows = fixed_data.chunks_exact(bytes_per_row).collect::<Vec<_>>();

    rows.iter().for_each(|r| {
        let pointer_offsets = [0, 12, 28, 36];

        let strings = pointer_offsets
            .iter()
            .map(|&p| u64::from_le_bytes(r[p..p + 8].try_into().unwrap()))
            .map(|p| take_utf16_string(&input[p as usize..]))
            .collect::<Vec<_>>();

        println!("{:?}", strings);

        let list_pointer_offset = 61;
        let list_pointer = u64::from_le_bytes(
            r[list_pointer_offset..list_pointer_offset + 8]
                .try_into()
                .unwrap(),
        ) as usize;

        println!("{:?}", &r[53..]);

        //println!("{:?}", &input[list_pointer..list_pointer + 32]);
    });

    // Debug - print out the values as hex
    rows.iter().for_each(|row| {
        let chars = row
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ");

        println!("{}", chars);
    });

    unimplemented!()
}

fn main() {
    let args = Cli::parse();

    println!("{:?}", args);

    let bytes = fs::read(args.dat64_path).expect("Failed to read table file");

    let (input, _) = parse_table(&bytes).expect("Failed to parse table");
}
