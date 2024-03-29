use std::{
    path::{
        Path
    }
};
use reqwest::{
    // Client,
    Method,
    RequestBuilder,
    Body
};
use reqwest_inspect_json::{
    InspectJson
};
use futures::{
    future::{
        join_all
    }
};
use tokio::{
    fs::{
        File
    }
};
use tokio_util::{
    codec::{
        FramedRead,
        BytesCodec
    }
};
use log::{
    debug,
    error
};
use super::{
    error::{
        AmazonError
    },
    request_builder::{
        AmazonAppRequestBuilder
    },
    responses::{
        *
    }
};

// Главная документация
// https://developer.amazon.com/docs/app-submission-api/overview.html
// https://developer.amazon.com/docs/app-submission-api/flows.html

///////////////////////////////////////////////////////

struct EditRequestBuilder<'a>{
    request_builder: AmazonAppRequestBuilder<'a>,
    edit_id: String
}
impl<'a> EditRequestBuilder<'a> {
    fn new(request_builder: AmazonAppRequestBuilder<'a>, edit_id: String) -> EditRequestBuilder{
        EditRequestBuilder{
            request_builder,
            edit_id
        }
    }
    fn build_request(&self, method: Method, path: &str) -> Result<RequestBuilder, AmazonError> {
        let path = format!("edits/{}/{}", self.edit_id, path.trim_matches('/'));
        self
            .request_builder
            .build_request(method, &path)
    }
}

///////////////////////////////////////////////////////

pub struct AppEdit<'a>{
    request_builder: EditRequestBuilder<'a>
}
impl<'a> AppEdit<'a> {
    pub async fn new(request_builder: AmazonAppRequestBuilder<'a>) -> Result<AppEdit<'a>, AmazonError>{
        // https://developer.amazon.com/docs/app-submission-api/appsubapi-endpoints.html#/Edits/createEdit_1

        // Получаем незавершенное редактирование или стартуем новое
        let edit_info = {
            // Пытаемся получить прошлое редактирование
            let previous_edit = request_builder
                .build_request(Method::GET, "edits")?
                .send()
                .await?
                .inspect_json::<DataOrErrorResponse<AmazonEditRespone>, AmazonError>(|data|{
                    debug!("Previous edit data: {}", data);
                })
                .await?
                .into_result()?;

            // Пустой ли ответ?
            match previous_edit{
                AmazonEditRespone::Exists(previous_edit_data) => {
                    debug!("Previous edit received: {:#?}", previous_edit_data);
                    previous_edit_data
                },
                AmazonEditRespone::Empty{} => {
                    // Создаем новое редактирование, пустые данные не ожидаем
                    let new_edit = request_builder
                        .build_request(Method::POST, "edits")?
                        .send()
                        .await?
                        .inspect_json::<DataOrErrorResponse<AmazonEditData>, AmazonError>(|data|{
                            debug!("New edit data: {}", data);
                        })
                        .await?
                        .into_result()?;
                    debug!("New edit received: {:#?}", new_edit);
                    new_edit
                }
            }
        };

        debug!("Edit info: {:#?}", edit_info);

        Ok(AppEdit{
            request_builder: EditRequestBuilder::new(request_builder, edit_info.id)
        })      
    }

    async fn get_apks_list(&self) -> Result<Option<Vec<ApkInfoResponse>>, AmazonError> {
        let resp = self.request_builder
            .build_request(Method::GET, "apks")?
            .send()
            .await?;

        if resp.status() == http::StatusCode::OK {
            let values = resp
                .inspect_json::<DataOrErrorResponse<Vec<ApkInfoResponse>>, AmazonError>(|data|{
                    debug!("{}", data);
                })
                .await?
                .into_result()?;
            Ok(Some(values))
        } else if resp.status() == http::StatusCode::NO_CONTENT {
            Ok(None)
        }else{
            Err(AmazonError::ApkListFailedWithCode(resp.status()))
        }
    }

    async fn get_etag_for_apk(&self, info: &ApkInfoResponse) -> Result<String, AmazonError> {
        // https://developer.amazon.com/docs/app-submission-api/appsubapi-endpoints.html#/Edits.apks/get_1
        let response = self.request_builder
            .build_request(Method::GET, &format!("apks/{}", info.id))?
            .send()
            .await?;

        match response.headers().get("ETag"){
            Some(header) => {
                let val = header
                    .to_str()
                    .map_err(|_| AmazonError::ETagParseFailed )?
                    .to_owned();
                Ok(val)
            },
            None => {
                Err(AmazonError::ETagReceiveFailed)
            }
        }
    }

    async fn delete_apk<'b>(&self, info: &'b ApkInfoResponse) -> Result<&'b ApkInfoResponse, AmazonError> {
        // https://developer.amazon.com/docs/app-submission-api/appsubapi-endpoints.html#/Edits.apks/delete_1
        debug!("Try to deleted: {:#?}", info);

