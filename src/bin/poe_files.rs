use anyhow::{ensure, Context, Result};
use clap::{ArgGroup, Parser, Subcommand};
use glob::Pattern;
use poe_tools::{
    bundle_fs::{from_cdn, from_steam},
    bundle_loader::cdn_base_url,
    commands::{
        cat::cat_file, dump_art::extract_art, dump_tables::dump_tables, extract::extract_files,
        list::list_files, Patch,
    },
};
use std::path::PathBuf;

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
    /// Converts datc64 files into CSV files
    DumpTables {
        /// Path to write out the parsed tables to
        output_folder: PathBuf,

        /// Glob pattern to filter the list of files
        #[clap(default_value = "*.datc64")]
        glob: Pattern,
    },
    DumpArt {
        /// Path to the folder to output the extracted files
        output_folder: PathBuf,
        /// Glob pattern to filter the list of files
        #[clap(default_value = "*.dds")]
        glob: Pattern,
    },
}

/// A simple CLI tool that extracts the virtual filenames from PoE data files.
/// File paths are printed to stdout.
#[derive(Parser, Debug)]
#[command(
    name = "poe_files",
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
    #[arg(long, required = true)]
    patch: Patch,

    /// Specify the Steam folder path (optional)
    #[arg(long)]
    steam: Option<PathBuf>,

    /// Specify the cache directory (optional)
    #[arg(long)]
    cache_dir: Option<PathBuf>,

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
    })
}

fn main() -> Result<()> {
    let args = parse_args()?;

    let mut fs = match args.source {
        Source::Cdn { cache_dir } => {
            let version_string = match &args.patch {
                Patch::One => "1",
                Patch::Two => "2",
                Patch::Specific(v) => v,
            };
            from_cdn(&cdn_base_url(&cache_dir, version_string)?, &cache_dir)
        }
        Source::Steam { steam_folder } => from_steam(steam_folder),
    }
    .context("Failed to initialise file system")?;

    match args.command {
        Command::List { glob } => list_files(&fs, &glob).context("List command failed")?,
        Command::Cat { path } => cat_file(&mut fs, &path).context("Cat command failed")?,
        Command::Extract {
            glob,
            output_folder,
        } => extract_files(&mut fs, &glob, &output_folder).context("Extract command filed")?,
        Command::DumpTables {
            output_folder,
            glob,
        } => dump_tables(&mut fs, &glob, &args.cache_dir, &output_folder, &args.patch)
            .context("Dump Tables command failed")?,
        Command::DumpArt {
            output_folder,
            glob,
        } => extract_art(&mut fs, &glob, &output_folder).context("Dump Art command failed")?,
    }

    Ok(())
}
