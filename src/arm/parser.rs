use itertools::{Itertools, izip};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{is_not, tag, take_till1, take_until},
    character::complete::{char as C, i32 as I, space0, space1, u32 as U},
    combinator::{self, all_consuming, rest},
    multi::{count, length_count, separated_list1},
    number::complete::float as F,
    sequence::{Tuple, delimited, preceded as P, separated_pair, terminated as T},
};

use super::{
    line_parser::{MultilineParser, length_prefixed, nom_adapter, single_line},
    types::*,
};
use crate::arm::line_parser::{repeated, terminated};

/// Parses a 0/1 as a bool
fn parse_bool(line: &str) -> IResult<&str, bool> {
    let (rest, item) = U(line)?;

    let item = match item {
        0 => false,
        1 => true,
        _ => {
            return Err(nom::Err::Error(nom::error::Error::new(
                line,
                // No better option here
                nom::error::ErrorKind::Digit,
            )));
        }
    };

    Ok((rest, item))
}

fn version_line<'a>() -> impl MultilineParser<'a, u32> {
    single_line(nom_adapter(P(tag("\u{feff}version "), U)))
}

/// Quoted string ending in newline
fn quoted_str(input: &str) -> IResult<&str, String> {
    delimited(C('"'), take_until("\""), C('"'))
        .map(String::from)
        .parse(input)
}

fn unquoted_str(input: &str) -> IResult<&str, String> {
    take_till1(|c: char| c.is_whitespace())
        .map(String::from)
        .parse(input)
}

/// Either length-prefixed or "-1"-terminated depending on version
fn group<'a, T: 'a>(
    version: u32,
    item_parser: impl MultilineParser<'a, T> + 'a,
) -> impl MultilineParser<'a, Vec<T>> {
    let group_parser: Box<dyn MultilineParser<'_, Vec<T>>> = match version {
        ..32 => Box::new(length_prefixed(item_parser)) as _,
        32.. => Box::new(terminated(item_parser, "-1")) as _,
    };

    group_parser
}

fn string_section<'a>() -> impl MultilineParser<'a, Vec<String>> {
    length_prefixed(single_line(nom_adapter(quoted_str)))
}

fn dimensions<'a>(version: u32) -> impl MultilineParser<'a, Dimension> {
    let parser = move |line| -> IResult<&str, Dimension> {
        let (line, side_length) = U(line)?;

        let line = match version {
            ..31 => {
                let (line, _) = P(space1, U)(line)?;
                line
            }
            31.. => line,
        };

        let (line, uint1) = match version {
            ..22 => (line, None),
            22.. => {
                let (line, bool1) = P(space1, U)(line)?;

                (line, Some(bool1))
            }
        };

        let dimension = Dimension { side_length, uint1 };

        Ok((line, dimension))
    };

    single_line(nom_adapter(parser))
}

/// Space-separated uinsigned ints
fn uints<'a>() -> impl MultilineParser<'a, Vec<u32>> {
    single_line(nom_adapter(separated_list1(C(' '), U)))
}

/// "k" followed by 23-24 numbers
fn slot_k<'a>(input: &'a str, strings: &[String]) -> IResult<&'a str, SlotK> {
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

