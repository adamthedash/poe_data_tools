use std::{rc::Rc, sync::Mutex};

use anyhow::Result;
use itertools::{Itertools, izip};
use nom::{
    Parser,
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{char as C, i32 as I, space0, space1, u32 as U},
    combinator::{all_consuming, cond, opt, rest, verify},
    multi::{count, length_count, many_till, separated_list1},
    number::complete::float as F,
    sequence::{preceded as P, separated_pair, terminated as T},
};

use super::types::*;
use crate::file_parsers::{
    lift_nom::{SliceParser, ToSliceParser},
    shared::{NomParser, parse_bool, quoted_str, separated_array, unquoted_str, version_line},
    slice::Slice,
};

fn length_prefixed2<'a, T>(
    item_parser: impl NomParser<'a, T>,
) -> impl SliceParser<'a, &'a str, Vec<T>> {
    length_count(U::<_, nom::error::Error<_>>.lift(), item_parser.lift())
}

fn terminated2<'a, T>(
    item_parser: impl NomParser<'a, T>,
    sentinel: &str,
) -> impl SliceParser<'a, &'a str, Vec<T>> {
    many_till(
        item_parser.lift(),
        tag::<_, _, nom::error::Error<_>>(sentinel).lift(),
    )
    .map(|(items, _)| items)
}

/// Either length-prefixed or "-1"-terminated depending on version
fn group<'a, const V: u32, T: 'a>(
    version: u32,
    item_parser: impl NomParser<'a, T> + 'a,
) -> impl SliceParser<'a, &'a str, Vec<T>> {
    let item_parser = Rc::new(Mutex::new(item_parser));

    let i1 = {
        let item_parser = item_parser.clone();
        move |input| item_parser.lock().unwrap().parse_complete(input)
    };
    let i2 = { move |input| item_parser.lock().unwrap().parse_complete(input) };

    (
        cond(version < V, length_prefixed2(i1)),
        cond(version >= V, terminated2(i2, "-1")),
    )
        .map(|(a, b)| a.or(b).expect("One parser should always be applied"))
}

fn string_section<'a>() -> impl SliceParser<'a, &'a str, Vec<String>> {
    length_count(
        U::<_, nom::error::Error<_>>.lift(), //
        quoted_str.lift(),
    )
}

fn dimensions<'a>(version: u32) -> impl NomParser<'a, Dimension> {
    (
        U,
        cond(version < 31, P(space1, U)),
        cond(version >= 22, P(space1, U)),
    )
        .map(|(side_length, _duplicate_side_length, uint1)| Dimension { side_length, uint1 })
}

/// "k" followed by 23-24 numbers
fn slot_k<'a>(strings: &[String]) -> impl NomParser<'a, SlotK> {
    (
        tag("k"),
        P(space1, separated_array::<2, _, _, _>(space1, U)),
        P(space1, separated_array::<4, _, _, _>(space1, U)),
        P(space1, separated_array::<8, _, _, _>(space1, U)),
        P(space1, separated_array::<4, _, _, _>(space1, U)),
        P(space1, separated_array::<4, _, _, _>(space1, I)),
        P(space1, U),
        opt(P(C(' '), U)),
    )
        .map(
            |(
                _letter,
                grid_dims,
                edges,
                exits,
                corner_grounds,
                corner_heights,
                slot_tag,
                origin_dir,
            )| {
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

                SlotK {
                    width: grid_dims[0],
                    height: grid_dims[1],
                    edges,
                    corners,
                    slot_tag: (slot_tag > 0).then(|| strings[slot_tag as usize - 1].clone()),
                    origin: Direction::diagonals()[origin_dir.unwrap_or(0) as usize],
                }
            },
        )
}

/// k, f, s, o, n slots
fn slot<'a>(strings: &[String]) -> impl NomParser<'a, Slot> {
    alt((
        C('n').map(|_| Slot::N),
        C('s').map(|_| Slot::S),
        C('o').map(|_| Slot::O),
        P(tag("f "), U).map(|i| Slot::F {
            fill: (i > 0).then(|| strings[i as usize - 1].clone()),
        }),
        slot_k(strings).map(Slot::K),
    ))
}

