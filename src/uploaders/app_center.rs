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
        UploadResultData
    }
};

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
        app_center_env_params.owner
    );
    
    // Информация по Git
    let git_info = git_info
        .map(|git|{
            AppCenterBuildGitInfo{
                branch: git.git_branch,
                commit: git.git_commit
            }
        });
    
    // Таска на выгрузку
    let task = AppCenterBuildUploadTask{
        file_path: file_path.as_path(),
        distribution_groups: app_center_app_params.distribution_groups,
        build_description: app_center_app_params.build_description,
        git_info,
        upload_threads_count: 5
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
                // Финальное сообщение
                let message = if let Some(url) = result.download_url{
                    format!(
                        "App Center uploading finished:\n- {}\n\nLoading url:\n- {}", 
                        file_name,
                        url
                    )
                }else{
                    format!("App Center uploading finished:\n- {}", file_name)
                };

                return Ok(UploadResultData{
                    target: "AppCenter",
                    message: Some(message),
                    install_url: result.install_url,
                })
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