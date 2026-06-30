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

use std::path::Path;

use serde::Serialize;

use self::{
    amd::AMDParser, ao::AOParser, arm::ARMParser, cht::CHTParser, clt::CLTParser, dct::DCTParser,
    ddt::DDTParser, dlp::DLPParser, ecf::ECFParser, epk::EPKParser, et::ETParser, fmt::FMTParser,
    gcf::GCFParser, gft::GFTParser, gt::GTParser, mat::MATParser, mtd::MTDParser, pet::PETParser,
    psg::PSGParser, rs::RSParser, sm::SMParser, smd::SMDParser, tgm::TGMParser, tgt::TGTParser,
    tmo::TMOParser, toy::TOYParser, trl::TRLParser, tsi::TSIParser, tst::TSTParser,
};

pub trait FileParser {
    /// Structured output type
    type Output;

    /// Attempt to parse a set of bytes. If the file contains a version before parsing fails, it is
    /// returned along with the result.
    fn parse(&self, bytes: &[u8]) -> error::Result<Self::Output>;

    /// Checks whether the file has been parsed successfully
    /// Also returns the file version if available
    fn validate(&self, bytes: &[u8]) -> (bool, Option<u32>)
    where
        Self::Output: VersionedFile,
    {
        let res = self.parse(bytes);

        match res {
            Ok(file) => (true, file.version()),
            Err(e) => (false, e.version),
        }
    }
}

pub trait VersionedFile {
    fn version(&self) -> Option<u32>;
}

/// All possible file parsers
#[non_exhaustive]
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

impl FileParser for Parser {
    type Output = ParserOutput;

    fn parse(&self, bytes: &[u8]) -> error::Result<Self::Output> {
        use Parser::*;
        let output = match self {
            Amd(p) => ParserOutput::Amd(p.parse(bytes)?),
            Ao(p) => ParserOutput::Ao(p.parse(bytes)?),
            Arm(p) => ParserOutput::Arm(Box::new(p.parse(bytes)?)),
            Cht(p) => ParserOutput::Cht(p.parse(bytes)?),
            Clt(p) => ParserOutput::Clt(p.parse(bytes)?),
            Dct(p) => ParserOutput::Dct(p.parse(bytes)?),
            Ddt(p) => ParserOutput::Ddt(p.parse(bytes)?),
            Dlp(p) => ParserOutput::Dlp(p.parse(bytes)?),
            Ecf(p) => ParserOutput::Ecf(p.parse(bytes)?),
            Epk(p) => ParserOutput::Epk(p.parse(bytes)?),
            Et(p) => ParserOutput::Et(Box::new(p.parse(bytes)?)),
            Fmt(p) => ParserOutput::Fmt(Box::new(p.parse(bytes)?)),
            Gcf(p) => ParserOutput::Gcf(p.parse(bytes)?),
            Gft(p) => ParserOutput::Gft(p.parse(bytes)?),
            Gt(p) => ParserOutput::Gt(p.parse(bytes)?),
            Mat(p) => ParserOutput::Mat(p.parse(bytes)?),
            Mtd(p) => ParserOutput::Mtd(p.parse(bytes)?),
            Pet(p) => ParserOutput::Pet(p.parse(bytes)?),
            Psg(p) => ParserOutput::Psg(p.parse(bytes)?),
            Rs(p) => ParserOutput::Rs(p.parse(bytes)?),
            Sm(p) => ParserOutput::Sm(p.parse(bytes)?),
            Smd(p) => ParserOutput::Smd(Box::new(p.parse(bytes)?)),
            Tgm(p) => ParserOutput::Tgm(p.parse(bytes)?),
            Tgt(p) => ParserOutput::Tgt(Box::new(p.parse(bytes)?)),
            Tmo(p) => ParserOutput::Tmo(p.parse(bytes)?),
            Toy(p) => ParserOutput::Toy(p.parse(bytes)?),
            Trl(p) => ParserOutput::Trl(p.parse(bytes)?),
            Tsi(p) => ParserOutput::Tsi(p.parse(bytes)?),
            Tst(p) => ParserOutput::Tst(p.parse(bytes)?),
        };

        Ok(output)
    }
}

/// All possible parsed file types
/// Some are boxes due to their size
#[derive(Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ParserOutput {
    Amd(<AMDParser as FileParser>::Output),
    Ao(<AOParser as FileParser>::Output),
    Arm(Box<<ARMParser as FileParser>::Output>),
    Cht(<CHTParser as FileParser>::Output),
    Clt(<CLTParser as FileParser>::Output),
    Dct(<DCTParser as FileParser>::Output),
    Ddt(<DDTParser as FileParser>::Output),
    Dlp(<DLPParser as FileParser>::Output),
    Ecf(<ECFParser as FileParser>::Output),
    Epk(<EPKParser as FileParser>::Output),
    Et(Box<<ETParser as FileParser>::Output>),
    Fmt(Box<<FMTParser as FileParser>::Output>),
    Gcf(<GCFParser as FileParser>::Output),
    Gft(<GFTParser as FileParser>::Output),
    Gt(<GTParser as FileParser>::Output),
    Mat(<MATParser as FileParser>::Output),
    Mtd(<MTDParser as FileParser>::Output),
    Pet(<PETParser as FileParser>::Output),
    Psg(<PSGParser as FileParser>::Output),
    Rs(<RSParser as FileParser>::Output),
    Sm(<SMParser as FileParser>::Output),
    Smd(Box<<SMDParser as FileParser>::Output>),
    Tgm(<TGMParser as FileParser>::Output),
    Tgt(Box<<TGTParser as FileParser>::Output>),
    Tmo(<TMOParser as FileParser>::Output),
    Toy(<TOYParser as FileParser>::Output),
    Trl(<TRLParser as FileParser>::Output),
    Tsi(<TSIParser as FileParser>::Output),
    Tst(<TSTParser as FileParser>::Output),
}

impl VersionedFile for ParserOutput {
    fn version(&self) -> Option<u32> {
        use ParserOutput::*;
        match self {
            Amd(o) => o.version(),
            Ao(o) => o.version(),
            Arm(o) => o.version(),
            Cht(o) => o.version(),
            Clt(o) => o.version(),
            Dct(o) => o.version(),
            Ddt(o) => o.version(),
            Dlp(o) => o.version(),
            Ecf(o) => o.version(),
            Epk(o) => o.version(),
            Et(o) => o.version(),
            Fmt(o) => o.version(),
            Gcf(o) => o.version(),
            Gft(o) => o.version(),
            Gt(o) => o.version(),
            Mat(o) => o.version(),
            Mtd(o) => o.version(),
            Pet(o) => o.version(),
            Psg(o) => o.version(),
            Rs(o) => o.version(),
            Sm(o) => o.version(),
            Smd(o) => o.version(),
            Tgm(o) => o.version(),
            Tgt(o) => o.version(),
            Tmo(o) => o.version(),
            Toy(o) => o.version(),
            Trl(o) => o.version(),
            Tsi(o) => o.version(),
            Tst(o) => o.version(),
        }
    }
}
