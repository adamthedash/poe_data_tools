use std::path::Path;

use anyhow::Context;
use poe_data_tools::{
    file_parsers::{FileParser2, psg::PSGParser},
    fs::{FS, FileSystem},
};

fn main() {
    // (I don't have a steam install, but it's the same stucture as the CDN cache, so I'm just using
    // that)
    let steam_folder =
        Path::new("/home/adam/.cache/poe_data_tools/patch-poe2.poecdn.com/4.3.1.2").to_owned();
    let fs = FS::from_steam(steam_folder)
        .context("cound't create file system")
        .unwrap();

    // Print out a summary of all the Passive Skill Graph (.psg) files
    let tree_files = fs
        .list()
        .filter(|f| f.ends_with(".psg"))
        .collect::<Vec<_>>();
    for (file, bytes) in fs.batch_read(&tree_files) {
        let Ok(bytes) = bytes else {
            eprintln!("Error reading {file:?}");
            continue;
        };

        let parser = PSGParser { version: 2 };
        let Ok(tree) = parser.parse(&bytes) else {
            eprintln!("Error parsing tree {file:?}");
            continue;
        };

        println!("=== {file} ===");
        println!("Starting points: {}", tree.root_passives.len());
        println!("Clusters: {}", tree.groups.len());
        println!(
            "Total passive skills: {}",
            tree.groups.iter().map(|g| g.passives.len()).sum::<usize>()
        );
    }
}
