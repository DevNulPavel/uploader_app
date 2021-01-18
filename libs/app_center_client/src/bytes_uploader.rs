use std::{
    path::{
        Path
    }
};
use tokio::{
    fs::{
        File,
    },
    io::{
        AsyncReadExt
    }
};
use futures::{
    FutureExt,
};
use serde::{
    Deserialize
};
use bytes::{
    Bytes,
    BytesMut
};
use log::{
    debug
};
use reqwest::{
    Client,
    Body
};
use super::{
    error::{
        AppCenterError
    },
    responses::{
        ReleasesResponse,
        MetaInfoSetResponse,
        UploadingFinishedResponse
    },
    helpers::{
        upload_content_type_for_file
    }
};

//////////////////////////////////////////////////////

async fn upload_file_chunk(http_client: &Client,
                           release_info: &ReleasesResponse,
                           chunk_number: usize,
                           total_chunks: usize,
                           data: Bytes) -> Result<usize, AppCenterError>{

    debug!("Chunk number {}/{} upload started with data length: {}", chunk_number+1, total_chunks, data.len());

    let url = format!("{}/upload/upload_chunk/{}",
                        release_info.upload_domain,
                        release_info.package_asset_id);

    let chunk_number_str = format!("{}", chunk_number + 1);

    let query_params = [
        ("token", &release_info.token),
        ("block_number", &chunk_number_str)
    ];
    
    let length = data.len();
    let body = Body::from(data);

    #[derive(Debug, Deserialize)]
    struct Response{
        error_code: String,
        chunk_num: usize,
        error: bool
    }
    let result = http_client
        .post(&url)
        .query(&query_params)
        .header("Content-Length", length)
        .body(body)
        .send()
        .await?
        .json::<Response>()
        .await?;

        /*.text()
        .await
        .map_err(|err| AppCenterError::ResponseError(err))?;*/
        /*.error_for_status()
        .map_err(|err| AppCenterError::ResponseError(err))? ;*/

    debug!("Chunk number {} upload result: {:#?}", chunk_number, result);

    if result.error {
        Err(AppCenterError::Custom(result.error_code))
    }else{
        Ok(chunk_number)
    }
}

//////////////////////////////////////////////////////

pub struct AppCenterUploader<'a>{
    http_client: Client,
    release_info: &'a ReleasesResponse,
    file_path: &'a Path,
    file: File,
    file_length: u64,
    upload_threads_count: usize
}
impl<'a> AppCenterUploader<'a> {
    pub async fn new(http_client: Client,
                     release_info: &'a ReleasesResponse,
                     file_path: &'a Path,
                     upload_threads_count: usize) -> Result<AppCenterUploader<'a>, AppCenterError> {

        let file = File::open(file_path)
            .await?;

        let file_length = file
            .metadata()
            .await?
            .len();

        Ok(AppCenterUploader{
            http_client,
            release_info,
            file_path,
            file,
            file_length,
            upload_threads_count
        })
    }

    async fn upload_file_stats(&self) -> Result<MetaInfoSetResponse, AppCenterError>{
        let file_name = self
            .file_path
            .file_name()
            .ok_or_else(|| AppCenterError::WrongFilePath )?
            .to_str()
            .ok_or_else(|| AppCenterError::WrongFilePath )?;

        let content_type = upload_content_type_for_file(&self.file_path);

        let file_length = format!("{}", self.file_length);

        let url = format!("{}/upload/set_metadata/{}",
                            self.release_info.upload_domain,
                            self.release_info.package_asset_id);

        let query_params = [
            ("file_name", file_name),
            ("file_size", &file_length),
            ("token", &self.release_info.token),
            ("content_type", content_type)
        ];

        let query = self.http_client
            .post(&url)
            //.header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Content-Length", "0")
            .query(&query_params);

        debug!("File stat upload query: {:#?}", query);

        let resp = query
            .send()
            .await;

        debug!("File stat upload response: {:#?}", resp);

        let result = resp?
            .json::<MetaInfoSetResponse>()
            .await?;

        debug!("File stat upload result: {:#?}", result);

        Ok(result)
    }

    async fn upload_file(&mut self, upload_info: MetaInfoSetResponse) -> Result<(), AppCenterError>{
        // const MAX_UPLOADS_COUNT: usize = 10;

        let mut futures_vec = Vec::with_capacity(self.upload_threads_count);

        let chunks_count = upload_info.chunk_list.len();
        for i in 0..chunks_count {
            // Выделяем буффер
            let buffer: Bytes = {
                let read_position = (i * upload_info.chunk_size) as i64;
                let file_bytes_left = (self.file_length as i64) - read_position;
                assert!(file_bytes_left > 0, "Bytes left must be greater than 0");
                let buffer_size = if file_bytes_left > (upload_info.chunk_size as i64) {
                    upload_info.chunk_size as usize
                }else{
                    file_bytes_left as usize
                };

                // Готовим буффер
                let mut buffer: Vec<u8> = Vec::new();
                buffer.resize(buffer_size, 0);

                // Читаем с текущего места файла только нужное количество байт
                let read_count = self.file
                    .read_exact(&mut buffer)
                    .await?;

                assert_eq!(read_count, buffer_size as usize, "Invalid read size from file");

                Bytes::from(buffer)
            };

            // Кидаем задачу на загрузку
            let fut_in_pined_box = upload_file_chunk(&self.http_client, &self.release_info, i, chunks_count, buffer).boxed();
            futures_vec.push(fut_in_pined_box);

            // Ждем возможности закинуть еще задачу либо ждем завершения всех тасков если дошли до конца
            let limit_val = if i < (chunks_count-1) {
                self.upload_threads_count
            }else{
                0
            };
            while futures_vec.len() > limit_val {
                let (result, _, left_futures) = futures::future::select_all(futures_vec).await;
                let finished_index = result?;
                debug!("Future number {}/{} finished", finished_index, chunks_count);
                futures_vec = left_futures;
            }
        }

        debug!("Uploading loop finished");

        Ok(())
    }

    async fn commit_uploading(&self) -> Result<UploadingFinishedResponse, AppCenterError>{
        let url = format!("{}/upload/finished/{}",
                            self.release_info.upload_domain,
                            self.release_info.package_asset_id);

        let result = self.http_client
            .post(&url)
            .header("Accept", "application/json")
            .header("Content-Length", "0")
            .query(&[("token", &self.release_info.token) ])
            .send()
            .await?
            .json::<UploadingFinishedResponse>()
            .await?;

        debug!("Commit uploading result: {:#?}", result);

        Ok(result)
    }

    pub async fn upload(mut self) -> Result<UploadingFinishedResponse, AppCenterError> {
        let meta_set_result = self
            .upload_file_stats()
            .await?;

        self
            .upload_file(meta_set_result)
            .await?;

        self
            .commit_uploading()
            .await
    }
}