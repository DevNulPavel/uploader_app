use std::{
    sync::{
        Arc,
        Mutex
    }, 
    time::{
        Instant,
        Duration
    },
    ops::{
        Add,
        Sub
    }
};
use log::{
    // info,
    debug
};
use quick_error::{
    ResultExt
};
use reqwest::{
    Client,
    header::{
        CONTENT_TYPE
    }
};
use url::{
    Url
};
use crate::{
    error::{
        MicrosoftAzureError
    },
    responses::{
        DataOrErrorResponse,
        TokenResponse
    }
};

////////////////////////////////////////////////////////////////

/// Внутренняя структура, содержащая общие переменные для запроса и сброса токена
struct InnerData {
    http_client: Client,  // ArcInside
    token_api_url: Url,
    client_id: String,
    client_secret: String,
    token_expire_pre_delay: Duration
}

////////////////////////////////////////////////////////////////

/// Внутренняя структура, которая содержит в себе токен, а так же время окончания его жизни
struct Token{
    data: TokenResponse,
    complete_time: Instant
}

impl From<TokenResponse> for Token {
    fn from(resp: TokenResponse) -> Self {
        let complete_time = Instant::now()
            .add(Duration::from_secs(resp.expires_in));
        Token{
            data: resp,
            complete_time
        }
    }
}

impl Token {
    /// Проверяем, что время жизни истекает скоро, при этом учитываем предзадержку
    fn is_will_be_expired_soon(&self, pre_delay: Duration) -> bool {
        let complete_time_val = self.complete_time.sub(pre_delay);
        Instant::now()
            .gt(&complete_time_val)
    }
}

////////////////////////////////////////////////////////////////

/// Данный провайдер запрашивает токен, выдает его наружу, 
/// если токен истекает через 5 минут, тогда перезапрашивает его самостоятельно.
/// Автоматически обновляя его внутри
pub struct MicrosoftAzureTokenProvider{
    inner: Arc<InnerData>,
    active_token: Arc<Mutex<Option<Token>>>
}

impl MicrosoftAzureTokenProvider{
    pub fn new(http_client: Client,
               api_url: &str,
               tenant_id: &str,
               client_id: String,
               client_secret: String,
               token_expire_pre_delay: Duration) -> Result<MicrosoftAzureTokenProvider, MicrosoftAzureError> {

        let token_api_url = Url::parse(api_url)
            .context("Token url base url error")?
            .join(&format!("{}/oauth2/token", tenant_id))
            .context("Token url join error")?;

        // Создаем Arc на получателя токена
        let inner = Arc::new(InnerData{
            http_client,
            token_api_url,
            client_id,
            client_secret,
            token_expire_pre_delay
        });

        // Создаем нашу структуру, изначально токен пустой
        Ok(MicrosoftAzureTokenProvider{
            inner,
            active_token: Default::default()
        })
    }

    pub async fn get_access_token(&self) -> Result<String, MicrosoftAzureError> {
        // TODO: Как-то сделать лучше

        // Делаем некую машину состояний, чтобы вернуть валидный рабочий токен после всех проверок
        let mut token_guard = self.active_token.lock().expect("Token mutex lock failed");
        loop{
            if let Some(token) = token_guard.as_mut() {
                // Если токен не заканчивает время жизни через 5 минут, то все норм
                // Иначе все делаем сброс
                if token.is_will_be_expired_soon(self.inner.token_expire_pre_delay) == false {
                    debug!("Token is OK");

                    // Все нормально - возвращаем склонированное значение токена
                    // Клонировать приходится, так как иначе придется долго держать блокировку
                    return Ok(token.data.access_token.clone());
                }else{
                    debug!("Token is expired soon, refresh it");

                    // Вызываем обновление токена
                    let new_token = self
                        .request_azure_token(Some(&token.data.refresh_token))
                        .await?;

                    // После установки идем на очередную итерацию цикла
                    token_guard.replace(new_token);
                }
            }else{
                debug!("Token is empty, request new one");

                // Если токена у нас вообще нету, тогда запрашиваем
                let new_token = self
                    .request_azure_token(None)
                    .await?;

                // После установки идем на очередную итерацию цикла
                token_guard.replace(new_token);
            }
        }
        // unreachable!("This code is unreacheble");
    }

    async fn request_azure_token(&self, refresh_token: Option<&str>) -> Result<Token, MicrosoftAzureError> {
        // https://docs.microsoft.com/en-us/azure/active-directory/azuread-dev/v1-protocols-oauth-code#refreshing-the-access-tokens
        // https://docs.microsoft.com/en-us/windows/uwp/monetize/create-and-manage-submissions-using-windows-store-services#obtain-an-azure-ad-access-token
        // https://docs.microsoft.com/en-us/windows/uwp/monetize/python-code-examples-for-the-windows-store-submission-api#create-app-submission
        // https://docs.microsoft.com/en-us/azure/active-directory/azuread-dev/v1-protocols-oauth-code#refreshing-the-access-tokens
        
        // Получение нового токена и сброс отличаются лишь параметрами запроса
        let query_params = {
            // Общая часть
            let mut query_params = vec![
                ("client_id", self.inner.client_id.as_str()),
                ("client_secret", self.inner.client_secret.as_str()),
                ("resource", "https://manage.devcenter.microsoft.com")
            ];

            // Отличная часть
            if let Some(ref refresh_token) = refresh_token {
                query_params.push(("grant_type", "refresh_token"));
                query_params.push(("refresh_token", refresh_token));
            }else{
                query_params.push(("grant_type", "client_credentials"));
            }

            query_params
        };

        // Запрос
        let result = self.inner.http_client
            .post(self.inner.token_api_url.clone())
            .query(&query_params)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded; charset=utf-8")
            .send()
            .await?
            .error_for_status()?
            .json::<DataOrErrorResponse<TokenResponse>>()
            .await?
            .into_result()?;

        debug!("Microsoft Azure token response: {:#?}", result);

        Ok(Token::from(result))
    }
}

