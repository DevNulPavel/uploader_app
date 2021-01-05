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
pub enum GoogleDriveError{
    InvalidBaseAddr(String),
    URLError(ParseError),
    NetErr(reqwest::Error),
    Custom(String),
    WrongFilePath,
    FileError(io::Error),
    TokenIsExpired,
    EmptyNewOwner,
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

impl Display for GoogleDriveError{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Google drive error: {:#?}", self)
    }
}

impl Error for GoogleDriveError {
}