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
pub mod smd;
pub mod tdt;
pub mod tmo;
pub mod toy;
pub mod trl;
pub mod tsi;
pub mod tst;

pub use shared::versioned_result::{VersionedResult, VersionedResultExt};

pub trait FileParser {
    type Output;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output>;
}
