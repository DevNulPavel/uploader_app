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
    Serialize,
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
    code: String,
    message: String,
    source: Option<String>,
    target: Option<String>,
    // error: String,
    // error_description: String,
    // error_codes: Vec<u32>,
    // timestamp: String, // TODO: Use DateTime "2016-04-11 18:59:01Z",
    // trace_id: String,
    // correlation_id: String,

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

/// Представляет из себя ответ от сервера на получение токена
/// Описание данных: `https://docs.microsoft.com/en-us/azure/active-directory/azuread-dev/v1-protocols-oauth-code#successful-response-2`
#[derive(Deserialize, Debug)]
pub struct TokenResponse{
    pub token_type: String, // TODO: Bearer enum
    pub resource: String,
    pub access_token: String,
    // pub refresh_token: String, // Refresh токен не отдается 

    #[serde(deserialize_with = "deserealize_from_str")]
    pub expires_in: u64,
    #[serde(deserialize_with = "deserealize_from_str")]
    pub expires_on: u64
}

//////////////////////////////////////////////////////////////////////

/*
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubmissionCreateStatusDetailInfo {
    pub code: String,
    
    pub details: String,

    #[serde(flatten)]
    pub other_fields: HashMap<String, Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubmissionCreateSertificationReport {
    date: String,

    #[serde(rename = "reportUrl")]
    report_url: String,

    #[serde(flatten)]
    pub other_fields: HashMap<String, Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubmissionStatusDetails {
    pub errors: Vec<SubmissionCreateStatusDetailInfo>,
    
    pub warnings: Vec<SubmissionCreateStatusDetailInfo>,

    #[serde(rename = "certificationReports")]
    pub certification_reports: Vec<SubmissionCreateSertificationReport>,
    
    #[serde(flatten)]
    pub other_fields: HashMap<String, Value>,
}*/

//////////////////////////////////////////////////////////////////////

/// Данная структура представляет собой ответ после инициализации
/// Описание данных: `https://docs.microsoft.com/en-us/windows/uwp/monetize/create-a-flight`
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FlightCreateResponse {
    #[serde(rename = "flightId")]
    pub flight_id: String,

    #[serde(rename = "friendlyName")]
    pub friendly_name: String,

    #[serde(rename = "groupIds")]
    pub group_ids: Vec<String>,

    #[serde(rename = "rankHigherThan")]
    pub rank_higher_than: String
}

//////////////////////////////////////////////////////////////////////

/// Данная структура представляет собой ответ после инициализации flight сабмиссии
/// Описание данных: `https://docs.microsoft.com/en-us/windows/uwp/monetize/create-a-flight-submission`
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FlightSubmissionsCreateResponse {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "flightId")]
    pub flight_id: String,

    // #[serde(rename = "statusDetails")]
    // pub status_details: SubmissionStatusDetails,

    #[serde(rename = "fileUploadUrl")]
    pub file_upload_url: String
}

//////////////////////////////////////////////////////////////////////

/// Данная структура представляет собой ответ после инициализации
/// Описание данных: `https://docs.microsoft.com/en-us/windows/uwp/monetize/manage-app-submissions#app-submission-object`
#[derive(Deserialize, Debug, Clone)]
pub struct FlightSubmissionCommitResponse {
    pub status: String
}

//////////////////////////////////////////////////////////////////////

/// Данная структура представляет собой ответ получения статуса после коммита
/// Описание данных: `https://docs.microsoft.com/en-us/windows/uwp/monetize/get-status-for-an-app-submission`
#[derive(Deserialize, Debug, Clone)]
pub struct SubmissionStatusResponse {
    pub status: String,

    // #[serde(rename = "statusDetails")]
    // pub status_details: SubmissionStatusDetails
}