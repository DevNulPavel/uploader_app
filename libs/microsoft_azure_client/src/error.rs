use super::responses::{ErrorResponseValue, SubmissionStatusResponse};
use quick_error::quick_error;
use serde_json_string_parse::JsonParseError;

quick_error! {
    #[derive(Debug)]
    pub enum MicrosoftAzureError{
        /// Ошибка парсинга контекста с дополнительной информацией того места, где произашла ошибка
        UrlParseError(err: url::ParseError){
            from()
            display("{}", err)
        }

        /// Ошибка при работе с сетью
        NetErr(err: reqwest::Error){
            from()
            display("{}", err)
        }

        /// Токен просрочен
        TokenIsExpired {
            display("Microsoft Azure token is expired")
        }

        /// Ошибка в парсинге JSON ответа
        JsonParsingError(err: JsonParseError<String>){
            from()
            display("{}", err)
        }

        /// REST API вернуло ошибку
        RestApiResponseError(err: ErrorResponseValue){
            from()
            display("{:?}", err)
            //from()                                  // Конвертируем из типа ErrorResponseValue
            //from(err: ErrorResponse) -> (err.error) // Конвертируем из типа ErrorResponse
        }

        /// Ошибка парсинга урла на компоненты в RequestBuilder
        UnvalidUrlSegments {
        }

        /// Ошибка построения урла через RequestBuilder
        RequestBuilderFail(info: &'static str){
            display("{}", info)
        }

        /// Ошибка, что расширение файлика для выгрузки неправильное
        InvalidUploadFileExtention{
        }

        /// Системная IO ошибка
        IOError(err: std::io::Error){
            from()
            display("{}", err)
        }

        /// Нету файлика по этому пути
        NoFile(path: std::path::PathBuf){
            display("{}", path.display())
        }

        /// Проблема красивой записи размера
        HumanSizeError(info: String){
            display("{}", info)
        }

        /// Не выгрузилось нормально на сервер
        UploadingError(info: String){
            display("{}", info)
        }

        /// Ошибка при открытии файлика
        ZipOpenFailed(err: zip::result::ZipError){
            from()
            display("{}", err)
        }

        /// Внутри .zip файлика нету .appx / .appxupload
        NoAppxFilesInZip{
            display("No file for store in provided .zip archive")
        }

        /// Получили какой-то кривой статус коммита
        InvalidCommitStatus(status: String){
            display("Unknown commit status: {}", status)
        }

        /// Получили какой-то кривой статус коммита
        CommitFailed(response_data: SubmissionStatusResponse){
            display("{:?}", response_data)
        }

        // /// Проблема с Mutex в корутине выгрузки
        // MutexError{

        // }
    }
}
