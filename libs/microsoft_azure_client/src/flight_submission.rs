use bytes::Bytes;
use humansize::FileSize;
use serde_json::json;
use std::path::Path;
use tokio::time;
use tokio::{fs::File, io::AsyncReadExt};
use tracing::debug;
use tracing::{instrument, warn};
use tracing_error::SpanTrace;
// use serde_json::{
//     json
// };
use crate::{
    error::MicrosoftAzureError,
    helpers::check_file_extention,
    request_builder::RequestBuilder,
    responses::{
        DataOrErrorResponse, FlightCreateResponse, FlightInfoResponse,
        FlightSubmissionCommitResponse, FlightSubmissionsCreateResponse, SubmissionStatusResponse,
    },
};

/// Внутренняя структура по работе с submission
pub struct FlightSubmission {
    request_builder: RequestBuilder,
    data: FlightSubmissionsCreateResponse,
}

impl FlightSubmission {
    /// Инициализируем новый экземпляр выливки
    #[instrument(skip(request_builder))]
    pub async fn start_new(
        request_builder: RequestBuilder,
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
                "groupIds": [
                    "1152921504607280735", // TODO: ???
                ],
                "friendlyName": "Test_submission_API", // TODO: ???
                // "rankHigherThan": null
            }))
            .send()
            .await?
            .json::<DataOrErrorResponse<FlightCreateResponse>>()
            .await?
            .into_result()?;
        debug!(
            "Microsoft Azure, new flight response: {:#?}",
            new_flight_info
        );

        // Создаем новый реквест билдер на основании старого, но уже с полученным submission id
        let request_builder = request_builder
            .clone()
            .flight_id(new_flight_info.flight_id.clone());

        /*// Создаем новую flight submission
        let new_submission_info = request_builder
            .clone()
            .method(reqwest::Method::POST)
            .join_path("submissions".to_owned())
            .build()
            .await?
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send()
            .await?
            .json::<DataOrErrorResponse<FlightSubmissionsCreateResponse>>()
            .await?
            .into_result()?;
        debug!("Microsoft Azure, new flight submission response: {:#?}", new_submission_info);*/

        // Получим информацию для данного flightId
        let flight_info = request_builder
            .clone()
            .method(reqwest::Method::GET)
            .build()
            .await?
            .send()
            .await?
            .json::<DataOrErrorResponse<FlightInfoResponse>>()
            .await?
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
                .json::<DataOrErrorResponse<FlightSubmissionsCreateResponse>>()
                .await?
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
                .json::<DataOrErrorResponse<FlightSubmissionsCreateResponse>>()
                .await?
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

    /// Выполнение выгрузки непосредственно файлика с билдом
    async fn perform_file_uploading(&self, file_path: &Path) -> Result<(), MicrosoftAzureError> {
        debug!("Microsoft Azure: file uploading start");

        // Получаем чистый HTTP клиент для выгрузки файлика
        let http_client = self.request_builder.get_http_client();

        // Первым этапом идет выставление режима AppendBlob для выгрузки
        // https://docs.microsoft.com/en-us/rest/api/storageservices/put-blob
        // https://docs.microsoft.com/en-us/rest/api/storageservices/put-blob#remarks
        // https://stackoverflow.com/questions/58724878/upload-large-files-1-gb-to-azure-blob-storage-through-web-api
        let blob_create_response = http_client
            .put(&self.data.file_upload_url)
            .header("x-ms-blob-type", "BlockBlob")
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!(
            "Microsoft Azure: blob create response: {:?}",
            blob_create_response
        );

        // Создаем непосредственно урл для добавления данных
        let append_data_url = reqwest::Url::parse(&self.data.file_upload_url)?;

        // Подготавливаем файлик для потоковой выгрузки
        let mut source_file = File::open(file_path).await?;

        // Получаем суммарный размер данных
        let source_file_length = source_file.metadata().await?.len();

        let mut blocks = Vec::<UploadResult>::new();

        #[derive(Debug)]
        struct UploadTask {
            data: Bytes,
            block_id: String,
            index: usize,
        }
        #[derive(Debug)]
        struct UploadResult {
            block_id: String,
            index: usize,
        }

        const UPLOAD_THREADS_COUNT: usize = 8;
        const BUFFER_MAX_SIZE: i64 = 1024 * 1024 * 8; // 8Mb - ограничение для отдельного куска

        let (mut task_sender, task_receiver) =
            tokio::sync::mpsc::channel::<UploadTask>(UPLOAD_THREADS_COUNT * 2);
        let task_receiver = std::sync::Arc::new(tokio::sync::Mutex::new(task_receiver));

        let (result_sender, mut result_receiver) = tokio::sync::mpsc::channel::<
            Result<UploadResult, MicrosoftAzureError>,
        >(UPLOAD_THREADS_COUNT * 8);

        for _ in 0..UPLOAD_THREADS_COUNT {
            let task_receiver = task_receiver.clone();
            let append_data_url = append_data_url.clone();
            let http_client = http_client.clone();
            let mut result_sender = result_sender.clone();

            tokio::spawn(async move {
                // Если делать: while let Some(task) = task_receiver.lock().await.recv().await
                // Тогда блокировка висит во время всего выполнения цикла
                loop {
                    let task = task_receiver.lock().await.recv().await;
                    if let Some(UploadTask {
                        data,
                        block_id,
                        index,
                    }) = task
                    {
                        debug!("Start uploading for block index: {}", index);

                        let mut url = append_data_url.clone();
                        url.query_pairs_mut()
                            .append_pair("comp", "block")
                            .append_pair("blockid", &block_id);

                        let data_size = data.len();

                        let mut iter_count = 0;
                        let result = loop {
                            let upload_fn = async {
                                // Непосредственно выгрузка
                                http_client
                                    .put(url.clone())
                                    .header(reqwest::header::CONTENT_LENGTH, data_size)
                                    .body(data.clone())
                                    .send()
                                    .await?
                                    .error_for_status()?;

                                Result::<UploadResult, MicrosoftAzureError>::Ok(UploadResult {
                                    block_id: block_id.clone(),
                                    index,
                                })
                            };

                            let res = upload_fn.await;
                            if res.is_ok() {
                                break res;
                            } else {
                                iter_count += 1;
                                if iter_count <= 3 {
                                    warn!(
                                        "Retry uploading for url: {}, iteration: {}, res: {:?}",
                                        url, iter_count, res
                                    );
                                    tokio::time::delay_for(time::Duration::from_secs(3)).await;
                                    continue;
                                } else {
                                    break res;
                                }
                            }
                        };
                        if result_sender.send(result).await.is_err() {
                            return;
                        }
                    } else {
                        break;
                    }
                }
            });
        }
        drop(result_sender);
        drop(task_receiver);

        // Оставшийся размер выгрузки
        let mut data_left = source_file_length as i64;
        // let mut data_offset: i64 = 0;
        let mut index = 0;
        loop {
            // Размер буффера
            let buffer_size_limit = std::cmp::min(BUFFER_MAX_SIZE, data_left);
            if buffer_size_limit <= 0 {
                break;
            }

            // TODO: Убрать создание нового буффера каждый раз,
            // Вроде бы как Hyper позволяет использовать slice для выгрузки
            let mut buffer = vec![0_u8; buffer_size_limit as usize];

            // Читаем из файлика данные в буффер
            let read_size = source_file.read_exact(&mut buffer).await?;

            // trace!(
            //     "Microsoft azure: bytes read from file {}",
            //     read_size
            //         .file_size(humansize::file_size_opts::BINARY)
            //         .map_err(|err| {
            //             MicrosoftAzureError::HumanSizeError(SpanTrace::capture(), err)
            //         })?
            // );

            // Отнимаем нужное значения размера данных
            data_left -= read_size as i64;

            // Обрезаем буффер на нужный размер
            buffer.truncate(read_size);

            task_sender
                .send(UploadTask {
                    data: Bytes::from(buffer),
                    block_id: format!("{:08}", index),
                    index,
                })
                .await
                .map_err(|e| {
                    MicrosoftAzureError::UploadingError(
                        SpanTrace::capture(),
                        format!("Upload task send failed ({})", e),
                    )
                })?;

            index += 1;

            // Может уже есть какие-то результаты, получим их тогда заранее
            loop {
                match result_receiver.try_recv() {
                    Result::Ok(result) => {
                        let result = result?;
                        debug!("Finished uploading for block: {:?}", result);
                        blocks.push(result);
                    }
                    Result::Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                        break;
                    }
                    Result::Err(tokio::sync::mpsc::error::TryRecvError::Closed) => {
                        return Err(MicrosoftAzureError::UploadingError(
                            SpanTrace::capture(),
                            "Receive channel cannot be closed in progress of uploading".to_owned(),
                        ));
                    }
                }
            }

            debug!(
                "Microsoft azure: bytes upload left {}",
                data_left
                    .file_size(humansize::file_size_opts::BINARY)
                    .map_err(|err| {
                        MicrosoftAzureError::HumanSizeError(SpanTrace::capture(), err)
                    })?
            );
        }
        drop(task_sender);

        // Получаем накопленные результаты
        while let Some(result) = result_receiver.recv().await {
            let result = result?;
            blocks.push(result);
        }
        blocks.sort_by_key(|v| v.index);
        drop(result_receiver);

        // Непосредственно выгрузка
        // https://docs.microsoft.com/en-us/rest/api/storageservices/put-block-list
        let data = {
            let mut data = String::from(r#"<?xml version="1.0" encoding="utf-8"?><BlockList>"#);
            for block_info in blocks.into_iter() {
                data.push_str("<Latest>");
                data.push_str(&block_info.block_id);
                data.push_str("</Latest>");
            }
            data.push_str("</BlockList>");
            data
            //   <Committed>first-base64-encoded-block-id</Committed>
            //   <Uncommitted>second-base64-encoded-block-id</Uncommitted>
            //   <Latest>third-base64-encoded-block-id</Latest>
            //   ...
            // </BlockList>"#;
        };
        let mut url = append_data_url.clone();
        url.query_pairs_mut().append_pair("comp", "blocklist");
        http_client
            .put(url)
            .body(data)
            .send()
            .await?
            .error_for_status()?;

        assert!(
            data_left == 0,
            "Data left must be zero after file uploading"
        );

        Ok(())
    }

    /// Выполнение выгрузки непосредственно файлика с билдом
    /*async fn perform_file_uploading(&self, file_path: &Path) -> Result<(), MicrosoftAzureError> {
        debug!("Microsoft Azure: file uploading start");

        // Получаем чистый HTTP клиент для выгрузки файлика
        let http_client = self.request_builder.get_http_client();

        // Первым этапом идет выставление режима AppendBlob для выгрузки
        // https://docs.microsoft.com/en-us/rest/api/storageservices/put-blob
        // https://docs.microsoft.com/en-us/rest/api/storageservices/put-blob#remarks
        // https://stackoverflow.com/questions/58724878/upload-large-files-1-gb-to-azure-blob-storage-through-web-api
        let blob_create_response = http_client
            .put(&self.data.file_upload_url)
            .header("x-ms-blob-type", "AppendBlob")
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!(
            "Microsoft Azure: blob create response: {:?}",
            blob_create_response
        );

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
        let mut data_offset: i64 = 0;
        loop {
            const BUFFER_MAX_SIZE: i64 = 1024 * 1024 * 4; // 4Mb - ограничение для отдельного куска

            // Размер буффера
            let buffer_size_limit = std::cmp::min(BUFFER_MAX_SIZE, data_left);
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
                    .map_err(|err| {
                        MicrosoftAzureError::HumanSizeError(SpanTrace::capture(), err)
                    })?
            );

            // Отнимаем нужное значения размера данных
            data_left -= read_size as i64;

            // Обрезаем буффер на нужный размер
            buffer.truncate(read_size);

            // Непосредственно выгрузка
            http_client
                .put(append_data_url.clone())
                .header(reqwest::header::CONTENT_LENGTH, read_size.to_string())
                .header("x-ms-blob-condition-appendpos", data_offset)
                .body(buffer)
                .send()
                .await?
                .error_for_status()?;

            data_offset += read_size as i64;

            debug!(
                "Microsoft azure: bytes upload left {}",
                data_left
                    .file_size(humansize::file_size_opts::BINARY)
                    .map_err(|err| {
                        MicrosoftAzureError::HumanSizeError(SpanTrace::capture(), err)
                    })?
            );
        }

        assert!(
            data_left == 0,
            "Data left must be zero after file uploading"
        );

        Ok(())
    }*/

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
                "PreProcessing" | "PendingPublication" | "Certification" | "Publishing"
                | "Published" | "Release" => {
                    break;
                }

                // Ошибочный статус - ошибка
                "CommitFailed"
                | "None"
                | "Canceled"
                | "PublishFailed"
                | "PreProcessingFailed"
                | "CertificationFailed"
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
        if !check_file_extention(zip_file_path, "zip") {
            return Err(MicrosoftAzureError::InvalidUploadFileExtention);
        }

        // Открываем zip файлик и получаем имя .appx там
        let filename = {
            let zip = zip::ZipArchive::new(std::fs::File::open(zip_file_path)?)?;
            let filename_in_zip = zip
                .file_names()
                .find(|full_path_str| {
                    let file_name = std::path::Path::new(full_path_str)
                        .file_name()
                        .and_then(|f| f.to_str());
                    if let Some(file_name) = file_name {
                        !file_name.starts_with('.')
                            && (file_name.ends_with(".appx") || file_name.ends_with(".appxupload"))
                    } else {
                        false
                    }
                })
                .ok_or(MicrosoftAzureError::NoAppxFileInZip)?;
            debug!("Microsoft Azure: filename in zip {}", filename_in_zip);
            filename_in_zip.to_owned() // TODO: не аллоцировать
        };

        // let filename = zip_file_path.file_name().and_then(|n| n.to_str()).unwrap();

        // Обновляем имя пакета
        self.data = self
            .request_builder
            .clone()
            .method(reqwest::Method::PUT)
            .build()
            .await?
            .json(&json!({
                "flightPackages": [
                    {
                      "fileName": filename,
                      "fileStatus": "PendingUpload",
                      "minimumDirectXVersion": "None",
                      "minimumSystemRam": "None"
                    }
                ],
                "targetPublishMode": "Manual",
                // "notesForCertification": "No special steps are required for certification of this app."
            }))
            .send()
            .await?
            .json::<DataOrErrorResponse<FlightSubmissionsCreateResponse>>()
            .await?
            .into_result()?;
        debug!("Microsoft Azure: update response {:#?}", self.data);

        // Выполняем непосредственно выгрузку на сервер нашего архива
        self.perform_file_uploading(zip_file_path).await?;

        self.data = self
            .request_builder
            .clone()
            .method(reqwest::Method::GET)
            .build()
            .await?
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send()
            .await?
            .json::<DataOrErrorResponse<FlightSubmissionsCreateResponse>>()
            .await?
            .into_result()?;
        debug!(
            "Microsoft Azure: update response after upload {:#?}",
            self.data
        );

        // Обновляем имя пакета
        /*self.data = self
            .request_builder
            .clone()
            .method(reqwest::Method::PUT)
            .build()
            .await?
            .json(&json!({
                "flightPackages": [
                    {
                      "fileName": filename,
                      "fileStatus": "Uploaded",
                      "minimumDirectXVersion": "None",
                      "minimumSystemRam": "None"
                    }
                ],
                "targetPublishMode": "Manual",
                // "notesForCertification": "No special steps are required for certification of this app."
            }))
            .send()
            .await?
            .json::<DataOrErrorResponse<FlightSubmissionsCreateResponse>>()
            .await?
            .into_result()?;
        debug!("Microsoft Azure: update response {:#?}", self.data);*/

        // Пытаемся закоммитить
        self.commit_changes().await?;

        // Ждем завершения коммита
        self.wait_commit_finished().await?;

        Ok(())
    }
}
