use crate::error::MicrosoftAzureError;
use bytes::Bytes;
use humansize::FileSize;
use reqwest::Client;
use std::path::Path;
use tokio::{fs::File, io::AsyncReadExt, sync::mpsc, time};
use tracing::{debug, warn, Instrument};
use tracing_error::SpanTrace;
use url::Url;

const UPLOAD_THREADS_COUNT: usize = 8;
const BUFFER_MAX_SIZE: i64 = 1024 * 1024 * 8; // 8Mb - ограничение для отдельного куска

// Задача выгрузки
#[derive(Debug)]
struct UploadTask {
    data: Bytes,
    block_id: String,
    index: usize,
}

// Результат выгрузки
#[derive(Debug)]
struct UploadResult {
    block_id: String,
    index: usize,
}

async fn enable_block_mode(http_client: &Client, url: &Url) -> Result<(), MicrosoftAzureError> {
    // Первым этапом идет выставление режима AppendBlob для выгрузки
    // https://docs.microsoft.com/en-us/rest/api/storageservices/put-blob
    // https://docs.microsoft.com/en-us/rest/api/storageservices/put-blob#remarks
    // https://stackoverflow.com/questions/58724878/upload-large-files-1-gb-to-azure-blob-storage-through-web-api
    http_client
        .put(url.clone())
        .header("x-ms-blob-type", "BlockBlob")
        .header(reqwest::header::CONTENT_LENGTH, "0")
        .send()
        .in_current_span()
        .await?
        .error_for_status()?;
    Ok(())
}

/// Запускаем отдельные корутины с воркерами откгрузки
fn spawn_uploaders(
    http_client: &Client,
    url: &Url,
    task_receiver: async_channel::Receiver<UploadTask>,
    result_sender: mpsc::Sender<Result<UploadResult, MicrosoftAzureError>>,
) {
    for _ in 0..UPLOAD_THREADS_COUNT {
        let task_receiver = task_receiver.clone();
        let url = url.clone();
        let http_client = http_client.clone();
        let mut result_sender = result_sender.clone();

        tokio::spawn(async move {
            // Если делать: while let Some(task) = task_receiver.lock().await.recv().await
            // Тогда блокировка висит во время всего выполнения цикла
            while let Ok(task) = task_receiver.recv().in_current_span().await {
                let UploadTask {
                    data,
                    block_id,
                    index,
                } = task;

                debug!("Start uploading for block index: {}", index);

                let mut url = url.clone();
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
                            .in_current_span()
                            .await?
                            .error_for_status()?;

                        Result::<UploadResult, MicrosoftAzureError>::Ok(UploadResult {
                            block_id: block_id.clone(),
                            index,
                        })
                    };

                    let res = upload_fn.in_current_span().await;
                    if res.is_ok() {
                        break res;
                    } else {
                        iter_count += 1;
                        if iter_count <= 3 {
                            warn!(
                                "Retry uploading for url: {}, iteration: {}, res: {:?}",
                                url, iter_count, res
                            );
                            tokio::time::delay_for(time::Duration::from_secs(3))
                                .in_current_span()
                                .await;
                            continue;
                        } else {
                            break res;
                        }
                    }
                };
                if result_sender.send(result).in_current_span().await.is_err() {
                    return;
                }
            }
        });
    }
}

async fn commit_blocks(
    http_client: Client,
    url: &Url,
    blocks: Vec<UploadResult>,
) -> Result<(), MicrosoftAzureError> {
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
        let mut list_commit_url = url.clone();
        list_commit_url
            .query_pairs_mut()
            .append_pair("comp", "blocklist");
        list_commit_url
    };
    http_client
        .put(list_commit_url)
        .body(data)
        .send()
        .in_current_span()
        .await?
        .error_for_status()?;
    Ok(())
}

/// Выполнение выгрузки непосредственно файлика с билдом
pub async fn perform_file_uploading(
    http_client: Client,
    url: Url,
    file_path: &Path,
) -> Result<(), MicrosoftAzureError> {
    debug!("Microsoft Azure: file uploading start");

    // Первым этапом идет выставление режима AppendBlob для выгрузки
    enable_block_mode(&http_client, &url)
        .in_current_span()
        .await?;

    // Подготавливаем файлик для потоковой выгрузки
    let mut source_file = File::open(file_path).in_current_span().await?;

    // Получаем суммарный размер данных
    let source_file_length = source_file.metadata().in_current_span().await?.len();

    // Массив с результатами
    let mut blocks = Vec::<UploadResult>::new();

    // Создаем каналы для задач и результатов
    let (task_sender, task_receiver) =
        async_channel::bounded::<UploadTask>(UPLOAD_THREADS_COUNT * 2);
    let (result_sender, mut result_receiver) =
        mpsc::channel::<Result<UploadResult, MicrosoftAzureError>>(UPLOAD_THREADS_COUNT * 8);

    // Создаем воркеры для отгрузки
    spawn_uploaders(&http_client, &url, task_receiver, result_sender);

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
        let read_size = source_file
            .read_exact(&mut buffer)
            .in_current_span()
            .await?;

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
                .map_err(|err| MicrosoftAzureError::HumanSizeError(SpanTrace::capture(), err))?,
            source_file_length
                .file_size(humansize::file_size_opts::BINARY)
                .map_err(|err| MicrosoftAzureError::HumanSizeError(SpanTrace::capture(), err))?,
        );
    }
    drop(task_sender);

    // Получаем накопленные результаты, которые еще не получили
    while let Some(result) = result_receiver.recv().in_current_span().await {
        let result = result?;
        blocks.push(result);
    }
    drop(result_receiver);

    // Теперь сортируем результаты дл правильного порядка следования
    blocks.sort_by_key(|v| v.index);

    // Непосредственно выгрузка списка в правильном порядке
    // https://docs.microsoft.com/en-us/rest/api/storageservices/put-block-list
    commit_blocks(http_client, &url, blocks).await?;

    // TODO: !!!
    assert!(
        data_left == 0,
        "Data left must be zero after file uploading"
    );

    Ok(())
}
