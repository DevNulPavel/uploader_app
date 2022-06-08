mod error;
mod responses;

use crate::{
    error::{convert_error, FacebookInstantError},
    responses::{ResponseWrapper, TokenResponse, UploadResponse},
};
use serde_json_string_parse_helper::ParseJson;
use reqwest::{
    multipart::{Form, Part},
    Body, Client,
};
use std::path::PathBuf;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use tracing::{debug, Instrument};
// use backtrace::Backtrace as BacktraceNoStd;

/// Клиент для работы с отгрузкой Facebook instant games
pub struct FacebookInstantClient {
    http_client: Client,
    app_id: String,
    token: String,
}

impl FacebookInstantClient {
    /// Создаем новы клиент для работы с Facebook instant
    pub async fn new(
        http_client: Client,
        app_id: String,
        app_secret: String,
    ) -> Result<Self, FacebookInstantError> {
        // Делаем запрос на получение токена работы с FB
        let token_info = http_client
            .get("https://graph.facebook.com/oauth/access_token")
            .query(&[
                ("client_id", app_id.as_str()),
                ("client_secret", app_secret.as_str()),
                ("grant_type", "client_credentials"),
            ])
            .send()
            .in_current_span()
            .await
            .map_err(convert_error!(Request, "Token request"))?
            .text()
            .in_current_span()
            .await
            .map_err(convert_error!(ResponseReceiving, "Token request"))?
            .parse_json_with_data_err::<ResponseWrapper<TokenResponse>>()
            .map_err(convert_error!(ResponseParsing, "Token request"))?
            .into_result()
            .map_err(convert_error!(ApiResponse, "Token request"))?;
        drop(app_secret);
        debug!("Received token info from Facebook: {:?}", token_info);

        Ok(FacebookInstantClient {
            app_id,
            token: token_info.access_token,
            http_client,
        })
    }

    /// Непосредтственно выгрузка билда
    pub async fn upload(
        &self,
        zip_file_path: PathBuf,
        commentary: String,
    ) -> Result<(), FacebookInstantError> {
        debug!("Start facebook uploading");

        // Есть ли файлик?
        if !zip_file_path.exists() {
            return Err(FacebookInstantError::NoFileAtPath {
                path: zip_file_path,
                // backtrace: BacktraceNoStd::new(),
            });
        }

        // Это .zip файлик? Проверим расширение
        if zip_file_path
            .extension()
            .and_then(|v| v.to_str())
            .map(|v| v.to_lowercase())
            .as_deref()
            != Some("zip")
        {
            return Err(FacebookInstantError::NotZipFile {
                path: zip_file_path,
                // backtrace: BacktraceNoStd::new(),
            });
        }

        // Сразу получим имя файлика
        let file_name = match zip_file_path.as_path().file_name().and_then(|v| v.to_str()) {
            Some(v) => v,
            None => {
                return Err(FacebookInstantError::NoZipFilename {
                    path: zip_file_path,
                    // backtrace: BacktraceNoStd::new(),
                });
            }
        };

        // Файлик в виде стрима
        let file = File::open(&zip_file_path)
            .await
            .map_err(convert_error!(IO, "Zip file opening"))?;
        let file_length = file
            .metadata()
            .await
            .map_err(convert_error!(IO, "Zip file metadata"))?
            .len();
        let body = Body::wrap_stream(FramedRead::new(file, BytesCodec::new()));

        // Оформляем в multipart
        let multipart = Form::new()
            .part("access_token", Part::text(self.token.clone()))
            .part("type", Part::text("BUNDLE"))
            .part("comment", Part::text(commentary))
            .part(
                "asset",
                Part::stream_with_length(body, file_length)
                    .file_name(file_name.to_owned())
                    .mime_str(mime::APPLICATION_OCTET_STREAM.essence_str())
                    .map_err(convert_error!(Request, "Multipart request building"))?,
            );

        // Выполняем выгрузку
        let response = self
            .http_client
            .post(format!(
                "https://graph-video.facebook.com/{}/assets",
                self.app_id
            ))
            .multipart(multipart)
            .send()
            .in_current_span()
            .await
            .map_err(convert_error!(Request, "Uploading request"))?
            .error_for_status()
            .map_err(convert_error!(
                ResponseReceiving,
                "Uploading request, error status"
            ))?
            .text()
            .in_current_span()
            .await
            .map_err(convert_error!(
                ResponseReceiving,
                "Uploading request, responce receiving"
            ))?
            .parse_json_with_data_err::<ResponseWrapper<UploadResponse>>()
            .map_err(convert_error!(ResponseParsing, "Token request"))?
            .into_result()
            .map_err(convert_error!(ApiResponse, "Token request"))?;

        debug!("Response after uploading: {:?}", response);

        // Адреса запуска игр:
        // https://www.facebook.com/gaming/play/420544052753044/?
        // source=dev_site&
        // game_url=https%3A%2F%2Fapps-420544052753044.apps.fbsbx.com%2Finstant-bundle%2F177318527884154%2F5275548972507693%2Findex.html%3F__cci%3DFQAREhIA.ARbkHwKtTJDaJWqMw26tTsrfLT0al1jLa0I8vhMPYpxufB3i&
        // ext=1654267178&
        // hash=AeTBYbnFdO2aD2mMf3o

        // Url-decoded game_url:
        // https://apps-420544052753044.apps.fbsbx.com/instant-bundle/177318527884154/5275548972507693/index.html?__cci=FQAREhIA.ARbkHwKtTJDaJWqMw26tTsrfLT0al1jLa0I8vhMPYpxufB3i&

        // https://www.facebook.com/gaming/play/420544052753044/?
        // source=dev_site&
        // game_url=https%3A%2F%2Fapps-420544052753044.apps.fbsbx.com%2Finstant-bundle%2F177318527884154%2F7915669388443981%2Findex.html%3F__cci%3DFQAREhIA.ARbkHwKtTJDaJWqMw26tTsrfLT0al1jLa0I8vhMPYpxufCiR&
        // ext=1654267481&
        // hash=AeQgDH0v4O1h8mEFBKY

        Ok(())
    }
}
