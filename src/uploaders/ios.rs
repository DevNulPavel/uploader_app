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
use serde_json::{
    Value,
    json
};
use log::{
    debug,
    // error
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
        UploadResultData,
        UploadResultMessage
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

//////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct IOSUploadMessage{
    plain: String,
    blocks: Vec<Value>
}
impl UploadResultMessage for IOSUploadMessage {
    fn get_slack_blocks(&self) -> &[Value] {
        self.blocks.as_slice()   
    }
    fn get_plain(&self) -> &str {
        &self.plain
    }
}

//////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct IOSUploadResult{
    message: IOSUploadMessage
}
impl IOSUploadResult{
    fn new(file_name: String) -> IOSUploadResult {
        let text = format!("IOS uploading finished:\n- {}", file_name);
        let message = IOSUploadMessage{
            plain: text.clone(),
            blocks: vec![
                json!({
                    "type": "section", 
                    "text": {
                        "type": "mrkdwn", 
                        "text": text
                    }
                })
            ]
        };
        IOSUploadResult{
            message
        }
    }
}
impl UploadResultData for IOSUploadResult {
    fn get_target(&self) -> &'static str {
        "SSH"   
    }
    fn get_message(&self) -> Option<&dyn UploadResultMessage> {
        Some(&self.message)
    }
    fn get_qr_data(&self) -> Option<&str> {
        None
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn upload_in_ios(env_params: IOSEnvironment, app_params: IOSParams) -> UploadResult {
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
        .spawn()
        .map_err(|err| IOSError::SpawnFailed(err))?
        .wait_with_output()
        .await?;

    // TODO: Как-то не очень хорошо получается вывод

    // Проверим ошибку
    if child.status.code() != Some(0) {
        let output = String::from_utf8(child.stdout)
            .map_err(|err|{
                IOSError::ErrorParseFailed(err)
            })?;
        let err = String::from_utf8(child.stderr)
            .map_err(|err|{
                IOSError::ErrorParseFailed(err)
            })?;
        let error_text = format!(
            "Spawn failed with code {:?}\nStdOut: '{}'\nStdError: '{}'", 
            child.status.code(),
            output, 
            err
        );

        return Err(Box::new(IOSError::InvalidSpawn(error_text)));
    }        

    // Получим вывод приложения
    let text = String::from_utf8(child.stdout)
        .map_err(|err|{
            IOSError::OutputParseFailed(err)
        })?;

    debug!("Uploading util output: {}", text);

    Ok(Box::new(IOSUploadResult::new(file_name.to_owned())))  
}