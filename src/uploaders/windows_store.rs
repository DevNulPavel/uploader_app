use super::upload_result::{UploadResult, UploadResultData};
use crate::{app_parameters::WindowsStoreParams, env_parameters::WindowsStoreEnvironment};
use microsoft_azure_client::MicrosoftAzureClient;
use std::path::Path;
use tap::TapFallible;
use tracing::{error, info, instrument};

fn get_file_name(path: &Path) -> Result<&str, &str> {
    let file_name = path
        .file_name()
        .ok_or("Microsoft Azure: invalid file name")?
        .to_str()
        .ok_or("Microsoft Azure : Invalid file name")?;
    Ok(file_name)
}

#[instrument(skip(http_client, env_params, app_params))]
pub async fn upload_in_windows_store(
    http_client: reqwest::Client,
    env_params: WindowsStoreEnvironment,
    app_params: WindowsStoreParams,
) -> UploadResult {
    info!("Start windows store uploading");

    // Создаем клиента
    let client = MicrosoftAzureClient::new(
        http_client,
        env_params.tenant_id,
        env_params.client_id,
        env_params.secret_key,
        app_params.app_id,
    )
    .tap_err(|err| {
        error!("Microsoft Azure client create failed with error: {}", err);
    })?;

    let mut messages = Vec::new();

    // Продакшен выгрузка
    if let Some(production_zip) = app_params.production_zip_file_path {
        // Файлик выгрузки
        let upload_file_path = Path::new(&production_zip);

        // Генерация имени выгрузки
        let submission_name = match app_params.production_submission_name {
            Some(name) => name,
            None => format!(
                "Production (UTC: {})",
                chrono::Utc::now().format("%Y-%m-%d %H:%M")
            ),
        };

        // Делавем попытку выгрузки
        client
            .upload_production_build(upload_file_path, submission_name)
            .await
            .tap_err(|err| {
                error!(
                    "Microsoft Azure production uploading failed with error: {}",
                    err
                );
            })?;

        // Финальное сообщение
        messages.push(format!(
            "Windows store production uploading finished:\n- {}",
            get_file_name(upload_file_path)?
        ));
    }

    // Тестовая выгрузка
    if let (Some(test_zip), Some(groups)) = (
        app_params.test_flight_zip_file_path,
        app_params.test_flight_groups,
    ) {
        // Файлик выгрузки
        let upload_file_path = Path::new(&test_zip);

        // Генерация имени выгрузки
        let flight_name = match app_params.test_flight_name {
            Some(name) => name,
            None => format!(
                "Test (UTC: {})",
                chrono::Utc::now().format("%Y-%m-%d %H:%M")
            ),
        };

        // Делавем попытку выгрузки
        client
            .upload_flight_build(upload_file_path, groups, flight_name)
            .await
            .tap_err(|err| {
                error!("Microsoft Azure test uploading failed with error: {}", err);
            })?;

        messages.push(format!(
            "Windows store test uploading finished:\n- {}",
            get_file_name(upload_file_path)?
        ));
    }

    Ok(UploadResultData {
        target: "Windows store",
        message: Some(messages.join("\n\n")),
        install_url: None,
    })
}
