use clap::{error::ErrorKind, ArgAction, Args, CommandFactory, Parser, Subcommand};
use glob::Pattern;
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
    /// List files
    List {
        /// Glob pattern to filter the list of files
        #[clap(default_value = "*")]
        glob: Pattern,
    },
    /// Extract matched files to a folder
    Extract {
        /// Path to the folder to output the extracted files
        output_folder: PathBuf,
        /// Glob pattern to filter the list of files
        #[clap(default_value = "*")]
        glob: Pattern,
    },
    /// Extract a single file to stdout
    Cat {
        /// Path to the file to extract
        path: String,
    },
}

#[derive(Debug, Args)]
struct GlobalOpts {
    /// Use the steam folder as the source of the data instead of the CDN
    #[clap(
        long,
        group = "source",
        action = ArgAction::SetTrue,
        default_value_t=false,
        conflicts_with_all=&["cdn", "cache_dir"],
    )]
    steam: bool,

    /// Version of the game to use: 1 for PoE 1, 2 for PoE 2, or a specific CDN patch version if --steam isn't used
    #[clap(long, default_value = "2")]
    patch: String,

    /// The path to the dir to store the local CDN cache; only required without --steam
    #[clap(
        long,
        default_value=dirs::cache_dir().unwrap().join("poe_data_tools").into_os_string(),
    )]
    cache_dir: PathBuf,

    /// The path to the game install folder for --steam [default: autodetect based on --patch]
    #[clap(
        long,
        default_value_ifs([
            ("patch", "1", steam_folder_search("1").unwrap_or(PathBuf::new()).into_os_string()),
            ("patch", "2", steam_folder_search("2").unwrap_or(PathBuf::new()).into_os_string()),
        ])
    )]
    steam_folder: Option<PathBuf>,
}

fn steam_folder_search(patch: &str) -> Option<PathBuf> {
    let home = dirs::home_dir().unwrap();
    let game = match patch {
        "1" => "Path of Exile",
        "2" => "Path of Exile 2",
        _ => return None,
    };
    [
        home.join(".local/share/Steam/steamapps/common"),
        home.join("Library/Application Support/Steam/steamapps/common"),
        PathBuf::from("C:\\Program Files (x86)\\Grinding Gear Games"),
        PathBuf::from("/mnt/e/SteamLibrary/steamapps/common"),
    ]
    .iter()
    .map(|p| p.join(game))
    .find(|p| p.exists())
}

fn main() {
    let args = Cli::parse();

    let mut fs = match args.global_opts.steam {
        true => match args.global_opts.steam_folder {
            Some(f) => from_steam(f),
            None => Cli::command()
                .error(
                    ErrorKind::ArgumentConflict,
                    "Invalid steam folder. Patch must be 1 or 2 when using --steam without --steam-folder",
                )
                .exit(),
        },
        false => from_cdn(&args.global_opts.cache_dir, args.global_opts.patch.as_str()),
    };

    match args.command {
        Command::List { glob } => {
            // Use a buffered writer since we're dumping a lot of data
            let stdout = io::stdout().lock();
            let mut out = BufWriter::new(stdout);

            fs.list().iter().filter(|p| glob.matches(p)).for_each(|p| {
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
        Command::Extract {
            glob,
            output_folder,
        } => {
            fs.list().iter().filter(|p| glob.matches(p)).for_each(|p| {
                // Dump it to disk
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
