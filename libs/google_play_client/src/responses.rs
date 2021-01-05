use serde::{
    Deserialize
};

// https://developers.google.com/play/api/v3/reference/files#resource
#[derive(Deserialize, Debug)]
pub struct AppEditResponse{
    pub id: String,
    // expiryTimeSeconds: String
}

// https://developers.google.com/android-publisher/api-ref/rest/v3/edits.bundles
// https://developers.google.com/android-publisher/api-ref/rest/v3/edits.apks
#[derive(Deserialize, Debug)]
pub struct UploadResponse{
    // #[serde(rename = "versionCode")]
    pub version_code: u64
}


#[derive(Deserialize, Debug)]
pub struct TrackUpdateResponse{
    pub track: String
}

// #[serde(rename = "webContentLink")]
// pub web_content_link: Option<String>
