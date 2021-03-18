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
    source: String,
    message: String,
    code: String,
    target: String,
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubmissionCreateStatusDetailInfo {
    code: String,
    details: String,

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
    errors: Vec<SubmissionCreateStatusDetailInfo>,

    warnings: Vec<SubmissionCreateStatusDetailInfo>,

    #[serde(rename = "certificationReports")]
    certification_reports: Vec<SubmissionCreateSertificationReport>,

    #[serde(flatten)]
    pub other_fields: HashMap<String, Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubmissionCreateAppPackageInfo {
    #[serde(rename = "fileName")]
    pub file_name: String,

    #[serde(rename = "fileStatus")]
    pub file_status: String,

    #[serde(rename = "minimumDirectXVersion")]
    pub minimum_direct_x: String,

    #[serde(rename = "minimumSystemRam")]
    pub minimum_ram: String,

    #[serde(flatten)]
    pub other_fields: HashMap<String, Value>,
}

/// Данная структура представляет собой ответ после инициализации
/// Описание данных: `https://docs.microsoft.com/en-us/windows/uwp/monetize/manage-app-submissions#app-submission-object`
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubmissionCreateResponse {
    pub id: String,

    pub status: String, // TODO: ENUM

    #[serde(rename = "statusDetails")]
    pub status_details: SubmissionStatusDetails,

    #[serde(rename = "fileUploadUrl")]
    pub file_upload_url: String,

    #[serde(rename = "applicationPackages")]
    pub app_packages: Vec<SubmissionCreateAppPackageInfo>,

    #[serde(flatten)]
    pub other_fields: HashMap<String, Value>,
}

//////////////////////////////////////////////////////////////////////

/// Данная структура представляет собой ответ после инициализации
/// Описание данных: `https://docs.microsoft.com/en-us/windows/uwp/monetize/manage-app-submissions#app-submission-object`
#[derive(Deserialize, Debug, Clone)]
pub struct SubmissionCommitResponse {
    pub status: String
}

//////////////////////////////////////////////////////////////////////

/// Данная структура представляет собой ответ получения статуса после коммита
/// Описание данных: `https://docs.microsoft.com/en-us/windows/uwp/monetize/get-status-for-an-app-submission`
#[derive(Deserialize, Debug, Clone)]
pub struct SubmissionStatusResponse {
    pub status: String,

    #[serde(rename = "statusDetails")]
    pub status_details: SubmissionStatusDetails
}