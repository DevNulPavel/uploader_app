use std::{
    path::{
        Path
    }
};
use tracing::{
    debug
};
use humansize::{
    FileSize
};
use tokio::{
    fs::{
        File
    },
    io::{
        // AsyncBufRead,
        // AsyncBufReadExt,
        // AsyncRead,
        AsyncReadExt
    }
};
// use tokio_util::{
//     codec::{
//         BytesCodec,
//         FramedRead
//     }
// };
// use reqwest::{
    // Body,
    // multipart::{
        // Form,
        // Part
    // }
// };
// use serde_json::{
//     json
// };
use crate::{
    request_builder::{
        RequestBuilder
    },
    error::{
        MicrosoftAzureError
    },
    helpers::{
        check_file_extention
    },
    responses::{
        DataOrErrorResponse,
        SubmissionCreateResponse,
        SubmissionCreateAppPackageInfo,
        SubmissionCommitResponse,
        // SubmissionStatusDetails,
        SubmissionStatusResponse
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

    /// Данный метод необходим для обновления информации приложения по текущему сабмишену,
    /// параметры передаются просто в виде Json словаря, так как список параметров огромный
    /// После успешного выполнения запроса, обновляем внутренние данные сабмишена
    /// Информация по параметрам: `https://docs.microsoft.com/en-us/windows/uwp/monetize/update-an-app-submission`
    async fn update_server_application_submission_info(&mut self) -> Result<(), MicrosoftAzureError> {
        debug!("Microsoft Azure: new submission update");

        let new_info = self.request_builder
            .clone()
            .method(reqwest::Method::PUT)
            .build()
            .await?
            .json(&self.data)
            .send()
            .await?
            //.error_for_status()?
            .json::<DataOrErrorResponse<SubmissionCreateResponse>>()
            .await?
            .into_result()?;

        debug!("Microsoft Azure: new submission update result: {:#?}", new_info);

        // Сохряняем новые данные
        self.data = new_info;
        
        Ok(())
    }

    /// Выполнение выгрузки непосредственно файлика с билдом
    async fn perform_file_uploading(&mut self, zip_file_path: &Path) -> Result<(), MicrosoftAzureError>{
        debug!("Microsoft Azure: file uploading start");

        // Получаем чистый HTTP клиент для выгрузки файлика
        let http_client = self.request_builder
            .get_http_client();

        // Первым этапом идет выставление режима AppendBlob для выгрузки
        // https://docs.microsoft.com/en-us/rest/api/storageservices/put-blob
        // https://docs.microsoft.com/en-us/rest/api/storageservices/put-blob#remarks
        // https://stackoverflow.com/questions/58724878/upload-large-files-1-gb-to-azure-blob-storage-through-web-api
        http_client
            .put(&self.data.file_upload_url)
            .header("x-ms-blob-type", "AppendBlob")
            .header(reqwest::header::CONTENT_LENGTH, 0)
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
        let mut source_file = File::open(zip_file_path)
            .await?;

        // Получаем суммарный размер данных
        let source_file_length = source_file
            .metadata()
            .await?
            .len();

        // Оставшийся размер выгрузки
        let mut data_left = source_file_length as i64;
        loop{
            const BUFFER_MAX_SIZE: i64 = 1024*1024*3; // 4Mb - ограничение для отдельного куска

            // Размер буффера
            let buffer_size_limit = std::cmp::min(BUFFER_MAX_SIZE, data_left);
            if buffer_size_limit <= 0 {
                break;
            }

            // TODO: Убрать создание нового буффера каждый раз,
            // Вроде бы как Hyper позволяет использовать slice для выгрузки
            let mut buffer = Vec::<u8>::with_capacity(buffer_size_limit as usize);

            // В данный буффер будем лишь писать сначала, поэтому можно не инициализировать данные ничем
            unsafe{ buffer.set_len(buffer_size_limit as usize); }

            // Читаем из файлика данные в буффер
            let read_size = source_file
                .read_exact(&mut buffer)
                .await?;
            
            debug!("Microsoft azure: bytes read from file {}", read_size.file_size(humansize::file_size_opts::BINARY).unwrap());

            // Отнимаем нужное значения размера данных
            data_left = data_left - (read_size as i64);

            // Обрезаем буффер на нужный размер
            buffer.truncate(read_size);

            // Непосредственно выгрузка
            http_client
                .put(append_data_url.clone())
                .header(reqwest::header::CONTENT_LENGTH, read_size)
                .body(buffer)
                .send()
                .await?
                .error_for_status()?;
            
            debug!("Microsoft azure: bytes upload left {}", data_left.file_size(humansize::file_size_opts::BINARY).unwrap());
        }

        assert!(data_left == 0, "Data left must be zero after file uploading");

        Ok(())
    }

    /// Данный метод занимается тем, что коммитит изменения на сервере
    /// Описание: `https://docs.microsoft.com/en-us/windows/uwp/monetize/commit-an-app-submission` 
    async fn commit_changes(&mut self) -> Result<(), MicrosoftAzureError> {
        debug!("Microsoft Azure: commit request");

        let new_info = self.request_builder
            .clone()
            .method(reqwest::Method::POST)
            .submission_command("commit".to_string())
            .build()
            .await?
            .header(reqwest::header::CONTENT_LENGTH, 0)
            .send()
            .await?
            // .error_for_status()?
            .json::<DataOrErrorResponse<SubmissionCommitResponse>>()
            .await?
            .into_result()?;

        debug!("Microsoft Azure: commit response {:#?}", new_info);

        if !new_info.status.eq("CommitStarted") {
            return Err(MicrosoftAzureError::InvalidCommitStatus(new_info.status));
        }
        
        Ok(())
    }

    /// C помощью данного метода мы ждем завершения выполнения коммита
    /// Описание: `https://docs.microsoft.com/en-us/windows/uwp/monetize/get-status-for-an-app-submission`
    async fn wait_commit_finished(&mut self) -> Result<(), MicrosoftAzureError> {
        debug!("Microsoft Azure: wait submission commit result");

        loop {
            let status_response = self.request_builder
                .clone()
                .method(reqwest::Method::GET)
                .submission_command("status".to_string())
                .build()
                .await?
                .header(reqwest::header::CONTENT_LENGTH, 0)
                .send()
                .await?
                // .error_for_status()?
                .json::<DataOrErrorResponse<SubmissionStatusResponse>>()
                .await?
                .into_result()?;

            debug!("Microsoft Azure: submission status response {:#?}", status_response);

            match status_response.status.as_str() {
                // Нормальное состояние для ожидания
                "CommitStarted" => {
                    tokio::time::delay_for(std::time::Duration::from_secs(15)).await;
                },

                // Коммит прошел успешно, прерываем ожидание
                "PreProcessing" => {
                    break;
                }

                // Ошибочный статус - ошибка
                "CommitFailed" |
                "None" |
                "Canceled" | 
                "PublishFailed" |
                "PendingPublication" |
                "Certification" |
                "Publishing" |
                "Published" |
                "PreProcessingFailed" |
                "CertificationFailed" |
                "Release" |
                "ReleaseFailed" =>{
                    return Err(MicrosoftAzureError::CommitFailed(status_response));
                }

                // Неизвестный статус - ошибка
                _ => {
                    return Err(MicrosoftAzureError::InvalidCommitStatus(status_response.status));
                }
            }
        }
        
        Ok(())
    }

    pub async fn upload_build(&mut self, zip_file_path: &Path) -> Result<(), MicrosoftAzureError>{
        // Может быть нет фалика по этому пути
        if !zip_file_path.exists() {
            return Err(MicrosoftAzureError::NoFile(zip_file_path.to_owned()));
        }

        // Проверяем расширение данного файлика
        if check_file_extention(zip_file_path, "zip") == false{
            return Err(MicrosoftAzureError::InvalidUploadFileExtention);
        }

        // Открываем zip файлик и получаем имя .appx / .appxupload
        let appx_file_path = {
            let zip = zip::ZipArchive::new(std::fs::File::open(zip_file_path)?)?;
            let path = zip
                .file_names()
                .filter(|full_path_str|{
                    let file_name = std::path::Path::new(full_path_str)
                        .file_name()
                        .and_then(|f|{
                            f.to_str()
                        });
                    if let Some(file_name) = file_name {
                        if !file_name.starts_with(".") &&
                            (file_name.ends_with(".appx") || file_name.ends_with(".appxupload")){
                            true
                        }else{
                            false
                        }
                    }else{
                        false
                    }
                })
                .next()
                .ok_or(MicrosoftAzureError::NoAppxFileInZip)?;
            path.to_owned()
        };

        // Передаем имя файлика в информацию о сабмишене
        debug!("Microsoft Azure: .appx or .appxupload file in zip: {}", appx_file_path);

        // Модифицируем текущие полученные данные, добавляя туда имя файлика
        // https://docs.microsoft.com/en-us/windows/uwp/monetize/update-an-app-submission
        self.data.app_packages.push(SubmissionCreateAppPackageInfo{
            file_name: appx_file_path.to_owned(),
            file_status: "PendingUpload".to_owned(),
            minimum_direct_x: "None".to_owned(),
            minimum_ram: "None".to_owned(),
            other_fields: Default::default()
        });

        // Обновление данных у сабмишена на основе изменений текущих
        self
            .update_server_application_submission_info()
            .await?;

        // Выполняем непосредственно выгрузку на сервер нашего архива
        self
            .perform_file_uploading(zip_file_path)
            .await?;
        
        // Пытаемся закоммитить
        self
            .commit_changes()
            .await?;

        // Ждем завершения коммита
        self
            .wait_commit_finished()
            .await?;

        Ok(())
    }
}