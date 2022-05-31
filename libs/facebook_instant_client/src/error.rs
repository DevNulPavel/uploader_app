use crate::responses::ErrorResponse;
use backtrace::Backtrace as BacktraceNoStd;
use std::{error::Error as StdError, fmt::Display};

#[derive(thiserror::Error, Debug)]
pub enum FacebookInstantError {
    #[error("Request error")]
    Request {
        source: reqwest::Error,
        context: &'static str,
        backtrace: BacktraceNoStd,
    },

    #[error("Response receiving error")]
    ResponseReceiving {
        source: reqwest::Error,
        context: &'static str,
        backtrace: BacktraceNoStd,
    },

    #[error("Response receiving error")]
    ResponseParsing {
        source: JsonParseError,
        context: &'static str,
        backtrace: BacktraceNoStd,
    },

    #[error("Response with error")]
    Response {
        source: ErrorResponse,
        context: &'static str,
        backtrace: BacktraceNoStd,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct JsonParseError {
    pub source: serde_json::Error,
    pub original_data: String,
}

impl StdError for JsonParseError {}

impl Display for JsonParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "source: {}, data: {}", self.source, self.original_data)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

// #[macro_export]
macro_rules! convert_error {
    ($type: ident, $info: literal) => {
        |err| crate::error::FacebookInstantError::$type {
            backtrace: BacktraceNoStd::new(),
            source: err,
            context: $info,
        }
    };
}

pub(crate) use convert_error;
