use std::{
    path::{
        Path
    }
};
use log::{
    debug
};
use quick_error::{
    ResultExt
};
use tokio::{
    fs::{
        File
    }
};
use tokio_util::{
    codec::{
        BytesCodec,
        FramedRead
    }
};
use reqwest::{
    Body
};
use serde_json::{
    json
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
        SubmissionCreateResponse,
        SubmissionCreateAppPackageInfo
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
    async fn update_application_submission_info(&mut self) -> Result<(), MicrosoftAzureError> {
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

        //     .text()
        //     .await?;
        // debug!("RESPONE: {}", new_info);
        // Err(MicrosoftAzureError::InvalidUploadFileExtention)
    }

    pub async fn upload_build_file(&mut self, appxupload_file_path: &Path) -> Result<(), MicrosoftAzureError>{
        // TODO: Внутри файла еще лежит символьная информация, может быть ее тоже можно как-то указать

        // Может быть нет фалика по этому пути
        if !appxupload_file_path.exists() {
            return Err(MicrosoftAzureError::NoFile(appxupload_file_path.to_owned()));
        }

        // Проверяем расширение данного файлика
        {
            appxupload_file_path
                .extension()
                .and_then(|ext|{
                    ext.to_str()
                })
                .and_then(|ext|{
                    if ext.eq("appxupload") {
                        Some(())
                    }else{
                        None
                    }
                })
                .ok_or(MicrosoftAzureError::InvalidUploadFileExtention)?;
        }

        // Передаем имя файлика в информацию о сабмишене
        {
            // Получаем имя файлика внутри нашего .appxupload архива
            let internal_appx_file_name = {
                // Получаем имя файлика без расширения appxupload
                let file_stem = appxupload_file_path
                    .file_stem()
                    .and_then(|file_stem|{
                        file_stem.to_str()
                    })
                    .ok_or(MicrosoftAzureError::InvalidUploadFileExtention)?;
                
                format!("{}.appx", file_stem)
            };

            debug!("New file name: {}", internal_appx_file_name);

            // Данные
            // https://docs.microsoft.com/en-us/windows/uwp/monetize/update-an-app-submission
            /*let json_data = json!(
                {
                    "applicationPackages": [
                        {
                            "fileName": internal_appx_file_name,
                            "fileStatus": "PendingUpload"
                        }
                    ]
                }
            );*/
            self.data.app_packages.push(SubmissionCreateAppPackageInfo{
                file_name: internal_appx_file_name,
                file_status: "PendingUpload".to_owned()
            });

            // Обновление данных у сабмишена
            self
                .update_application_submission_info()
                .await?;
        }

        // Подготавливаем файлик для потоковой выгрузки
        let file = File::open(appxupload_file_path).await.context("Upload file open error")?;
        let file_length = file.metadata().await.context("Upload file metadate receive error")?.len();
        let reader = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(reader);

        // Получаем чистый HTTP клиент для выгрузки файлика
        let http_client = self.request_builder.get_http_client();

        // Выполняем выгрузку
        http_client
            .put(&self.data.file_upload_url)
            .header("x-ms-blob-type", "BlockBlob")
            .header(reqwest::header::CONTENT_LENGTH, file_length)
            .body(body)
            .send()
            .await?
            .error_for_status()?;
        
        Ok(())
    }
}