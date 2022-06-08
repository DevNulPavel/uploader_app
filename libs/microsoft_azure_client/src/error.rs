use super::responses::{ErrorResponseValue, SubmissionStatusResponse};
use quick_error::quick_error;
use serde_json_string_parse::JsonParseError;

quick_error! {
    #[derive(Debug)]
    pub enum MicrosoftAzureError{
        /// Ошибка парсинга контекста с дополнительной информацией того места, где произашла ошибка
        UrlParseError(err: url::ParseError){
            from()
        }

        /// Ошибка при работе с сетью
        NetErr(err: reqwest::Error){
            from()
        }

        /// Токен просрочен
        TokenIsExpired {
            display("Microsoft Azure token is expired")
        }

        /// Ошибка в парсинге JSON ответа
        JsonParsingError(err: JsonParseError<String>){
            from()
        }

        /// REST API вернуло ошибку
        RestApiResponseError(err: ErrorResponseValue){
            from()
            //from()                                  // Конвертируем из типа ErrorResponseValue
            //from(err: ErrorResponse) -> (err.error) // Конвертируем из типа ErrorResponse
        }

        /// Ошибка парсинга урла на компоненты в RequestBuilder
        UnvalidUrlSegments {
        }

        /// Ошибка построения урла через RequestBuilder
        RequestBuilderFail(info: &'static str){
        }

        /// Ошибка, что расширение файлика для выгрузки неправильное
        InvalidUploadFileExtention{
        }

        /// Системная IO ошибка
        IOError(err: std::io::Error){
            from()
        }

        /// Нету файлика по этому пути
        NoFile(path: std::path::PathBuf){
        }

        /// Проблема красивой записи размера
        HumanSizeError(info: String){
        }

        /// Не выгрузилось нормально на сервер
        UploadingError(info: String){
        }

        /// Ошибка при открытии файлика
        ZipOpenFailed(err: zip::result::ZipError){
            from()
        }

        /// Внутри .zip файлика нету .appx / .appxupload
        NoAppxFilesInZip{
        }

        /// Получили какой-то кривой статус коммита
        InvalidCommitStatus(status: String){
        }

        /// Получили какой-то кривой статус коммита
        CommitFailed(response_data: SubmissionStatusResponse){
        }

        // /// Проблема с Mutex в корутине выгрузки
        // MutexError{

        // }
    }
}
