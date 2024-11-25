use clap::Parser;
use murmurhash64::murmur_hash64a;
use poe_game_data_parser::{bundle::load_bundle_content, bundle_index::load_index_file};
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

    let hash_seed = 0x1337b33f;

    // Process input filenames
    let stdin = io::stdin().lock();
    stdin.lines().for_each(|l| {
        // Look up the filename
        let filename = l.expect("Failed to read line from stdin.");
        let hash = murmur_hash64a(filename.to_lowercase().as_bytes(), hash_seed);
        let file = file_lut
            .get(&hash)
            .unwrap_or_else(|| panic!("File not found: {}", filename));

        println!("Extracting: {}", filename);

        // Go get the bundle where the data is
        // todo: batch filenames from the same bundle to reduce reads
        let bundle_path = args.steam_folder.as_path().join(format!(
            "Bundles2/{}.bundle.bin",
            index.bundles[file.bundle_index as usize].name
        ));
        let bundle = load_bundle_content(bundle_path.as_path());

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
    })
}
