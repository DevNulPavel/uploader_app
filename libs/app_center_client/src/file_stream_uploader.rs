use std::{
    path::{
        Path
    },
    io::{
        SeekFrom
    }
};
use tokio::{
    fs::{
        File,
    },
};
use futures::{
    FutureExt,
    // Stream,
    TryStream,
    // TryStreamExt,
    // StreamExt
};
use serde::{
    Deserialize
};
use bytes::{
    Bytes,
    // BytesMut
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
        UploadingFinishedOkResponse,
        // UploadingFinishedErrorResponse,
        UploadingFinishedResponse
    },
    helpers::{
        upload_content_type_for_file
    }
};

//////////////////////////////////////////////////////

async fn upload_file_chunk<S>(http_client: &Client,
                              release_info: &ReleasesResponse,
                              chunk_number: usize,
                              total_chunks: usize,
                              data: S,
                              length: u64) -> Result<usize, AppCenterError>
where
    S: TryStream + Send + Sync + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>, // Ограничение на подтип можно записывать вот так
    S::Ok: Into<Bytes>,
    Bytes: From<S::Ok> // Так же можно записывать ограничения, где первым параметром будет структура, а вторым - трейт
{

    debug!("Chunk number {}/{} upload started with data length: {}", chunk_number+1, total_chunks, length);

    let url = format!("{}/upload/upload_chunk/{}",
                        release_info.upload_domain,
                        release_info.package_asset_id);

    let chunk_number_str = format!("{}", chunk_number + 1);

    let query_params = [
        ("token", &release_info.token),
        ("block_number", &chunk_number_str)
    ];
    
    let body = Body::wrap_stream(data);

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
    file_length: u64,
    upload_threads_count: usize
}
impl<'a> AppCenterUploader<'a> {
    pub async fn new(http_client: Client,
                     release_info: &'a ReleasesResponse,
                     file_path: &'a Path,
                     upload_threads_count: usize) -> Result<AppCenterUploader<'a>, AppCenterError> {

        let file_length = File::open(file_path)
            .await?
            .metadata()
            .await?
            .len();

        Ok(AppCenterUploader{
            http_client,
            release_info,
            file_path,
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

        let mut total_uploaded_size = 0;
        let chunks_count = upload_info.chunk_list.len();
        for i in 0..chunks_count {
            // Есть проблема в Reqwest, если использовать не Stream в качестве Body, тогда сильно много уходит оперативки
            // Внутри Body происходит клонирование буффера
            // Поэтому просто открываем файлик много раз и читаем из разных мест
            let (stream, length) = {
                // Вычисление оставшегося размера буффера
                let read_position = (i * upload_info.chunk_size) as i64;
                let file_bytes_left = (self.file_length as i64) - read_position;
                assert!(file_bytes_left > 0, "Bytes left must be greater than 0");
                let buffer_size = if file_bytes_left > (upload_info.chunk_size as i64) {
                    upload_info.chunk_size as u64
                }else{
                    file_bytes_left as u64
                };
                total_uploaded_size += buffer_size;
                assert!(buffer_size > 0, "Buffer size must be greater than 0");

                // Открыли файлик
                let mut file = File::open(self.file_path)
                    .await?;

                // Сместились на нужное место
                let seek_value = file
                    .seek(SeekFrom::Start(read_position as u64))
                    .await?;
                assert_eq!(seek_value, read_position as u64, "Seek position must be valid");

                // Берем только нужное количество данных из файлика
                // TODO: Почему-то работает криво, используем Content-Length для ограничения
                let file = file.take(buffer_size as u64);

                // Файлик преобразуем в stream
                let stream = tokio_util::codec::FramedRead::new(file, tokio_util::codec::BytesCodec::new());

                // Читаем только нужный размер буффера
                (stream, buffer_size)
            };

            // Кидаем задачу на загрузку
            let fut_in_pined_box = upload_file_chunk(&self.http_client, &self.release_info, i, chunks_count, stream, length).boxed();
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
                debug!("Future number {}/{} finished", finished_index+1, chunks_count);
                futures_vec = left_futures;
            }
        }

        assert_eq!(futures_vec.len(), 0, "Futures count must be zero");
        assert_eq!(total_uploaded_size, self.file_length, "Total size and file length must be equal");

        debug!("Uploading loop finished");

        Ok(())
    }

    async fn commit_uploading(&self) -> Result<UploadingFinishedOkResponse, AppCenterError>{
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
        
        match result {
            UploadingFinishedResponse::Ok(res) => {
                Ok(res)
            },
            UploadingFinishedResponse::Error(err) => {
                Err(AppCenterError::Custom(format!("Commit failed with error: {}", err.error_code)))
            }
        }
    }

    pub async fn upload(mut self) -> Result<UploadingFinishedOkResponse, AppCenterError> {
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