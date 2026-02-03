use std::path::PathBuf;

use anyhow::{ensure, Context, Result};
use clap::{ArgGroup, Parser, Subcommand};
use glob::Pattern;
use poe_data_tools::{
    bundle_fs::FS,
    bundle_loader::cdn_base_url,
    commands::{
        cat::cat_file, dump_art::extract_art, dump_maps::dump_maps, dump_tables::dump_tables,
        dump_trees::dump_trees, extract::extract_files, list::list_files, Patch,
    },
    VERBOSE,
};

#[derive(Debug, Subcommand)]
enum Command {
    /// List files
    List {
        /// Glob patterns to filter the list of files
        #[clap(default_value = "**")]
        #[arg(num_args = 1..)]
        globs: Vec<Pattern>,
    },
    /// Extract matched files to a folder
    Extract {
        /// Path to the folder to output the extracted files
        output_folder: PathBuf,
        /// Glob patterns to filter the list of files
        #[clap(default_value = "**")]
        #[arg(num_args = 1..)]
        globs: Vec<Pattern>,
    },
    /// Extract a single file to stdout
    Cat {
        /// Path to the file to extract
        path: String,
    },
    /// Converts datc64 files into CSV files
    DumpTables {
        /// Path to write out the parsed tables to
        output_folder: PathBuf,

        /// Glob patterns to filter the list of files
        #[clap(default_value = "**/*.datc64")]
        #[arg(num_args = 1..)]
        globs: Vec<Pattern>,
    },
    DumpArt {
        /// Path to the folder to output the extracted files
        output_folder: PathBuf,
        /// Glob pattern to filter the list of files
        #[clap(default_value = "**/*.dds")]
        #[arg(num_args = 1..)]
        globs: Vec<Pattern>,
    },
    /// Extracts the passive trees as JSON
    DumpTrees {
        output_folder: PathBuf,

        /// Glob patterns to filter the list of files
        #[clap(default_value = "**/*.psg")]
        #[arg(num_args = 1..)]
        globs: Vec<Pattern>,
    },
    /// Extracts map files as JSON
    DumpMaps {
        output_folder: PathBuf,

        /// Glob patterns to filter the list of files
        #[clap(default_value = "**/*.arm")]
        #[arg(num_args = 1..)]
        globs: Vec<Pattern>,
    },
}

/// A simple CLI tool that extracts the virtual filenames from PoE data files.
/// File paths are printed to stdout.
#[derive(Parser, Debug)]
#[command(
    name = "poe_data_tools",
    group(
        ArgGroup::new("source")
        .args(&["steam", "cache_dir"])
        .required(false) // At least one is not required, but they are mutually exclusive
        .multiple(false) // Only one can be used at a time
    )
)]
#[clap(version)]
struct Cli {
    /// Specify the patch version (1, 2, or specific_patch)
    #[arg(short, long, required = true)]
    patch: Patch,

    /// Specify the Steam folder path (optional)
    #[arg(long)]
    steam: Option<PathBuf>,

    /// Specify the cache directory (optional)
    #[arg(long)]
    cache_dir: Option<PathBuf>,

    /// Verbose printing of non-fatal error messages
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug)]
enum Source {
    Cdn { cache_dir: PathBuf },
    Steam { steam_folder: PathBuf },
}

#[derive(Debug)]
struct Args {
    patch: Patch,
    source: Source,
    command: Command,
    cache_dir: PathBuf,
    verbose: bool,
}

/// Validates user input and constructs a valid input state
fn parse_args() -> Result<Args> {
    let cli = Cli::parse();

    let cache_dir = cli
        .cache_dir
        .unwrap_or_else(|| dirs::cache_dir().unwrap().join("poe_data_tools"));

    let source = if let Some(steam_folder) = cli.steam {
        ensure!(steam_folder.exists(), "Steam folder doesn't exist");
        Source::Steam { steam_folder }
    } else {
        Source::Cdn {
            cache_dir: cache_dir.clone(),
        }
    };

    if matches!(source, Source::Steam { .. }) {
        ensure!(
            !matches!(cli.patch, Patch::Specific { .. }),
            "When using steam, specific patch versions are not supported."
        );
    }

    Ok(Args {
        patch: cli.patch,
        source,
        command: cli.command,
        cache_dir,
        verbose: cli.verbose,
    })
}

fn main() -> Result<()> {
    let args = parse_args()?;
    VERBOSE.set(args.verbose).unwrap();

    let mut fs = match args.source {
        Source::Cdn { cache_dir } => {
            let version_string = match &args.patch {
                Patch::One => "1",
                Patch::Two => "2",
                Patch::Specific(v) => v,
            };
            FS::from_cdn(&cdn_base_url(&cache_dir, version_string)?, &cache_dir)
        }
        Source::Steam { steam_folder } => FS::from_steam(steam_folder),
    }
    .context("Failed to initialise file system")?;

    match args.command {
        Command::List { globs } => list_files(&fs, &globs).context("List command failed")?,
        Command::Cat { path } => cat_file(&mut fs, &path).context("Cat command failed")?,
        Command::Extract {
            globs,
            output_folder,
        } => extract_files(&mut fs, &globs, &output_folder).context("Extract command filed")?,
        Command::DumpTables {
            output_folder,
            globs,
        } => dump_tables(
            &mut fs,
            &globs,
            &args.cache_dir,
            &output_folder,
            &args.patch,
        )
        .context("Dump Tables command failed")?,
        Command::DumpArt {
            output_folder,
            globs,
        } => extract_art(&mut fs, &globs, &output_folder).context("Dump Art command failed")?,
        Command::DumpTrees {
            output_folder,
            globs,
        } => {
            dump_trees(
                &mut fs,
                &globs,
                &output_folder,
                &args.patch,
                &args.cache_dir,
            )
            .context("Dump Tree command failed")?;
        }
        Command::DumpMaps {
            output_folder,
            globs,
        } => {
            dump_maps(&mut fs, &globs, &output_folder).context("Dump Maps command failed")?;
        }
    }

    Ok(())
}
