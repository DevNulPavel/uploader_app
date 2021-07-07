use std::{
    string::{
        FromUtf8Error
    },
    path::{
        Path
    },
    io::{
        self,
    },
    fmt::{
        Display,
        Formatter,
        self
    },
    error::{
        self
    }
};
use tokio::{
    process::{
        Command
    }
};
use tracing::{
    debug,
    error
};
use tap::{
    TapFallible
};
use crate::{
    app_parameters::{
        IOSParams
    },
    env_parameters::{
        IOSEnvironment
    },
    uploaders::{
        UploadResult,
        UploadResultData
    }
};

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
enum IOSError{
    FileDoesNotExist(String),
    SpawnFailed(io::Error),
    InvalidSpawn(String),
    ErrorParseFailed(FromUtf8Error),
    OutputParseFailed(FromUtf8Error)
}
impl Display for IOSError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl error::Error for IOSError {
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn upload_in_ios(env_params: IOSEnvironment, app_params: IOSParams) -> UploadResult {
    let _span = tracing::info_span!("upload_in_ios");

    // Проверка наличия файлика
    let path = Path::new(&app_params.ipa_file_path);
    if !path.exists(){
        return Err(Box::new(IOSError::FileDoesNotExist(app_params.ipa_file_path.to_owned())));
    }

    // Имя файла
    let file_name = path
        .file_name()
        .ok_or("iOS: invalid file name")?
        .to_str()
        .ok_or("iOS: Invalid file name")?;

    // Запуск altool
    let child = Command::new("xcrun")
        .args(&[
            "altool", "--upload-app",
            "-f", &app_params.ipa_file_path, 
            "-u", &env_params.user,
            "-p", &env_params.pass
        ])
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::inherit())
        .stdin(std::process::Stdio::null())
        .spawn()
        .map_err(|err| {
            IOSError::SpawnFailed(err)
        })?
        .wait_with_output()
        .await
        .tap_err(|err|{
            error!(%err, "xcrun start failed");
        })?;

    // TODO: Как-то не очень хорошо получается вывод

    // Проверим ошибку
    if child.status.code() != Some(0) {
        let err = String::from_utf8(child.stderr)
            .map_err(|err|{
                IOSError::ErrorParseFailed(err)
            })?;
        let error_text = format!(
            "Spawn failed with code {:?}, stderr: '{}'", 
            child.status.code(),
            err
        );

        return Err(Box::new(IOSError::InvalidSpawn(error_text)));
    }        

    // Получим вывод приложения
    let text = String::from_utf8(child.stdout)
        .map_err(|err|{
            IOSError::OutputParseFailed(err)
        })?;

    debug!(%text, "Uploading util output");

    // Финальное сообщение
    let message = format!("IOS uploading finished:\n- {}", file_name);

    Ok(UploadResultData{
        target: "iOS",
        message: Some(message),
        install_url: None
    })  
}