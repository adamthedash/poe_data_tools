use std::{
    convert::Infallible,
    ops::{ControlFlow, FromResidual, Residual, Try},
};

use anyhow::Context;

pub type VersionedResult<T> = VersionedResult2<T, anyhow::Error>;

pub struct VersionedResult2<T, E> {
    pub version: Option<u32>,
    pub inner: Result<T, E>,
}

impl<T> From<VersionedResult2<T, anyhow::Error>> for anyhow::Result<T> {
    fn from(value: VersionedResult2<T, anyhow::Error>) -> Self {
        value
            .inner
            .with_context(|| format!("Fail for version: {:?}", value.version))
    }
}

impl<T> VersionedResult2<T, anyhow::Error> {
    pub fn as_anyhow(self) -> anyhow::Result<T> {
        self.into()
    }
}

impl<T, E> Try for VersionedResult2<T, E> {
    type Output = T;

    type Residual = VersionedResult2<!, E>;

    fn from_output(output: Self::Output) -> Self {
        Self {
            version: None,
            inner: Ok(output),
        }
    }

    fn branch(self) -> ControlFlow<Self::Residual, T> {
        match self.inner {
            Ok(v) => ControlFlow::Continue(v),
            Err(e) => ControlFlow::Break(VersionedResult2 {
                version: self.version,
                inner: Err(e),
            }),
        }
    }
}

pub trait VersionedResultExt<T, E> {
    fn with_version(self, version: Option<u32>) -> VersionedResult2<T, E>;
    fn unversioned(self) -> VersionedResult2<T, E>;
}

impl<T, E> VersionedResultExt<T, E> for Result<T, E> {
    fn with_version(self, version: Option<u32>) -> VersionedResult2<T, E> {
        VersionedResult2 {
            version,
            inner: self,
        }
    }

    fn unversioned(self) -> VersionedResult2<T, E> {
        VersionedResult2 {
            version: None,
            inner: self,
        }
    }
}

/// Coerce non-versioned result
impl<T, E> FromResidual<Result<Infallible, E>> for VersionedResult2<T, E> {
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        let Err(e) = residual;

        Self {
            version: None,
            inner: Err(e),
        }
    }
}

/// For propagating versioned results upwards
impl<T, E> FromResidual<VersionedResult2<!, E>> for VersionedResult2<T, E> {
    fn from_residual(residual: VersionedResult2<!, E>) -> Self {
        let Err(e) = residual.inner;

        Self {
            version: residual.version,
            inner: Err(e),
        }
    }
}

// Required due to https://github.com/rust-lang/rust/pull/154451
impl<T, E> Residual<T> for VersionedResult2<!, E> {
    type TryType = VersionedResult2<T, E>;
}
