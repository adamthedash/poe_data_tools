use std::{fs, path::Path};

use anyhow::{anyhow, ensure, Context, Result};
use glob::{MatchOptions, Pattern};
use itertools::{izip, Itertools};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{self},
    multi::separated_list1,
    sequence::{delimited, preceded, terminated},
    IResult, Parser,
};
use serde::Serialize;

use crate::bundle_fs::FS;

#[derive(Debug, Serialize)]
struct Map {
    version: u32,
    strings: Vec<String>,
    /// Have seen either 2 or 3 elements here
    dimensions: Vec<u32>,
    numbers: Vec<u32>,
    tag: String,
    numbers2: Vec<u32>,
    root_slot: SlotK,
    numbers3: Vec<Vec<u32>>,
    points_of_interest: Vec<Vec<String>>,
    grid: Vec<Vec<Slot>>,
    doodads: Vec<String>,
}

/// Quoted string ending in newline
fn quoted_str(input: &str) -> IResult<&str, String> {
    delimited(complete::char('"'), take_until("\""), complete::char('"'))
        .map(String::from)
        .parse(input)
}

/// Letter + 24 chars
fn letter_numbers(input: &str) -> IResult<&str, (char, Vec<u32>)> {
    let (input, letter) = terminated(complete::char('k'), complete::char(' '))(input)?;
    let (input, numbers) = separated_list1(complete::char(' '), complete::u32)(input)?;

    Ok((input, (letter, numbers)))
}

#[derive(Debug, Clone, Copy, Serialize)]
enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Direction {
    fn diagonals() -> [Self; 4] {
        use Direction::*;
        [SW, SE, NE, NW]
    }

    fn cardinal() -> [Self; 4] {
        use Direction::*;
        [N, W, S, E]
    }
}

#[derive(Debug, Serialize)]
struct Edge {
    direction: Direction,
    edge: String,
    exit: u32,
    virtual_exit: u32,
}

#[derive(Debug, Serialize)]
struct Corner {
    direction: Direction,
    ground: String,
    height: u32,
}

#[derive(Debug, Serialize)]
struct SlotK {
    height: u32,
    width: u32,
    edges: [Edge; 4],
    corners: [Corner; 4],
    slot_tag: String,
    origin: Direction,
}

#[derive(Debug, Serialize)]
enum Slot {
    K(SlotK),
    N,
    // TODO: Thought this was a string index, but doesn't look like it
    F { fill: u32 },
    S,
}

