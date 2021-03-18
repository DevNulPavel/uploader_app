use quick_error::{
    quick_error
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
        UrlParseError(context: &'static str, err: url::ParseError){
            context(context: &'static str, err: url::ParseError) -> (context, err)
        }

        /// Ошибка при работе с сетью
        NetErr(err: reqwest::Error){
            from()
        }

        /// Токен просрочен
        TokenIsExpired{
            display("Microsoft Azure token is expired")
        }

        /// REST API вернуло ошибку
        RestApiResponseError(err: ErrorResponseValue){
            from()                                  // Конвертируем из типа ErrorResponseValue
            //from(err: ErrorResponse) -> (err.error) // Конвертируем из типа ErrorResponse
        }

        /// Ошибка парсинга урла на компоненты в RequestBuilder
        UnvalidUrlSegments{
        }

        /// Ошибка построения урла через RequestBuilder
        RequestBuilderFail(info: &'static str){
        }

        /// Ошибка, что расширение файлика для выгрузки неправильное
        InvalidUploadFileExtention{
        }

        /// Системная IO ошибка
        IOError(context: &'static str, err: std::io::Error){
            context(context: &'static str, err: std::io::Error) -> (context, err)
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