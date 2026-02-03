use itertools::{izip, Itertools};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{self},
    multi::{count, separated_list1},
    sequence::{delimited, preceded},
    IResult, Parser,
};

use super::{
    line_parser::{length_prefixed, nom_adapter, single_line, MultilineParser},
    types::*,
};
use crate::arm::line_parser::{repeated, terminated};

fn version_line<'a>() -> impl MultilineParser<'a, u32> {
    single_line(nom_adapter(preceded(
        tag("\u{feff}version "),
        complete::u32,
    )))
}

/// Quoted string ending in newline
fn quoted_str(input: &str) -> IResult<&str, String> {
    delimited(complete::char('"'), take_until("\""), complete::char('"'))
        .map(String::from)
        .parse(input)
}

fn string_section<'a>() -> impl MultilineParser<'a, Vec<String>> {
    length_prefixed(single_line(nom_adapter(quoted_str)))
}

/// Space-separated uinsigned ints
fn uints<'a>() -> impl MultilineParser<'a, Vec<u32>> {
    single_line(nom_adapter(separated_list1(
        complete::char(' '),
        complete::u32,
    )))
}

/// "k" followed by 24 numbers
fn slot_k(input: &str) -> IResult<&str, (char, Vec<u32>)> {
    let (input, letter) =
        nom::sequence::terminated(complete::char('k'), complete::char(' '))(input)?;
    let (input, numbers) = separated_list1(complete::char(' '), complete::u32)(input)?;

    Ok((input, (letter, numbers)))
}

/// k, f, s, n slots
fn parse_slot<'a>(input: &'a str, strings: &[String]) -> IResult<&'a str, Slot> {
    alt((
        complete::char('n').map(|_| Slot::N),
        complete::char('s').map(|_| Slot::S),
        preceded(tag("f "), complete::u32).map(|i| Slot::F { fill: i }),
        slot_k.map(|(_, nums)| {
            // width, height, edge_[n, w, s, e](index), [exit_n, virtual_exit_n, exit_w, ...], corner_ground_[sw, se, ne,
            // nw](index), corner_height_[sw, se, ne, nw], slot_tag(index)
            let edges = izip!(
                Direction::cardinal().into_iter(),
                &nums[2..6],
                nums[6..14].iter().tuples(),
            )
            .map(|(direction, edge, (exit, virtual_exit))| Edge {
                direction,
                edge: strings[*edge as usize].clone(),
                exit: *exit,
                virtual_exit: *virtual_exit,
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("Failed to parse edges");

            let corners = izip!(
                Direction::diagonals().into_iter(),
                &nums[14..18],
                &nums[18..22],
            )
            .map(|(direction, ground, height)| Corner {
                direction,
                ground: strings[*ground as usize].clone(),
                height: *height,
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("Failed to parse corners");

            Slot::K(SlotK {
                width: nums[0],
                height: nums[1],
                edges,
                corners,
                slot_tag: strings[nums[22] as usize].clone(),
                origin: Direction::diagonals()[*nums.get(23).unwrap_or(&0) as usize],
            })
        }),
    ))(input)
}

fn root_slot<'a>(strings: &'a [String]) -> impl MultilineParser<'a, Slot> {
    single_line(nom_adapter(|line| parse_slot(line, strings)))
}

/// Grid of Slots
fn grid<'a>(
    height: usize,
    _width: usize,
    strings: &'a [String],
) -> impl MultilineParser<'a, Vec<Vec<Slot>>> {
    let row_parser = single_line(nom_adapter(separated_list1(complete::char(' '), |l| {
        parse_slot(l, strings)
    })));

    repeated(row_parser, height)
}

/// Single PoI line - 3 uints & a string
fn poi<'a>() -> impl MultilineParser<'a, PoI> {
    let line_parser = count(
        nom::sequence::terminated(complete::u32, complete::char(' ')),
        3,
    )
    .and(quoted_str)
    .map(|(nums, string)| PoI {
        num1: nums[0],
        num2: nums[1],
        num3: nums[2],
        tag: string,
    });

    single_line(nom_adapter(line_parser))
}

fn poi_groups<'a>(version: u32) -> impl MultilineParser<'a, Vec<Vec<PoI>>> {
    // Counts determined by version, manually curated
    let count = match version {
        ..22 => 9,
        22..26 => 10,
        26..29 => 5,
        29.. => 6,
    };

    let group_parser: Box<dyn MultilineParser<'_, Vec<PoI>>> = match version {
        ..23 => Box::new(length_prefixed(poi())) as _,
        23.. => Box::new(terminated(poi(), "-1")) as _,
    };

    repeated(group_parser, count)
}

