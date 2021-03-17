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
    Body,
    // multipart::{
        // Form,
        // Part
    // }
};
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

        // Получаем имя файлика внутри нашего .appxupload архива
        /*let internal_appx_file_name = {
            // Получаем имя файлика без расширения appxupload
            let file_stem = appxupload_file_path
                .file_stem()
                .and_then(|file_stem|{
                    file_stem.to_str()
                })
                .ok_or(MicrosoftAzureError::InvalidUploadFileExtention)?;
            
            format!("{}.appx", file_stem)
        };*/
        let file_name = appxupload_file_path
            .file_name()
            .and_then(|name|{
                name.to_str()
            })
            .ok_or(MicrosoftAzureError::InvalidUploadFileExtention)?;

        debug!("New file name: {}", file_name);

        // Модифицируем текущие полученные данные, добавляя туда имя файлика
        // https://docs.microsoft.com/en-us/windows/uwp/monetize/update-an-app-submission
        self.data.app_packages.push(SubmissionCreateAppPackageInfo{
            file_name: file_name.to_owned(),
            file_status: "PendingUpload".to_owned(),
            minimum_direct_x: "None".to_owned(),
            minimum_ram: "None".to_owned(),
            other_fields: Default::default()
        });

        // Обновление данных у сабмишена
        self
            .update_server_application_submission_info()
            .await?;

        // Подготавливаем файлик для потоковой выгрузки
        let file = File::open(appxupload_file_path).await.context("Upload file open error")?;
        let file_length = file.metadata().await.context("Upload file metadate receive error")?.len();
        let reader = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(reader);

        // let multipart = Form::new()
            // .part("meta", Part::text("{}")
            //         .mime_str("application/json; charset=UTF-8")
            //         .expect("Meta set failed"))
            // .part("body", Part::stream_with_length(body, file_length)
            //         .file_name(file_name.to_owned()));

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