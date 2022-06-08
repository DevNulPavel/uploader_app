use serde::{
    de::{self, Deserializer},
    Deserialize, Serialize,
};
use serde_json::value::Value;
use std::{collections::HashMap, fmt::Display, str::FromStr};

/// Вспомогательная функция для serde, чтобы конвертировать строки в u64 во время парсинга
fn deserealize_from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

//////////////////////////////////////////////////////////////////////

/// Тип ошибки, в который мы можем парсить наши данные
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct ErrorResponseValue {
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
    other: HashMap<String, Value>,
}

//////////////////////////////////////////////////////////////////////

/// Специальный шаблонный тип, чтобы можно было парсить возвращаемые ошибки в ответах
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum DataOrErrorResponse<D> {
    Ok(D),
    Err(ErrorResponseValue),
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
pub struct TokenResponse {
    pub token_type: String, // TODO: Bearer enum
    pub resource: String,
    pub access_token: String,
    // pub refresh_token: String, // Refresh токен не отдается
    #[serde(deserialize_with = "deserealize_from_str")]
    pub expires_in: u64,
    #[serde(deserialize_with = "deserealize_from_str")]
    pub expires_on: u64,
}

//////////////////////////////////////////////////////////////////////

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
}

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
    pub rank_higher_than: String,
}

//////////////////////////////////////////////////////////////////////

/// Данная структура представляет собой информацию о конретном flight
/// Описание данных: `https://docs.microsoft.com/en-us/windows/uwp/monetize/get-a-flight`
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FlightInfoSubmission {
    pub id: String,

    #[serde(rename = "resourceLocation")]
    pub resource_location: String,
}

/// Данная структура представляет собой информацию о конретном flight
/// Описание данных: `https://docs.microsoft.com/en-us/windows/uwp/monetize/get-a-flight`
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FlightInfoResponse {
    #[serde(rename = "flightId")]
    pub flight_id: String,

    #[serde(rename = "friendlyName")]
    pub friendly_name: String,

    #[serde(rename = "lastPublishedFlightSubmission")]
    pub last_published_flight_submission: Option<FlightInfoSubmission>,

    #[serde(rename = "pendingFlightSubmission")]
    pub pending_flight_submission: Option<FlightInfoSubmission>,

    #[serde(rename = "groupIds")]
    pub group_ids: Vec<String>,

    #[serde(rename = "rankHigherThan")]
    pub rank_higher_than: String,
}

//////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FlightPackage {
    #[serde(rename = "fileName")]
    file_name: Option<String>,

    #[serde(rename = "fileStatus")]
    file_status: Option<String>,

    id: Option<String>,
    version: Option<String>,
    languages: Option<serde_json::Value>,
    capabilities: Option<serde_json::Value>,

    #[serde(rename = "minimumDirectXVersion")]
    minimum_directx_version: Option<String>,

    #[serde(rename = "minimumSystemRam")]
    minimum_system_ram: Option<String>,
}

/// Данная структура представляет собой ответ после инициализации flight сабмиссии
/// Описание данных: `https://docs.microsoft.com/en-us/windows/uwp/monetize/create-a-flight-submission`
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FlightSubmissionsCreateResponse {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "flightId")]
    pub flight_id: String,

    #[serde(rename = "statusDetails")]
    pub status_details: SubmissionStatusDetails,

    #[serde(rename = "fileUploadUrl")]
    pub file_upload_url: String,

    #[serde(rename = "flightPackages")]
    pub flight_packages: Option<Vec<FlightPackage>>,
}

//////////////////////////////////////////////////////////////////////

/// Данная структура представляет собой ответ после инициализации
/// Описание данных: `https://docs.microsoft.com/en-us/windows/uwp/monetize/manage-app-submissions#app-submission-object`
#[derive(Deserialize, Debug, Clone)]
pub struct FlightSubmissionCommitResponse {
    pub status: String,
}

//////////////////////////////////////////////////////////////////////

/// Данная структура представляет собой ответ после инициализации
/// Описание данных: `https://docs.microsoft.com/en-us/windows/uwp/monetize/create-an-app-submission#response`
#[derive(Deserialize, Default, Serialize, Debug, Clone)]
pub struct AppPackage {
    #[serde(rename = "fileName")]
    pub file_name: String,

