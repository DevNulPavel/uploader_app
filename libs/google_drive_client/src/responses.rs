use serde::{
    Deserialize
};

// https://developers.google.com/drive/api/v3/reference/files#resource
#[derive(Deserialize, Debug)]
pub struct FilesUploadResponse{
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

// https://developers.google.com/drive/api/v3/reference/files#resource
#[derive(Deserialize, Debug)]
pub struct FilePermissionResponse{
    pub id: String,
    // pub domain: String,
    
    // #[serde(rename = "emailAddress")]
    // pub email_address: String,
}

// https://developers.google.com/drive/api/v3/reference/files#resource
#[derive(Deserialize, Debug)]
pub struct FilesListResponse{
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,

    // #[serde(rename = "incompleteSearch")]
    // pub incomplete_search: bool,

    pub files: Vec<FilesUploadResponse>
}