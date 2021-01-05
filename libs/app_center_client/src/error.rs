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
    Custom(String),
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

impl Display for AppCenterError{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "AppCenter error: {:#?}", self)
    }
}

impl Error for AppCenterError {
}