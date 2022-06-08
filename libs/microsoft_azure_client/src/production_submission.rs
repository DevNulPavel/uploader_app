use crate::responses::AppPackage;
use crate::{
    blob_uploader::perform_blob_file_uploading,
    error::MicrosoftAzureError,
    helpers::find_appx_filenames_in_zip,
    request_builder::RequestBuilder,
    responses::{DataOrErrorResponse, SubmissionCreateResponse},
    submission_helpers::{commit_changes, wait_commit_finished},
};
use serde_json_string_parse_helper::ParseJson;
use std::path::Path;
use tracing::debug;
use tracing::{instrument, Instrument};

/// Внутренняя структура по работе с submission
pub struct ProductionSubmission {
    request_builder: RequestBuilder,
    data: SubmissionCreateResponse,
}

impl ProductionSubmission {
    /// Инициализируем новый экземпляр выливки
    #[instrument(skip(request_builder))]
    pub async fn start_new(
        request_builder: RequestBuilder,
    ) -> Result<ProductionSubmission, MicrosoftAzureError> {
        // Выполняем запрос создания нового сабмишена
        // https://docs.microsoft.com/en-us/windows/uwp/monetize/create-a-flight
        let data = request_builder
            .clone()
            .method(reqwest::Method::POST)
            .join_path("submissions".to_owned())
            .build()
            .in_current_span()
            .await?
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send()
            .in_current_span()
            .await?
            .text()
            .in_current_span()
            .await?
            .parse_json_with_data_err::<DataOrErrorResponse<SubmissionCreateResponse>>()?
            .into_result()?;
        debug!("Microsoft Azure, new submission response: {:#?}", data);

        // Создаем новый реквест билдер на основании старого, но уже с полученным submission id
        let request_builder = request_builder.clone().submission_id(data.id.clone());

        Ok(ProductionSubmission {
            request_builder,
            data,
        })
    }

    #[instrument(skip(self, zip_file_path, submission_name))]
    pub async fn upload_build(
        &mut self,
        zip_file_path: &Path,
        submission_name: String,
    ) -> Result<(), MicrosoftAzureError> {
        // Может быть нет фалика по этому пути
        if !zip_file_path.exists() {
            return Err(MicrosoftAzureError::NoFile(zip_file_path.to_owned()));
        }

        // Открываем zip файлик и получаем имя .appx/.appxupload там
        let filenames_in_zip = find_appx_filenames_in_zip(zip_file_path)?;

        // Обновляем имя файликов и способ активации
        let mut new_params = self.data.common_data.clone();

        // Выставляем параметры активации и имя выгрузки
        new_params.friendly_name = Some(submission_name);
        new_params.target_publish_mode = "Manual".to_owned();

        // У старых пакетов помечаем статус необходимости удаления
        new_params.application_packages.iter_mut().for_each(|val| {
            val.file_status = "PendingDelete".to_owned();
        });

        // Добавляем новые пакеты без заполнения параметров
        new_params
            .application_packages
            .extend(filenames_in_zip.into_iter().map(|file_name| AppPackage {
                file_name,
                file_status: "PendingUpload".to_owned(),
                ..Default::default()
            }));

        // Запрос обновления данных
        self.data = self
            .request_builder
            .clone()
            .method(reqwest::Method::PUT)
            .build()
            .in_current_span()
            .await?
            .json(&new_params)
            .send()
            .in_current_span()
            .await?
            .text()
            .await?
            .parse_json_with_data_err::<DataOrErrorResponse<SubmissionCreateResponse>>()?
            .into_result()?;
        debug!("Microsoft Azure: update response {:#?}", self.data);

        // Получаем чистый HTTP клиент для выгрузки файлика
        let http_client = self.request_builder.get_http_client();

        // Создаем непосредственно урл для добавления данных
        let append_data_url = reqwest::Url::parse(&self.data.file_upload_url)?;

        // Выполняем непосредственно выгрузку на сервер нашего архива
        perform_blob_file_uploading(&http_client, &append_data_url, zip_file_path)
            .in_current_span()
            .await?;

        // Пытаемся закоммитить
        commit_changes(&self.request_builder)
            .in_current_span()
            .await?;

        // Ждем завершения коммита
        wait_commit_finished(&self.request_builder)
            .in_current_span()
            .await?;

        Ok(())
    }
}
