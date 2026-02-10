use std::{io::BufWriter, path::Path};

use anyhow::{Context, Result};
use glob::{MatchOptions, Pattern};
use serde::Serialize;

use crate::{
    bundle_fs::FS,
    commands::Patch,
    file_parsers::{
        FileParser, ao::AOParser, arm::parser::ARMParser, ddt::DDTParser, ecf::ECFParser,
        et::ETParser, gft::GFTParser, gt::GTParser, rs::RSParser, tsi::TSIParser,
    },
};

pub trait FileParserExt: FileParser {
    fn parse_to_json_file(&self, bytes: &[u8], output_path: &Path) -> Result<()>
    where
        Self::Output: Serialize,
    {
        let parsed = self.parse(bytes)?;

        std::fs::create_dir_all(output_path.parent().unwrap())
            .context("Failed to create folder")?;

        let f = std::fs::File::create(output_path)
            .with_context(|| format!("Failed to create file {:?}", output_path))?;
        let f = BufWriter::new(f);

        serde_json::to_writer(f, &parsed).context("Failed to serialise")?;

        Ok(())
    }
}

impl<P> FileParserExt for P where P: FileParser {}

enum Parser {
    Tsi(TSIParser),
    Rs(RSParser),
    Arm(ARMParser),
    Ecf(ECFParser),
    Et(ETParser),
    Gt(GTParser),
    Gft(GFTParser),
    Ddt(DDTParser),
    AO(AOParser),
}

impl Parser {
    fn parse_to_json_file(&self, bytes: &[u8], output_folder: &Path) -> Result<()> {
        use Parser::*;
        match self {
            Tsi(p) => p.parse_to_json_file(bytes, output_folder),
            Rs(p) => p.parse_to_json_file(bytes, output_folder),
            Arm(p) => p.parse_to_json_file(bytes, output_folder),
            Ecf(p) => p.parse_to_json_file(bytes, output_folder),
            Et(p) => p.parse_to_json_file(bytes, output_folder),
            Gt(p) => p.parse_to_json_file(bytes, output_folder),
            Gft(p) => p.parse_to_json_file(bytes, output_folder),
            Ddt(p) => p.parse_to_json_file(bytes, output_folder),
            AO(p) => p.parse_to_json_file(bytes, output_folder),
        }
    }

    fn from_filename(filename: &Path) -> Option<Self> {
        let ext = filename.extension()?.to_str()?;

        let f = match ext {
            "rs" => Parser::Rs(RSParser),
            "tsi" => Parser::Tsi(TSIParser),
            "arm" => Parser::Arm(ARMParser),
            "ecf" => Parser::Ecf(ECFParser),
            "et" => Parser::Et(ETParser),
            "gt" => Parser::Gt(GTParser),
            "gft" => Parser::Gft(GFTParser),
            "ddt" => Parser::Ddt(DDTParser),
            "ao" => Parser::AO(AOParser),
            _ => return None,
        };

        Some(f)
    }
}

/// Extract, parse and transform files into easier to parse alternatives
pub fn translate(
    fs: &mut FS,
    patterns: &[Pattern],
    _cache_dir: &Path,
    output_folder: &Path,
    _version: &Patch,
) -> Result<()> {
    let filenames = fs
        .list()
        // Filter on globs
        .filter(|filename| {
            patterns.iter().any(|pattern| {
                pattern.matches_with(
                    filename,
                    MatchOptions {
                        require_literal_separator: true,
                        ..Default::default()
                    },
                )
            })
        })
        // Filter out files that we can't parse
        // TODO: This might be expensive, also we might want to log skips?
        .filter(|filename| Parser::from_filename(Path::new(filename)).is_some())
        .collect::<Vec<_>>();

    let filenames = filenames.iter().map(|f| f.as_str()).collect::<Vec<_>>();

    fs.batch_read(&filenames)
        // Print and filter out errors
        .filter_map(|f| match f {
            Ok(x) => Some(x),
            Err((path, e)) => {
                eprintln!("Failed to extract file: {:?}: {:?}", path, e);
                None
            }
        })
        // Attempt to read file contents
        .map(|(filename, contents)| -> Result<_, anyhow::Error> {
            eprintln!("Extracting file: {filename}");
            let parser = Parser::from_filename(Path::new(filename))
                .expect("Already verified parser exists above");

            let out_path = output_folder.join(filename).with_added_extension("json");
            parser
                .parse_to_json_file(&contents, &out_path)
                .with_context(|| format!("Failed to process file: {:?}", filename))?;

            Ok(filename)
        })
        // Report results
        .for_each(|result| match result {
            Ok(filename) => eprintln!("Extracted file: {}", filename),
            Err(e) => eprintln!("{:?}", e),
        });

    Ok(())
}
