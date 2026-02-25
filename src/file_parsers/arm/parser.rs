use anyhow::Result;
use itertools::{Itertools, izip};
use winnow::{
    Parser,
    ascii::{dec_int as I, dec_uint, float, space0, space1 as S},
    binary::length_repeat,
    combinator::{
        cond, dispatch, empty, fail, opt, preceded as P, repeat, repeat_till, separated,
        separated_pair, terminated as T,
    },
    token::{any, literal, rest, take_while},
};

use super::types::*;
use crate::file_parsers::shared::{
    lift::{SliceParser, lift},
    winnow::{
        TraceHelper, WinnowParser, parse_bool, quoted_str, separated_array, unquoted_str,
        version_line,
    },
};

// ==================================
// Some winnow funcs with explicit types to help type inference down below

#[allow(non_snake_case)]
#[inline(always)]
fn U(input: &mut &str) -> winnow::Result<u32> {
    dec_uint(input)
}

#[allow(non_snake_case)]
#[inline(always)]
fn F(input: &mut &str) -> winnow::Result<f32> {
    float(input)
}

// ==================================

fn length_prefixed<'a, T>(
    item_parser: impl WinnowParser<&'a str, T>,
) -> impl SliceParser<'a, &'a str, Vec<T>> {
    length_repeat(
        lift(U), //
        lift(item_parser),
    )
    .trace("length_prefixed")
}

fn terminated<'a, T>(
    item_parser: impl WinnowParser<&'a str, T>,
    sentinel: &str,
) -> impl SliceParser<'a, &'a str, Vec<T>> {
    repeat_till(
        0.., //
        lift(item_parser),
        lift(literal(sentinel)),
    )
    .map(|(items, _)| items)
    .trace("terminated")
}

/// Either length-prefixed or "-1"-terminated depending on version
fn group<'a, const V: u32, T>(
    version: u32,
    mut item_parser: impl WinnowParser<&'a str, T>,
) -> impl SliceParser<'a, &'a str, Vec<T>> {
    dispatch! {
        empty.value(version);
        v if v < V => length_prefixed(item_parser.by_ref()),
        v if v >= V => terminated(item_parser.by_ref(), "-1"),
        _ => fail,
    }
    .trace("group")
}

fn string_section<'a>() -> impl SliceParser<'a, &'a str, Vec<String>> {
    length_repeat(
        lift(U), //
        lift(quoted_str),
    )
    .trace("string_section")
}

fn dimensions<'a>(version: u32) -> impl WinnowParser<&'a str, Dimension> {
    (U, cond(version < 31, P(S, U)), cond(version >= 22, P(S, U)))
        .map(|(side_length, _duplicate_side_length, uint1)| Dimension { side_length, uint1 })
        .trace("dimensions")
}

