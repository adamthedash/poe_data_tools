use clap::Parser;
use poe_game_data_parser::bundle_index::{fetch_index_file, load_index_file, BundleIndex};
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
    /// Version of the game to use: 1 for PoE 1, 2 for PoE 2, or a specific CDN patch version
    #[arg(short, long, default_value = "1")]
    patch: String,
    /// The path to the Path of Exile folder for steam - if not provided, will fetch from the CDN
    #[arg(short, long)]
    steam_folder: Option<PathBuf>,
    /// The path to the dir to store the local CDN cache
    #[arg(short, long, default_value=dirs::cache_dir().unwrap().join("poe_data_tools").into_os_string())]
    cache_dir: PathBuf,
}

fn main() {
    let args = Cli::parse();

    // Load up index file
    let index: BundleIndex;
    if let Some(steam_folder) = args.steam_folder {
        let index_path = steam_folder.as_path().join("Bundles2/_.index.bin");
        index = load_index_file(index_path.as_ref());
    } else {
        index = fetch_index_file(
            args.patch.as_str(),
            args.cache_dir.as_ref(),
            PathBuf::from("Bundles2/_.index.bin").as_ref(),
        );
    }

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
