use std::{
    io,
    error::{
        Error
    },
    fmt::{
        self,
        Display,
        Formatter
    }
};
use url::{
    ParseError
};

#[derive(Debug)]
pub enum AppCenterError{
    InvalidBaseAddr(String),
    URLError(ParseError),
    NetErr(reqwest::Error),
    JsonParseErr(serde_json::Error),
    Custom(String),
    CustomDyn(Box<dyn std::error::Error + Send>),
    WrongFilePath,
    FileError(io::Error),
    ReleaseIdReceiveFailed(String),
}

impl From<ParseError> for AppCenterError {
    fn from(err: ParseError) -> AppCenterError {
        AppCenterError::URLError(err)
    }
}
impl From<io::Error> for AppCenterError {
    fn from(err: io::Error) -> AppCenterError {
        AppCenterError::FileError(err)
    }
}
impl From<reqwest::Error> for AppCenterError {
    fn from(err: reqwest::Error) -> AppCenterError {
        AppCenterError::NetErr(err)
    }
}
impl From<serde_json::Error> for AppCenterError {
    fn from(err: serde_json::Error) -> AppCenterError {
        AppCenterError::JsonParseErr(err)
    }
}

impl Display for AppCenterError{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "AppCenter error: {:#?}", self)
    }
}

impl Error for AppCenterError {
}