/// "k" followed by 23-24 numbers
fn slot_k<'a>(strings: &[String]) -> impl WinnowParser<&'a str, SlotK> {
    (
        separated_array(S, U),
        P(S, separated_array(S, U)),
        P(S, separated_array(S, U)),
        P(S, separated_array(S, U)),
        P(S, separated_array(S, I)),
        P(S, U),
        opt(P(' ', U)),
    )
        .map(
            |(
                grid_dims, //
                edges,
                exits,
                corner_grounds,
                corner_heights,
                slot_tag,
                origin_dir,
            ): ([_; 2], [_; 4], [_; 8], [_; 4], [_; 4], _, _)| {
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
        .trace("slot_k")
}

/// k, f, s, o, n slots
fn slot<'a>(strings: &[String]) -> impl WinnowParser<&'a str, Slot> {
    dispatch! {
        any;
        'n' => empty.value(Slot::N),
        's' => empty.value(Slot::S),
        'o' => empty.value(Slot::O),
        'f' => P(S, U).map(|i | Slot::F {
            fill: (i > 0).then(|| strings[i as usize - 1].clone()),
        }),
        'k' => P(S, slot_k(strings)).map(Slot::K),
        _ => fail,
    }
    .trace("slot")
}

/// Grid of Slots
fn grid<'a>(
    height: usize,
    width: usize,
    strings: &'a [String],
) -> impl SliceParser<'a, &'a str, Vec<Vec<Slot>>> {
    repeat(
        height, //
        lift::<_, _, Vec<_>, _>(
            separated(width, slot(strings), S), //
        )
        .trace("grid_row"),
    )
    .trace("grid")
}

/// Single PoI line - 3 uints & a string
fn poi<'a>() -> impl WinnowParser<&'a str, PoI> {
    (
        U,
        P(S, U),
        P(S, F),
        // TODO: Remove \u0000 chars from this
        P(S, quoted_str),
    )
        .map(|(x, y, rotation, tag)| PoI {
            x,
            y,
            rotation,
            tag,
        })
        .trace("poi")
}

fn poi_groups<'a>(version: u32) -> impl SliceParser<'a, &'a str, Vec<Vec<PoI>>> {
    // Counts determined by version, manually curated
    let group_count = match version {
        ..20 => 9,
        20..26 => 10,
        26..29 => 5,
        29.. => 6,
    };

    repeat(group_count, group::<32, _>(version, poi())).trace("poI_groups")
}

/// key=value
fn key_value<'a>() -> impl WinnowParser<&'a str, (&'a str, &'a str)> {
    separated_pair(
        take_while(1.., |c| c != '='),
        '=',
        take_while(1.., |c| c != ' '),
    )
    .trace("key_value")
}

