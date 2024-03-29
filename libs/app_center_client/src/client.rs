use std::{
    path::{
        Path
    }
};
use log::{
    debug, 
    info
};
use serde_json::{
    Value
};
use reqwest::{
    Client,
    Method
};
use reqwest_inspect_json::{
    InspectJson
};
use serde_json::{
    json
};
use super::{
    request_builder::{
        AppCenterRequestBuilder,
        AppCenterUrlTarget::{
            // API,
            Application
        }
    },
    responses::{
        *
    },
    // bytes_uploader::{
    //     AppCenterUploader
    // },
    // file_stream_uploader::{
    //     AppCenterUploader
    // },
    hyper_uploader::{
        AppCenterUploader
    },
    error::{
        AppCenterError
    }
};

//////////////////////////////////////////////////////////////////////////////////////////

pub struct AppCenterBuildVersionInfo{
    pub version: String,
    pub build_code: u32
}

pub struct AppCenterBuildGitInfo{
    pub branch: String,
    pub commit: String
}

pub struct AppCenterBuildUploadTask<'a>{
    pub file_path: &'a Path,
    pub distribution_groups: Option<Vec<String>>,
    pub build_description: Option<String>,
    pub version_info: Option<AppCenterBuildVersionInfo>,
    pub git_info: Option<AppCenterBuildGitInfo>,
    pub upload_threads_count: usize
}

//////////////////////////////////////////////////////////////////////////////////////////

pub struct AppCenterClient{
    request_builder: AppCenterRequestBuilder
}
impl AppCenterClient {
    pub fn new(http_client: Client,
               token: String,
               app_name: String,
               app_owner: String) -> AppCenterClient {

        let request_builder = AppCenterRequestBuilder::new(http_client, 
                                                           token, 
                                                           "https://api.appcenter.ms/v0.1/",
                                                           &app_owner,
                                                           &app_name)
            .expect("App center client create failed");

        info!("App center request builder created");

        AppCenterClient{
            request_builder
        }
    }

    async fn initialize_release(&self, version: &Option<AppCenterBuildVersionInfo>) -> Result<ReleasesResponse, AppCenterError>{
        // Базовый запрос
        let req = self
            .request_builder
            .build_request(Application, Method::POST, "uploads/releases", true)?;
        
        // Закидываем версию
        let req = if let Some(v) = version{
            let json_data = json!({
                "build_version": v.version,
                "build_number": format!("{}", v.build_code)
            });
            req
                .json(&json_data)
        }else{
            req
        };
        
        // Выполняем
        let releases_resp = req
            .send()
            .await?
            .error_for_status()?
            .inspect_json::<ReleasesResponse, AppCenterError>(|d| { 
                debug!("{}", d); 
            })
            .await?;
        
        debug!("App center releases response: {:#?}", releases_resp);

        Ok(releases_resp)
    }

    async fn set_upload_finished_status(&self, release_info: &ReleasesResponse) -> Result<UploadingFinishedSetStatusResponse, AppCenterError>{
        let path = format!("uploads/releases/{}", release_info.id);
        let update_status_resp = self.request_builder
            .build_request(Application, Method::PATCH, &path, true)?
            .json(&json!({
                "upload_status": "uploadFinished",
                "id": release_info.id
            }))
            .send()
            .await?
            .error_for_status()?
            .inspect_json::<UploadingFinishedSetStatusResponse, AppCenterError>(|d| { 
                debug!("{}", d); 
            })
            .await?;

        debug!("Update status response: {:#?}", update_status_resp);

        Ok(update_status_resp)
    }

    async fn wait_release_id(&self, release_info: &ReleasesResponse) -> Result<u64, AppCenterError>{
        debug!("Await uploading finished status set");

        let path = format!("uploads/releases/{}", release_info.id);

        loop {
            let update_status_resp = self.request_builder
                .build_request(Application, Method::GET, &path, true)?
                .send()
                .await?
                .error_for_status()?         
                .inspect_json::<UploadingFinishedGetStatusResponse, AppCenterError>(|d| { 
                    debug!("{}", d); 
                })
                .await?;

            debug!("Update status response: {:#?}", update_status_resp);

            match update_status_resp{
                UploadingFinishedGetStatusResponse::Ready{release_distinct_id, ..} => {
                    return Ok(release_distinct_id)
                },
                UploadingFinishedGetStatusResponse::Waiting{..} => {
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                },
                UploadingFinishedGetStatusResponse::Error{error_details, ..} => {
                    return Err(AppCenterError::ReleaseIdReceiveFailed(error_details));
                }
                val @ UploadingFinishedGetStatusResponse::Unknown(_) => {
                    return Err(AppCenterError::ReleaseIdReceiveFailed(format!("{:#?}", val)));
                }
            }
        }
    }

