use std::{
    path::{
        Path
    }
};
use tracing::{
    debug
};
use tap::{
    TapFallible
};
use tracing::{
    error,
    instrument
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
        UploadResultData
    }
};

#[instrument(skip(http_client, env_params, app_params))]
pub async fn upload_in_amazon(http_client: reqwest::Client, 
                              env_params: AmazonEnvironment, 
                              app_params: AmazonParams) -> UploadResult {

    let token: AmazonAccessToken = request_token(&http_client, &env_params.client_id, &env_params.client_secret)
        .await
        .tap_err(|err|{
            error!(%err, "Access token request failed");
        })?;

    {
        let token_str = token
            .as_str_checked()
            .tap_err(|err|{
                error!(%err, "Invalid token");
            })?;
            debug!(%token_str, "Amazon token");
    }

    let file_path = Path::new(&app_params.file_path);

    // Грузим
    let client = AmazonClient::new(http_client, token);
    let task = AmazonUploadTask{
        application_id: &env_params.app_id,
        file_path: file_path
    };
    client
        .upload(task)
        .await
        .tap_err(|err|{
            error!(%err, "Amazon uploading error");
        })?;

    // Имя файла
    let file_name = file_path
        .file_name()
        .ok_or("Amazon: invalid file name")?
        .to_str()
        .ok_or("Amazon: Invalid file name")?;

    // Финальное сообщение
    let message = format!("Amazon uploading finished:\n- {}", file_name);

    Ok(UploadResultData{
        target: "Amazon",
        message: Some(message),
        install_url: None
    })  
}