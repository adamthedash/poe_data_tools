use std::collections::HashMap;

use itertools::{izip, Itertools};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{self, space1},
    combinator,
    multi::{count, separated_list1},
    sequence::{delimited, preceded, separated_pair},
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

/// Space-separated signed ints
fn ints<'a>() -> impl MultilineParser<'a, Vec<i32>> {
    single_line(nom_adapter(separated_list1(
        complete::char(' '),
        complete::i32,
    )))
}

/// "k" followed by 24 numbers
fn slot_k<'a>(input: &'a str, strings: &[String]) -> IResult<&'a str, SlotK> {
    use nom::{
        character::complete::{char as C, i32 as I, u32 as U},
        sequence::{preceded as P, terminated as T},
    };

    let (input, _letter) = T(C('k'), C(' '))(input)?;

    let (input, grid_dims) = count(T(U, C(' ')), 2)(input)?;

    let (input, edges) = count(T(U, C(' ')), 4)(input)?;

    let (input, exits) = count(T(U, C(' ')), 8)(input)?;

    let (input, corner_grounds) = count(T(U, C(' ')), 4)(input)?;

    let (input, corner_heights) = count(T(I, C(' ')), 4)(input)?;

    let (input, slot_tag) = U(input)?;

    let (input, origin_dir) = combinator::opt(P(C(' '), U))(input)?;

    let edges = izip!(
        Direction::cardinal().into_iter(),
        edges,
        exits.into_iter().tuples(),
    )
    .map(|(direction, edge, (exit, virtual_exit))| Edge {
        direction,
        // 0 -> None,
        // i -> Some(i - 1)
        edge: (edge > 0).then(|| strings[(edge - 1) as usize].clone()),
        exit,
        virtual_exit,
    })
    .collect::<Vec<_>>()
    .try_into()
    .expect("Parser should take care of counts");

    let corners = izip!(
        Direction::diagonals().into_iter(),
        corner_grounds,
        corner_heights,
    )
    .map(|(direction, ground, height)| Corner {
        direction,
        ground: (ground > 0).then(|| strings[ground as usize - 1].clone()),
        height,
    })
    .collect::<Vec<_>>()
    .try_into()
    .expect("Parser should take care of counts");

    let slot = SlotK {
        width: grid_dims[0],
        height: grid_dims[1],
        edges,
        corners,
        slot_tag: (slot_tag > 0).then(|| strings[slot_tag as usize - 1].clone()),
        origin: Direction::diagonals()[origin_dir.unwrap_or(0) as usize],
    };

    Ok((input, slot))
}

/// k, f, s, n slots
fn parse_slot<'a>(input: &'a str, strings: &[String]) -> IResult<&'a str, Slot> {
    alt((
        complete::char('n').map(|_| Slot::N),
        complete::char('s').map(|_| Slot::S),
        complete::char('o').map(|_| Slot::O),
        preceded(tag("f "), complete::u32).map(|i| Slot::F {
            fill: (i > 0).then(|| strings[i as usize - 1].clone()),
        }),
        (|input| slot_k(input, strings)).map(Slot::K),
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
        2,
    )
    .and(nom::sequence::terminated(
        nom::number::complete::float,
        complete::char(' '),
    ))
    .and(quoted_str)
    .map(|((nums12, num3), string)| PoI {
        num1: nums12[0],
        num2: nums12[1],
        num3,
        tag: string,
    });

    single_line(nom_adapter(line_parser))
}

fn poi_groups<'a>(version: u32) -> impl MultilineParser<'a, Vec<Vec<PoI>>> {
    // Counts determined by version, manually curated
    let count = match version {
        ..20 => 9,
        20..26 => 10,
        26..29 => 5,
        29.. => 6,
    };

    let group_parser: Box<dyn MultilineParser<'_, Vec<PoI>>> = match version {
        ..32 => Box::new(length_prefixed(poi())) as _,
        32.. => Box::new(terminated(poi(), "-1")) as _,
    };

    repeated(group_parser, count)
}

