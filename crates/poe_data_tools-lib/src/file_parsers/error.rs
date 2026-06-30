use std::fmt::Display;

use annotated_parser::{Annotation, AnnotationResult};

use crate::file_parsers::shared::BOMError;

pub type Result<T, E = ParseError> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
#[error("error for file version: {version:?}")]
pub struct ParseError {
    /// Optional version if parsing failed midway through
    pub version: Option<u32>,
    #[source]
    inner: ParseErrorInner,
}

impl ParseError {
    pub(crate) fn other(e: impl std::error::Error + Send + Sync + 'static) -> Self {
        ParseErrorInner::Other(e.into()).into()
    }

    pub(crate) fn processing(e: impl std::error::Error + Send + Sync + 'static) -> Self {
        ParseErrorInner::Preprocessing(e.into()).into()
    }
}

impl From<ParseErrorInner> for ParseError {
    fn from(value: ParseErrorInner) -> Self {
        Self {
            version: None,
            inner: value,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum ParseErrorInner {
    /// Errors from winnow parsers
    // TODO: Figure out how to properly bubble up context for winnow parsers
    #[error("error originating from winnow parser: {0:?}")]
    Winnow(#[from] winnow::error::ContextError),

    /// Errors from annotated_parser
    #[error(transparent)]
    Annotated(#[from] AnnotatedError),

    /// Error from anything before a parser is applied
    #[error("error during file pre-processing: {0}")]
    Preprocessing(Box<dyn std::error::Error + Send + Sync>),

    /// Catchall
    #[error("parsing error: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl<I, E> From<winnow::error::ParseError<I, E>> for ParseErrorInner
where
    E: Into<ParseErrorInner>,
{
    fn from(value: winnow::error::ParseError<I, E>) -> Self {
        value.into_inner().into()
    }
}

impl From<annotated_parser::Annotation> for ParseErrorInner {
    fn from(value: annotated_parser::Annotation) -> Self {
        AnnotatedError::from(value).into()
    }
}

impl From<BOMError> for ParseErrorInner {
    fn from(value: BOMError) -> Self {
        Self::Preprocessing(Box::new(value))
    }
}

// ==============================================================

/// Wrap an error in an unversioned ParseError
pub(crate) trait AsParseError {
    type Output;
    fn to_parse_error(self) -> Self::Output;
}

impl<T, E> AsParseError for Result<T, E>
where
    E: Into<ParseErrorInner>,
{
    type Output = Result<T, ParseError>;

    fn to_parse_error(self) -> Self::Output {
        self.map_err(|e| ParseError {
            version: None,
            inner: e.into(),
        })
    }
}

/// For adding version information to parse errors
pub(crate) trait ParseResultEx {
    fn with_version(self, version: u32) -> Self;
    fn with_maybe_version(self, version: Option<u32>) -> Self;
}

impl<T> ParseResultEx for Result<T, ParseError> {
    fn with_version(self, version: u32) -> Self {
        self.map_err(|e| ParseError {
            version: Some(version),
            ..e
        })
    }

    fn with_maybe_version(self, version: Option<u32>) -> Self {
        self.map_err(|e| ParseError { version, ..e })
    }
}

// ==============================================================
// TODO: This goes in annotated_parser

#[derive(Debug, thiserror::Error)]
pub(crate) struct AnnotatedError {
    /// Materialsed parser ID
    parser_id: String,
    /// One of failure cases
    result: AnnotationResult,

    source: Option<Box<Self>>,
}

impl Display for AnnotatedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.result {
            AnnotationResult::Success { .. } => {
                unreachable!("annotated errors cannot contain successful results")
            }
            AnnotationResult::Incomplete { start } => {
                write!(
                    f,
                    "[incomplete] @ {start} \"extra data expected\" {}",
                    self.parser_id
                )
            }
            AnnotationResult::Child { start } => write!(f, "[child] @ {start} {}", self.parser_id),
            AnnotationResult::Invalid { span, reason } => {
                write!(f, "[invalid] @ {span:?} {reason:?} {}", self.parser_id)
            }
        }
    }
}

impl From<Annotation> for AnnotatedError {
    fn from(mut value: Annotation) -> Self {
        // TODO: A better error display for annotation trees so we don't need to do this
        value = value.to_failure_tree().expect("on error path");
        value.materialize();

        let mut errors = value.failure_path().into_iter();

        // Prep bottom of stack
        let first = errors.by_ref().next().expect("must be at least 1 error");
        let first = Self {
            parser_id: first.parser_id,
            result: first.result,
            source: None,
        };

        // Fold rest of stack in
        errors.fold(first, |source, anno| Self {
            parser_id: anno.parser_id,
            result: anno.result,
            source: Some(Box::new(source)),
        })
    }
}
