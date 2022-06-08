use crate::{
    app_parameters::AmazonParams,
    env_parameters::AmazonEnvironment,
    uploaders::{UploadResult, UploadResultData},
};
use amazon_client::{request_token, AmazonAccessToken, AmazonClient, AmazonUploadTask};
use log::{debug, error};
use std::path::Path;
use tap::TapFallible;

pub async fn upload_in_amazon(
    http_client: reqwest::Client,
    env_params: AmazonEnvironment,
    app_params: AmazonParams,
) -> UploadResult {
    let token: AmazonAccessToken = request_token(
        &http_client,
        &env_params.client_id,
        &env_params.client_secret,
    )
    .await
    .tap_err(|err| {
        error!("Access token request failed: {err}");
    })?;

    {
        let token_str = token.as_str_checked().tap_err(|err| {
            error!("Invalid token: {err}");
        })?;
        debug!("Amazon token: {token_str}");
    }

    let file_path = Path::new(&app_params.file_path);

    // Грузим
    let client = AmazonClient::new(http_client, token);
    let task = AmazonUploadTask {
        application_id: &env_params.app_id,
        file_path,
    };
    client.upload(task).await.tap_err(|err| {
        error!("Amazon uploading error: {err}");
    })?;

    // Имя файла
    let file_name = file_path
        .file_name()
        .ok_or("Amazon: invalid file name")?
        .to_str()
        .ok_or("Amazon: Invalid file name")?;

    // Финальное сообщение
    let message = format!("Amazon uploading finished:\n- {}", file_name);

    Ok(UploadResultData {
        target: "Amazon",
        message: Some(message),
        install_url: None,
    })
}
