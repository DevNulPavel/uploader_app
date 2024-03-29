use super::upload_result::{UploadResult, UploadResultData};
use crate::{app_parameters::FacebookInstantParams, env_parameters::FacebookInstantEnvironment};
use facebook_instant_client::FacebookInstantClient;
use log::{error, info};
use std::path::{Path, PathBuf};
use tap::TapFallible;

/// Функция помогалка для получения пути
fn get_file_name(path: &Path) -> Result<&str, &str> {
    let file_name = path
        .file_name()
        .ok_or("Facebook instant: invalid file name")?
        .to_str()
        .ok_or("Facebook instant: Invalid file name")?;
    Ok(file_name)
}

pub async fn upload_in_facebook_instant(
    http_client: reqwest::Client,
    env_params: FacebookInstantEnvironment,
    app_params: FacebookInstantParams,
) -> UploadResult {
    info!("Start facebook uploading");

    // Заранее формируем путь
    let upload_file_path = PathBuf::from(app_params.zip_file_path);

    // Финальное сообщение заранее делаем, так как переменную перемещаем
    let message = format!(
        "Facebook instant games uploading finished:\n- {}\n\n\
        Console url:\n- https://developers.facebook.com/apps/{}/instant-games/hosting/",
        get_file_name(&upload_file_path)?,
        env_params.app_id
    );

    // Создаем клиента
    let client = FacebookInstantClient::new(http_client, env_params.app_id, env_params.app_secret)
        .await
        .tap_err(|err| {
            error!("Facebook instant client create failed with error: {}", err);
        })?;

    // Выгрузка
    client
        .upload(upload_file_path, app_params.commentary)
        .await
        .tap_err(|err| {
            error!("Facebook instant uploading failed with error: {}", err);
        })?;

    Ok(UploadResultData {
        target: "Facebook instant",
        message: Some(message),
        install_url: None,
    })
}
