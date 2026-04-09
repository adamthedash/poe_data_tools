use std::{collections::HashMap, path::Path};

use itertools::Itertools;
use poe_data_tools::{
    commands::{
        Patch,
        translate::{FileParserExt, Parser},
    },
    fs::{FS, FileSystem, cdn::cdn_base_url},
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
            let parser = Parser::from_filename(Path::new(filename.as_ref()), version.major())
                .expect("Already validated above");

            let res = parser.validate(&contents);

            let ext = Path::new(filename.as_ref())
                .extension()
                .unwrap()
                .to_str()
                .unwrap();

            (ext.to_owned(), res)
        })
        .fold(HashMap::new(), |mut hm, (ext, res)| {
            let version_counts = if let Some(version_counts) = hm.get_mut(&ext) {
                version_counts
            } else {
                hm.entry(ext.to_string()).or_insert(HashMap::new())
            };

            let counts = if let Some(counts) = version_counts.get_mut(&res.version) {
                counts
            } else {
                version_counts.entry(res.version).or_insert([0_usize; 2])
            };

            counts[res.inner.is_ok() as usize] += 1;

            hm
        });

    // Print game & lib version
    let patch_version = base_url.path().trim_matches('/');

    println!(
        "PoE {} (patch {}), PoE Data Tools v{}",
        version.major(),
        patch_version,
        env!("CARGO_PKG_VERSION")
    );
    println!();

    // Print to markdown table
    println!("|Format|Fail|Success|Total|Success %|");
    println!("|-|-|-|-|-|");
    results
        .iter()
        .sorted_unstable_by_key(|(ext, _)| ext.to_string())
        .map(|(ext, version_counts)| {
            let counts = version_counts
                .iter()
                .fold([0, 0], |[fails, successes], (_, [f, s])| {
                    [fails + *f, successes + *s]
                });

            (ext, counts)
        })
        .for_each(|(ext, [fails, successes])| {
            println!(
                "|{ext:<6}|{fails:>6}|{successes:>6}|{:>6}|{:>6.2}%|",
                fails + successes,
                100. * (successes as f32 / (fails + successes) as f32)
            );
        });
    println!();

    // Print by version number
    let max_version = results
        .values()
        .flat_map(|versions| versions.keys().flatten().copied().collect::<Vec<_>>())
        .max();

    let mut header = "||Unknown|".to_owned();
    for i in 0..max_version.unwrap() {
        header.push_str(&format!("{i}|"));
    }
    println!("By file format version (fails : successes)");
    println!("{header}");
    println!(
        "|{}",
        "-|".repeat(max_version.unwrap_or_default() as usize + 2)
    );

    results
        .iter()
        .sorted_unstable_by_key(|(ext, _)| ext.to_string())
        .for_each(|(ext, version_counts)| {
            let mut counts = vec!["".to_owned(); max_version.unwrap_or_default() as usize + 2];
            for (version, [fails, successes]) in version_counts {
                let index = if let Some(v) = version {
                    *v as usize + 1
                } else {
                    0
                };

                counts[index] = format!("{fails} : {successes}");
            }

            let mut row = format!("|{ext}|");
            for c in counts {
                row.push_str(&c);
                row.push('|');
            }

            println!("{row}");
        });
}

fn main() {
    bench_version(Patch::One);
    println!();
    bench_version(Patch::Two);
}