/// Grid of Slots
fn grid<'a>(
    height: usize,
    width: usize,
    strings: &'a [String],
) -> impl SliceParser<'a, &'a str, Vec<Vec<Slot>>> {
    let line_parser = verify(
        separated_list1(C(' '), slot(strings)),
        move |row: &Vec<_>| {
            let pass = row.len() == width;
            if !pass {
                println!("grid fail");
            }
            pass
        },
    );

    count(line_parser.lift(), height)
}

/// Single PoI line - 3 uints & a string
fn poi<'a>() -> impl NomParser<'a, PoI> {
    (
        U,
        P(space1, U),
        P(space1, F),
        // TODO: Remove \u0000 chars from this
        P(space1, quoted_str),
    )
        .map(|(x, y, rotation, tag)| PoI {
            x,
            y,
            rotation,
            tag,
        })
}

fn poi_groups<'a>(version: u32) -> impl SliceParser<'a, &'a str, Vec<Vec<PoI>>> {
    // Counts determined by version, manually curated
    let group_count = match version {
        ..20 => 9,
        20..26 => 10,
        26..29 => 5,
        29.. => 6,
    };

    count(group::<32, _>(version, poi()), group_count)
}

/// Single doodad string
fn doodad<'a>(version: u32) -> impl NomParser<'a, Doodad> {
    (
        U,
        P(space1, U),
        cond(
            version >= 34,
            P(space1, length_count(U, (P(space1, F), P(space1, F)))),
        ),
        P(space1, F),
        cond(
            version >= 18,
            P(space1, separated_array::<4, _, _, _>(space1, F)),
        ),
        P(space1, parse_bool),
        cond(version >= 25, P(space1, parse_bool)),
        P(space1, length_count(U, P(space1, F))),
        P(space1, F),
        P(space1, quoted_str),
        P(space1, quoted_str),
        cond(
            version >= 36,
            P(
                space1,
                length_count(
                    U,
                    P(space1, separated_pair(is_not("="), C('='), is_not(" "))),
                ),
            ),
        ),
    )
        .map(
            |(
                x,
                y,
                float_pairs,
                radians1,
                trigs,
                bool1,
                bool2,
                floats,
                scale,
                ao_file,
                stub,
                key_values,
            )| {
                let [trig1, trig2, trig3, trig4] = if let Some(trigs) = trigs {
                    trigs.map(Some)
                } else {
                    [None; _]
                };

                let key_values = key_values.map(|key_values| {
                    key_values
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect()
                });

                Doodad {
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
                }
            },
        )
}

fn doodad_connections<'a>(version: u32) -> impl SliceParser<'a, &'a str, Vec<DoodadConnection>> {
    let line_parser = (
        U, //
        P(space1, U),
        P(space1, quoted_str),
    )
        .map(|(from, to, tag)| DoodadConnection { from, to, tag });

    group::<32, _>(version, line_parser)
}

/// Decale on a line
fn decal<'a>(version: u32) -> impl NomParser<'a, Decal> {
    (
        separated_array::<3, _, _, _>(space1, F),
        cond(version >= 17, P(space1, parse_bool)),
        P(space1, F),
        P(space1, quoted_str),
        P(space1, quoted_str),
    )
        .map(|([x, y, rotation], bool1, scale, atlas_file, tag)| Decal {
            x,
            y,
            rotation,
            bool1,
            scale,
            atlas_file,
            tag,
        })
}

fn boss_lines2<'a>() -> impl SliceParser<'a, &'a str, Vec<Vec<String>>> {
    // NOTE: Boss string line is often not properly ended with a newline, causing the next
    // line to be appended on the end instead. We're accounting for this by cropping off
    // the end and prepending it to our list of lines

    |lines| {
        let line_parser = separated_pair(separated_list1(space1, quoted_str), space1, rest);

        let mut parser = length_prefixed2(line_parser).map(|items| {
            let (boss_lines, mut trailing) =
                items
                    .into_iter()
                    .fold((vec![], vec![]), |(mut blines, mut tlines), (b, t)| {
                        blines.push(b);
                        tlines.push(t);

                        (blines, tlines)
                    });

            let trailing = trailing.pop().filter(|t| !t.is_empty());

            (boss_lines, trailing)
        });

        let (lines, (boss_lines, extra)) = parser.parse_complete(lines)?;

        let lines = if let Some(extra) = extra {
            let mut lines = lines.to_vec();
            lines.insert(0, extra);

            // TODO: Figure out if we can avoid this leak
            let lines = Box::new(lines).leak();
            Slice(&*lines)
        } else {
            lines
        };

        Ok((lines, boss_lines))
    }
}

