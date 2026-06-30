use std::path::Path;

use poe_data_tools::{
    file_parsers::{FileParser2, psg::PSGParser},
    fs::{FS, FileSystem, cdn::cdn_base_url},
};

fn main() {
    env_logger::init();

    let cache_dir = Path::new("./cache");
    let base_url = cdn_base_url(cache_dir, "2").expect("couldn't get CDN URL");
    let fs = FS::from_cdn(&base_url, cache_dir).expect("couldn't create filesystem");

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