    async fn update_build_meta(&self, 
                               release_id: u64,
                               task: &AppCenterBuildUploadTask<'_>) -> Result<(), AppCenterError>{
        let text = match task.build_description {
            Some(ref desc) =>{
                match task.git_info {
                    Some(ref git) => {
                        format!("• Branch: {}\n\n• Commit: {}\n\n\n\n{}", git.branch, git.commit, desc)
                    },
                    None => {
                        desc.to_string()
                    }
                }
            },
            None => {
                match task.git_info {
                    Some(ref git) => {
                        format!("Branch: {}\n\nCommit: {}", git.branch, git.commit)
                    },
                    None => {
                        return Ok(());
                    }
                }
            }
        };
        let (branch, commit) = match task.git_info{
            Some(ref git) => (git.branch.as_str(), git.commit.as_str()),
            None => ("", "")
        };

        let path = format!("releases/{}", release_id);
        let set_info_result = self
            .request_builder
            .build_request(Application, Method::PUT, &path, true)?
            .json(&json!(
                {
                    "enabled": true,
                    "release_notes": text,
                    "build": {
                        "branch_name": branch,
                        "commit_hash": commit,
                        "commit_message": ""
                    }
                }
            ))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        debug!("Information set result: {:#?}", set_info_result);

        Ok(())
    }

    async fn update_distribution_groups(&self, release_id: u64, task: &AppCenterBuildUploadTask<'_>) -> Result<(), AppCenterError>{
        if let Some(ref groups) = task.distribution_groups {
            let path = format!("releases/{}", release_id);

            let groups_json_array: Vec<Value> = groups
                .iter()
                .map(|val|{
                    json!({
                        "name": val
                    })
                })
                .collect();

            let request = self
                .request_builder
                .build_request(Application, Method::PATCH, &path, true)?
                .json(&json!(
                    {
                        "notify_testers": false,
                        "destinations": groups_json_array
                    }
                ));

            debug!("Distribution groups request: {:#?}", request);

            let result = request
                .send()
                .await?
                .error_for_status()?
                /*.text()
                .await?*/
                .inspect_json::<ReleaseUpdateResponse, AppCenterError>(|d| { 
                    debug!("{}", d); 
                })
                .await?;
             
            debug!("Distribution groups set result: {:#?}", result);

            match result {
                ReleaseUpdateResponse::Success{..} => {
                    return Ok(());
                },
                ReleaseUpdateResponse::Failure{message, ..} => {
                    return Err(AppCenterError::Custom(format!("Groups distribution failed: {}", message)));
                }
            }
        }

        Ok(())
    }

    async fn request_release_information(&self, release_id: u64) -> Result<ReleaseInfoResponse, AppCenterError>{
        let path = format!("releases/{}", release_id);
        let result = self
            .request_builder
            .build_request(Application, Method::GET, &path, true)?
            .send()
            .await?
            .error_for_status()?
            .inspect_json::<ReleaseInfoResponse, AppCenterError>(|d| { 
                debug!("{}", d); 
            })
            .await?;
        
        Ok(result)
    }

    pub async fn upload_build(&self, task: &AppCenterBuildUploadTask<'_>) -> Result<ReleaseInfoResponse, AppCenterError>{
        // Инициирование отгрузки
        let release_info = self
            .initialize_release(&task.version_info)
            .await?;

        // Выгрузка файлика
        AppCenterUploader::new(self.request_builder.get_http_client().clone(), 
                               &release_info, 
                               task.file_path,
                               task.upload_threads_count)
            .await?
            .upload()
            .await?;

        // Запрашиваем обновление статуса выгрузки
        self
            .set_upload_finished_status(&release_info)
            .await?;

        // Дожидаемся id релиза
        let release_id = self
            .wait_release_id(&release_info)
            .await?;

        // Обновляем мету
        self
            .update_build_meta(release_id, task)
            .await?;

        // Обновляем группы дистрибуции
        self
            .update_distribution_groups(release_id, task)
            .await?;

        // Получение информации по релизу
        self
            .request_release_information(release_id)
            .await
    }
}