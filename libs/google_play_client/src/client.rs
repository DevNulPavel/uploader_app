use std::{
    path::{
        Path
    }
};
use log::{
    debug, 
    info
};
use yup_oauth2::{
    AccessToken
};
use reqwest::{
    Client   
};
use super::{
    request_builder::{
        GooglePlayRequestBuilder,
    },
    app_edit::{
        AppEdit
    },
    error::{
        GooglePlayError
    }
};

//////////////////////////////////////////////////////////////////////////////////////////

pub struct GooglePlayUploadTask<'a>{
    pub file_path: &'a Path,
    pub package_name: &'a str,
    pub target_track: Option<&'a str>
}


//////////////////////////////////////////////////////////////////////////////////////////

pub struct GooglePlayClient{
    http_client: Client,
    token: AccessToken
}
impl GooglePlayClient {
    pub fn new(http_client: Client, token: AccessToken) -> GooglePlayClient {
        info!("Google play request builder created");

        GooglePlayClient{
            http_client,
            token   
        }
    }

    async fn start_insert<'a>(&'a self, package_name: &str) -> Result<AppEdit<'a>, GooglePlayError>{
        let request_builder = GooglePlayRequestBuilder::new(self.http_client.clone(), package_name, &self.token)?;

        let edit = AppEdit::new(request_builder)
            .await?;

        Ok(edit)
    }

    pub async fn upload(&self, task: GooglePlayUploadTask<'_>) -> Result<u64, GooglePlayError> {
        info!("Before upload");

        // https://developers.google.com/android-publisher/api-ref/rest

        // Старт редактирования
        let edit = self
            .start_insert(&task.package_name)
            .await?;

        // debug!("Google play edit result: {:#?}", edit.get_info());

        // Выгрузка
        let upload_result = edit
            .upload_build(&task.file_path)
            .await?;

        debug!("Google play upload result: {:#?}", upload_result);

        // Обновляем таргет если надо
        if let Some(ref target_track) = task.target_track{
            let track_update_result = edit
                .update_track_to_complete(target_track, &upload_result)
                .await?;
            debug!("Google play track update result: {:#?}", track_update_result);
        }

        // Валидация
        edit
            .validate()
            .await?;

        // Коммит
        edit
            .commit()
            .await?;

        Ok(upload_result.version_code)
    }
}