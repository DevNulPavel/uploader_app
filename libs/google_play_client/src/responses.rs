use std::{
    collections::{
        HashMap
    }
};
use serde::{
    Deserialize
};
use into_result::{
    IntoResult
};
use serde_json::{
    Value
};


#[derive(Deserialize, Debug)]
pub struct ErrorResponseValue{
    pub code: i64,
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

// https://developers.google.com/play/api/v3/reference/files#resource
#[derive(Deserialize, Debug)]
pub struct AppEditResponse{
    pub id: String
}

//////////////////////////////////////////////////////////////////////

// https://developers.google.com/android-publisher/api-ref/rest/v3/edits.bundles
// https://developers.google.com/android-publisher/api-ref/rest/v3/edits.apks
#[derive(Deserialize, Debug)]
pub struct UploadResponseOk{
    #[serde(rename = "versionCode")]
    pub version_code: u64
}
#[derive(Deserialize, Debug)]
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
}

//////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct TrackUpdateResponse{
    pub track: String
}