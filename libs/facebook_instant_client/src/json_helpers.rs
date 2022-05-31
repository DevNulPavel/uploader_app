use crate::error::JsonParseError;
use serde::Deserialize;

pub fn parse_json<T: for<'a> Deserialize<'a>>(data: String) -> Result<T, JsonParseError> {
    serde_json::from_str(&data).map_err(|err| JsonParseError {
        original_data: data,
        source: err,
    })
}

pub trait ParseJson {
    fn parse_json_with_data_err<T>(self) -> Result<T, JsonParseError>
    where
        T: for<'a> Deserialize<'a>;
}

impl ParseJson for std::string::String {
    fn parse_json_with_data_err<T>(self) -> Result<T, JsonParseError>
    where
        T: for<'a> Deserialize<'a>,
    {
        parse_json::<T>(self)
    }
}
