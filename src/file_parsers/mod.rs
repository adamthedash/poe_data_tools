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

#[derive(Debug)]
pub struct VersionedError {
    pub version: Option<u32>,
    pub inner: anyhow::Error,
}

pub type VersionedResult<T> = Result<T, VersionedError>;

impl From<anyhow::Error> for VersionedError {
    fn from(value: anyhow::Error) -> Self {
        Self {
            version: None,
            inner: value,
        }
    }
}

impl From<VersionedError> for anyhow::Error {
    fn from(value: VersionedError) -> Self {
        value
            .inner
            .context(format!("Fail for file version: {:?}", value.version))
    }
}

trait VersionedResultExt<T> {
    fn with_version(self, version: Option<u32>) -> VersionedResult<T>;
}

impl<T> VersionedResultExt<T> for anyhow::Result<T> {
    fn with_version(self, version: Option<u32>) -> VersionedResult<T> {
        self.map_err(|e| VersionedError { version, inner: e })
    }
}

pub trait FileParser {
    type Output;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output>;
}
