use super::{
    app_edit::AppEdit, error::AmazonError, request_builder::AmazonAppRequestBuilder,
    token::AmazonAccessToken,
};
use log::error;
use reqwest::Client;
use std::path::Path;
use tap::TapFallible;

// Доки
// https://developer.amazon.com/docs/app-submission-api/appsubapi-endpoints.html

//////////////////////////////////////////////////////////////////////////////////////////

pub struct AmazonUploadTask<'a> {
    pub application_id: &'a str,
    pub file_path: &'a Path,
}

//////////////////////////////////////////////////////////////////////////////////////////

pub struct AmazonClient {
    http_client: Client,
    token: AmazonAccessToken,
}
impl AmazonClient {
    pub fn new(http_client: Client, token: AmazonAccessToken) -> AmazonClient {
        AmazonClient { http_client, token }
    }

    async fn build_edit<'a>(&'a self, app_id: &str) -> Result<AppEdit<'a>, AmazonError> {
        let request_builder =
            AmazonAppRequestBuilder::new(self.http_client.clone(), &self.token, app_id).tap_err(
                |err| {
                    error!("Request builder create failed: {}", err);
                },
            )?;

        let edit = AppEdit::new(request_builder).await?;

        Ok(edit)
    }

    pub async fn upload(&self, task: AmazonUploadTask<'_>) -> Result<(), AmazonError> {
        let edit = self.build_edit(task.application_id).await.tap_err(|err| {
            error!("Edit create failed: {}", err);
        })?;

        edit.remove_old_apks().await.tap_err(|err| {
            error!("Remove old apps failed: {}", err);
        })?;

        let _info = edit.upload_new_apk(task.file_path).await.tap_err(|err| {
            error!("Upload failed: {}", err);
        })?;

        // Валидация и коммит вроде как запрещены текущим аккаунтом, делаем лишь выгрузку
        // edit.validate().await?;
        // edit.commit_apk(&info).await?;

        Ok(())
    }
}
