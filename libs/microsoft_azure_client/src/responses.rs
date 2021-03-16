use std::{
    collections::{
        HashMap
    },
    str::{
        FromStr
    },
    fmt::{
        Display
    }
};
use serde::{
    Deserialize,
    de::{
        self, 
        Deserializer
    }
};
use serde_json::{
    value::{
        Value
    }
};

/// Вспомогательная функция для serde, чтобы конвертировать строки в u64 во время парсинга
fn deserealize_from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s)
        .map_err(de::Error::custom)
}

//////////////////////////////////////////////////////////////////////

/// Тип ошибки, в который мы можем парсить наши данные
#[derive(Deserialize, Debug)]
pub struct ErrorResponseValue{
    error: String,
    error_description: String,
    error_codes: Vec<u32>,
    timestamp: String, // TODO: Use DateTime "2016-04-11 18:59:01Z",
    trace_id: String,
    correlation_id: String,

    #[serde(flatten)]
    other: HashMap<String, Value>
}

//////////////////////////////////////////////////////////////////////

/// Специальный шаблонный тип, чтобы можно было парсить возвращаемые ошибки в ответах
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum DataOrErrorResponse<D>{
    Ok(D),
    Err(ErrorResponseValue)
}
impl<D> DataOrErrorResponse<D> {
    pub fn into_result(self) -> Result<D, ErrorResponseValue> {
        match self {
            DataOrErrorResponse::Ok(ok) => Ok(ok),
            DataOrErrorResponse::Err(err) => Err(err),
        }
    }
}

//////////////////////////////////////////////////////////////////////

// https://docs.microsoft.com/en-us/azure/active-directory/azuread-dev/v1-protocols-oauth-code#successful-response-2
#[derive(Deserialize, Debug)]
pub struct TokenResponse{
    pub token_type: String, // TODO: Bearer enum
    pub resource: String,
    pub access_token: String,
    pub refresh_token: String,

    #[serde(deserialize_with = "deserealize_from_str")]
    pub expires_in: u64,
    #[serde(deserialize_with = "deserealize_from_str")]
    pub expires_on: u64
}