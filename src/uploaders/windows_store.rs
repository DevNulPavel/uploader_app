use super::upload_result::{UploadResult, UploadResultData};
use crate::{app_parameters::WindowsStoreParams, env_parameters::WindowsStoreEnvironment};
use microsoft_azure_client::MicrosoftAzureClient;
use std::path::Path;
use tap::TapFallible;
use tracing::{error, info, instrument};

#[instrument(skip(http_client, env_params, app_params))]
pub async fn upload_in_windows_store(
    http_client: reqwest::Client,
    env_params: WindowsStoreEnvironment,
    app_params: WindowsStoreParams,
) -> UploadResult {
    info!("Start google play uploading");

    // Создаем клиента
    let client = MicrosoftAzureClient::new(
        http_client,
        env_params.tenant_id,
        env_params.client_id,
        env_params.secret_key,
        env_params.app_id,
    )
    .tap_err(|err| {
        error!("Microsoft Azure client create failed with error: {}", err);
    })?;

    // Файлик выгрузки
    let upload_file_path = Path::new(&app_params.zip_file_path);

    // Делавем попытку выгрузки
    client
        .upload_production_build(
            upload_file_path,
            app_params.test_groups,
            app_params.test_flight_name,
        )
        .await
        .expect("Upload failed");

    let file_name = upload_file_path
        .file_name()
        .ok_or("Google play: invalid file name")?
        .to_str()
        .ok_or("Google play: Invalid file name")?;

    // Финальное сообщение
    let message = format!("Windows store uploading finished:\n- {}", file_name);

    Ok(UploadResultData {
        target: "Windows store",
        message: Some(message),
        install_url: None,
    })
}
