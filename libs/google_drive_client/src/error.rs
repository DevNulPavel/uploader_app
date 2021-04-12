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
use super::{
    responses::{
        ResponseErrorValue,
        ResponseErr
    }
};

#[derive(Debug)]
pub enum GoogleDriveError{
    InvalidBaseAddr(String),
    URLError(ParseError),
    NetErr(reqwest::Error),
    JsonError(serde_json::Error),
    Custom(String),
    WrongFilePath,
    FileError(io::Error),
    TokenIsExpired,
    EmptyNewOwner,
    ErrorResponse(ResponseErrorValue)
}

impl From<ParseError> for GoogleDriveError {
    fn from(err: ParseError) -> GoogleDriveError {
        GoogleDriveError::URLError(err)
    }
}
impl From<io::Error> for GoogleDriveError {
    fn from(err: io::Error) -> GoogleDriveError {
        GoogleDriveError::FileError(err)
    }
}
impl From<reqwest::Error> for GoogleDriveError {
    fn from(err: reqwest::Error) -> GoogleDriveError {
        GoogleDriveError::NetErr(err)
    }
}
impl From<ResponseErr> for GoogleDriveError {
    fn from(err: ResponseErr) -> GoogleDriveError {
        GoogleDriveError::ErrorResponse(err.error)
    }
}
impl From<serde_json::Error> for GoogleDriveError {
    fn from(err: serde_json::Error) -> GoogleDriveError {
        GoogleDriveError::JsonError(err)
    }
}

impl Display for GoogleDriveError{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Google drive error: {:#?}", self)
    }
}

impl Error for GoogleDriveError {
}