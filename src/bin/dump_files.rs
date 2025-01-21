#![feature(hash_raw_entry)]

use clap::Parser;
use poe_game_data_parser::bundle::fetch_bundle_content;
use poe_game_data_parser::bundle_index::{fetch_index_file, BundleIndex};
use poe_game_data_parser::{
    bundle::load_bundle_content, bundle_index::load_index_file, hasher::BuildMurmurHash64A,
};
use std::hash::{BuildHasher, Hasher};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufRead, Write},
    path::PathBuf,
};

/// A simple CLI tool that extracts virtual files from the PoE data files.
/// Filenames are read from stdin, one per line.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the Path of Exile folder for steam - if not provided, will fetch from the CDN
    #[arg(short, long)]
    steam_folder: Option<PathBuf>,
    /// Version of the game to use: 1 for PoE 1, 2 for PoE 2, or a specific CDN patch version
    #[arg(short, long, default_value = "1")]
    patch: String,
    /// The path to the dir to store the local CDN cache
    #[arg(short, long, default_value=dirs::cache_dir().unwrap().join("poe_data_tools").into_os_string())]
    cache_dir: PathBuf,
    /// The folder to which to output extracted files
    #[arg(short, long)]
    output_folder: PathBuf,
}

fn main() {
    let args = Cli::parse();

    // Load up index file
    let index: BundleIndex;
    if let Some(steam_folder) = &args.steam_folder {
        let index_path = steam_folder.as_path().join("Bundles2/_.index.bin");
        index = load_index_file(index_path.as_ref());
    } else {
        index = fetch_index_file(
            args.patch.as_str(),
            args.cache_dir.as_ref(),
            PathBuf::from("Bundles2/_.index.bin").as_ref(),
        );
    }

    // Effficient LUT for filenames -> bundle
    let file_lut = index
        .files
        .iter()
        .map(|f| (f.hash, f))
        .collect::<HashMap<_, _>>();

    // Output folder
    let hash_builder = BuildMurmurHash64A { seed: 0x1337b33f };

    // Process input filenames, grouping them into their corresponding bundles
    let mut files = HashMap::new();
    io::stdin().lock().lines().for_each(|l| {
        // Look up the filename
        let filename = l.expect("Failed to read line from stdin.");
        let mut hasher = hash_builder.build_hasher();
        hasher.write(filename.to_lowercase().as_bytes());
        let hash = hasher.finish();

        let file = *file_lut
            .get(&hash)
            .unwrap_or_else(|| panic!("File not found: {}", filename));

        files
            .entry(file.bundle_index)
            .or_insert_with(Vec::new)
            .push((filename, file));
    });

    // Pull out the data
    files.iter().for_each(|(&bundle_index, bundle_files)| {
        // Load up bundle file
        let bundle: Vec<u8>;
        if let Some(steam_folder) = &args.steam_folder {
            let bundle_path = steam_folder.as_path().join(format!(
                "Bundles2/{}.bundle.bin",
                index.bundles[bundle_index as usize].name
            ));
            bundle = load_bundle_content(bundle_path.as_ref());
        } else {
            bundle = fetch_bundle_content(
                args.patch.as_str(),
                args.cache_dir.as_ref(),
                PathBuf::from(format!(
                    "Bundles2/{}.bundle.bin",
                    index.bundles[bundle_index as usize].name
                ))
                .as_ref(),
            );
        }

        // Extract all the files we want
        bundle_files.iter().for_each(|(filename, file)| {
            println!("Extracting: {}", filename);

            // Pull out the file's contents
            let file_contents =
                &bundle[file.offset as usize..file.offset as usize + file.size as usize];

            // Dump it to disk
            let out_filename = args.output_folder.as_path().join(filename);
            fs::create_dir_all(out_filename.parent().unwrap()).expect("Failed to create folder");

            let mut out_file = File::create(out_filename).expect("Failed to create file.");
            out_file
                .write_all(file_contents)
                .expect("Failed to write to file.");
        });
    });
}
