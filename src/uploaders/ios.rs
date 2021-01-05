use std::{
    path::{
        Path
    }
};
use tokio::{
    process::{
        Child,
        Command
    }
};
use log::{
    debug
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


pub async fn upload_in_ios(env_params: IOSEnvironment, 
                           app_params: IOSParams) -> UploadResult {

    // Имя файла
    // let file_name = file_path
    //     .file_name()
    //     .ok_or("iOS: invalid file name")?
    //     .to_str()
    //     .ok_or("iOS: Invalid file name")?;

    // Финальное сообщение
    let message = format!("Amazon uploading finished:\n- {}", file_name);

    Ok(UploadResultData{
        target: "iOS",
        message: Some(message),
        install_url: None
    })  
}