use std::{
    path::{
        Path
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
use google_play_client::{
    GooglePlayClient,
    GooglePlayUploadTask
};
use serde_json::{
    Value,
    json
};
use crate::{
    app_parameters::{
        GooglePlayParams
    },
    env_parameters::{
        GooglePlayEnvironment
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
struct GooglePlayUploadMessage{
    plain: String,
    blocks: Vec<Value>
}
impl UploadResultMessage for GooglePlayUploadMessage {
    fn get_slack_blocks(&self) -> &[Value] {
        self.blocks.as_slice()   
    }
    fn get_plain(&self) -> &str {
        &self.plain
    }
}

//////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct GooglePlayUploadResult{
    message: GooglePlayUploadMessage
}
impl GooglePlayUploadResult{
    fn new(file_name: String) -> GooglePlayUploadResult {
        let text = format!("Google play uploading finished:\n- {}", file_name);
        let message = GooglePlayUploadMessage {
            plain: text.clone(),
            blocks: vec![
                json!({
                    "type": "section", 
                    "text": {
                        "type": "mrkdwn", 
                        "text": text
                    }
                })
            ]
        };
        GooglePlayUploadResult{
            message
        }
    }
}
impl UploadResultData for GooglePlayUploadResult {
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

pub async fn upload_in_google_play(client: reqwest::Client, 
                                   env_params: GooglePlayEnvironment, 
                                   app_params: GooglePlayParams) -> UploadResult {
    info!("Start google play uploading");

    // Содержимое Json файлика ключа 
    let key = read_service_account_key(env_params.auth_file)
        .await
        .expect("Google play auth file parsing failed");

    // Аутентификация на основе прочитанного файлика
    let auth = ServiceAccountAuthenticator::builder(key)
          .build()
          .await
          .expect("Failed to create google play authenticator");
 
    // Add the scopes to the secret and get the token.
    let token = auth
        .token(&["https://www.googleapis.com/auth/androidpublisher"])
        .await
        .expect("Failed to get google play token");

    // Клиент
    let client = GooglePlayClient::new(client, token);

    // Грузим файлы
    let path = Path::new(app_params.file_path.as_str());
    let task = GooglePlayUploadTask{
        file_path: &path,
        target_track: app_params.target_track.as_deref(),
        package_name: app_params.package_name.as_str()
    };
    let uploaded_version = client
        .upload(task)
        .await?;

    debug!("Google play: uploaded version {}", uploaded_version);

    let file_name = path
        .file_name()
        .ok_or("Google play: invalid file name")?
        .to_str()
        .ok_or("Google play: Invalid file name")?;

    Ok(Box::new(GooglePlayUploadResult::new(file_name.to_owned())))
}