use std::{
    path::{
        Path
    }
};
use tracing::{
    info,
    debug,
    error,
    instrument
};
use tap::{
    TapFallible
};
use yup_oauth2::{
    read_service_account_key, 
    ServiceAccountAuthenticator
};
use google_play_client::{
    GooglePlayClient,
    GooglePlayUploadTask
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
        UploadResultData
    }
};

#[instrument(skip(client, env_params, app_params))]
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
        .tap_err(|err|{
            error!(%err, "Token receive failed");
        })?;
    debug!(?token, "Token received");

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
        .await
        .tap_err(|err|{
            error!(%err, "Uploading failed");
        })?;

    debug!(uploaded_version, "Google play: uploaded version");

    let file_name = path
        .file_name()
        .ok_or("Google play: invalid file name")?
        .to_str()
        .ok_or("Google play: Invalid file name")?;

    // Финальное сообщение
    let message = format!("Google play uploading finished:\n- {}", file_name);

    Ok(UploadResultData{
        target: "Google play",
        message: Some(message),
        install_url: None
    })
}