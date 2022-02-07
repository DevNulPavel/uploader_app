use reqwest::{
    Method,
    multipart::{
        Form,
        Part
    }
};
use into_result::{
    IntoResult
};
use tracing::{
    debug
};
use serde_json::{
    json,
};
use reqwest_inspect_json::{
    InspectJson
};
use super::{
    request_builder::{
        GoogleDriveRequestBuilder
    },
    responses::{
        FilesUploadResponse,
        FilesUploadResponseOk,
    },
    helpers::{
        get_files_list_with_query
    },
    error::{
        GoogleDriveError
    }
};

//////////////////////////////////////////////////////////////////////////////

pub struct GoogleDriveFolder{
    request_builder: GoogleDriveRequestBuilder,
    info: FilesUploadResponseOk
}
impl From<GoogleDriveFolder> for FilesUploadResponseOk {
    fn from(file: GoogleDriveFolder) -> Self {
        file.info
    }
}
impl GoogleDriveFolder {
    pub(super) fn new(request_builder: GoogleDriveRequestBuilder, info: FilesUploadResponseOk) -> GoogleDriveFolder{
        GoogleDriveFolder{
            request_builder,
            info
        }
    }
    
    pub fn get_info(&self) -> &FilesUploadResponseOk{
        &self.info
    }

    async fn find_sub_folder_id_for_name(&self, subfolder_name: &str) -> Result<Option<GoogleDriveFolder>, GoogleDriveError>{
        // https://developers.google.com/drive/api/v3/search-files
        let query = format!("(mimeType = 'application/vnd.google-apps.folder') and \
                             ('{}' in parents) and \
                             (name = '{}')", self.info.id, subfolder_name);

        let mut files_list = get_files_list_with_query(&self.request_builder, &query, None)
            .await?;
        
        while !files_list.files.is_empty() {
            // Обходим найденный список
            // TODO: Можно было бы убрать фильтрацию тут
            let found_res = files_list
                .files
                .into_iter()
                .find(|val|{
                    let parents = match val.parents{
                        Some(ref parents) => parents,
                        None => return false
                    };

                    val.mime_type.eq("application/vnd.google-apps.folder") &&
                    val.name.eq(subfolder_name) &&
                    parents.iter().any(|id| id.as_str().eq(self.info.id.as_str()))
                });

            // Нашли - возврат
            if let Some(found) = found_res{
                return Ok(Some(GoogleDriveFolder::new(self.request_builder.clone(), found)));
            }

            // Иначе заново запрашиваем новую страницу
            if files_list.next_page_token.is_some(){
                files_list = get_files_list_with_query(&self.request_builder, &query, files_list.next_page_token,)
                    .await?;
            }else{
                break;
            }
        }

        Ok(None)
    }

    async fn create_sub_folder(&self, subfolder_name: &str) -> Result<GoogleDriveFolder, GoogleDriveError>{
        // https://developers.google.com/drive/api/v3/reference/files/create
        let parent_id = &self.info.id;
        let meta = json!({
            "name": subfolder_name,
            "parents": [
                parent_id
            ],
            "mimeType": "application/vnd.google-apps.folder"
        }).to_string();

        let multipart = Form::new()
            .part("meta", Part::text(meta)
                    .mime_str("application/json; charset=UTF-8")
                    .expect("Meta set failed"));

        let response = self.request_builder
            .build_request(Method::POST, "upload/drive/v3/files")?
            .query(&[
                ("uploadType", "multipart"),
                ("supportsAllDrives", "true"),
                ("includeTeamDriveItems", "true"),
                ("supportsTeamDrives", "true"),
                ("fields", "id,parents,owners,name,size,mimeType,webContentLink,webViewLink")
            ])
            .multipart(multipart)
            .send()
            .await?
            .inspect_json::<FilesUploadResponse, GoogleDriveError>(|d| { debug!("{}", d) })
            .await?
            .into_result()?;

        Ok(GoogleDriveFolder::new(self.request_builder.clone(), response))
    }

    pub async fn create_subfolder_if_needed(&self, subfolder_name: &str) -> Result<GoogleDriveFolder, GoogleDriveError> {
        let found_sub_folder_id = self
            .find_sub_folder_id_for_name(&*subfolder_name)
            .await?;

        if let Some(found) = found_sub_folder_id{
            debug!(id = ?found.get_info(), "Found subfolder id");
            Ok(found)
        }else{
            return self.create_sub_folder(subfolder_name).await;
        }
    }
}