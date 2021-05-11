//! Common error scenarios handled by this crate.
use std::fmt::Formatter;

/// Represents possible errors that might happen during the Lambda
/// function execution.
///
/// As observed in a few projects available at GitHub, most of the time
/// main function will return a [Result] instance that will handled by
/// the async/await runtime. Therefore, we don't need a fancy concrete type
/// for error handling here, but a way to convey what happened upon the
/// occurrence of an error.
#[derive(Debug, Eq, PartialEq)]
pub struct Error(String);

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error(s.to_string())
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(cause: std::string::FromUtf8Error) -> Self {
        Error(format!("{}", cause))
    }
}

impl From<serde_json::Error> for Error {
    fn from(cause: serde_json::Error) -> Self {
        Error(format!("{}", cause))
    }
}

impl From<hyper::Error> for Error {
    fn from(cause: hyper::Error) -> Self {
        Error(format!("{}", cause))
    }
}

impl From<hyper::http::Error> for Error {
    fn from(cause: hyper::http::Error) -> Self {
        Error(format!("{}", cause))
    }
}

impl From<hyper::http::uri::InvalidUri> for Error {
    fn from(cause: hyper::http::uri::InvalidUri) -> Self {
        Error(format!("{}", cause))
    }
}

/// Short-hand result definition.
pub type Result<T> = std::result::Result<T, Error>;