fn zone<'a>(version: u32) -> impl NomParser<'a, Zone> {
    (
        move |line| match version {
            ..35 => unquoted_str(line),
            35.. => quoted_str(line),
        },
        P(space1, separated_array::<4, _, _, _>(space1, I)),
        cond(
            version >= 35,
            (
                P(space1, quoted_str), //
                P(space1, quoted_str),
                P(space1, U),
            ),
        ),
    )
        .map(|(name, [x_min, y_min, x_max, y_max], optionals)| {
            let (disable_teleports, env_file, uint1) = if let Some((d, e, u)) = optionals {
                (Some(d), Some(e), Some(u))
            } else {
                (None, None, None)
            };

            Zone {
                name,
                x_min,
                y_min,
                x_max,
                y_max,
                disable_teleports,
                env_file,
                uint1,
            }
        })
}

fn tags<'a>() -> impl NomParser<'a, Vec<String>> {
    length_count(U, P(space1, unquoted_str))
}

/// Line of space separated uints, sometimes with a space a the end
fn ground_overrides<'a>(
    strings: &[String],
    grid_height: usize,
    grid_width: usize,
) -> impl NomParser<'a, Vec<Vec<Option<String>>>> {
    T(
        verify(separated_list1(space1, U), move |items: &Vec<_>| {
            items.len() == (grid_height - 1) * (grid_width - 1)
        }),
        space0,
    )
    .map(move |indices| {
        // Chunk into 2D grid
        indices
            .into_iter()
            .chunks(grid_width - 1)
            .into_iter()
            .map(|c| {
                c.map(|i| (i > 0).then(|| strings[i as usize - 1].clone()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    })
}

fn thingy<'a>(strings: &[String]) -> impl NomParser<'a, Thingy> {
    (
        U, //
        P(space1, I),
        count(opt(P(space1, parse_bool)), 3),
    )
        .map(|(i, int, bools)| {
            let et_file = (i > 0).then(|| strings[i as usize - 1].clone());

            Thingy {
                et_file,
                int,
                bool1: bools[0],
                bool2: bools[1],
                bool3: bools[2],
            }
        })
}

pub fn parse_arm_str(input: &str) -> Result<ARMFile> {
    let lines = input.lines().filter(|l| !l.is_empty()).collect::<Vec<_>>();
    let lines = Slice(lines.as_slice());

    let (lines, (version, strings)) = (
        version_line().lift(), //
        string_section(),
    )
        .parse_complete(lines)
        .map_err(|e| anyhow::anyhow!("Failed to parse file: {:?}", e))?;

    let (lines, (dimensions, numbers1, tag1, bools, root_slot)) = (
        dimensions(version).lift(),
        separated_list1(C::<_, nom::error::Error<_>>(' '), U).lift(),
        quoted_str.lift(),
        separated_list1(space1, parse_bool).lift(),
        slot(&strings).lift(),
    )
        .parse_complete(lines)
        .map_err(|e| anyhow::anyhow!("Failed to parse file: {:?}", e))?;

    let (grid_height, grid_width) = if let Slot::K(slot) = &root_slot {
        (slot.height as usize, slot.width as usize)
    } else {
        (1, 1)
    };

    let parser = (
        count(
            thingy(&strings).lift(),
            numbers1.iter().sum::<u32>() as usize * 2,
        ),
        poi_groups(version),
        cond(version >= 35, quoted_str.lift()),
        grid(grid_height, grid_width, &strings),
        group::<32, _>(version, doodad(version)),
        cond(version >= 23, doodad_connections(version)),
        group::<32, _>(version, decal(version)),
        cond(version >= 22, boss_lines2()),
        cond(version >= 27, group::<33, _>(version, zone(version))),
        opt(tags().lift()),
        opt(ground_overrides(&strings, grid_height, grid_width).lift()),
    );

    let (
        _,
        (
            thingies,
            poi_groups,
            string1,
            grid,
            doodads,
            doodad_connections,
            decals,
            boss_lines,
            zones,
            tags,
            ground_overrides,
        ),
    ) = all_consuming(parser)
        .parse_complete(lines)
        .map_err(|e| anyhow::anyhow!("Failed to parse file: {:?}", e))?;

    let arm_file = ARMFile {
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
        ground_overrides,
    };

    Ok(arm_file)
}
