use clap::{Args, Parser, Subcommand};
use poe_game_data_parser::bundle_fs::{from_cdn, from_steam};
use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::PathBuf,
};

/// A simple CLI tool that extracts the virtual filenames from PoE data files.
/// File paths are printed to stdout.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(flatten)]
    global_opts: GlobalOpts,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    List {
        // Glob pattern to filter the list of files
        #[clap(default_value = "*")]
        glob: String,
    },
    Dump {
        // Path to the folder to output the extracted files
        output_folder: PathBuf,
        // Glob pattern to filter the list of files
        #[clap(default_value = "*")]
        glob: String,
    },
    Cat {
        // Path to the file to extract
        path: String,
    },
}

#[derive(Debug, Args)]
struct GlobalOpts {
    /// Version of the game to use: 1 for PoE 1, 2 for PoE 2, or a specific CDN patch version
    #[clap(short, long, default_value = "1")]
    patch: String,
    /// The path to the Path of Exile folder for steam - if not provided, will fetch from the CDN
    #[clap(short, long)]
    steam_folder: Option<PathBuf>,
    /// The path to the dir to store the local CDN cache
    #[clap(short, long, default_value=dirs::cache_dir().unwrap().join("poe_data_tools").into_os_string())]
    cache_dir: PathBuf,
}

fn main() {
    let args = Cli::parse();

    let mut fs = match args.global_opts.steam_folder {
        Some(steam_folder) => from_steam(steam_folder),
        None => from_cdn(&args.global_opts.cache_dir, args.global_opts.patch.as_str()),
    };

    match args.command {
        Command::List { glob } => {
            let pattern = glob::Pattern::new(&glob).expect("Failed to parse glob pattern");

            // Use a buffered writer since we're dumping a lot of data
            let stdout = io::stdout().lock();
            let mut out = BufWriter::new(stdout);

            fs.list()
                .iter()
                .filter(|p| pattern.matches(p))
                .for_each(|p| {
                    writeln!(out, "{}", p).expect("Failed to write to stdout");
                });

            out.flush().expect("Failed to flush stdout");
        }
        Command::Cat { path } => {
            let result = fs.read(path).expect("Failed to read file");
            let stdout = io::stdout().lock();
            let mut out = BufWriter::new(stdout);
            out.write_all(&result).expect("Failed to write to stdout");
            out.flush().expect("Failed to flush stdout");
        }
        Command::Dump {
            glob,
            output_folder,
        } => {
            let pattern = glob::Pattern::new(&glob).expect("Failed to parse glob pattern");
            eprintln!("Dumping files to: {:?}", output_folder);

            fs.list()
                .iter()
                .filter(|p| pattern.matches(p))
                .for_each(|p| {
                    // Dump it to disk
                    eprintln!("{}", p);
                    let contents = fs.read(p.to_string()).expect("Failed to read file");
                    let out_filename = output_folder.as_path().join(p);
                    fs::create_dir_all(out_filename.parent().unwrap())
                        .expect("Failed to create folder");
                    let mut out_file = File::create(out_filename).expect("Failed to create file.");
                    out_file
                        .write_all(&contents)
                        .expect("Failed to write to file.");
                });
        }
    }
}