        let etag = self.get_etag_for_apk(info).await?;
        debug!("ETag received: {:#?}", etag);

        let resp = self.request_builder
            .build_request(Method::DELETE, &format!("apks/{}", info.id))?
            .header("IF-Match", etag)
            .send()
            .await?;
        if resp.status() == 204 {
            debug!("Delete success: {:#?}", resp);
            Ok(info)
        }else{
            let responce_status = resp.status();
            match resp.json::<ErrorResponseValue>().await{
                Ok(api_err) => {
                    error!("Delete failed: code={}, err={:?}", responce_status, api_err);
                    if let Some(error) = api_err.errors.and_then(|v| v.into_iter().next() ) {
                        Err(AmazonError::ApkDeleteFailedWithCode{
                            code: responce_status, 
                            message: api_err.message, 
                            desc: Some(error.error_message)
                        })
                    }else{
                        Err(AmazonError::ApkDeleteFailedWithCode{code: responce_status, message: None, desc: None})
                    }
                },
                Err(_) => {
                    error!("Delete failed: code={}", responce_status);
                    Err(AmazonError::ApkDeleteFailedWithCode{code: responce_status, message: None, desc: None})
                }
            }
        }
    }

    pub async fn remove_old_apks(&self) -> Result<(), AmazonError> {
        let old_apks = match self.get_apks_list().await?{
            Some(apks) => apks,
            None => return Ok(())
        };

        debug!("Old apks list: {:#?}", old_apks);

        // Итератор по футурам
        let delete_futures_iter = old_apks
            .iter()
            .map(|info|{
                self.delete_apk(info)
            });

        // Ждем результатов
        let results = join_all(delete_futures_iter).await;
            
        // Проверяем ошибки в запросах
        for result in results{
            let deleted_info = result?;
            debug!("Apk deleted: {:#?}", deleted_info);
        }

        Ok(())
    }

    pub async fn upload_new_apk(&self, file_path: &Path) -> Result<ApkInfoResponse, AmazonError>{
        // https://developer.amazon.com/docs/app-submission-api/appsubapi-endpoints.html#/Edits.apks/upload_1

        debug!("Uploading started");

        // Имя
        // let file_name = file_path
        //     .file_name()
        //     .ok_or(AmazonError::WrongFilePath)?
        //     .to_str()
        //     .ok_or(AmazonError::WrongFilePath)?;

        // Файлик в виде стрима
        let file = File::open(file_path).await?;
        let file_length = file.metadata().await?.len();
        let reader = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(reader);

        let response = self.request_builder
            .build_request(Method::POST, "apks/upload")?
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", file_length)
            // .header("fileName", file_name)
            .body(body)
            .send()
            .await?
            .inspect_json::<DataOrErrorResponse<ApkInfoResponse>, AmazonError>(|data|{
                debug!("{}", data);
            })
            .await?
            .into_result()?;

        debug!("Uploading finished: {:#?}", response);

        Ok(response)
    }

    /*pub async fn validate(&self) -> Result<AmazonEditRespone, AmazonError>{
        // https://developer.amazon.com/docs/app-submission-api/appsubapi-endpoints.html#/Edits/validateEdit_1
        let response = self.request_builder
            .build_request(Method::POST, "validate")?
            .send()
            .await?
            .json::<AmazonEditRespone>()
            .await?;

        Ok(response)
    }*/

    /*pub async fn commit_apk<'b>(&self, info: &'b ApkInfoResponse) -> Result<AmazonEditRespone, AmazonError>{
        debug!("Commit with info: {:#?}", info);

        let etag = self
            .get_etag_for_apk(info).await?;

        let response = self.request_builder
            .build_request(Method::POST, "commit")?
            .header("If-Match", etag)
            .send()
            .await?
            .json::<AmazonEditRespone>()
            .await?;

        Ok(response)
    }*/
}