////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests{
    use httpmock::{
        MockServer,
        Method::{
            POST
        },
    };
    use super::{
        *
    };

    fn setup_logs() {
        if std::env::var("RUST_LOG").is_err(){
            // export RUST_LOG=reqwest=trace
            // unset RUST_LOG
            std::env::set_var("RUST_LOG", "microsoft_azure_client=trace,integration_test=trace,reqwest=trace");
        }
        std::env::set_var("RUST_LOG_STYLE", "auto");
        env_logger::builder()
            //.is_test(true) // Выводить логи только в случае ошибки
            .try_init() // Позволяет инициализировать много раз
            .ok();
    }

    #[tokio::test]
    async fn mock_test_azure_token(){
        const TENANT_ID: &str = "test_tenant_id";
        const CLIENT_ID: &str = "test_client_id";
        const CLIENT_SECRET: &str = "test_secret_value_id";
        const RESULT_TOKEN_FIRST: &str = "aaaaaaa";
        const REFRESH_TOKEN: &str = "bbbbbbb";
        const RESULT_TOKEN_SECOND: &str = "eeeeeeeeee";

        setup_logs();

        // Стартуем легковесный mock сервер
        let server = MockServer::start();
        let server_url = server.base_url();

        // Конфигурируем ответы сервера для первоначального получения токена
        let first_token_mock_controller = server
            .mock(|when, then| {
                // TODO: Использовать format! как-то
                let response_text = r#"
                    {
                        "token_type": "Bearer",
                        "expires_in": "15",
                        "expires_on": "123213",
                        "resource": "https://manage.devcenter.microsoft.com",
                        "access_token": "aaaaaaa",
                        "refresh_token": "bbbbbbb"
                    }
                "#;

                when
                    .method(POST)
                    .path(format!("/{}/oauth2/token", TENANT_ID))
                    .query_param("grant_type", "client_credentials")
                    .query_param("client_id", CLIENT_ID)
                    .query_param("client_secret", CLIENT_SECRET)
                    .query_param("resource", "https://manage.devcenter.microsoft.com")
                    .header(reqwest::header::CONTENT_TYPE.as_str(), "application/x-www-form-urlencoded; charset=utf-8");

                then
                    .status(200)
                    .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json; charset=utf-8")
                    .body(response_text);
            });

        // Конфигурируем ответы сервера на сброс токена
        let refresh_token_mock_controller = server
            .mock(|when, then| {
                // TODO: Использовать format! как-то
                let response_text = r#"
                    {
                        "token_type": "Bearer",
                        "expires_in": "15",
                        "expires_on": "123123",
                        "resource": "https://manage.devcenter.microsoft.com",
                        "access_token": "eeeeeeeeee",
                        "refresh_token": "bbbbbbbb"
                    }
                "#;

                when
                    .method(POST)
                    .path(format!("/{}/oauth2/token", TENANT_ID))
                    .query_param("grant_type", "refresh_token")
                    .query_param("refresh_token", REFRESH_TOKEN)
                    .query_param("client_id", CLIENT_ID)
                    .query_param("client_secret", CLIENT_SECRET)
                    .query_param("resource", "https://manage.devcenter.microsoft.com")
                    .header(reqwest::header::CONTENT_TYPE.as_str(), "application/x-www-form-urlencoded; charset=utf-8");

                then
                    .status(200)
                    .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json; charset=utf-8")
                    .body(response_text);
            });

        let token_provider = MicrosoftAzureTokenProvider::new(reqwest::Client::new(), 
                                                              &server_url, 
                                                              TENANT_ID, 
                                                              CLIENT_ID.to_owned(), 
                                                              CLIENT_SECRET.to_owned(),
                                                              Duration::from_secs(5))
            .expect("Token provider create failed");

        // Первоначальное получение токена
        {
            let token_string = token_provider
                .get_access_token()
                .await
                .expect("First access token receive failed");

            assert_eq!(token_string.as_str(), RESULT_TOKEN_FIRST);
        }

        // Ждем, чтобы проверить, что токен валидный после 5 секунд все еще
        tokio::time::delay_for(std::time::Duration::from_secs(5)).await;

        // Первоначальное получение токена
        {
            let token_string = token_provider
                .get_access_token()
                .await
                .expect("First access token receive failed");

            assert_eq!(token_string.as_str(), RESULT_TOKEN_FIRST);
        }

        // Ждем, чтобы токен стал невалидным
        tokio::time::delay_for(std::time::Duration::from_secs(20)).await;

        // Получение токена после сброса
        {
            let token_string = token_provider
                .get_access_token()
                .await
                .expect("Second access token receive failed");

            assert_eq!(token_string.as_str(), RESULT_TOKEN_SECOND);
        }

        // Удостоверяемся, что моки были вызваны в точности один раз
        first_token_mock_controller.assert();
        refresh_token_mock_controller.assert()
    }
}