/// Single doodad string
fn doodad<'a>(version: u32) -> impl MultilineParser<'a, Doodad> {
    use nom::{
        character::complete::{char as C, u32 as U},
        number::complete::float as F,
        sequence::{preceded as P, terminated as T},
    };

    let parser = for<'b> move |line: &'b str| -> IResult<&'b str, Doodad> {
        // println!("Line: {:?}", line);
        let (line, x) = U(line)?;
        let (line, _) = space1(line)?;

        let (line, y) = U(line)?;
        let (line, _) = space1(line)?;

        let (line, float_pairs) = match version {
            ..34 => (line, vec![]),
            34.. => {
                let (line, count1) = U(line)?;
                let (line, _) = space1(line)?;

                count(T(separated_pair(F, space1, F), space1), count1 as usize)(line)?
            }
        };

        let (line, radians1) = F(line)?;
        let (line, _) = space1(line)?;

        let (line, (radians2, radians3, radians4, radians5)) = match version {
            ..18 => (line, (None, None, None, None)),
            18.. => {
                let (line, radians2) = F(line)?;
                let (line, _) = space1(line)?;

                let (line, radians3) = F(line)?;
                let (line, _) = space1(line)?;

                let (line, radians4) = F(line)?;
                let (line, _) = space1(line)?;

                let (line, radians5) = F(line)?;
                let (line, _) = space1(line)?;

                (
                    line,
                    (
                        Some(radians2),
                        Some(radians3),
                        Some(radians4),
                        Some(radians5),
                    ),
                )
            }
        };

        let (line, uint3) = U(line)?;
        let (line, _) = space1(line)?;

        // TODO: Unclear whether it is uint3 or 4 that's missing.
        let (line, uint4) = match version {
            ..25 => (line, None),
            25.. => {
                let (line, uint4) = U(line)?;
                let (line, _) = space1(line)?;

                (line, Some(uint4))
            }
        };

        let (line, count1) = U(line)?;
        let (line, _) = space1(line)?;

        let (line, floats) = count(T(F, space1), count1 as usize)(line)?;

        let (line, scale) = F(line)?;
        let (line, _) = space1(line)?;

        let (line, ao_file) = quoted_str(line)?;
        let (line, _) = space1(line)?;

        let (line, stub) = quoted_str(line)?;

        let (line, key_values) = match version {
            ..36 => (line, HashMap::new()),
            36.. => {
                let (line, _) = space1(line)?;
                let (line, count1) = U(line)?;

                let (line, key_values) = count(
                    P(space1, separated_pair(is_not("="), C('='), is_not(" "))),
                    count1 as usize,
                )(line)?;

                let key_values = key_values
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();

                (line, key_values)
            }
        };

        let doodad = Doodad {
            x,
            y,
            float_pairs,
            radians1,
            radians2,
            radians3,
            radians4,
            radians5,
            uint3,
            uint4,
            floats,
            scale,
            ao_file,
            stub,
            key_values,
        };

        Ok((line, doodad))
    };

    single_line(nom_adapter(parser))
}

/// Group of doodads
fn doodads<'a>(version: u32) -> impl MultilineParser<'a, Vec<Doodad>> {
    let doodad = doodad(version);

    let group_parser: Box<dyn MultilineParser<'_, Vec<Doodad>>> = match version {
        ..32 => Box::new(length_prefixed(doodad)) as _,
        32.. => Box::new(terminated(doodad, "-1")) as _,
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
        ..32 => Box::new(length_prefixed(doodad_connection)) as _,
        32.. => Box::new(terminated(doodad_connection, "-1")) as _,
    };

    group_parser
}

fn decals<'a>(version: u32) -> impl MultilineParser<'a, Vec<String>> {
    let decal = noop();

    let group_parser: Box<dyn MultilineParser<'_, Vec<String>>> = match version {
        ..32 => Box::new(length_prefixed(decal)) as _,
        32.. => Box::new(terminated(decal, "-1")) as _,
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

    let (lines, numbers3) = repeated(ints(), numbers1.iter().sum::<u32>() as usize * 2)(lines)?;

    let (lines, poi_groups) = poi_groups(version)(lines)?;

    let (lines, string1) = match version {
        ..35 => (lines, None),
        35.. => {
            let (lines, string1) = single_line(nom_adapter(quoted_str))(lines)?;
            (lines, Some(string1))
        }
    };

    let (grid_height, grid_width) = if let Slot::K(slot) = &root_slot {
        (slot.height as usize, slot.width as usize)
    } else {
        (1, 1)
    };
    let (lines, grid) = grid(grid_height, grid_width, &strings)(lines)?;

    // println!("{:#?}", lines);
    let (lines, doodads) = doodads(version)(lines)?;

    let (lines, doodad_connections) = match version {
        ..15 => (lines, vec![]),
        15.. => doodad_connections(version)(lines)?,
    };

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