/// Single doodad string
fn doodad<'a>(version: u32) -> impl WinnowParser<&'a str, Doodad> {
    (
        U,
        P(S, U),
        cond(
            version >= 34,
            P(
                S,
                length_repeat(
                    U, //
                    (P(S, F), P(S, F)),
                ),
            ),
        ),
        P(S, F),
        cond(
            version >= 18, //
            P(S, separated_array(S, F)),
        ),
        P(S, parse_bool),
        cond(
            version >= 25, //
            P(S, parse_bool),
        ),
        P(S, length_repeat(U, P(S, F))),
        P(S, F),
        P(S, quoted_str),
        P(S, quoted_str),
        cond(version >= 36, P(S, length_repeat(U, P(S, key_value())))),
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

                let key_values = key_values.map(|key_values: Vec<_>| {
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
        .trace("doodad")
}

fn doodad_connections<'a>(version: u32) -> impl SliceParser<'a, &'a str, Vec<DoodadConnection>> {
    let line_parser = (
        U, //
        P(S, U),
        P(S, quoted_str),
    )
        .map(|(from, to, tag)| DoodadConnection { from, to, tag });

    group::<32, _>(version, line_parser).trace("doodad_connections")
}

/// Decal on a line
fn decal<'a>(version: u32) -> impl WinnowParser<&'a str, Decal> {
    (
        separated_array::<3, _, _, _, _, _>(S, F),
        cond(version >= 17, P(S, parse_bool)),
        P(S, F),
        P(S, quoted_str),
        P(S, quoted_str),
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
        .trace("decal")
}

fn boss_lines<'a>() -> impl SliceParser<'a, &'a str, Vec<Vec<String>>> {
    // NOTE: Boss string line is often not properly ended with a newline, causing the next
    // line to be appended on the end instead. We're accounting for this by cropping off
    // the end and prepending it to our list of lines

    let parser = |lines: &mut &'a [&'a str]| -> winnow::Result<Vec<Vec<String>>> {
        let line_parser = separated_pair(
            separated(1.., quoted_str, S), //
            S,
            rest,
        );

        let mut parser = length_prefixed(line_parser).map(|items| {
            let (boss_lines, mut trailing) = items.into_iter().fold(
                (vec![], vec![]),
                |(mut blines, mut tlines), (b, t): (Vec<_>, _)| {
                    blines.push(b);
                    tlines.push(t);

                    (blines, tlines)
                },
            );

            let trailing = trailing.pop().filter(|t| !t.is_empty());

            (boss_lines, trailing)
        });

        let (boss_lines, extra) = parser.parse_next(lines)?;

        if let Some(extra) = extra {
            let mut box_lines = lines.to_vec();
            box_lines.insert(0, extra);

            // TODO: Figure out if we can avoid this leak
            let box_lines = Box::new(box_lines).leak();
            *lines = box_lines;
        }

        Ok(boss_lines)
    };

    parser.trace("boss_lines")
}

fn zone<'a>(version: u32) -> impl WinnowParser<&'a str, Zone> {
    (
        dispatch! {
            empty.value(version);
            ..35 => unquoted_str,
            35.. => quoted_str,
        },
        P(S, separated_array::<4, _, _, _, _, _>(S, I)),
        cond(
            version >= 35,
            (
                P(S, quoted_str), //
                P(S, quoted_str),
                P(S, U),
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
        .trace("zone")
}

fn tags<'a>() -> impl WinnowParser<&'a str, Vec<String>> {
    length_repeat(
        U, //
        P(S, unquoted_str),
    )
    .trace("tags")
}

/// Line of space separated uints, sometimes with a space a the end
fn ground_overrides<'a>(
    strings: &[String],
    grid_height: usize,
    grid_width: usize,
) -> impl WinnowParser<&'a str, Vec<Vec<Option<String>>>> {
    T(
        separated((grid_height - 1) * (grid_width - 1), U, S),
        space0,
    )
    .map(move |indices: Vec<_>| {
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
    .trace("ground_overrides")
}

fn thingy<'a>(strings: &[String]) -> impl WinnowParser<&'a str, Thingy> {
    (
        U, //
        P(S, I),
        opt(P(S, parse_bool)),
        opt(P(S, parse_bool)),
        opt(P(S, parse_bool)),
    )
        .map(|(i, int, bool1, bool2, bool3)| {
            let et_file = (i > 0).then(|| strings[i as usize - 1].clone());

            Thingy {
                et_file,
                int,
                bool1,
                bool2,
                bool3,
            }
        })
        .trace("thingy")
}

pub fn parse_arm_str(input: &str) -> Result<ARMFile> {
    let lines = input.lines().filter(|l| !l.is_empty()).collect::<Vec<_>>();
    let mut lines = lines.as_slice();

    let (version, strings) = (
        lift(version_line()), //
        string_section(),
    )
        .parse_next(&mut lines)
        .map_err(|e| anyhow::anyhow!("Failed to parse file: {:?}", e))?;

    let (dimensions, numbers1, tag1, bools, root_slot) = (
        lift(dimensions(version)),
        lift::<_, _, Vec<_>, _>(separated(1.., U, S)),
        lift(quoted_str),
        lift(separated(1.., parse_bool, S)),
        lift(slot(&strings)),
    )
        .parse_next(&mut lines)
        .map_err(|e| anyhow::anyhow!("Failed to parse file: {:?}", e))?;

    let (grid_height, grid_width) = if let Slot::K(slot) = &root_slot {
        (slot.height as usize, slot.width as usize)
    } else {
        (1, 1)
    };

    let mut parser = (
        repeat(
            numbers1.iter().sum::<u32>() as usize * 2,
            lift(thingy(&strings)),
        ),
        poi_groups(version),
        cond(version >= 35, lift(quoted_str)),
        grid(grid_height, grid_width, &strings),
        group::<32, _>(version, doodad(version)),
        cond(version >= 23, doodad_connections(version)),
        group::<32, _>(version, decal(version)),
        cond(version >= 22, boss_lines()),
        cond(version >= 27, group::<33, _>(version, zone(version))),
        opt(lift(tags())),
        opt(lift(ground_overrides(&strings, grid_height, grid_width))),
    );

    let (
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
    ) = parser
        .parse(lines)
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
