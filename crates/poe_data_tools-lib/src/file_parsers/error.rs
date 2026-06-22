use winnow::error::ContextError;

pub type Result<T, E = ParseError> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
#[error("error for file version: {version:?}")]
pub struct ParseError {
    /// Optional version if parsing failed midway through
    pub version: Option<u32>,
    #[source]
    inner: ParseErrorInner,
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum ParseErrorInner {
    /// Errors from winnow parsers
    #[error(transparent)]
    Winnow(#[from] winnow::error::ContextError),

    /// Errors from annotated_parser
    // TODO: Transform into proper error type
    #[error(transparent)]
    Annotated(annotated_parser::Annotation),

    /// Error from anything before a parser is applied
    #[error("error during file pre-processing: {0}")]
    Preprocessing(Box<dyn std::error::Error>),

    /// Catchall
    #[error("parsing error: {0}")]
    Other(String),
}

impl<I, E> From<winnow::error::ParseError<I, E>> for ParseErrorInner
where
    E: Into<ParseErrorInner>,
{
    fn from(value: winnow::error::ParseError<I, E>) -> Self {
        value.into_inner().into()
    }
}

// ==============================================================

/// Wrap an error in an unversioned ParseError
pub(crate) trait AsParseError {
    type Output;
    fn as_parse_error(self) -> Self::Output;
}

impl<T, E> AsParseError for Result<T, E>
where
    E: Into<ParseErrorInner>,
{
    type Output = Result<T, ParseError>;

    fn as_parse_error(self) -> Self::Output {
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
