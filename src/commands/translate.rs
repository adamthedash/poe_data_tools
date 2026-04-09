use std::{io::BufWriter, path::Path};

use anyhow::{Context, Result};
use enum_dispatch::enum_dispatch;
use glob::{MatchOptions, Pattern};
use serde::Serialize;

use crate::{
    commands::Patch,
    file_parsers::{
        FileParser, amd::AMDParser, ao::AOParser, arm::ARMParser, cht::CHTParser, clt::CLTParser,
        dct::DCTParser, ddt::DDTParser, dlp::DLPParser, ecf::ECFParser, epk::EPKParser,
        et::ETParser, fmt::FMTParser, gcf::GCFParser, gft::GFTParser, gt::GTParser, mat::MATParser,
        mtd::MTDParser, pet::PETParser, psg::PSGParser, rs::RSParser,
        shared::versioned_result::VersionedResult2, smd::SMDParser, tmo::TMOParser, toy::TOYParser,
        trl::TRLParser, tsi::TSIParser, tst::TSTParser,
    },
    fs::{FS, FileSystem},
};

#[enum_dispatch]
pub trait FileParserExt {
    /// Parse and serialise to JSON
    fn parse_to_json_file(&self, bytes: &[u8], output_path: &Path) -> Result<()>;

    /// Checks whether the file has been parsed successfully
    fn validate(&self, bytes: &[u8]) -> VersionedResult2<(), ()>;
}

impl<P> FileParserExt for P
where
    P: FileParser,
    P::Output: Serialize,
{
    fn parse_to_json_file(&self, bytes: &[u8], output_path: &Path) -> Result<()> {
        let parsed = self.parse(bytes).as_anyhow()?;

        std::fs::create_dir_all(output_path.parent().unwrap())
            .context("Failed to create folder")?;

        let f = std::fs::File::create(output_path)
            .with_context(|| format!("Failed to create file {:?}", output_path))?;
        let f = BufWriter::new(f);

        serde_json::to_writer(f, &parsed).context("Failed to serialise")?;

        Ok(())
    }

    fn validate(&self, bytes: &[u8]) -> VersionedResult2<(), ()> {
        let res = self.parse(bytes);
        VersionedResult2 {
            version: res.version,
            inner: match res.inner {
                Ok(_) => Ok(()),
                Err(_) => Err(()),
            },
        }
    }
}

/// Parser for different file formats
#[enum_dispatch(FileParserExt)]
pub enum Parser {
    Amd(AMDParser),
    Ao(AOParser),
    Arm(ARMParser),
    Cht(CHTParser),
    Clt(CLTParser),
    Dct(DCTParser),
    Ddt(DDTParser),
    Dlp(DLPParser),
    Ecf(ECFParser),
    Epk(EPKParser),
    Et(ETParser),
    Fmt(FMTParser),
    Gcf(GCFParser),
    Gft(GFTParser),
    Gt(GTParser),
    Mat(MATParser),
    Mtd(MTDParser),
    Pet(PETParser),
    Psg(PSGParser),
    Rs(RSParser),
    Smd(SMDParser),
    Tmo(TMOParser),
    Toy(TOYParser),
    Trl(TRLParser),
    Tsi(TSIParser),
    Tst(TSTParser),
}

impl Parser {
    pub fn from_filename(filename: &Path, poe_version: u32) -> Option<Self> {
        let ext = filename.extension()?.to_str()?;

        use Parser::*;
        let f = match ext {
            "amd" => Amd(AMDParser),
            "ao" => Ao(AOParser),
            "arm" => Arm(ARMParser),
            "cht" => Cht(CHTParser),
            "clt" => Clt(CLTParser),
            "dct" => Dct(DCTParser),
            "ddt" => Ddt(DDTParser),
            "dlp" => Dlp(DLPParser),
            "ecf" => Ecf(ECFParser),
            "epk" => Epk(EPKParser),
            "et" => Et(ETParser),
            "fmt" => Fmt(FMTParser),
            "gcf" => Gcf(GCFParser),
            "gft" => Gft(GFTParser),
            "gt" => Gt(GTParser),
            "mat" => Mat(MATParser),
            "mtd" => Mtd(MTDParser),
            "pet" => Pet(PETParser),
            "psg" => Psg(PSGParser {
                version: poe_version,
            }),
            "rs" => Rs(RSParser),
            "smd" => Smd(SMDParser),
            "tmo" => Tmo(TMOParser),
            "toy" => Toy(TOYParser),
            "trl" => Trl(TRLParser),
            "tsi" => Tsi(TSIParser),
            "tst" => Tst(TSTParser),
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

    fs.batch_read(&filenames)
        // Print and filter out errors
        .filter_map(|f| match f {
            Ok(x) => Some(x),
            Err((path, e)) => {
                log::error!("Failed to extract file: {:?}: {:?}", path, e);
                None
            }
        })
        // Attempt to read file contents
        .map(|(filename, contents)| -> Result<_, anyhow::Error> {
            log::info!("Extracting file: {filename}");
            let parser = Parser::from_filename(Path::new(filename.as_ref()), poe_version.major())
                .expect("Already verified parser exists above");

            let out_path = output_folder
                .join(filename.as_ref())
                .with_added_extension("json");
            parser
                .parse_to_json_file(&contents, &out_path)
                .with_context(|| format!("Failed to process file: {:?}", filename))?;

            Ok(filename)
        })
        // Report results
        .for_each(|result| match result {
            Ok(filename) => log::info!("Extracted file: {}", filename),
            Err(e) => log::error!("{:?}", e),
        });

    Ok(())
}
