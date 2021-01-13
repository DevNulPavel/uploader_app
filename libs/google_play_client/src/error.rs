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
        ErrorResponse,
        ErrorResponseValue
    }
};

#[derive(Debug)]
pub enum GooglePlayError{
    InvalidBaseAddr(String),
    URLError(ParseError),
    NetErr(reqwest::Error),
    WrongFilePath,
    InvalidFileExtention(&'static str),
    FileError(io::Error),
    TokenIsExpired,
    ResponseError(ErrorResponseValue),
    Custom(String),
}

impl From<ParseError> for GooglePlayError {
    fn from(err: ParseError) -> GooglePlayError {
        GooglePlayError::URLError(err)
    }
}
impl From<io::Error> for GooglePlayError {
    fn from(err: io::Error) -> GooglePlayError {
        GooglePlayError::FileError(err)
    }
}
impl From<reqwest::Error> for GooglePlayError {
    fn from(err: reqwest::Error) -> GooglePlayError {
        GooglePlayError::NetErr(err)
    }
}
impl From<ErrorResponse> for GooglePlayError {
    fn from(err: ErrorResponse) -> GooglePlayError {
        GooglePlayError::ResponseError(err.error)
    }
}

impl Display for GooglePlayError{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Google play error: {:#?}", self)
    }
}

impl Error for GooglePlayError {
}