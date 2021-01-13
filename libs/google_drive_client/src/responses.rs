use std::{
    collections::{
        HashMap
    }
};
use serde::{
    Deserialize
};
use serde_json::{
    Value
};

#[derive(Deserialize, Debug)]
pub struct ResponseErrorValue{
    pub code: i64,
    pub message: String,
    // #[serde(flatten)]
    // other: HashMap<String, Value>
}
#[derive(Deserialize, Debug)]
pub struct ResponseErr{
    pub error: ResponseErrorValue,
}

///////////////////////////////////////////////////////////////////////////////////////////

// https://developers.google.com/drive/api/v3/reference/files#resource
#[derive(Deserialize, Debug)]
pub struct FilesUploadResponseOk{
    pub id: String,
    pub name: String,
    pub parents: Option<Vec<String>>,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(rename = "webViewLink")]
    pub web_view_link: String,
    #[serde(rename = "webContentLink")]
    pub web_content_link: Option<String>
}
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum FilesUploadResponse{
    Ok(FilesUploadResponseOk),
    Error(ResponseErr)
}
impl into_result::IntoResult<FilesUploadResponseOk, ResponseErr> for FilesUploadResponse{
    fn into_result(self) -> Result<FilesUploadResponseOk, ResponseErr> {
        match self {
            FilesUploadResponse::Ok(ok) => Ok(ok),
            FilesUploadResponse::Error(err) => Err(err)
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////

// https://developers.google.com/drive/api/v3/reference/files#resource
#[derive(Deserialize, Debug)]
pub struct FilePermissionResponse{
    pub id: String,
    // pub domain: String,
    
    // #[serde(rename = "emailAddress")]
    // pub email_address: String,
}

///////////////////////////////////////////////////////////////////////////////////////////

// https://developers.google.com/drive/api/v3/reference/files#resource
#[derive(Deserialize, Debug)]
pub struct FilesListResponse{
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,

    // #[serde(rename = "incompleteSearch")]
    // pub incomplete_search: bool,

    pub files: Vec<FilesUploadResponseOk>
}