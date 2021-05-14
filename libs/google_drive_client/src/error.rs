use std::{
    io
};
use tracing_error::{
    SpanTrace
};
use quick_error::{
    quick_error
};
// use thiserror::{
//     Error
// };
use url::{
    ParseError
};
use super::{
    responses::{
        ResponseErrorValue,
        ResponseErr
    }
};

quick_error!{
    #[derive(Debug)]
    pub enum GoogleDriveError{
        InvalidBaseAddr(trace: SpanTrace, info: String){
        }
        
        URLError(trace: SpanTrace, err: ParseError){
            from(err: ParseError) -> (SpanTrace::capture(), err)
        }

        NetErr(trace: SpanTrace, err: reqwest::Error){
            from(err: reqwest::Error) -> (SpanTrace::capture(), err)
        }

        JsonError(trace: SpanTrace, err: serde_json::Error){
            from(err: serde_json::Error) -> (SpanTrace::capture(), err)
        }

        Custom(trace: SpanTrace, info: String){
        }

        WrongFilePath {
        }

        FileError(err: io::Error){
            from()
        }

        TokenIsExpired(trace: SpanTrace){
        }

        EmptyNewOwner{
        }

        ErrorResponse(err: ResponseErrorValue){
            from()
            from(err: ResponseErr) -> (err.error)
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