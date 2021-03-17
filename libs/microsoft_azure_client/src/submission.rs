use log::debug;
use reqwest::{
    Client
};
use crate::{
    request_builder::{
        RequestBuilder
    },
    error::{
        MicrosoftAzureError
    },
    responses::{
        DataOrErrorResponse,
        SubmissionCreateResponse
    }
};


/// Внутренняя структура по работе с submission
pub struct Submission {
    request_builder: RequestBuilder,
    data: SubmissionCreateResponse
}

impl Submission {
    /// Инициализируем новый экземпляр выливки
    pub async fn start_new(request_builder: RequestBuilder) -> Result<Submission, MicrosoftAzureError> {
        // Выполняем запрос создания нового сабмишена
        // https://docs.microsoft.com/en-us/windows/uwp/monetize/create-an-app-submission
        let info = request_builder
            .clone()
            .method(reqwest::Method::POST)
            .join_path("submissions".to_owned())    
            .build()
            .await?
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send()
            .await?
            .error_for_status()?
            .json::<DataOrErrorResponse<SubmissionCreateResponse>>()
            .await?
            .into_result()?;
        
        debug!("Microsoft Azure, new submission response: {:#?}", info);

        // Создаем новый реквест билдер на основании старого, но уже с полученным submission id 
        let new_builder = request_builder
            .clone()
            .submission_id(info.id.clone());

        Ok(Submission{
            request_builder: new_builder,
            data: info
        })
    }
}