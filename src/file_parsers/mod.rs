pub mod ao;
pub mod arm;
pub mod ddt;
pub mod ecf;
pub mod et;
pub mod gft;
pub mod gt;
pub mod lift;
pub mod lift_winnow;
pub mod mtd;
pub mod rs;
pub mod shared;
pub mod slice;
pub mod tsi;

use anyhow::Result;

pub trait FileParser {
    type Output;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output>;
}
