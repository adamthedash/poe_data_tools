pub mod amd;
pub mod ao;
pub mod arm;
pub mod cht;
pub mod clt;
pub mod ddt;
pub mod dlp;
pub mod ecf;
pub mod epk;
pub mod et;
pub mod gft;
pub mod gt;
pub mod lift_nom;
pub mod lift_winnow;
pub mod mat;
pub mod mtd;
pub mod pet;
pub mod rs;
pub mod shared;
pub mod slice;
pub mod trl;
pub mod tsi;
pub mod tst;

use anyhow::Result;

pub trait FileParser {
    type Output;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output>;
}
