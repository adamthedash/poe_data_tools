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
pub mod ecf;
pub mod epk;
pub mod et;
pub mod gcf;
pub mod gft;
pub mod gt;
pub mod mat;
pub mod mtd;
pub mod pet;
pub mod psg;
pub mod rs;
pub mod shared;
pub mod tmo;
pub mod toy;
pub mod trl;
pub mod tsi;
pub mod tst;

use anyhow::Result;

pub trait FileParser {
    type Output;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output>;
}
