use super::upload_result::{UploadResult, UploadResultData};
use crate::{app_parameters::GoogleDriveParams, env_parameters::GoogleDriveEnvironment};
use google_drive_client::{GoogleDriveClient, GoogleDriveUploadTask};
use std::{path::PathBuf, time::Duration};
use tap::TapFallible;
use tracing::{debug, error, info, instrument};
use yup_oauth2::{read_service_account_key, ServiceAccountAuthenticator};

#[instrument(skip(client, env_params, app_params))]
pub async fn upload_in_google_drive(
    client: reqwest::Client,
    env_params: GoogleDriveEnvironment,
    app_params: GoogleDriveParams,
) -> UploadResult {
    info!("Start google drive uploading");

    // Содержимое Json файлика ключа
    let key = read_service_account_key(env_params.auth_file)
        .await
        .tap_err(|err| {
            error!(%err, "Credentials read failed");
        })?;
    info!("Google drive key read success");

    // Аутентификация на основе прочитанного файлика
    let auth = ServiceAccountAuthenticator::builder(key)
        .build()
        .await
        .tap_err(|err| {
            error!(%err, "Service account build failed");
        })?;
    info!("Google drive auth success");

    // Add scopes to the secret and get the token.
    let token = auth
        .token(&["https://www.googleapis.com/auth/drive"])
        .await
        .tap_err(|err| {
            error!(%err, "Token receive failed");
        })?;
    info!("Google drive token received");

    // Клиент
    let client = GoogleDriveClient::new(client, token);

    // Целевая папка
    let folder = {
        let folder = client
            .get_folder_for_id(&app_params.target_folder_id)
            .await?
            .ok_or("Target google drive folder is not found")
            .tap_err(|err| {
                error!(%err, "Folder find failed");
            })?;
        if let Some(sub_folder_name) = app_params.target_subfolder_name {
            folder
                .create_subfolder_if_needed(&sub_folder_name)
                .await
                .tap_err(|err| {
                    error!(%err, "Subfolder create failed");
                })?
        } else {
            folder
        }
    };
    debug!(folder_id = %folder.get_info().id, "Target folder received");

    // Грузим файлы
    let mut results = Vec::with_capacity(app_params.files.len());
    for file_path_str in app_params.files {
        let task = GoogleDriveUploadTask {
            file_path: PathBuf::from(file_path_str),
            owner_domain: app_params.target_owner_email.as_deref(),
            owner_email: app_params.target_owner_email.as_deref(),
            parent_folder: &folder,
        };

        // Делаем 3 попытки повторной выгрузки файлика с паузой в 20 секунд
        let mut current_retry_count = 0;
        let result = loop {
            match client.upload(&task).await {
                Ok(result) => {
                    break result;
                }
                Err(err) => {
                    error!(%err, "Upload failed");
                    if current_retry_count < 3 {
                        current_retry_count += 1;
                        tokio::time::sleep(Duration::from_secs(20)).await;
                    } else {
                        return UploadResult::Err(Box::new(err));
                    }
                }
            }
        };

        debug!(?result, "Google drive uploading result");
        results.push(result);
    }

    // Финальное сообщение
    let message_begin = format!(
        "Google drive folder:\n- {}\n  => {}\n\nFiles:",
        folder.get_info().name,
        folder.get_info().web_view_link
    );
    let message = results.into_iter().fold(message_begin, |prev, res| {
        format!("{}\n- {}\n  => {}", prev, res.file_name, res.web_view_link)
    });

    Ok(UploadResultData {
        target: "Google drive",
        message: Some(message),
        install_url: None,
    })
}
