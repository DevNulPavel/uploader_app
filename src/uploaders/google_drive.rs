use std::{
    path::{
        PathBuf
    }
};
use log::{
    info,
    debug,
    //error
};
use yup_oauth2::{
    read_service_account_key, 
    ServiceAccountAuthenticator
};
use serde_json::{
    Value,
    json
};
use google_drive_client::{
    GoogleDriveClient, 
    GoogleDriveFolder, 
    GoogleDriveUploadResult, 
    GoogleDriveUploadTask
};
use crate::{
    app_parameters::{
        GoogleDriveParams
    },
    env_parameters::{
        GoogleDriveEnvironment
    }
};
use super::{
    upload_result::{
        UploadResult,
        UploadResultData,
        UploadResultMessage
    }
};

//////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct GoogleDriveUploadMessage{
    plain: String,
    blocks: Vec<Value>
}
impl UploadResultMessage for GoogleDriveUploadMessage {
    fn get_slack_blocks(&self) -> &[Value] {
        self.blocks.as_slice()   
    }
    fn get_plain(&self) -> &str {
        &self.plain
    }
}

//////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct DriveUploadResult{
    message: GoogleDriveUploadMessage
}
impl DriveUploadResult{
    fn new(folder: GoogleDriveFolder, files: Vec<GoogleDriveUploadResult>) -> DriveUploadResult {
        // Финальное сообщение
        let files_message = files
            .into_iter()
            .fold("Files:".to_owned(), |prev, res|{
                format!("{}\n- <{}|{}>", prev, res.web_view_link, res.file_name)
            });

        let message = GoogleDriveUploadMessage {
            plain: format!("Google drive uploading finished:\n- {}", folder.get_info().web_view_link),
            blocks: vec![
                json!({
                    "type": "section", 
                    "text": {
                        "type": "mrkdwn", 
                        "text": format!("Google drive folder:\n- <{}|{}>", folder.get_info().web_view_link, folder.get_info().name)
                    }
                }),
                json!({
                    "type": "section", 
                    "text": {
                        "type": "mrkdwn", 
                        "text": files_message
                    }
                }),
            ]
        };
        DriveUploadResult{
            message
        }
    }
}
impl UploadResultData for DriveUploadResult {
    fn get_target(&self) -> &'static str {
        "SSH"   
    }
    fn get_message(&self) -> Option<&dyn UploadResultMessage> {
        Some(&self.message)
    }
    fn get_qr_data(&self) -> Option<&str> {
        None
    }
}

//////////////////////////////////////////////////////////////////

pub async fn upload_in_google_drive(client: reqwest::Client, env_params: GoogleDriveEnvironment, app_params: GoogleDriveParams) -> UploadResult {
    info!("Start google drive uploading");

    // Содержимое Json файлика ключа 
    let key = read_service_account_key(env_params.auth_file)
        .await?;
    
    debug!("Google drive key read success");

    // Аутентификация на основе прочитанного файлика
    let auth = ServiceAccountAuthenticator::builder(key)
          .build()
          .await?;

    debug!("Google drive auth success");
 
    // Add the scopes to the secret and get the token.
    let token = auth
        .token(&["https://www.googleapis.com/auth/drive"])
        .await?;
        
    debug!("Google drive token received");

    // Клиент
    let client = GoogleDriveClient::new(client, token);

    // Целевая папка
    let folder = {
        let folder = client
            .get_folder_for_id(&app_params.target_folder_id)
            .await?
            .ok_or_else(||{
                "Target google drive folder is not found"
            })?;
        if let Some(sub_folder_name) = app_params.target_subfolder_name{
            folder
                .create_subfolder_if_needed(&sub_folder_name)
                .await?
        }else{
            folder
        }
    };

    debug!("Google drive target folder received: {}", folder.get_info().id);

    // Грузим файлы
    let mut results = Vec::with_capacity(app_params.files.len());
    for file_path_str in app_params.files {
        let task = GoogleDriveUploadTask{
            file_path: PathBuf::from(file_path_str),
            owner_domain: app_params.target_owner_email.as_deref(),
            owner_email: app_params.target_owner_email.as_deref(),
            parent_folder: &folder
        };
        let result = client
            .upload(task)
            .await?;
        
        debug!("Google drive uploading result: {:#?}", result);
        results.push(result);
    }

    Ok(Box::new(DriveUploadResult::new(folder, results)))
}