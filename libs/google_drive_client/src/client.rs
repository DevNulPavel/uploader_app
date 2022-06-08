use std::{
    path::{
        Path, 
        PathBuf
    }
};
use into_result::{
    IntoResult
};
use tracing::{
    debug, 
    trace,
    info,
    instrument
};
use tokio::{
    fs::{
        File
    }
};
use reqwest_inspect_json::{
    InspectJson
};
use tokio_util::{
    codec::{
        BytesCodec,
        FramedRead
    }
};
use yup_oauth2::{
    AccessToken
};
use reqwest::{
    Client,
    Method,
    Body,
    multipart::{
        Form,
        Part
    }
};
use serde_json::{
    json
};
use super::{
    request_builder::{
        GoogleDriveRequestBuilder,
    },
    file::{
        GoogleDriveFile,
        DomainFileOwner,
        EmailFileOwner
    },
    folder::{
        GoogleDriveFolder
    },
    helpers::{
        get_files_list_with_query
    },
    responses::{
        *
    },
    error::{
        GoogleDriveError
    }
};

//////////////////////////////////////////////////////////////////////////////////////////

// #[derive(Clone)]
pub struct GoogleDriveUploadTask<'a>{
    pub file_path: PathBuf,
    pub parent_folder: &'a GoogleDriveFolder,
    pub owner_email: Option<&'a str>,
    pub owner_domain: Option<&'a str>
}

#[derive(Debug)]
pub struct GoogleDriveUploadResult{
    pub file_name: String,
    pub web_view_link: String,
    pub web_content_link: String
}

//////////////////////////////////////////////////////////////////////////////////////////

pub struct GoogleDriveClient{
    request_builder: GoogleDriveRequestBuilder
}
impl GoogleDriveClient {
    pub fn new(http_client: Client,
               token: AccessToken) -> GoogleDriveClient {

        let request_builder = GoogleDriveRequestBuilder::new(http_client, token)
            .expect("Google drive client create failed");

        info!("Google drive request builder created");

        GoogleDriveClient{
            request_builder
        }
    }

    #[instrument(skip(self, parent_folder, file_path))]
    async fn upload_file(&self, parent_folder: &GoogleDriveFolder, file_path: &Path) -> Result<GoogleDriveFile, GoogleDriveError> {
        // https://developers.google.com/drive/api/v3/reference/files/create
        // https://developers.google.com/drive/api/v3/manage-uploads

        let file_name = file_path
            .file_name()
            .ok_or(GoogleDriveError::WrongFilePath)?
            .to_str()
            .ok_or(GoogleDriveError::WrongFilePath)?;
        debug!(%file_name, "File name");
        
        // let mut total_uploaded = 0;
        // let file_name_stream = file_name.to_owned();

        let file = File::open(file_path).await?;
        let file_length = file.metadata().await?.len();
        let reader = FramedRead::new(file, BytesCodec::new())
            /*.map(move |v| {
                if let Ok(ref v) = v{
                    total_uploaded += v.len();
                    info!("Uploaded {}: {}/{}", file_name_stream, total_uploaded, file_length);
                }
                v
            })*/;
        let body = Body::wrap_stream(reader);

        // Первой секцией идет метаинформация в формате json
        let meta = json!({
            "name": file_name.to_owned(),
            "parents": [
                parent_folder.get_info().id.to_owned()
            ]
        }).to_string();

        let multipart = Form::new()
            .part("meta", Part::text(meta)
                    .mime_str("application/json; charset=UTF-8")
                    .expect("Meta set failed"))
            .part("body", Part::stream_with_length(body, file_length)
                    .file_name(file_name.to_owned()));

        let info = self.request_builder
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
            .inspect_json::<FilesUploadResponse, GoogleDriveError>(|data|{
                trace!(?data, "Files response");
            })
            .await?
            .into_result()?;

        trace!(?info, "Uploading response", );

        Ok(GoogleDriveFile::new(self.request_builder.clone(), info))
    }

    #[instrument(skip(self, folder_id))]
    pub async fn get_folder_for_id(&self, folder_id: &str) -> Result<Option<GoogleDriveFolder>, GoogleDriveError> {
        // TODO: Указание конкретного ID не работает почему-то
        // let query = format!("(mimeType = 'application/vnd.google-apps.folder') and \
        //                      (files.id = '{}')", folder_id);

        let query = "(mimeType = 'application/vnd.google-apps.folder')";
        let mut found_list = get_files_list_with_query(&self.request_builder, query, None)
            .await?;

        while !found_list.files.is_empty() {
            let found = found_list
                .files
                .into_iter()
                .find(|info|{
                    info.id.eq(folder_id)
                });

            // Нашли - возврат
            if let Some(info) = found {
                return Ok(Some(GoogleDriveFolder::new(self.request_builder.clone(), info)));
            }

            // Иначе заново запрашиваем новую страницу
            if found_list.next_page_token.is_some(){
                found_list = get_files_list_with_query(&self.request_builder, query, found_list.next_page_token)
                    .await?;
            }else{
                info!("Folder search finished, next token is empty");
                break;
            }
        }

        Ok(None)
    }

    #[instrument(skip(self, task))]
    pub async fn upload(&self, task: &GoogleDriveUploadTask<'_>) -> Result<GoogleDriveUploadResult, GoogleDriveError> {
        info!("Before upload");

        // Выгрузка файлика
        let upload_res = self
            .upload_file(task.parent_folder, &task.file_path)
            .await?;

        debug!(res = ?upload_res.get_info(), "Upload res");

        // Смена владельца
        if let Some(email) = task.owner_email{
            upload_res
                .update_owner(EmailFileOwner::new(email))
                .await?;
        }else if let Some(domain) = task.owner_domain{
            upload_res
                .update_owner(DomainFileOwner::new(domain))
                .await?;
        }else{
            return Err(GoogleDriveError::EmptyNewOwner);
        }

        // Результат
        let FilesUploadResponseOk { name, web_view_link, web_content_link, .. } = upload_res.into();
        match web_content_link {
            Some(web_content_link) => {
                Ok(GoogleDriveUploadResult{
                    file_name: name,
                    web_content_link,
                    web_view_link
                })
            },
            _ => {
                Err(GoogleDriveError::Custom(tracing_error::SpanTrace::capture(), "Web view link is missing".to_owned()))
            }
        }
    }
}