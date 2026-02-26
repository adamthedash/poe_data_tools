use std::{collections::HashMap, path::Path};

use itertools::Itertools;
use poe_data_tools::{
    bundle_fs::FS,
    bundle_loader::cdn_base_url,
    commands::{
        Patch,
        translate::{FileParserExt, Parser},
    },
};

fn bench_version(version: Patch) {
    // Set up file system
    let cache_dir = dirs::cache_dir().unwrap().join("poe_data_tools");
    let base_url = cdn_base_url(&cache_dir, version.as_str()).expect("Failed to get CDN URL");
    let fs = FS::from_cdn(&base_url, &cache_dir).expect("Failed to create filesystem");

    let filenames = fs
        .list()
        // Filter out files that we can't parse
        .filter(|filename| Parser::from_filename(Path::new(filename), version.major()).is_some())
        .collect::<Vec<_>>();

    let results = fs
        .batch_read(&filenames)
        .filter_map(Result::ok)
        .map(|(filename, contents)| {
            let parser = Parser::from_filename(Path::new(filename), version.major())
                .expect("Already validated above");

            let success = parser.validate(&contents);

            let ext = Path::new(filename).extension().unwrap().to_str().unwrap();

            (ext, success)
        })
        .fold(HashMap::new(), |mut hm, (ext, success)| {
            let counts = if let Some(counts) = hm.get_mut(ext) {
                counts
            } else {
                hm.entry(ext.to_string()).or_insert([0_usize; 2])
            };

            counts[success as usize] += 1;

            hm
        });

    // Print game & lib version
    let patch_version = base_url.path().trim_matches('/');

    println!(
        "PoE {} (patch {}), poe_data_tools v{}",
        version.major(),
        patch_version,
        env!("CARGO_PKG_VERSION")
    );

    // Print to markdown table
    println!("|Format|Fail|Success|Total|Success %|");
    println!("|-|-|-|-|-|");
    results
        .iter()
        .sorted_unstable_by_key(|(ext, _)| ext.to_string())
        .for_each(|(ext, [fails, successes])| {
            println!(
                "|{ext:<6}|{fails:>6}|{successes:>6}|{:>6}|{:>6.2}%|",
                fails + successes,
                100. * (*successes as f32 / (fails + successes) as f32)
            );
        });
}

fn main() {
    bench_version(Patch::One);
    println!();
    bench_version(Patch::Two);
}
