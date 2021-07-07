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
use crate::{
    responses::{
        ErrorResponseValue
    }
};

#[derive(Debug)]
pub enum AmazonError{
    InvalidBaseAddr(String),
    URLError(ParseError),
    NetErr(reqwest::Error),
    JsonParseErr(serde_json::Error),
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
    ApiError(ErrorResponseValue),
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
impl From<serde_json::Error> for AmazonError {
    fn from(err: serde_json::Error) -> AmazonError {
        AmazonError::JsonParseErr(err)
    }
}
impl From<ErrorResponseValue> for AmazonError {
    fn from(err: ErrorResponseValue) -> AmazonError {
        AmazonError::ApiError(err)
    }
}

impl Display for AmazonError{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Amazon error: {:#?}", self)
    }
}

impl Error for AmazonError {
}