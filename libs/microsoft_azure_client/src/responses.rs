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

/// Представляет из себя информацию о сабмите приложения в рамках информации о приложении
/// Описание данных: `https://docs.microsoft.com/en-us/windows/uwp/monetize/get-app-data`
#[derive(Deserialize, Debug)]
pub struct ApplicationInfoSubmissionData{
    pub id: String,

    #[serde(rename = "resourceLocation")]
    pub resource_location: String
}

/// Представляет из себя информацию о конкретном приложении
/// Описание данных: `https://docs.microsoft.com/en-us/windows/uwp/monetize/get-app-data`
#[derive(Deserialize, Debug)]
pub struct ApplicationInfoResponse{
    pub id: String,

    #[serde(rename = "primaryName")]
    pub primary_name: String,

    #[serde(rename = "packageFamilyName")]
    pub package_family_name: Option<String>,

    #[serde(rename = "packageIdentityName")]
    pub package_identity_name: String,

    #[serde(rename = "publisherName")]
    pub publisher_name: String,

    #[serde(rename = "firstPublishedDate")]
    pub first_publisher_date: String,

    #[serde(rename = "lastPublishedApplicationSubmission")]
    pub last_publisher_app_submission: Option<ApplicationInfoSubmissionData>,

    #[serde(rename = "pendingApplicationSubmission")]
    pub pending_app_submission: Option<ApplicationInfoSubmissionData>,

    #[serde(rename = "hasAdvancedListingPermission")]
    pub has_advanced_listing_perm: bool
}

//////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct SubmissionCreateStatusDetailInfo {
    code: String,
    details: String
}

#[derive(Deserialize, Debug)]
pub struct SubmissionCreateSertificationReport {
    date: String,

    #[serde(rename = "reportUrl")]
    report_url: String
}

#[derive(Deserialize, Debug)]
pub struct SubmissionCreateStatusDetails {
    errors: Vec<SubmissionCreateStatusDetailInfo>,

    warnings: Vec<SubmissionCreateStatusDetailInfo>,

    #[serde(rename = "certificationReports")]
    certification_reports: Vec<SubmissionCreateSertificationReport>
}

/// Данная структура представляет собой ответ после инициализации
/// Описание данных: `https://docs.microsoft.com/en-us/windows/uwp/monetize/manage-app-submissions#app-submission-object`
#[derive(Deserialize, Debug)]
pub struct SubmissionCreateResponse {
    pub id: String,

    #[serde(rename = "applicationCategory")]
    pub app_category: String,

    pub visibility: String,

    #[serde(rename = "targetPublishMode")]
    pub target_publish_mode: String,

    #[serde(rename = "targetPublishDate")]
    pub target_publish_date: String,

    pub status: String, // TODO: ENUM

    #[serde(rename = "statusDetails")]
    pub status_details: SubmissionCreateStatusDetails,

    #[serde(rename = "fileUploadUrl")]
    pub file_upload_url: String,

    #[serde(rename = "friendlyName")]
    pub friendly_name: String
}