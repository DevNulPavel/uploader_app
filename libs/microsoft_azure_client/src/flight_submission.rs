use crate::{
    error::MicrosoftAzureError,
    helpers::check_file_extention,
    request_builder::RequestBuilder,
    responses::{
        DataOrErrorResponse, FlightCreateResponse, FlightInfoResponse,
        FlightSubmissionCommitResponse, FlightSubmissionsCreateResponse, SubmissionStatusResponse,
    },
};
use bytes::Bytes;
use humansize::FileSize;
use serde_json::json;
use std::path::Path;
use tokio::{fs::File, io::AsyncReadExt, sync::mpsc, time};
use tracing::{debug, instrument, warn, Instrument};
use tracing_error::SpanTrace;

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
        groups: &[&str],
        test_flight_name: &str,
    ) -> Result<FlightSubmission, MicrosoftAzureError> {
        // Выполняем запрос создания нового сабмишена
        // https://docs.microsoft.com/en-us/windows/uwp/monetize/create-a-flight
        let new_flight_info = request_builder
            .clone()
            .method(reqwest::Method::POST)
            .join_path("flights".to_owned())
            .build()
            .in_current_span()
            .await?
            .json(&json!({
                "groupIds": groups,
                "friendlyName": test_flight_name,
                // "rankHigherThan": null
            }))
            .send()
            .in_current_span()
            .await?
            .json::<DataOrErrorResponse<FlightCreateResponse>>()
            .in_current_span()
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

        // Получим информацию для данного flightId
        let flight_info = request_builder
            .clone()
            .method(reqwest::Method::GET)
            .build()
            .in_current_span()
            .await?
            .send()
            .in_current_span()
            .await?
            .json::<DataOrErrorResponse<FlightInfoResponse>>()
            .in_current_span()
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
                .in_current_span()
                .await?
                .send()
                .in_current_span()
                .await?
                .json::<DataOrErrorResponse<FlightSubmissionsCreateResponse>>()
                .in_current_span()
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
                .in_current_span()
                .await?
                .header(reqwest::header::CONTENT_LENGTH, 0)
                .send()
                .in_current_span()
                .await?
                .json::<DataOrErrorResponse<FlightSubmissionsCreateResponse>>()
                .in_current_span()
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
        http_client
            .put(&self.data.file_upload_url)
            .header("x-ms-blob-type", "BlockBlob")
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send()
            .await?
            .error_for_status()?;

        // Создаем непосредственно урл для добавления данных
        let append_data_url = reqwest::Url::parse(&self.data.file_upload_url)?;

        // Подготавливаем файлик для потоковой выгрузки
        let mut source_file = File::open(file_path).await?;

        // Получаем суммарный размер данных
        let source_file_length = source_file.metadata().await?.len();

        // Массив с результатами
        let mut blocks = Vec::<UploadResult>::new();

        // Задача и результат выгрузки
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

        // Создаем каналы для задач и результатов
        let (task_sender, task_receiver) =
            async_channel::bounded::<UploadTask>(UPLOAD_THREADS_COUNT * 2);
        let (result_sender, mut result_receiver) =
            mpsc::channel::<Result<UploadResult, MicrosoftAzureError>>(UPLOAD_THREADS_COUNT * 8);

        // Создаем воркеры для отгрузки
        for _ in 0..UPLOAD_THREADS_COUNT {
            let task_receiver = task_receiver.clone();
            let append_data_url = append_data_url.clone();
            let http_client = http_client.clone();
            let mut result_sender = result_sender.clone();

            tokio::spawn(async move {
                // Если делать: while let Some(task) = task_receiver.lock().await.recv().await
                // Тогда блокировка висит во время всего выполнения цикла
                while let Ok(task) = task_receiver.recv().await {
                    let UploadTask {
                        data,
                        block_id,
                        index,
                    } = task;

                    debug!("Start uploading for block index: {}", index);

                    let mut url = append_data_url.clone();
                    url.query_pairs_mut()
                        .append_pair("comp", "block")
                        .append_pair("blockid", &block_id);

                    let mut iter_count = 0;
                    let result = loop {
                        let upload_fn = async {
                            // Непосредственно выгрузка
                            http_client
                                .put(url.clone())
                                .header(reqwest::header::CONTENT_LENGTH, data.len())
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
                }
            });
        }
        drop(result_sender);
        drop(task_receiver);

        // Оставшийся размер выгрузки
        let mut data_left = source_file_length as i64;
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

            // Отнимаем нужное значения размера данных
            data_left -= read_size as i64;

            // Обрезаем буффер на нужный размер
            buffer.truncate(read_size);

            // Отправляем задачу выгрузки
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

            // +1 к индексу
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
                "Microsoft azure: bytes upload progress {}/{}",
                data_left
                    .file_size(humansize::file_size_opts::BINARY)
                    .map_err(|err| MicrosoftAzureError::HumanSizeError(
                        SpanTrace::capture(),
                        err
                    ))?,
                source_file_length
                    .file_size(humansize::file_size_opts::BINARY)
                    .map_err(|err| MicrosoftAzureError::HumanSizeError(
                        SpanTrace::capture(),
                        err
                    ))?,
            );
        }
        drop(task_sender);

        // Получаем накопленные результаты, которые еще не получили
        while let Some(result) = result_receiver.recv().await {
            let result = result?;
            blocks.push(result);
        }
        drop(result_receiver);

        // Теперь сортируем результаты дл правильного порядка следования
        blocks.sort_by_key(|v| v.index);

        // Непосредственно выгрузка списка в правильном порядке
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
        };
        let list_commit_url = {
            let mut list_commit_url = append_data_url.clone();
            list_commit_url
                .query_pairs_mut()
                .append_pair("comp", "blocklist");
            list_commit_url
        };
        http_client
            .put(list_commit_url)
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

    /// Выгружаем наш билд
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
        let filename_in_zip = {
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
                .ok_or(MicrosoftAzureError::NoAppxFileInZip)?
                .to_owned();
            filename_in_zip
        };
        debug!("Microsoft Azure: filename in zip {}", filename_in_zip);

        // Обновляем имя пакета
        self.data = self
            .request_builder
            .clone()
            .method(reqwest::Method::PUT)
            .build()
            .in_current_span()
            .await?
            .json(&json!({
                "flightPackages": [
                    {
                      "fileName": filename_in_zip,
                      "fileStatus": "PendingUpload",
                      "minimumDirectXVersion": "None",
                      "minimumSystemRam": "None"
                    }
                ],
                "targetPublishMode": "Manual",
                // "notesForCertification": "No special steps are required for certification of this app."
            }))
            .send()
            .in_current_span()
            .await?
            .json::<DataOrErrorResponse<FlightSubmissionsCreateResponse>>()
            .in_current_span()
            .await?
            .into_result()?;
        debug!("Microsoft Azure: update response {:#?}", self.data);

        // Выполняем непосредственно выгрузку на сервер нашего архива
        self.perform_file_uploading(zip_file_path)
            .in_current_span()
            .await?;

        // Пытаемся закоммитить
        self.commit_changes().in_current_span().await?;

        // Ждем завершения коммита
        self.wait_commit_finished().in_current_span().await?;

        Ok(())
    }
}
