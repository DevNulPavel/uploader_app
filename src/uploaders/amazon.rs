use std::{
    path::{
        Path
    }
};
use log::{
    debug
};
use amazon_client::{
    AmazonClient,
    AmazonUploadTask,
    AmazonAccessToken,
    request_token
};
use crate::{
    app_parameters::{
        AmazonParams
    },
    env_parameters::{
        AmazonEnvironment
    },
    uploaders::{
        UploadResult,
        UploadResultData,
        UploadResultMessage
    }
};

//////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct AmazonUploadMessage{
    text: String
}
impl UploadResultMessage for AmazonUploadMessage {
    fn get_markdown(&self) -> &str {
        &self.text
    }
    fn get_plain(&self) -> &str {
        &self.text
    }
}

//////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct AmazonUploadResult{
    message: AmazonUploadMessage
}
impl UploadResultData for AmazonUploadResult {
    fn get_target(&self) -> &'static str {
        "Amazon"   
    }
    fn get_message(&self) -> Option<&dyn UploadResultMessage> {
        Some(&self.message)
    }
    fn get_qr_data(&self) -> Option<&str> {
        None
    }
}

//////////////////////////////////////////////////////////////////

pub async fn upload_in_amazon(http_client: reqwest::Client, 
                              env_params: AmazonEnvironment, 
                              app_params: AmazonParams) -> UploadResult {

    let token: AmazonAccessToken = request_token(&http_client, &env_params.client_id, &env_params.client_secret)
        .await?;

    let token_str = token
        .as_str_checked()
        .expect("Token string get failed");

    debug!("Amazon token: {:#?}", token_str);

    let file_path = Path::new(&app_params.file_path);

    // Грузим
    let client = AmazonClient::new(http_client, token);
    let task = AmazonUploadTask{
        application_id: &env_params.app_id,
        file_path: file_path
    };
    client
        .upload(task)
        .await?;

    // Имя файла
    let file_name = file_path
        .file_name()
        .ok_or("Amazon: invalid file name")?
        .to_str()
        .ok_or("Amazon: Invalid file name")?;

    // Финальное сообщение
    let res = AmazonUploadResult{
        message: AmazonUploadMessage{
            text: format!("Amazon uploading finished:\n- {}", file_name)
        }
    };

    Ok(Box::new(res))
}