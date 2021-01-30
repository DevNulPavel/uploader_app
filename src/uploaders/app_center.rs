use std::{
    time::{
        Duration
    },
    path::{
        PathBuf
    }
};
use tokio::{
    time::{
        delay_for
    }
};
use log::{
    info,
    //debug,
    error
};
use app_center_client::{
    AppCenterClient,
    AppCenterBuildGitInfo,
    AppCenterBuildVersionInfo,
    AppCenterBuildUploadTask,
};
use crate::{
    app_parameters::{
        AppCenterParams
    },
    env_parameters::{
        AppCenterEnvironment,
        GitEnvironment
    }
};
use super::{
    upload_result::{
        UploadResult,
        UploadResultData,
        UploadResultMessage
    }
};

//////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct AppCenterUploadMessage{
    markdown: String,
    plain: String,
}
impl UploadResultMessage for AppCenterUploadMessage {
    fn get_markdown(&self) -> &str {
        &self.markdown
    }
    fn get_plain(&self) -> &str {
        &self.plain
    }
}

//////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct AppCenterUploadResult {
    message: AppCenterUploadMessage,
    qr_data: String
}
impl AppCenterUploadResult{
    fn new(file_name: String, result_url: String) -> AppCenterUploadResult {
        let markdown = format!(
            "App Center uploading finished:\n- [{}]({})", 
            file_name, 
            result_url
        );
        let plain = format!(
            "App Center uploading finished:\n- file: {}\n- url: {}", 
            file_name, 
            result_url
        );
        let message = AppCenterUploadMessage{
            markdown,
            plain 
        };
        AppCenterUploadResult{
            message,
            qr_data: result_url
        }
    }
}
impl UploadResultData for AppCenterUploadResult {
    fn get_target(&self) -> &'static str {
        "AppCenter"   
    }
    fn get_message(&self) -> Option<&dyn UploadResultMessage> {
        Some(&self.message)
    }
    fn get_qr_data(&self) -> Option<&str> {
        Some(&self.qr_data)
    }
}

//////////////////////////////////////////////////////////////////

pub async fn upload_in_app_center(http_client: reqwest::Client, 
                                  app_center_env_params: AppCenterEnvironment,
                                  app_center_app_params: AppCenterParams,
                                  git_info: Option<GitEnvironment>) -> UploadResult {

    info!("Start app center uploading");

    let file_path: PathBuf = app_center_app_params.input_file.into();

    let file_name = file_path
        .file_name()
        .ok_or("App center: invalid file name")?
        .to_str()
        .ok_or("App center: Invalid file name")?;

    // Создаем клиента
    let app_center_client = AppCenterClient::new(
        http_client.clone(), 
        app_center_env_params.token, 
        app_center_env_params.app, 
        app_center_env_params.owner.clone()
    );
    
    // Информация по Git
    let git_info = git_info
        .map(|git|{
            AppCenterBuildGitInfo{
                branch: git.git_branch,
                commit: git.git_commit
            }
        });
        
    // Инфа по версии
    let version = match (app_center_app_params.build_version, app_center_app_params.build_code) {
        (Some(version), Some(code)) => {
            let code = code.parse::<u32>()?;
            Some(AppCenterBuildVersionInfo{
                build_code: code,
                version
            })
        },
        _ => {
            None
        }
    };
    
    // Таска на выгрузку
    let task = AppCenterBuildUploadTask{
        file_path: file_path.as_path(),
        distribution_groups: app_center_app_params.distribution_groups,
        build_description: app_center_app_params.build_description,
        git_info,
        version_info: version,
        upload_threads_count: 10
    };

    let mut iteration_number = 0_u32;
    loop {
        info!("App center uploading iteration number: {}", iteration_number);
        iteration_number += 1;

        // Результат
        let upload_result = app_center_client
            .upload_build(&task)
            .await
            .map_err(|err| Box::new(err));

        match upload_result {
            // Если все хорошо - возвращаем результат
            Ok(result) => {
                // Финальная ссылка с авторизацией
                let result_url = format!(
                    "https://install.appcenter.ms/orgs/{}/apps/{}/releases/{}",
                    app_center_env_params.owner,
                    result.app_name,
                    result.id
                );

                // Финальное сообщение
                let res = AppCenterUploadResult::new(file_name.to_owned(), result_url);

                return Ok(Box::new(res))
            },

            // Если все плохо - делаем несколько попыток c паузой
            Err(err) => {
                error!("App center uploading failed with error: {}", err);

                if iteration_number <= 5 {
                    info!("Wait some time before new iteration");
                    delay_for(Duration::from_secs(20)).await;
                }else{
                    return Err(format!("AppCenter uploading failed with error: {}", err).into());
                }
            }
        }
    }
}