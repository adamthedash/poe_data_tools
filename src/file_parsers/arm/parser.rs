use itertools::{Itertools, izip};
use nom::{
    Parser,
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{char as C, i32 as I, space0, space1, u32 as U},
    combinator::{all_consuming, cond, opt, rest, verify},
    multi::{count, length_count, separated_list1},
    number::complete::float as F,
    sequence::{preceded as P, separated_pair, terminated as T},
};

use super::types::*;
use crate::file_parsers::{
    FileParser,
    line_parser::{
        MultilineParser, NomParser, Result as LResult, length_prefixed, nom_adapter, optional,
        repeated, single_line, terminated,
    },
    shared::{
        parse_bool, quoted_str, separated_array, unquoted_str, utf16_bom_to_string, version_line,
    },
};

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
    let parser = (
        U,
        cond(version < 31, P(space1, U)),
        cond(version >= 22, P(space1, U)),
    )
        .map(|(side_length, _duplicate_side_length, uint1)| Dimension { side_length, uint1 });

    single_line(nom_adapter(parser))
}

/// Space-separated uinsigned ints
fn uints<'a>() -> impl MultilineParser<'a, Vec<u32>> {
    single_line(nom_adapter(separated_list1(C(' '), U)))
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

fn root_slot<'a>(strings: &'a [String]) -> impl MultilineParser<'a, Slot> {
    single_line(nom_adapter(slot(strings)))
}

/// Grid of Slots
fn grid<'a>(
    height: usize,
    width: usize,
    strings: &'a [String],
) -> impl MultilineParser<'a, Vec<Vec<Slot>>> {
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

    let row_parser = single_line(nom_adapter(line_parser));

    repeated(row_parser, height)
}

/// Single PoI line - 3 uints & a string
fn poi<'a>() -> impl MultilineParser<'a, PoI> {
    let line_parser = (
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

    let group_parser = group(version, poi());

    repeated(group_parser, count)
}

/// Single doodad string
fn doodad<'a>(version: u32) -> impl MultilineParser<'a, Doodad> {
    let line_parser = (
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
        );

    single_line(nom_adapter(line_parser))
}

fn doodad_connections<'a>(version: u32) -> impl MultilineParser<'a, Vec<DoodadConnection>> {
    let line_parser = (
        U, //
        P(space1, U),
        P(space1, quoted_str),
    )
        .map(|(from, to, tag)| DoodadConnection { from, to, tag });

    let connection_parser = single_line(nom_adapter(line_parser));

    group(version, connection_parser)
}

/// Decale on a line
fn decal<'a>(version: u32) -> impl MultilineParser<'a, Decal> {
    let line_parser = (
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
        });

    single_line(nom_adapter(line_parser))
}

fn boss_lines<'a>(lines: &'a [&'a str]) -> LResult<(Vec<&'a str>, Vec<Vec<String>>)> {
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
    let line_parser = (
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
        });

    single_line(nom_adapter(line_parser))
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
    let mut line_parser = opt(all_consuming(length_count(U, P(space1, unquoted_str))));

    move |lines| {
        if let Some(&first) = lines.first() {
            let (_, tags) = line_parser.parse_complete(first)?;

            // Crop off last line if successful
            let lines = if tags.is_some() { &lines[1..] } else { lines };

            Ok((lines, tags))
        } else {
            Ok((lines, None))
        }
    }
}

/// Line of space separated uints, sometimes with a space a the end
fn ground_overrides<'a>(
    strings: &[String],
    grid_height: usize,
    grid_width: usize,
) -> impl MultilineParser<'a, Option<Vec<Vec<Option<String>>>>> {
    let line_parser = T(
        verify(separated_list1(space1, U), move |items: &Vec<_>| {
            items.len() == (grid_height - 1) * (grid_width - 1)
        }),
        space0,
    );

    let line_parser = line_parser.map(move |indices| {
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
    });

    optional(single_line(nom_adapter(line_parser)))
}

fn thingy<'a>(strings: &[String]) -> impl MultilineParser<'a, Thingy> {
    let line_parser = (
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
        });

    single_line(nom_adapter(line_parser))
}

fn parse_arm_str(input: &str) -> LResult<Arm> {
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
    let (lines, ground_overrides) = ground_overrides(&strings, grid_height, grid_width)(lines)?;

    assert!(lines.is_empty(), "Extra lines: {:#?}", lines);

    let map = Arm {
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

    Ok(map)
}

pub struct ARMParser;

impl FileParser for ARMParser {
    type Output = Arm;

    fn parse(&self, bytes: &[u8]) -> anyhow::Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        let arm = parse_arm_str(&contents)
            .map_err(|e| anyhow::anyhow!("Failed to parse file: {:?}", e))?;

        Ok(arm)
    }
}