    #[serde(rename = "fileStatus")]
    pub file_status: String,

    #[serde(flatten)]
    pub other_fields: serde_json::Value,
    //
    // pub id: Option<String>,
    // pub version: Option<String>,
    // pub architecture: Option<String>,
    // pub languages: Option<Vec<String>>,
    // pub capabilities: Option<Vec<String>>,
    // #[serde(rename = "minimumDirectXVersion")]
    // pub minimum_directx_version: Option<String>,
    // #[serde(rename = "minimumSystemRam")]
    // pub minimum_system_ram: Option<String>,
    // #[serde(rename = "targetDeviceFamilies")]
    // pub target_device_families: Option<Vec<String>>,
}

/// Данная структура представляет собой ответ после инициализации
/// Описание данных: `https://docs.microsoft.com/en-us/windows/uwp/monetize/create-an-app-submission#response`
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubmissionCommonData {
    // Имя, которое отображается в админке
    #[serde(rename = "friendlyName")]
    pub friendly_name: Option<String>,

    // Режим публикации
    #[serde(rename = "targetPublishMode")]
    pub target_publish_mode: String,

    // Пакеты
    #[serde(rename = "applicationPackages")]
    pub application_packages: Vec<AppPackage>,

    // Все закомментированные поля выше просто размещаем внутри плоской структуры
    #[serde(flatten)]
    pub other_fields: serde_json::Value,
    //
    // #[serde(rename = "applicationCategory")]
    // pub application_category: String,

    // pub pricing: serde_json::Value,

    // pub visibility: String,

    // #[serde(rename = "targetPublishDate")]
    // pub target_date: String,

    // pub listings: serde_json::Value,

    // #[serde(rename = "hardwarePreferences")]
    // pub hardware_preferences: serde_json::Value,

    // #[serde(rename = "automaticBackupEnabled")]
    // pub automatic_backup_enabled: bool,

    // #[serde(rename = "canInstallOnRemovableMedia")]
    // pub can_install_on_removable_media: bool,

    // #[serde(rename = "isGameDvrEnabled")]
    // pub is_game_dvr_enabled: bool,

    // #[serde(rename = "gamingOptions")]
    // pub gaming_options: serde_json::Value,

    // #[serde(rename = "hasExternalInAppProducts")]
    // pub has_external_in_app_products: bool,

    // #[serde(rename = "meetAccessibilityGuidelines")]
    // pub meet_accessibility_guidelines: Option<bool>,

    // #[serde(rename = "notesForCertification")]
    // pub notes_for_certification: Option<String>,

    // #[serde(rename = "packageDeliveryOptions")]
    // pub package_delivery_options: serde_json::Value,

    // #[serde(rename = "enterpriseLicensing")]
    // pub enterprise_licensing: String,

    // #[serde(rename = "allowMicrosoftDecideAppAvailabilityToFutureDeviceFamilies")]
    // pub allow_microsoft_decide_app_availability_to_future_device_families: bool,

    // #[serde(rename = "allowTargetFutureDeviceFamilies")]
    // pub allow_target_future_device_families: serde_json::Value,

    // pub trailers: serde_json::Value,
}

/// Данная структура представляет собой ответ после инициализации
/// Описание данных: `https://docs.microsoft.com/en-us/windows/uwp/monetize/create-an-app-submission#response`
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubmissionCreateResponse {
    #[serde(flatten)]
    pub common_data: SubmissionCommonData,

    pub id: String,

    pub status: String,

    #[serde(rename = "statusDetails")]
    pub status_details: serde_json::Value,

    #[serde(rename = "fileUploadUrl")]
    pub file_upload_url: String,
    //
    // Все остальные поля в виде плоской структуры
    // #[serde(flatten)]
    // pub other_fields: serde_json::Value,
}

//////////////////////////////////////////////////////////////////////

/// Данная структура представляет собой ответ получения статуса после коммита
/// Описание данных: `https://docs.microsoft.com/en-us/windows/uwp/monetize/get-status-for-a-flight-submission`
#[derive(Deserialize, Debug, Clone)]
pub struct SubmissionStatusResponse {
    pub status: String,

    #[serde(rename = "statusDetails")]
    pub status_details: Option<SubmissionStatusDetails>,
}
