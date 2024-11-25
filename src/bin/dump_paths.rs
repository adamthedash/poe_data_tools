use clap::Parser;
use poe_game_data_parser::bundle_index::load_index_file;
use poe_game_data_parser::path::parse_paths;
use std::io::Write;
use std::{
    io::{self, BufWriter},
    path::PathBuf,
};

/// A simple CLI tool that extracts the virtual filenames from PoE data files.
/// File paths are printed to stdout.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the Path of Exile folder for steam
    steam_folder: PathBuf,
}

fn main() {
    let args = Cli::parse();

    // Load up index file
    let index_path = args.steam_folder.as_path().join("Bundles2/_.index.bin");
    let index = load_index_file(index_path.as_ref());

    // Use a buffered writer since we're dumping a lot of data
    let stdout = io::stdout().lock();
    let mut out = BufWriter::new(stdout);

    // Loop over each folder
    index.paths.iter().for_each(|pr| {
        let parsed = parse_paths(&index.path_rep_bundle, pr);

        // Print out the fully qualified file paths
        parsed.get_paths().iter().for_each(|p| {
            writeln!(out, "{}", p).expect("Failed to write to stdout");
        })
    });

    out.flush().expect("Failed to flush stdout");
}
