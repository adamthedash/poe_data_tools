use std::{io::BufWriter, path::Path};

use anyhow::{Context, Result};
use glob::{MatchOptions, Pattern};
use serde::Serialize;

use crate::{
    bundle_fs::FS,
    commands::Patch,
    file_parsers::{
        FileParser, amd::AMDParser, ao::AOParser, arm::ARMParser, cht::CHTParser, clt::CLTParser,
        dct::DCTParser, ddt::DDTParser, dlp::DLPParser, ecf::ECFParser, epk::EPKParser,
        et::ETParser, gcf::GCFParser, gft::GFTParser, gt::GTParser, mat::MATParser, mtd::MTDParser,
        pet::PETParser, psg::PSGParser, rs::RSParser, tmo::TMOParser, toy::TOYParser,
        trl::TRLParser, tsi::TSIParser, tst::TSTParser,
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
    Ao(AOParser),
    Mtd(MTDParser),
    Mat(MATParser),
    Tst(TSTParser),
    Clt(CLTParser),
    Amd(AMDParser),
    Epk(EPKParser),
    Pet(PETParser),
    Trl(TRLParser),
    Dlp(DLPParser),
    Cht(CHTParser),
    Dct(DCTParser),
    Toy(TOYParser),
    Tmo(TMOParser),
    Gcf(GCFParser),
    Psg(PSGParser),
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
            Ao(p) => p.parse_to_json_file(bytes, output_folder),
            Mtd(p) => p.parse_to_json_file(bytes, output_folder),
            Mat(p) => p.parse_to_json_file(bytes, output_folder),
            Tst(p) => p.parse_to_json_file(bytes, output_folder),
            Clt(p) => p.parse_to_json_file(bytes, output_folder),
            Amd(p) => p.parse_to_json_file(bytes, output_folder),
            Epk(p) => p.parse_to_json_file(bytes, output_folder),
            Pet(p) => p.parse_to_json_file(bytes, output_folder),
            Trl(p) => p.parse_to_json_file(bytes, output_folder),
            Dlp(p) => p.parse_to_json_file(bytes, output_folder),
            Cht(p) => p.parse_to_json_file(bytes, output_folder),
            Dct(p) => p.parse_to_json_file(bytes, output_folder),
            Toy(p) => p.parse_to_json_file(bytes, output_folder),
            Tmo(p) => p.parse_to_json_file(bytes, output_folder),
            Gcf(p) => p.parse_to_json_file(bytes, output_folder),
            Psg(p) => p.parse_to_json_file(bytes, output_folder),
        }
    }

    fn from_filename(filename: &Path, poe_version: u32) -> Option<Self> {
        let ext = filename.extension()?.to_str()?;

        use Parser::*;
        let f = match ext {
            "rs" => Rs(RSParser),
            "tsi" => Tsi(TSIParser),
            "arm" => Arm(ARMParser),
            "ecf" => Ecf(ECFParser),
            "et" => Et(ETParser),
            "gt" => Gt(GTParser),
            "gft" => Gft(GFTParser),
            "ddt" => Ddt(DDTParser),
            "ao" => Ao(AOParser),
            "mtd" => Mtd(MTDParser),
            "mat" => Mat(MATParser),
            "tst" => Tst(TSTParser),
            "clt" => Clt(CLTParser),
            "amd" => Amd(AMDParser),
            "epk" => Epk(EPKParser),
            "pet" => Pet(PETParser),
            "trl" => Trl(TRLParser),
            "dlp" => Dlp(DLPParser),
            "cht" => Cht(CHTParser),
            "dct" => Dct(DCTParser),
            "toy" => Toy(TOYParser),
            "tmo" => Tmo(TMOParser),
            "gcf" => Gcf(GCFParser),
            "psg" => Psg(PSGParser {
                version: poe_version,
            }),
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
    poe_version: &Patch,
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
        .filter(|filename| {
            Parser::from_filename(Path::new(filename), poe_version.major()).is_some()
        })
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
            let parser = Parser::from_filename(Path::new(filename), poe_version.major())
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
