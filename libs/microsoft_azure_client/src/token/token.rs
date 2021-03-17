use std::{
    sync::{
        Arc
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
// use log::{
//     // info,
//     debug
// };
use crate::{
    responses::{
        TokenResponse
    }
};

////////////////////////////////////////////////////////////////

/// Внутренняя структура, которая содержит в себе токен, а так же время окончания его жизни
#[derive(Debug)]
pub struct Token{
    access_token: Arc<String>,
    // refresh_token: String,
    complete_time: Instant
}

impl From<TokenResponse> for Token {
    fn from(resp: TokenResponse) -> Self {
        let complete_time = Instant::now()
            .add(Duration::from_secs(resp.expires_in));
        Token{
            access_token: Arc::new(resp.access_token),
            // refresh_token: resp.refresh_token,
            complete_time
        }
    }
}

impl Token {
    /// Проверяем, что время жизни истекает скоро, при этом учитываем предзадержку
    pub fn is_will_be_expired_soon(&self, pre_delay: Duration) -> bool {
        let complete_time_val = self.complete_time.sub(pre_delay);
        Instant::now()
            .gt(&complete_time_val)
    }

    pub fn get_token_value(&self) -> Arc<String>{
        self.access_token.clone()
    }
}

////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////

// TODO: Mock сервер с поддержкой form data
/*#[cfg(test)]
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
                    .json_body(serde_json::json!(
                        {
                            "grant_type": "client_credentials",
                            "client_id": "test_client_id",
                            "client_secret": "test_secret_value_id",
                            "resource": "https://manage.devcenter.microsoft.com"
                        }
                    ))
                    .header(reqwest::header::CONTENT_TYPE.as_str(), "application/x-www-form-urlencoded");

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

        let token_provider = MicrosoftAzureTokenProvider::new_custom(reqwest::Client::new(), 
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
}*/