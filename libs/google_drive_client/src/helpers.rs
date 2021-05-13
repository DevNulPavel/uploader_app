use tracing::{
    trace
};
use reqwest::{
    Method,
};
use reqwest_inspect_json::{
    InspectJson
};
use super::{
    request_builder::{
        GoogleDriveRequestBuilder,
    },
    responses::{
        *
    },
    error::{
        GoogleDriveError
    }
};

pub async fn get_files_list_with_query(request_builder: &GoogleDriveRequestBuilder, query: &str, page_token: Option<String>) -> Result<FilesListResponse, GoogleDriveError> {
    // https://developers.google.com/drive/api/v3/reference/files/list
    // https://developers.google.com/drive/api/v3/search-files
    // https://developers.google.com/drive/api/v3/ref-search-terms
    let page_token = page_token.unwrap_or_default();
    let info = request_builder
        .build_request(Method::GET, "drive/v3/files")?
        .query(&[
            ("corpora", "allDrives"),
            ("includeItemsFromAllDrives", "true"),
            ("supportsAllDrives", "true"),
            ("includeTeamDriveItems", "true"),
            ("supportsTeamDrives", "true"),
            ("fields", "nextPageToken,files(id,mimeType,name,webContentLink,webViewLink,parents)"),
            ("pageToken", &page_token),
            ("q", query)
        ])
        .send()
        .await?
        .inspect_json::<FilesListResponse, GoogleDriveError>(|d| { trace!(d, "Info resp") })
        .await?;

    trace!(?info, "Files list response");

    Ok(info)
}