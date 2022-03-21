use crate::{
    app_parameters::IOSParams,
    env_parameters::IOSEnvironment,
    uploaders::{UploadResult, UploadResultData},
};
use std::{
    error::{self},
    fmt::{self, Display, Formatter},
    io::{self},
    path::Path,
    string::FromUtf8Error,
    time::Duration,
};
use tap::TapFallible;
use tokio::{process::Command, time::delay_for};
use tracing::{debug, error, instrument};

/*

#!/bin/bash -ex

ALTOOL_PATH="xcrun altool"
IPA_DIR="$HOME/IPAs_build"
IPA_PATH="$IPA_DIR/$IPA_NAME"
ls -lat "$IPA_DIR"
#| grep ".ipa" | true
if [ -f $IPA_PATH ]; then
  $ALTOOL_PATH --upload-app -f "$IPA_PATH" -u $USER -p $PASS
fi
*/
////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum IOSError {
    FileDoesNotExist(String),
    SpawnFailed(io::Error),
    InvalidSpawn(String),
    ErrorParseFailed(FromUtf8Error),
    OutputParseFailed(FromUtf8Error),
}
impl Display for IOSError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl error::Error for IOSError {}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[instrument(skip(env_params, app_params))]
pub async fn upload_in_ios(env_params: IOSEnvironment, app_params: IOSParams) -> UploadResult {
    let _span = tracing::info_span!("upload_in_ios");

    // Проверка наличия файлика
    let path = Path::new(&app_params.ipa_file_path);
    if !path.exists() {
        return UploadResult::Err(Box::new(IOSError::FileDoesNotExist(
            app_params.ipa_file_path.to_owned(),
        )));
    }

    // Имя файла
    let file_name = path
        .file_name()
        .ok_or("iOS: invalid file name")?
        .to_str()
        .ok_or("iOS: Invalid file name")?;

    // Объект комманды
    #[rustfmt::skip]
    let mut command = {
        let mut command = Command::new("xcrun");
        command
            .args(&[
                "altool", "--upload-app",
                "-f", &app_params.ipa_file_path, 
                "-t", "ios",
                "-u", &env_params.user,
                "-p", &env_params.pass
            ])
            .stderr(std::process::Stdio::piped())
            .stdout(std::process::Stdio::inherit())
            .stdin(std::process::Stdio::null());
        command
    };

    // Делаем 3 попытки с паузой
    const ITER_COUNT: u8 = 3;
    let mut i = 0;
    let output = loop {
        i += 1;

        // Запуск altool
        let output = command
            .spawn()
            .map_err(|err| Box::new(IOSError::SpawnFailed(err)))?
            .wait_with_output()
            .await
            .tap_err(|err| {
                error!(%err, "xcrun start failed");
            })?;

        // Проверим ошибку
        if output.status.code() != Some(0) {
            let err = String::from_utf8(output.stderr)
                .map_err(|err| Box::new(IOSError::ErrorParseFailed(err)))?;
            let error_text = format!(
                "Spawn failed with code {:?}, stderr: '{err}'",
                output.status.code()
            );

            // Было ли превышено количество итераций?
            if i > ITER_COUNT {
                return UploadResult::Err(Box::new(IOSError::InvalidSpawn(error_text)));
            } else {
                // Пока что выводим ошибку как есть
                error!("{error_text}");

                // Подождем 10 секунд, может быть следующая итерация будет успешной
                delay_for(Duration::from_secs(10)).await;
            }
        } else {
            break output;
        }
    };

    // Получим вывод приложения
    let text = String::from_utf8(output.stdout)
        .map_err(|err| Box::new(IOSError::OutputParseFailed(err)))?;
    debug!(%text, "Uploading util output");

    // Финальное сообщение
    let message = format!("IOS uploading finished:\n- {}", file_name);

    Ok(UploadResultData {
        target: "iOS",
        message: Some(message),
        install_url: None,
    })
}
