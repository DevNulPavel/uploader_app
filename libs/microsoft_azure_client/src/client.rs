use std::{
    path::{
        Path
    }
};
use log::{
    debug
};
use reqwest::{
    Client
};
use crate::{
    token::{
        TokenProviderDefault
    },
    request_builder::{
        RequestBuilder
    },
    error::{
        MicrosoftAzureError
    },
    responses::{
        DataOrErrorResponse,
        ApplicationInfoResponse,
        ApplicationInfoSubmissionData
    },
    submission::{
        Submission
    }
};


pub struct MicrosoftAzureClient{
    request_builder: RequestBuilder
}

impl MicrosoftAzureClient {
    pub fn new<D: std::fmt::Display>(http_client: Client, 
                                     tenant_id: D,
                                     client_id: String,
                                     client_secret: String,
                                     application_id: String) -> MicrosoftAzureClient {

        // Создаем провайдер токена
        let token_provider = Box::new(TokenProviderDefault::new(http_client.clone(), 
                                                                tenant_id, 
                                                                client_id, 
                                                                client_secret)
            .expect("Token create failed"));           
        
        // Уже с провайдером токенов создаем билдер запросов
        let request_builder = RequestBuilder::new(http_client, 
                                                  token_provider, 
                                                  application_id);

        MicrosoftAzureClient{
            request_builder
        }
    }

    /// Данный метод позволяет получить информацию по текущему приложению
    async fn get_application_info(&self) -> Result<ApplicationInfoResponse, MicrosoftAzureError>{
        // https://docs.microsoft.com/en-us/windows/uwp/monetize/get-an-app
        let response = self.request_builder
            .clone()
            .method(reqwest::Method::GET)
            .build()
            .await?
            .send()
            .await?
            .error_for_status()?
            .json::<DataOrErrorResponse<ApplicationInfoResponse>>()
            .await?
            .into_result()?;

        Ok(response)
    }

    /// Данный метод позволяет удалить ожидающий билд
    async fn remove_pending_submission(&self, pending_info: ApplicationInfoSubmissionData) -> Result<(), MicrosoftAzureError>{
        // https://docs.microsoft.com/en-us/windows/uwp/monetize/delete-an-app-submission

        // Если вернулся валидный статус 200, значит все хорошо
        self.request_builder
            .clone()
            .method(reqwest::Method::DELETE)
            .submission_id(pending_info.id)
            .build()
            .await?
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn upload_production_build(&self, zip_upload_file_path: &Path) -> Result<(), MicrosoftAzureError> {
        // https://docs.microsoft.com/en-us/windows/uwp/monetize/manage-app-submissions
        // https://docs.microsoft.com/en-us/windows/uwp/monetize/python-code-examples-for-the-windows-store-submission-api

        // Сначала запрашиваем информацию о текущем приложении
        debug!("Microsoft Azure: request application info");
        let current_app_info = self.get_application_info().await?;
        debug!("Microsoft Azure: current application info: {:#?}", current_app_info);

        // Затем нужно проверить, нет ли у нас сейчас каких-то ожидающих сабмитов,
        // если есть - выполняем удаление
        if let Some(pending_info) = current_app_info.pending_app_submission {
            debug!("Microsoft Azure: remove current pending: {:#?}", pending_info);
            self.remove_pending_submission(pending_info).await?;
            debug!("Microsoft Azure: current pending removed");
        }

        return Ok(());

        // Создаем новый Submission для данного приложения
        debug!("Microsoft Azure: submission create try");
        let mut submission = Submission::start_new(self.request_builder.clone())
            .await?;
        debug!("Microsoft Azure: submission created");

        // Выполняем выгрузку файлика
        debug!("Microsoft Azure: File uploading start");
        submission
            .upload_build(zip_upload_file_path)
            .await?;
        debug!("Microsoft Azure: File uploading finished");

        Ok(())
    }
}