fn parse_slot<'a>(input: &'a str, strings: &[String]) -> IResult<&'a str, Slot> {
    alt((
        complete::char('n').map(|_| Slot::N),
        complete::char('s').map(|_| Slot::S),
        preceded(tag("f "), complete::u32).map(|i| Slot::F { fill: i }),
        letter_numbers.map(|(_, nums)| {
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

fn parse_map_str(input: &str) -> IResult<&str, Map> {
    let mut lines = input.lines().filter(|l| !l.is_empty()).peekable();

    // Magic version
    // TODO: Propogate error up
    let line = lines.next().unwrap();
    let (_, version) = preceded(tag("\u{feff}version "), complete::u32)(line)?;

    let line = lines.next().unwrap();
    let (_, num_strings) = complete::u32(line)?;

    let strings = lines
        .by_ref()
        .take(num_strings as usize)
        .map(quoted_str)
        .map_ok(|(_, s)| s)
        .collect::<Result<Vec<_>, _>>()?;

    //TODO: ensure!(strings.len() == num_strings);
    assert_eq!(strings.len(), num_strings as usize);

    let line = lines.next().unwrap();
    let (_, dimensions) = separated_list1(complete::char(' '), complete::u32)(line)?;

    let line = lines.next().unwrap();
    let (_, numbers) = separated_list1(complete::char(' '), complete::u32)(line)?;

    let line = lines.next().unwrap();
    let (_, tag1) = quoted_str(line)?;

    let line = lines.next().unwrap();
    let (input, numbers2) = separated_list1(complete::char(' '), complete::u32)(line)?;

    let line = lines.next().unwrap();
    let (_, root_slot) = parse_slot(line, &strings)?;
    let Slot::K(root_slot) = root_slot else {
        // TODO: Check this assumption
        panic!("Root slot shoulk be K type");
    };

    let numbers3 = lines
        .by_ref()
        .take(numbers.iter().sum::<u32>() as usize * 2)
        .map(separated_list1(complete::char(' '), complete::u32))
        // TODO: [u32; 2]? testingdoodads file has 3 elements per line
        .map_ok(|(_, s)| s)
        .collect::<Result<Vec<_>, _>>()?;

    // TODO: # of PoI groups seems to be always 6?
    //      Doesn't seem the case: metadata/terrain/ruinedcity/sewers/rooms/unique/shrine_dry_02.arm
    //      Maybe 6 + sum(numbers) * 2? Nope
    //      Maybe related to numbers2? "0 1" -> 6, "1 1" -> 10, "1 0" -> 10, "0 1" -> 9 (version 16)
    //      Does index of group mean something specific?
    //
    let poi_groups = (0..6)
        .map(|_| -> IResult<&str, Vec<String>> {
            let line = lines.next().unwrap();
            let (_, num_pois) = complete::u32(line)?;

            // TODO: Interpret these lines
            let poi_lines = lines
                .by_ref()
                .take(num_pois as usize)
                .map(String::from)
                .collect::<Vec<_>>();

            Ok(("", poi_lines))
        })
        .map_ok(|(_, s)| s)
        .collect::<Result<Vec<_>, _>>()?;

    // // TODO: Figure out what determines the # of PoI groups
    // let line = lines.next().unwrap();
    // let (_, num_pois) = complete::u32(line)?;
    //
    // // TODO: Interpret these lines
    // let poi_lines = lines
    //     .by_ref()
    //     .take(num_pois as usize)
    //     .map(String::from)
    //     .collect::<Vec<_>>();
    //
    // let line = lines.next().unwrap();
    // let (_, num_pois) = complete::u32(line)?;
    //
    // // TODO: Interpret these lines
    // let poi_lines2 = lines
    //     .by_ref()
    //     .take(num_pois as usize)
    //     .map(String::from)
    //     .collect::<Vec<_>>();
    //
    // // Skip rest of "0" lines. TODO: Make sure this is really how things work
    // let zeros = lines
    //     .peeking_take_while(|line| *line == "0")
    //     .map(|_| vec![])
    //     .collect::<Vec<_>>();

    // Grid
    let grid = lines
        .by_ref()
        .take(root_slot.height as usize)
        .map(separated_list1(complete::char(' '), |l| {
            parse_slot(l, &strings)
        }))
        .map_ok(|(_, s)| s)
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(grid.len(), root_slot.height as usize);
    for row in &grid {
        assert_eq!(row.len(), root_slot.width as usize);
    }

    // Doodads
    let line = lines.next().unwrap();
    let (_, num_doodads) = complete::u32(line)?;

    // TODO: Interpret these lines
    let doodad_lines = lines
        .by_ref()
        .take(num_doodads as usize)
        .map(String::from)
        .collect::<Vec<_>>();

    let map = Map {
        version,
        strings,
        dimensions,
        numbers,
        tag: tag1,
        numbers2,
        root_slot,
        numbers3,
        points_of_interest: poi_groups,
        grid,
        doodads: doodad_lines,
    };

    Ok((input, map))
}

fn parse_map(contents: &[u8]) -> Result<Map> {
    ensure!(contents[..2] == [0xff, 0xfe], ".arm magic number mismatch");

    let input =
        String::from_utf16le(contents).context("Failed to parse contents as UTF16LE string")?;

    let (_, map) =
        parse_map_str(&input).map_err(|e| anyhow!("Failed to parse map file: {:?}", e))?;
    println!("{:#?}", map);

    Ok(map)
}

/// Extract files to disk matching a glob pattern
pub fn dump_maps(fs: &mut FS, patterns: &[Pattern], output_folder: &Path) -> Result<()> {
    for pattern in patterns {
        ensure!(
            pattern.as_str().ends_with(".arm"),
            "Only .arm map export is supported."
        );
    }

    let filenames = fs
        .list()
        .filter(|filename| {
            patterns.iter().any(|pattern| {
                pattern.matches_with(
                    filename,
                    MatchOptions {
                        require_literal_separator: true,
                        ..Default::default()
                    },
                )
            })
        })
        .collect::<Vec<_>>();
    let filenames = filenames.iter().map(|f| f.as_str()).collect::<Vec<_>>();

    fs.batch_read(&filenames)
        // Print and filter out errors
        .filter_map(|f| match f {
            Ok(x) => Some(x),
            Err((path, e)) => {
                eprintln!("Failed to extract file: {:?}: {:?}", path, e);
                None
            }
        })
        // Attempt to read file contents
        .map(|(filename, contents)| -> Result<_, anyhow::Error> {
            println!("extracting: {:?}", filename);
            let map = parse_map(&contents)
                .with_context(|| format!("Failed to parse file: {:?}", filename))
                .unwrap();

            let out_path = output_folder.join(filename).with_extension("json");
            fs::create_dir_all(out_path.parent().unwrap()).context("Failed to create folder")?;

            let f = std::fs::File::create(&out_path)
                .with_context(|| format!("Failed to create file {:?}", out_path))?;
            serde_json::to_writer_pretty(f, &map).context("Failed to serialise map")?;

            Ok(filename)
        })
        // Report results
        .for_each(|result| match result {
            Ok(filename) => eprintln!("Extracted file: {}", filename),
            Err(e) => eprintln!("Failed to extract file: {:?}", e),
        });

    Ok(())
}
