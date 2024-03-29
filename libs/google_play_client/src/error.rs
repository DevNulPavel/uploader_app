use std::{
    io
};
use quick_error::{
    quick_error
};
use url::{
    ParseError
};
use super::{
    responses::{
        ErrorResponse
    }
};

quick_error! {
    #[derive(Debug)]
    pub enum GooglePlayError {
        InvalidBaseAddr(err: String){
            display("{}", err)
        }
        
        EmptyUrlSegments{
        }

        URLError(err: ParseError){
            from()
            display("{}", err)
        }

        // NetErr(context: &'static str, err: reqwest::Error){
        //     context(context: &'static str, err: reqwest::Error) -> (context, err)
        // }
        NetErr(err: reqwest::Error){
            from()
            display("{}", err)
        }

        // JsonParseErr(context: &'static str, err: serde_json::Error){
        //     context(context: &'static str, err: serde_json::Error) -> (context, err)
        // }
        JsonParseErr(err: serde_json::Error){
            from()
            display("{}", err)
        }

        WrongFilePath{
        }

        InvalidFileExtention(err: &'static str){
            display("{}", err)
        }

        FileError(err: io::Error){
            from()
            display("{}", err)
        }

        TokenIsExpired{
        }

        ResponseError(err: ErrorResponse){
            from()
            display("{:?}", err)
        }

        Custom(err: String){
            display("{}", err)
        }
    }
}

/*
#[derive(Debug)]
pub enum GooglePlayError{
    InvalidBaseAddr(String),
    EmptyUrlSegments,
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
}*/