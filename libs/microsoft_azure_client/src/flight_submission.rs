use crate::{
    blob_uploader::perform_blob_file_uploading,
    error::MicrosoftAzureError,
    helpers::find_appx_filenames_in_zip,
    request_builder::RequestBuilder,
    responses::{
        DataOrErrorResponse, FlightCreateResponse, FlightInfoResponse,
        FlightSubmissionsCreateResponse,
    },
    submission_helpers::{commit_changes, wait_commit_finished},
};
use log::debug;
use serde_json::json;
use serde_json_string_parse::ParseJson;
use std::path::Path;

/// Внутренняя структура по работе с submission
pub struct FlightSubmission {
    request_builder: RequestBuilder,
    data: FlightSubmissionsCreateResponse,
}

impl FlightSubmission {
    /// Инициализируем новый экземпляр выливки
    pub async fn start_new(
        request_builder: RequestBuilder,
        groups: Vec<String>,
        test_flight_name: String,
    ) -> Result<FlightSubmission, MicrosoftAzureError> {
        // Выполняем запрос создания нового сабмишена
        // https://docs.microsoft.com/en-us/windows/uwp/monetize/create-a-flight
        let new_flight_info = request_builder
            .clone()
            .method(reqwest::Method::POST)
            .join_path("flights".to_owned())
            .build()
            .await?
            .json(&json!({
                "groupIds": groups,
                "friendlyName": test_flight_name,
                // "rankHigherThan": null
            }))
            .send()
            .await?
            .text()
            .await?
            .parse_json_with_data_err::<DataOrErrorResponse<FlightCreateResponse>>()?
            .into_result()?;
        debug!(
            "Microsoft Azure, new flight response: {:#?}",
            new_flight_info
        );

        // Создаем новый реквест билдер на основании старого, но уже с полученным submission id
        let request_builder = request_builder
            .clone()
            .flight_id(new_flight_info.flight_id.clone());

        // Получим информацию для данного flightId
        let flight_info = request_builder
            .clone()
            .method(reqwest::Method::GET)
            .build()
            .await?
            .send()
            .await?
            .text()
            .await?
            .parse_json_with_data_err::<DataOrErrorResponse<FlightInfoResponse>>()?
            .into_result()?;
        debug!("Microsoft Azure, flight info: {:#?}", flight_info);

        // Если есть какие-то ожидающие сабмиссии
        let new_submission_info = if let Some(pending) = flight_info.pending_flight_submission {
            // Удаляем сабмиссии
            /*let resp = request_builder
                .clone()
                .submission_id(pending.id)
                .method(reqwest::Method::DELETE)
                .build()
                .await?
                .send()
                .await?;
            if !resp.status().is_success() {
                let err = resp.json::<ErrorResponseValue>().await?;
                return Err(MicrosoftAzureError::RestApiResponseError(SpanTrace::capture(), err));
            }
            debug!("Microsoft Azure, flight submission delete success");*/

            // Данные по имеющейся сабмиссии
            let info = request_builder
                .clone()
                .submission_id(pending.id)
                .method(reqwest::Method::GET)
                .build()
                .await?
                .send()
                .await?
                .text()
                .await?
                .parse_json_with_data_err::<DataOrErrorResponse<FlightSubmissionsCreateResponse>>()?
                .into_result()?;
            debug!(
                "Microsoft Azure, received active flight submission response: {:#?}",
                info
            );
            info
        } else {
            // Создаем новую flight submission
            let info = request_builder
                .clone()
                .method(reqwest::Method::POST)
                .join_path("submissions".to_owned())
                .build()
                .await?
                .header(reqwest::header::CONTENT_LENGTH, 0)
                .send()
                .await?
                .text()
                .await?
                .parse_json_with_data_err::<DataOrErrorResponse<FlightSubmissionsCreateResponse>>()?
                .into_result()?;
            debug!(
                "Microsoft Azure, new flight submission response: {:#?}",
                info
            );
            info
        };

        // Создаем новый реквест билдер на основании старого, но уже с полученным submission id
        let request_builder = request_builder
            .clone()
            .submission_id(new_submission_info.id.clone());

        Ok(FlightSubmission {
            request_builder,
            data: new_submission_info,
        })
    }

    /// Выгружаем наш билд
    pub async fn upload_build(&mut self, zip_file_path: &Path) -> Result<(), MicrosoftAzureError> {
        // Может быть нет фалика по этому пути
        if !zip_file_path.exists() {
            return Err(MicrosoftAzureError::NoFile(zip_file_path.to_owned()));
        }

        // Открываем zip файлик и получаем имя .appx/.appxupload там
        let filenames_in_zip = find_appx_filenames_in_zip(zip_file_path)?;

        // Формируем json с именами файлов
        let flight_packages_json: Vec<_> = filenames_in_zip
            .into_iter()
            .map(|filename_in_zip| {
                json!(                    {
                  "fileName": filename_in_zip,
                  "fileStatus": "PendingUpload",
                  "minimumDirectXVersion": "None",
                  "minimumSystemRam": "None"
                })
            })
            .collect();

        // Обновляем имя пакета
        self.data = self
            .request_builder
            .clone()
            .method(reqwest::Method::PUT)
            .build()
            .await?
            .json(&json!({
                "flightPackages": flight_packages_json,
                "targetPublishMode": "Manual",
                // "notesForCertification": "No special steps are required for certification of this app."
            }))
            .send()
            .await?
            .text()
            .await?
            .parse_json_with_data_err::<DataOrErrorResponse<FlightSubmissionsCreateResponse>>()?
            .into_result()?;
        debug!("Microsoft Azure: update response {:#?}", self.data);

        // Получаем чистый HTTP клиент для выгрузки файлика
        let http_client = self.request_builder.get_http_client();

        // Создаем непосредственно урл для добавления данных
        let append_data_url = reqwest::Url::parse(&self.data.file_upload_url)?;

        // Выполняем непосредственно выгрузку на сервер нашего архива
        perform_blob_file_uploading(&http_client, &append_data_url, zip_file_path).await?;

        // Пытаемся закоммитить
        commit_changes(&self.request_builder).await?;

        // Ждем завершения коммита
        wait_commit_finished(&self.request_builder).await?;

        Ok(())
    }
}
