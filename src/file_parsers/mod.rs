pub mod arm;
pub mod ecf;
pub mod line_parser;
pub mod rs;
pub mod shared;
pub mod tsi;

use anyhow::Result;

pub trait FileParser {
    type Output;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output>;
}
