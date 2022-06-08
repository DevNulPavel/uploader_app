use quick_error::quick_error;
use std::io;
// use thiserror::{
//     Error
// };
use super::responses::{ResponseErr, ResponseErrorValue};
use url::ParseError;

quick_error! {
    #[derive(Debug)]
    pub enum GoogleDriveError{
        InvalidBaseAddr(info: String){
            display("{}", info)
        }

        URLError(err: ParseError){
            from(err: ParseError) -> (err)
            display("{}", err)
        }

        NetErr(err: reqwest::Error){
            from(err: reqwest::Error) -> (err)
            display("{}", err)
        }

        JsonError(err: serde_json::Error){
            from(err: serde_json::Error) -> (err)
            display("{}", err)
        }

        Custom(info: String){
            display("{}", info)
        }

        WrongFilePath {
        }

        FileError(err: io::Error){
            from()
            display("{}", err)
        }

        TokenIsExpired{
        }

        EmptyNewOwner{
        }

        ErrorResponse(err: ResponseErrorValue){
            from()
            from(err: ResponseErr) -> (err.error)
            display("{}", err)
        }
    }
}

/*#[derive(Debug)]
pub struct GoogleDriveError{
    trace: SpanTrace,
    source: anyhow::Error
}
impl<E> From<E> for GoogleDriveError
where
    E: Into<anyhow::Error>
{
    fn from(err: E) -> GoogleDriveError {
        GoogleDriveError{
            trace: SpanTrace::capture(),
            source: anyhow::Error::from(err)
        }
    }
}
impl std::fmt::Display for GoogleDriveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SuperError is here!")
    }
}
impl std::error::Error for GoogleDriveError{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}*/

/*impl From<ParseError> for GoogleDriveError {
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
}*/