/// Single doodad string
fn doodad(line: &str) -> IResult<&str, Doodad> {
    let (line, num1) = complete::u32(line)?;
    let (line, _) = complete::char(' ')(line)?;

    let (line, num2) = complete::u32(line)?;
    let (line, _) = complete::char(' ')(line)?;

    let (line, float1) = nom::number::complete::float(line)?;
    let (line, _) = complete::char(' ')(line)?;

    let (line, rotation) = nom::number::complete::float(line)?;
    let (line, _) = complete::char(' ')(line)?;

    let (line, float2) = nom::number::complete::float(line)?;
    let (line, _) = complete::char(' ')(line)?;

    let (line, num3) = complete::u32(line)?;
    let (line, _) = complete::char(' ')(line)?;

    let (line, float3) = nom::number::complete::float(line)?;
    let (line, _) = complete::char(' ')(line)?;

    let (line, float4) = nom::number::complete::float(line)?;
    let (line, _) = complete::char(' ')(line)?;

    let (line, num4) = complete::u32(line)?;
    let (line, _) = complete::char(' ')(line)?;

    let (line, num5) = complete::u32(line)?;
    let (line, _) = complete::char(' ')(line)?;

    let (line, num6) = complete::u32(line)?;
    let (line, _) = complete::char(' ')(line)?;

    let (line, num7) = complete::u32(line)?;
    let (line, _) = complete::char(' ')(line)?;

    let (line, ao_file) = quoted_str(line)?;
    let (line, _) = complete::char(' ')(line)?;

    let (line, stub) = quoted_str(line)?;
    let (line, _) = complete::char(' ')(line)?;

    let doodad = Doodad {
        num1,
        num2,
        float1,
        rotation,
        float2,
        num3,
        float3,
        float4,
        num4,
        num5,
        num6,
        num7,
        ao_file,
        stub,
    };

    Ok((line, doodad))
}

/// Group of doodads
fn doodads<'a>(version: u32) -> impl MultilineParser<'a, Vec<Doodad>> {
    let doodad = single_line(nom_adapter(doodad));

    let group_parser: Box<dyn MultilineParser<'_, Vec<Doodad>>> = match version {
        ..23 => Box::new(length_prefixed(doodad)) as _,
        23.. => Box::new(terminated(doodad, "-1")) as _,
    };

    group_parser
}

/// Stores the entire line without interpreting
fn noop<'a>() -> impl MultilineParser<'a, String> {
    single_line(|line| Ok(("", line.to_string())))
}

/// TODO: Interpret lines
fn doodad_connections<'a>(version: u32) -> impl MultilineParser<'a, Vec<String>> {
    let doodad_connection = noop();

    let group_parser: Box<dyn MultilineParser<'_, Vec<String>>> = match version {
        ..23 => Box::new(length_prefixed(doodad_connection)) as _,
        23.. => Box::new(terminated(doodad_connection, "-1")) as _,
    };

    group_parser
}

fn decals<'a>(version: u32) -> impl MultilineParser<'a, Vec<String>> {
    let decal = noop();

    let group_parser: Box<dyn MultilineParser<'_, Vec<String>>> = match version {
        ..23 => Box::new(length_prefixed(decal)) as _,
        23.. => Box::new(terminated(decal, "-1")) as _,
    };

    group_parser
}

pub fn parse_map_str(input: &str) -> super::line_parser::Result<(Vec<String>, Map)> {
    let lines = input.lines().filter(|l| !l.is_empty()).collect::<Vec<_>>();

    let (lines, version) = version_line()(&lines)?;

    let (lines, strings) = string_section()(lines)?;

    let (lines, dimensions) = uints()(lines)?;

    let (lines, numbers1) = uints()(lines)?;

    let (lines, tag1) = single_line(nom_adapter(quoted_str))(lines)?;

    let (lines, numbers2) = uints()(lines)?;

    let (lines, root_slot) = root_slot(&strings)(lines)?;

    let (lines, numbers3) = repeated(uints(), numbers1.iter().sum::<u32>() as usize)(lines)?;

    let (lines, poi_groups) = poi_groups(version)(lines)?;

    let (lines, string1) = match version {
        ..35 => (lines, None),
        35.. => {
            let (lines, string1) = single_line(nom_adapter(quoted_str))(lines)?;
            (lines, Some(string1))
        }
    };

    let (lines, grid) = grid(dimensions[0] as usize, dimensions[1] as usize, &strings)(lines)?;

    let (lines, doodads) = doodads(version)(lines)?;

    // TODO: Might be optional in some versions
    let (lines, doodad_connections) = doodad_connections(version)(lines)?;

    // TODO: Might be optional in some versions
    let (lines, decals) = decals(version)(lines)?;

    let map = Map {
        version,
        strings: strings.to_vec(),
        dimensions,
        numbers1,
        tag: tag1,
        numbers2,
        root_slot,
        numbers3,
        points_of_interest: poi_groups,
        grid,
        doodads,
        string1,
        doodad_connections,
        decals,
    };

    // TODO: Return &str once we figure out lifetimes
    let lines = lines.iter().map(|s| s.to_string()).collect();

    Ok((lines, map))
}
