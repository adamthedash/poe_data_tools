use std::{fs, path::Path};

use anyhow::{anyhow, ensure, Context, Result};
use glob::{MatchOptions, Pattern};

use crate::{
    arm::{parser::parse_map_str, types::Map},
    bundle_fs::FS,
};

// fn parse_map_str(input: &str) -> IResult<&str, Map> {
//     let lines = input.lines().filter(|l| !l.is_empty()).collect::<Vec<_>>();
//
//     let (lines, version) = version_line(&lines)?;
//
//     let (lines, strings) = string_section(lines)?;
//
//     // ==============================================================
//
//     let mut lines = input.lines().filter(|l| !l.is_empty()).peekable();
//
//     let line = lines.next().unwrap();
//     let (_, dimensions) = separated_list1(complete::char(' '), complete::u32)(line)?;
//
//     let line = lines.next().unwrap();
//     let (_, numbers) = separated_list1(complete::char(' '), complete::u32)(line)?;
//
//     let line = lines.next().unwrap();
//     let (_, tag1) = quoted_str(line)?;
//
//     let line = lines.next().unwrap();
//     let (input, numbers2) = separated_list1(complete::char(' '), complete::u32)(line)?;
//
//     let line = lines.next().unwrap();
//     let (_, root_slot) = parse_slot(line, &strings)?;
//     let Slot::K(root_slot) = root_slot else {
//         // TODO: Check this assumption
//         panic!("Root slot shoulk be K type");
//     };
//
//     let numbers3 = lines
//         .by_ref()
//         .take(numbers.iter().sum::<u32>() as usize * 2)
//         .map(separated_list1(complete::char(' '), complete::u32))
//         // TODO: [u32; 2]? testingdoodads file has 3 elements per line
//         .map_ok(|(_, s)| s)
//         .collect::<Result<Vec<_>, _>>()?;
//
//     // TODO: # of PoI groups seems to be always 6?
//     //      Doesn't seem the case: metadata/terrain/ruinedcity/sewers/rooms/unique/shrine_dry_02.arm
//     //      Maybe 6 + sum(numbers) * 2? Nope
//     //      Maybe related to numbers2? "0 1" -> 6, "1 1" -> 10, "1 0" -> 10, "0 1" -> 9 (version 16)
//     //      Does index of group mean something specific?
//     //
//     let poi_groups = (0..6)
//         .map(|_| -> IResult<&str, Vec<String>> {
//             let line = lines.next().unwrap();
//             let (_, num_pois) = complete::u32(line)?;
//
//             // TODO: Interpret these lines
//             let poi_lines = lines
//                 .by_ref()
//                 .take(num_pois as usize)
//                 .map(String::from)
//                 .collect::<Vec<_>>();
//
//             Ok(("", poi_lines))
//         })
//         .map_ok(|(_, s)| s)
//         .collect::<Result<Vec<_>, _>>()?;
//
//     // // TODO: Figure out what determines the # of PoI groups
//     // let line = lines.next().unwrap();
//     // let (_, num_pois) = complete::u32(line)?;
//     //
//     // // TODO: Interpret these lines
//     // let poi_lines = lines
//     //     .by_ref()
//     //     .take(num_pois as usize)
//     //     .map(String::from)
//     //     .collect::<Vec<_>>();
//     //
//     // let line = lines.next().unwrap();
//     // let (_, num_pois) = complete::u32(line)?;
//     //
//     // // TODO: Interpret these lines
//     // let poi_lines2 = lines
//     //     .by_ref()
//     //     .take(num_pois as usize)
//     //     .map(String::from)
//     //     .collect::<Vec<_>>();
//     //
//     // // Skip rest of "0" lines. TODO: Make sure this is really how things work
//     // let zeros = lines
//     //     .peeking_take_while(|line| *line == "0")
//     //     .map(|_| vec![])
//     //     .collect::<Vec<_>>();
//
//     // Grid
//     let grid = lines
//         .by_ref()
//         .take(root_slot.height as usize)
//         .map(separated_list1(complete::char(' '), |l| {
//             parse_slot(l, &strings)
//         }))
//         .map_ok(|(_, s)| s)
//         .collect::<Result<Vec<_>, _>>()?;
//
//     assert_eq!(grid.len(), root_slot.height as usize);
//     for row in &grid {
//         assert_eq!(row.len(), root_slot.width as usize);
//     }
//
//     // Doodads
//     let line = lines.next().unwrap();
//     let (_, num_doodads) = complete::u32(line)?;
//
//     // TODO: Interpret these lines
//     let doodad_lines = lines
//         .by_ref()
//         .take(num_doodads as usize)
//         .map(String::from)
//         .collect::<Vec<_>>();
//
//     let map = Map {
//         version,
//         strings,
//         dimensions,
//         numbers,
//         tag: tag1,
//         numbers2,
//         root_slot,
//         numbers3,
//         points_of_interest: vec![],
//         grid,
//         doodads: vec![],
//     };
//
//     Ok((input, map))
// }

fn parse_map(contents: &[u8]) -> Result<Map> {
    ensure!(contents[..2] == [0xff, 0xfe], ".arm magic number mismatch");

    let input =
        String::from_utf16le(contents).context("Failed to parse contents as UTF16LE string")?;

    let (remaining, map) =
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
