use serde::Deserialize;
use std::{
    error::Error as StdError,
    fmt::{Debug, Display},
};

////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct JsonParseError<T> {
    pub source_error: serde_json::Error,
    pub original_data: T,
}

impl<T> StdError for JsonParseError<T> where T: Display + Debug {}

impl<T> Display for JsonParseError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "source: {}, data: {}", self.source_error, self.original_data)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

pub fn parse_json<T, R>(data: T) -> Result<R, JsonParseError<T>>
where
    T: AsRef<str>,
    R: for<'de> Deserialize<'de>,
{
    serde_json::from_str(data.as_ref()).map_err(|err| JsonParseError {
        original_data: data,
        source_error: err,
    })
}

////////////////////////////////////////////////////////////////////////////////////////////////

pub trait ParseJson {
    fn parse_json_with_data_err<T>(self) -> Result<T, JsonParseError<Self>>
    where
        T: for<'de> Deserialize<'de>,
        Self: Sized;
}

////////////////////////////////////////////////////////////////////////////////////////////////

impl ParseJson for std::string::String {
    fn parse_json_with_data_err<T>(self) -> Result<T, JsonParseError<Self>>
    where
        T: for<'de> Deserialize<'de>,
    {
        parse_json::<Self, T>(self)
    }
}

impl ParseJson for &str {
    fn parse_json_with_data_err<T>(self) -> Result<T, JsonParseError<Self>>
    where
        T: for<'de> Deserialize<'de>,
    {
        parse_json::<Self, T>(self)
    }
}
