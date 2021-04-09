// use std::{
//     collections::{
//         HashMap
//     }
// };
use serde::{
    Deserialize
};
// use into_result::{
//     IntoResult
// };
// use serde_json::{
//     Value
// };

#[derive(Deserialize, Debug)]
pub struct ErrorResponseValue{
    // pub code: i64,
    pub message: String,
    pub status: String
    // #[serde(flatten)]
    // other: HashMap<String, Value>
}
#[derive(Deserialize, Debug)]
pub struct ErrorResponse{
    pub error: ErrorResponseValue,
}

//////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum DataOrErrorResponse<D>{
    Ok(D),
    Err(ErrorResponse)
}
impl<D> DataOrErrorResponse<D> {
    pub fn into_result(self) -> Result<D, ErrorResponse> {
        match self {
            DataOrErrorResponse::Ok(ok) => Ok(ok),
            DataOrErrorResponse::Err(err) => Err(err),
        }
    }
}

//////////////////////////////////////////////////////////////////////

// https://developers.google.com/play/api/v3/reference/files#resource
#[derive(Deserialize, Debug)]
pub struct AppEditResponseOk{
    pub id: String
}
/*#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum AppEditResponse{
    Ok(AppEditResponseOk),
    Err(ErrorResponse)
}
impl IntoResult<AppEditResponseOk, ErrorResponse> for AppEditResponse {
    fn into_result(self) -> Result<AppEditResponseOk, ErrorResponse> {
        match self {
            AppEditResponse::Ok(ok) => Ok(ok),
            AppEditResponse::Err(err) => Err(err),
        }
    }
}*/

//////////////////////////////////////////////////////////////////////

// https://developers.google.com/android-publisher/api-ref/rest/v3/edits.bundles
// https://developers.google.com/android-publisher/api-ref/rest/v3/edits.apks
#[derive(Deserialize, Debug)]
pub struct UploadResponseOk{
    #[serde(rename = "versionCode")]
    pub version_code: u64
}
/*#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum UploadResponse{
    Ok(UploadResponseOk),
    Err(ErrorResponse)
}
impl IntoResult<UploadResponseOk, ErrorResponse> for UploadResponse {
    fn into_result(self) -> Result<UploadResponseOk, ErrorResponse> {
        match self {
            UploadResponse::Ok(ok) => Ok(ok),
            UploadResponse::Err(err) => Err(err),
        }
    }
}*/

//////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct TrackUpdateResponse{
    pub track: String
}