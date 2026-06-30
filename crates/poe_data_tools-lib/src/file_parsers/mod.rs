pub mod amd;
pub mod ao;
pub mod arm;
pub mod bundle;
pub mod bundle_index;
pub mod cht;
pub mod clt;
pub mod dat;
pub mod dct;
pub mod ddt;
pub mod dlp;
pub mod dolm;
pub mod ecf;
pub mod epk;
pub mod error;
pub mod et;
pub mod fmt;
pub mod gcf;
pub mod gft;
pub mod ggpk;
pub mod gt;
pub mod mat;
pub mod mtd;
pub mod pet;
pub mod psg;
pub mod rs;
pub mod shared;
pub mod sm;
pub mod smd;
pub mod tgm;
pub mod tgt;
pub mod tmo;
pub mod toy;
pub mod trl;
pub mod tsi;
pub mod tst;

use std::{io::BufWriter, path::Path};

use amd::AMDParser;
use anyhow::{Context, Result};
use ao::AOParser;
use arm::ARMParser;
use cht::CHTParser;
use clt::CLTParser;
use dct::DCTParser;
use ddt::DDTParser;
use dlp::DLPParser;
use ecf::ECFParser;
use enum_dispatch::enum_dispatch;
use epk::EPKParser;
use et::ETParser;
use fmt::FMTParser;
use gcf::GCFParser;
use gft::GFTParser;
use gt::GTParser;
use mat::MATParser;
use mtd::MTDParser;
use pet::PETParser;
use psg::PSGParser;
use rs::RSParser;
use serde::Serialize;
use shared::versioned_result::VersionedResult2;
pub use shared::versioned_result::{VersionedResult, VersionedResultExt};
use sm::SMParser;
use smd::SMDParser;
use tgm::TGMParser;
use tgt::TGTParser;
use tmo::TMOParser;
use toy::TOYParser;
use trl::TRLParser;
use tsi::TSIParser;
use tst::TSTParser;

pub trait FileParser2 {
    /// Structured output type
    type Output;

    /// Attempt to parse a set of bytes. If the file contains a version before parsing fails, it is
    /// returned along with the result.
    fn parse(&self, bytes: &[u8]) -> error::Result<Self::Output>;
}

/// A trait for parsing binary file contents into a structured type
pub trait FileParser {
    /// Structured output type
    type Output;

    /// Attempt to parse a set of bytes. If the file contains a version before parsing fails, it is
    /// returned along with the result.
    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output>;
}

impl<P> FileParser for P
where
    P: FileParser2,
{
    type Output = P::Output;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output> {
        match self.parse(bytes) {
            Ok(val) => VersionedResult {
                // TODO: Use VersionedFile trait
                version: None,
                inner: Ok(val),
            },
            Err(e) => {
                println!("{e:?}");
                VersionedResult {
                    version: e.version,
                    inner: Err(anyhow::anyhow!(e)),
                }
            }
        }
    }
}

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
    Sm(SMParser),
    Smd(SMDParser),
    Tgm(TGMParser),
    Tgt(TGTParser),
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
            "sm" => Sm(SMParser),
            "smd" => Smd(SMDParser),
            "tgm" => Tgm(TGMParser),
            "tgt" => Tgt(TGTParser),
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
