use crate::responses::ErrorResponse;
use serde_json_string_parse::JsonParseError;
use std::path::PathBuf;
// use backtrace::Backtrace as BacktraceNoStd;

#[derive(thiserror::Error, Debug)]
pub enum FacebookInstantError {
    #[error("Request error `{source}` at `{context}`")]
    Request {
        source: reqwest::Error,
        context: &'static str,
        // backtrace: BacktraceNoStd,
    },

    #[error("Response receiving error `{source}` at `{context}`")]
    ResponseReceiving {
        source: reqwest::Error,
        context: &'static str,
        // backtrace: BacktraceNoStd,
    },

    #[error("Response receiving error `{source}` at `{context}`")]
    ResponseParsing {
        source: JsonParseError<String>,
        context: &'static str,
        // backtrace: BacktraceNoStd,
    },

    #[error("Response with error from facebook API `{source}` at `{context}`")]
    ApiResponse {
        source: ErrorResponse,
        context: &'static str,
        // backtrace: BacktraceNoStd,
    },

    #[error("File does not exist at `{path}`")]
    NoFileAtPath {
        path: PathBuf,
        // backtrace: BacktraceNoStd,
    },

    #[error("File have no .zip extention `{path}`")]
    NotZipFile {
        path: PathBuf,
        // backtrace: BacktraceNoStd,
    },

    #[error("Filename is missing `{path}`")]
    NoZipFilename {
        path: PathBuf,
        // backtrace: BacktraceNoStd,
    },

    #[error("IO error `{source}` at `{context}`")]
    IO {
        source: std::io::Error,
        context: &'static str,
        // backtrace: BacktraceNoStd,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////

// #[macro_export]
macro_rules! convert_error {
    ($type: ident, $info: literal) => {
        |err| crate::error::FacebookInstantError::$type {
            // backtrace: BacktraceNoStd::new(),
            source: err,
            context: $info,
        }
    };
}

pub(crate) use convert_error;
