use quick_error::{
    quick_error
};
use tracing_error::{
    SpanTrace
};
use super::{
    responses::{
        ErrorResponseValue,
        SubmissionStatusResponse
    }
};

quick_error!{
    #[derive(Debug)]
    pub enum MicrosoftAzureError{
        /// Ошибка парсинга контекста с дополнительной информацией того места, где произашла ошибка
        UrlParseError(context: SpanTrace, err: url::ParseError){
            from(err: url::ParseError) -> (SpanTrace::capture(), err)
        }

        /// Ошибка при работе с сетью
        NetErr(context: SpanTrace, err: reqwest::Error){
            from(err: reqwest::Error) -> (SpanTrace::capture(), err)
        }

        /// Токен просрочен
        TokenIsExpired {
            display("Microsoft Azure token is expired")
        }

        /// REST API вернуло ошибку
        RestApiResponseError(context: SpanTrace, err: ErrorResponseValue){
            from(err: ErrorResponseValue) -> (SpanTrace::capture(), err)
            //from()                                  // Конвертируем из типа ErrorResponseValue
            //from(err: ErrorResponse) -> (err.error) // Конвертируем из типа ErrorResponse
        }

        /// Ошибка парсинга урла на компоненты в RequestBuilder
        UnvalidUrlSegments(context: SpanTrace){
        }

        /// Ошибка построения урла через RequestBuilder
        RequestBuilderFail(info: &'static str){
        }

        /// Ошибка, что расширение файлика для выгрузки неправильное
        InvalidUploadFileExtention{
        }

        /// Системная IO ошибка
        IOError(context: SpanTrace, err: std::io::Error){
            from(err: std::io::Error) -> (SpanTrace::capture(), err)
        }

        /// Нету файлика по этому пути
        NoFile(path: std::path::PathBuf){
        }

        /// Ошибка при открытии файлика
        ZipOpenFailed(err: zip::result::ZipError){
            from()
        }

        /// Внутри .zip файлика нету .appx / .appxupload
        NoAppxFileInZip{
        }

        /// Получили какой-то кривой статус коммита
        InvalidCommitStatus(status: String){
        }

        /// Получили какой-то кривой статус коммита
        CommitFailed(response_data: SubmissionStatusResponse){
        }
    }
}