
    /// Выполнение выгрузки непосредственно файлика с билдом
    /// Способ выгрузки через Append работает что-то не очень хорошо,
    /// поэтому переделано на вариант с блочной выгрузкой
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
