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
use reqwest::{
    StatusCode
};
use url::{
    ParseError
};

#[derive(Debug)]
pub enum AmazonError{
    InvalidBaseAddr(String),
    URLError(ParseError),
    NetErr(reqwest::Error),
    WrongFilePath,
    InvalidFileExtention(&'static str),
    FileError(io::Error),
    InvalidTokenDuration(u64),
    TokenIsExpired,
    StartEditFailed,
    ETagReceiveFailed,
    ETagParseFailed,
    EmptyApksForCommit,
    ApkListFailedWithCode(StatusCode),
    ApkDeleteFailedWithCode(StatusCode),
    UploadingFailedWithCode(StatusCode),
    Custom(String),
}

impl From<ParseError> for AmazonError {
    fn from(err: ParseError) -> AmazonError {
        AmazonError::URLError(err)
    }
}
impl From<io::Error> for AmazonError {
    fn from(err: io::Error) -> AmazonError {
        AmazonError::FileError(err)
    }
}
impl From<reqwest::Error> for AmazonError {
    fn from(err: reqwest::Error) -> AmazonError {
        AmazonError::NetErr(err)
    }
}

impl Display for AmazonError{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Amazon error: {:#?}", self)
    }
}

impl Error for AmazonError {
}