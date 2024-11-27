#![feature(hash_raw_entry)]

use clap::Parser;
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
    /// The path to the Path of Exile folder for steam
    steam_folder: PathBuf,
    /// The folder to which to output extracted files
    output_folder: PathBuf,
}

fn main() {
    let args = Cli::parse();

    // Load up index file
    let index_path = args.steam_folder.as_path().join("Bundles2/_.index.bin");
    let index = load_index_file(index_path.as_ref());

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
        // Go get the bundle where the data is
        let bundle_path = args.steam_folder.as_path().join(format!(
            "Bundles2/{}.bundle.bin",
            index.bundles[bundle_index as usize].name
        ));
        let bundle = load_bundle_content(bundle_path.as_path());

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
