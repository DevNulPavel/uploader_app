use super::upload_result::{UploadResult, UploadResultData};
use crate::{
    app_parameters::AppCenterParams,
    env_parameters::{AppCenterEnvironment, GitEnvironment},
};
use app_center_client::{
    AppCenterBuildGitInfo, AppCenterBuildUploadTask, AppCenterBuildVersionInfo, AppCenterClient,
};
use log::{
    //debug,
    error,
    info,
};
use std::{path::PathBuf, time::Duration};
use tokio::time::sleep;

pub async fn upload_in_app_center(
    http_client: reqwest::Client,
    app_center_env_params: AppCenterEnvironment,
    app_center_app_params: AppCenterParams,
    git_info: Option<GitEnvironment>,
) -> UploadResult {
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
        app_center_env_params.owner.clone(),
    );

    // Информация по Git
    let git_info = git_info.map(|git| AppCenterBuildGitInfo {
        branch: git.git_branch,
        commit: git.git_commit,
    });

    // Инфа по версии
    let version = match (
        app_center_app_params.build_version,
        app_center_app_params.build_code,
    ) {
        (Some(version), Some(code)) => {
            let code = code.parse::<u32>()?;
            Some(AppCenterBuildVersionInfo {
                build_code: code,
                version,
            })
        }
        _ => None,
    };

    // Таска на выгрузку
    let task = AppCenterBuildUploadTask {
        file_path: file_path.as_path(),
        distribution_groups: app_center_app_params.distribution_groups,
        build_description: app_center_app_params.build_description,
        git_info,
        version_info: version,
        upload_threads_count: 5,
    };

    let mut iteration_number = 0_u32;
    loop {
        info!("App center uploading iteration: {}", iteration_number);
        iteration_number += 1;

        // Результат
        let upload_result = app_center_client
            .upload_build(&task)
            .await
            .map_err(Box::new);

        match upload_result {
            // Если все хорошо - возвращаем результат
            Ok(result) => {
                // Финальная ссылка с авторизацией
                let result_url = format!(
                    "https://install.appcenter.ms/orgs/{}/apps/{}/releases/{}",
                    app_center_env_params.owner, result.app_name, result.id
                );

                // Финальное сообщение
                let message = format!(
                    "App Center uploading finished:\n- {}\n  => {}",
                    file_name, result_url
                );

                return Ok(UploadResultData {
                    target: "AppCenter",
                    message: Some(message),
                    install_url: Some(result_url),
                });
            }

            // Если все плохо - делаем несколько попыток c паузой
            Err(err) => {
                error!("App center uploading failed with error: {}", err);

                if iteration_number <= 5 {
                    info!("Wait some time before new iteration");
                    sleep(Duration::from_secs(20)).await;
                } else {
                    return Err(format!("AppCenter uploading failed with error: {}", err).into());
                }
            }
        }
    }
}
