use humansize::FileSize;
use serde_json::json;
use std::path::Path;
use tokio::{fs::File, io::AsyncReadExt};
use tracing::debug;
use tracing::instrument;
use tracing_error::SpanTrace;
// use serde_json::{
//     json
// };
use crate::{
    error::MicrosoftAzureError,
    request_builder::RequestBuilder,
    responses::{
        DataOrErrorResponse, FlightInfoResponse, FlightSubmissionCommitResponse,
        SubmissionCreateResponse, SubmissionStatusResponse,
    },
};

/// Внутренняя структура по работе с submission
pub struct Submission {
    request_builder: RequestBuilder,
    data: SubmissionCreateResponse,
}

impl Submission {
    /// Инициализируем новый экземпляр выливки
    #[instrument(skip(request_builder))]
    pub async fn start_new(
        request_builder: RequestBuilder,
    ) -> Result<Submission, MicrosoftAzureError> {
        // Выполняем запрос создания нового сабмишена
        // https://docs.microsoft.com/en-us/windows/uwp/monetize/create-a-flight
        let data = request_builder
            .clone()
            .method(reqwest::Method::POST)
            .join_path("submissions".to_owned())
            .build()
            .await?
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send()
            .await?
            .json::<DataOrErrorResponse<SubmissionCreateResponse>>()
            .await?
            .into_result()?;
        debug!("Microsoft Azure, new submission response: {:#?}", data);

        // Создаем новый реквест билдер на основании старого, но уже с полученным submission id
        let request_builder = request_builder.clone().submission_id(data.id.clone());

        Ok(Submission {
            request_builder,
            data,
        })
    }

    /// Выполнение выгрузки непосредственно файлика с билдом
    async fn perform_file_uploading(&self, file_path: &Path) -> Result<(), MicrosoftAzureError> {
        debug!("Microsoft Azure: file uploading start");

        // Получаем чистый HTTP клиент для выгрузки файлика
        let http_client = self.request_builder.get_http_client();

        // Первым этапом идет выставление режима AppendBlob для выгрузки
        // https://docs.microsoft.com/en-us/rest/api/storageservices/put-blob
        // https://docs.microsoft.com/en-us/rest/api/storageservices/put-blob#remarks
        // https://stackoverflow.com/questions/58724878/upload-large-files-1-gb-to-azure-blob-storage-through-web-api
        http_client
            .put(&self.data.file_upload_url)
            .header("x-ms-blob-type", "AppendBlob")
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send()
            .await?
            .error_for_status()?;

        // Создаем непосредственно урл для добавления данных
        let append_data_url = {
            // Парсим базовый Url
            let mut append_data_url = reqwest::Url::parse(&self.data.file_upload_url)?;

            // Дополнительно добавляем к параметрам запроса значение appendblock
            append_data_url
                .query_pairs_mut()
                .append_pair("comp", "appendblock");

            append_data_url
        };

        // Подготавливаем файлик для потоковой выгрузки
        let mut source_file = File::open(file_path).await?;

        // Получаем суммарный размер данных
        let source_file_length = source_file.metadata().await?.len();

        // Оставшийся размер выгрузки
        let mut data_left = source_file_length as i64;
        loop {
            // Размер буффера
            let buffer_size_limit = std::cmp::min(1024 * 1024 * 3, data_left); // 4Mb - ограничение для отдельного куска
            if buffer_size_limit <= 0 {
                break;
            }

            // TODO: Убрать создание нового буффера каждый раз,
            // Вроде бы как Hyper позволяет использовать slice для выгрузки
            let mut buffer = vec![0; buffer_size_limit as usize];

            // Читаем из файлика данные в буффер
            let read_size = source_file.read_exact(&mut buffer).await?;

            debug!(
                "Microsoft azure: bytes read from file {}",
                read_size
                    .file_size(humansize::file_size_opts::BINARY)
                    .unwrap()
            );

            // Отнимаем нужное значения размера данных
            data_left -= read_size as i64;

            // Обрезаем буффер на нужный размер
            buffer.truncate(read_size);

            // Непосредственно выгрузка
            http_client
                .put(append_data_url.clone())
                .header(reqwest::header::CONTENT_LENGTH, read_size.to_string())
                .body(buffer)
                .send()
                .await?
                .error_for_status()?;

            debug!(
                "Microsoft azure: bytes upload left {}",
                data_left
                    .file_size(humansize::file_size_opts::BINARY)
                    .unwrap()
            );
        }

        if data_left != 0 {
            return Err(MicrosoftAzureError::UploadingError(
                SpanTrace::capture(),
                "Left data size must be zero after uploading".to_owned(),
            ));
        }

        Ok(())
    }