/// k, f, s, o, n slots
fn parse_slot<'a>(input: &'a str, strings: &[String]) -> IResult<&'a str, Slot> {
    alt((
        C('n').map(|_| Slot::N),
        C('s').map(|_| Slot::S),
        C('o').map(|_| Slot::O),
        P(tag("f "), U).map(|i| Slot::F {
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
    let row_parser = single_line(nom_adapter(separated_list1(C(' '), |l| {
        parse_slot(l, strings)
    })));

    repeated(row_parser, height)
}

/// Single PoI line - 3 uints & a string
fn poi<'a>() -> impl MultilineParser<'a, PoI> {
    let line_parser = |line| -> IResult<&str, PoI> {
        let (line, (x, y, rotation, tag)) = (
            U,
            P(space1, U),
            P(space1, F),
            // TODO: Remove \u0000 chars from this
            P(space1, quoted_str),
        )
            .parse(line)?;

        let poi = PoI {
            x,
            y,
            rotation,
            tag,
        };

        Ok((line, poi))
    };

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

    let group_parser = group(version, poi());

    repeated(group_parser, count)
}

/// Single doodad string
fn doodad<'a>(version: u32) -> impl MultilineParser<'a, Doodad> {
    let parser = for<'b> move |line: &'b str| -> IResult<&'b str, Doodad> {
        let (line, x) = U(line)?;
        let (line, _) = space1(line)?;

        let (line, y) = U(line)?;
        let (line, _) = space1(line)?;

        let (line, float_pairs) = match version {
            ..34 => (line, None),
            34.. => {
                let (line, count1) = U(line)?;
                let (line, _) = space1(line)?;

                let (line, pairs) =
                    count(T(separated_pair(F, space1, F), space1), count1 as usize)(line)?;

                (line, Some(pairs))
            }
        };

        let (line, radians1) = F(line)?;
        let (line, _) = space1(line)?;

        let (line, (trig1, trig2, trig3, trig4)) = match version {
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

        let (line, bool1) = parse_bool(line)?;
        let (line, _) = space1(line)?;

        let (line, bool2) = match version {
            ..25 => (line, None),
            25.. => {
                let (line, bool2) = parse_bool(line)?;
                let (line, _) = space1(line)?;

                (line, Some(bool2))
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
            ..36 => (line, None),
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

                (line, Some(key_values))
            }
        };

        let doodad = Doodad {
            x,
            y,
            float_pairs,
            radians1,
            trig1,
            trig2,
            trig3,
            trig4,
            bool1,
            bool2,
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

fn doodad_connections<'a>(version: u32) -> impl MultilineParser<'a, Vec<DoodadConnection>> {
    let doodad_connection = single_line(nom_adapter(count(T(U, space1), 2).and(quoted_str).map(
        |(nums, tag)| DoodadConnection {
            from: nums[0],
            to: nums[1],
            tag,
        },
    )));

    group(version, doodad_connection)
}

/// Decale on a line
fn decal<'a>(version: u32) -> impl MultilineParser<'a, Decal> {
    let parser = move |line| -> IResult<&str, Decal> {
        let (line, floats) = count(T(F, space1), 3)(line)?;

        let (line, bool1) = match version {
            ..17 => (line, None),
            17.. => {
                let (line, uint1) = T(parse_bool, space1)(line)?;
                (line, Some(uint1))
            }
        };

        let (line, scale) = T(F, space1)(line)?;

        let (line, atlas_file) = T(quoted_str, space1)(line)?;

        let (line, tag) = quoted_str(line)?;

        let decal = Decal {
            x: floats[0],
            y: floats[1],
            rotation: floats[2],
            bool1,
            scale,
            atlas_file,
            tag,
        };

        Ok((line, decal))
    };

    single_line(nom_adapter(parser))
}

fn boss_lines<'a>(
    lines: &'a [&'a str],
) -> super::line_parser::Result<(Vec<&'a str>, Vec<Vec<String>>)> {
    // NOTE: Boss string line is often not properly ended with a newline, causing the next
    // line to be appended on the end instead. We're accounting for this by cropping off
    // the end and prepending it to our list of lines
    let line_parser = separated_pair(separated_list1(space1, quoted_str), space1, rest);
    let line_parser = single_line(nom_adapter(line_parser));

    let (lines, boss_lines) = length_prefixed(line_parser)(lines)?;
    let mut lines = lines.to_vec();

    let (boss_lines, mut trailing) =
        boss_lines
            .into_iter()
            .fold((vec![], vec![]), |(mut blines, mut tlines), (b, t)| {
                blines.push(b);
                tlines.push(t);

                (blines, tlines)
            });

    if let Some(t) = trailing.pop()
        && !t.is_empty()
    {
        lines.insert(0, t);
    }

    assert!(trailing.iter().all(|s| s.is_empty()));

    Ok((lines, boss_lines))
}

fn zone<'a>(version: u32) -> impl MultilineParser<'a, Zone> {
    let parser = move |line| -> IResult<&str, Zone> {
        let (line, name) = match version {
            ..35 => unquoted_str(line)?,
            35.. => quoted_str(line)?,
        };

        let (line, bbox) = count(P(space1, I), 4)(line)?;

        let (line, string1, env_file, uint1) = match version {
            ..35 => (line, None, None, None),
            35.. => {
                let (line, string1) = P(space1, quoted_str)(line)?;
                let (line, env_file) = P(space1, quoted_str)(line)?;
                let (line, uint1) = P(space1, U)(line)?;

                (line, Some(string1), Some(env_file), Some(uint1))
            }
        };

        let zone = Zone {
            name,
            x_min: bbox[0],
            y_min: bbox[1],
            x_max: bbox[2],
            y_max: bbox[3],
            disable_teleports: string1,
            env_file,
            uint1,
        };

        Ok((line, zone))
    };

    single_line(nom_adapter(parser))
}

fn zones<'a>(version: u32) -> impl MultilineParser<'a, Vec<Zone>> {
    let line_parser = zone(version);

    // NOTE: Appears to be LP until v33, then v36 appears another LP?
    let group_parser: Box<dyn MultilineParser<'_, Vec<Zone>>> = match version {
        ..33 => Box::new(length_prefixed(line_parser)) as _,
        33.. => Box::new(terminated(line_parser, "-1")) as _,
    };

    group_parser
}

fn tags<'a>() -> impl MultilineParser<'a, Option<Vec<String>>> {
    |lines| {
        if let Some(&first) = lines.first() {
            let mut line_parser =
                combinator::opt(all_consuming(length_count(U, P(space1, unquoted_str))));
            let (_, tags) = line_parser(first)?;

            // Crop off last line if successful
            let lines = if tags.is_some() { &lines[1..] } else { lines };

            Ok((lines, tags))
        } else {
            Ok((lines, None))
        }
    }
}

/// Line of space separated uints, sometimes with a space a the end
fn trailing<'a>() -> impl MultilineParser<'a, Option<Vec<u32>>> {
    |lines| {
        let (lines, trailing) = if let Some(&last) = lines.first() {
            let mut line_parser = combinator::opt(all_consuming(T(
                separated_list1(space1::<_, nom::error::Error<_>>, U),
                space0,
            )));
            let (_, trailing_nums) = line_parser(last)?;

            // Crop off last line if successful
            let lines = if trailing_nums.is_some() {
                &lines[1..]
            } else {
                lines
            };

            (lines, trailing_nums)
        } else {
            (lines, None)
        };

        Ok((lines, trailing))
    }
}

pub fn thingy<'a>(strings: &[String]) -> impl MultilineParser<'a, Thingy> {
    let parser = |line| -> IResult<&str, Thingy> {
        let (line, index) = U(line)?;

        let et_file = (index > 0).then(|| strings[index as usize - 1].clone());

        let (line, int) = P(space1, I)(line)?;

        let (line, bools) = count(combinator::opt(P(space1, parse_bool)), 3)(line)?;

        let thingy = Thingy {
            et_file,
            int,
            bool1: bools[0],
            bool2: bools[1],
            bool3: bools[2],
        };

        Ok((line, thingy))
    };

    single_line(nom_adapter(parser))
}

pub fn parse_map_str(input: &str) -> super::line_parser::Result<(Vec<String>, Map)> {
    let lines = input.lines().filter(|l| !l.is_empty()).collect::<Vec<_>>();

    let (lines, version) = version_line()(&lines)?;

    let (lines, strings) = string_section()(lines)?;

    let (lines, dimensions) = dimensions(version)(lines)?;

    let (lines, numbers1) = uints()(lines)?;

    let (lines, tag1) = single_line(nom_adapter(quoted_str))(lines)?;

    let (lines, bools) = single_line(nom_adapter(separated_list1(space1, parse_bool)))(lines)?;

    let (lines, root_slot) = root_slot(&strings)(lines)?;

    let (lines, thingies) =
        repeated(thingy(&strings), numbers1.iter().sum::<u32>() as usize * 2)(lines)?;

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

    let (lines, doodads) = group(version, doodad(version))(lines)?;

    let (lines, doodad_connections) = match version {
        ..23 => (lines, vec![]),
        23.. => doodad_connections(version)(lines)?,
    };

    let (lines, decals) = group(version, decal(version))(lines)?;

    let (lines, boss_lines) = match version {
        ..22 => (lines.to_vec(), None),
        22.. => {
            let (lines, boss_lines) = boss_lines(lines)?;

            (lines, Some(boss_lines))
        }
    };
    let lines = lines.as_slice();

    let (lines, zones) = match version {
        ..27 => (lines, None),
        27.. => {
            let (lines, item_lines) = zones(version)(lines)?;

            (lines, Some(item_lines))
        }
    };

    // Optional line with tags
    let (lines, tags) = match version {
        ..10 => (lines, None),
        10.. => tags()(lines)?,
    };

    // Optional trailing line with a bunch of numbers
    let (lines, trailing) = trailing()(lines)?;

    assert!(lines.is_empty(), "Extra lines: {:#?}", lines);

    let map = Map {
        version,
        strings: strings.to_vec(),
        dimensions,
        numbers1,
        tag: tag1,
        bools,
        root_slot,
        thingies,
        points_of_interest: poi_groups,
        grid,
        doodads,
        string1,
        doodad_connections,
        decals,
        boss_lines,
        zones,
        tags,
        trailing,
    };

    // TODO: Return &str once we figure out lifetimes
    let lines = lines.iter().map(|s| s.to_string()).collect::<Vec<_>>();

    Ok((lines, map))
}
