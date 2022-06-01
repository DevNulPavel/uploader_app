// use backtrace::Backtrace as BacktraceNoStd;
// use derive_more::Display;
use serde::Deserialize;
use std::{error::Error as StdError, fmt::Display};

////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct ErrorData {
    pub message: Option<String>,

    #[serde(rename = "type")]
    pub err_type: Option<String>,

    pub code: Option<String>,

    pub fbtrace_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ErrorResponse {
    pub error: ErrorData,
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StdError for ErrorResponse {}

////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ResponseWrapper<T> {
    Valid(T),
    Error(ErrorResponse),
}

impl<T> ResponseWrapper<T> {
    pub fn into_result(self) -> Result<T, ErrorResponse> {
        match self {
            Self::Valid(data) => Ok(data),
            Self::Error(e) => Err(e),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct UploadResponse {
    pub success: bool,
}