    /// Данный метод занимается тем, что коммитит изменения на сервере
    /// Описание: `https://docs.microsoft.com/en-us/windows/uwp/monetize/commit-a-flight-submission`
    async fn commit_changes(&self) -> Result<(), MicrosoftAzureError> {
        debug!("Microsoft Azure: commit request");

        let new_info = self
            .request_builder
            .clone()
            .method(reqwest::Method::POST)
            .submission_command("commit".to_string())
            .build()
            .await?
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send()
            .await?
            // .error_for_status()?
            .json::<DataOrErrorResponse<FlightSubmissionCommitResponse>>()
            .await?
            .into_result()?;

        debug!("Microsoft Azure: commit response {:#?}", new_info);

        if !new_info.status.eq("CommitStarted") {
            return Err(MicrosoftAzureError::InvalidCommitStatus(new_info.status));
        }

        Ok(())
    }

    /// C помощью данного метода мы ждем завершения выполнения коммита
    /// Описание: `https://docs.microsoft.com/en-us/windows/uwp/monetize/get-status-for-a-flight-submission`
    async fn wait_commit_finished(&self) -> Result<(), MicrosoftAzureError> {
        debug!("Microsoft Azure: wait submission commit result");

        loop {
            let status_response = self
                .request_builder
                .clone()
                .method(reqwest::Method::GET)
                .submission_command("status".to_string())
                .build()
                .await?
                .header(reqwest::header::CONTENT_LENGTH, "0")
                .send()
                .await?
                // .error_for_status()?
                .json::<DataOrErrorResponse<SubmissionStatusResponse>>()
                .await?
                .into_result()?;

            debug!(
                "Microsoft Azure: submission status response {:#?}",
                status_response
            );

            match status_response.status.as_str() {
                // Нормальное состояние для ожидания
                "CommitStarted" => {
                    tokio::time::delay_for(std::time::Duration::from_secs(15)).await;
                }

                // Коммит прошел успешно, прерываем ожидание
                "PreProcessing" => {
                    break;
                }

                // Ошибочный статус - ошибка
                "CommitFailed"
                | "None"
                | "Canceled"
                | "PublishFailed"
                | "PendingPublication"
                | "Certification"
                | "Publishing"
                | "Published"
                | "PreProcessingFailed"
                | "CertificationFailed"
                | "Release"
                | "ReleaseFailed" => {
                    return Err(MicrosoftAzureError::CommitFailed(status_response));
                }

                // Неизвестный статус - ошибка
                _ => {
                    return Err(MicrosoftAzureError::InvalidCommitStatus(
                        status_response.status,
                    ));
                }
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn upload_build(&mut self, zip_file_path: &Path) -> Result<(), MicrosoftAzureError> {
        // Может быть нет фалика по этому пути
        if !zip_file_path.exists() {
            return Err(MicrosoftAzureError::NoFile(zip_file_path.to_owned()));
        }

        // Проверяем расширение данного файлика
        /*if check_file_extention(zip_file_path, "zip") == false{
            return Err(MicrosoftAzureError::InvalidUploadFileExtention);
        }

        // Открываем zip файлик и получаем имя .appx там
        let filename = {
            let zip = zip::ZipArchive::new(std::fs::File::open(zip_file_path)?)?;
            let filename = zip
                .file_names()
                .find(|full_path_str|{
                    let file_name = std::path::Path::new(full_path_str)
                        .file_name()
                        .and_then(|f|{
                            f.to_str()
                        });
                    if let Some(file_name) = file_name {
                        if !file_name.starts_with(".") && file_name.ends_with(".appx"){
                            true
                        }else{
                            false
                        }
                    }else{
                        false
                    }
                })
                .ok_or(MicrosoftAzureError::NoAppxFileInZip)?;
            debug!("Microsoft Azure: filename in zip {}", filename);
            filename.to_owned() // TODO: не аллоцировать
        };*/

        let filename = zip_file_path.file_name().and_then(|n| n.to_str()).unwrap();

        // Обновляем имя пакета и имя файлика
        let mut new_params = self.data.common_data.clone();
        new_params.target_publish_mode = "Manual".to_owned();
        new_params.application_packages.iter_mut().for_each(|obj|{
            obj.file_name = filename.to_owned();
        });

        self.data = self
            .request_builder
            .clone()
            .method(reqwest::Method::PUT)
            .build()
            .await?
            .json(&new_params)
            .send()
            .await?
            .json::<DataOrErrorResponse<SubmissionCreateResponse>>()
            .await?
            .into_result()?;
        debug!("Microsoft Azure: update response {:#?}", self.data);

        // Выполняем непосредственно выгрузку на сервер нашего архива
        self.perform_file_uploading(zip_file_path).await?;

        // Пытаемся закоммитить
        self.commit_changes().await?;

        // Ждем завершения коммита
        //self.wait_commit_finished().await?;

        Ok(())
    }
}
