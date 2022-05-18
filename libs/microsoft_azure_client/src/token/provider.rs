use super::token_struct::Token;
use crate::{
    error::MicrosoftAzureError,
    responses::{DataOrErrorResponse, TokenResponse},
};
use reqwest::{header::CONTENT_TYPE, Client};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tracing::debug;
use url::Url;


////////////////////////////////////////////////////////////////

/// Внутренняя структура, содержащая общие переменные для запроса и сброса токена
#[derive(Debug)]
struct InnerData {
    http_client: Client, // ArcInside
    token_api_url: Url,
    client_id: String,
    client_secret: String,
    token_expire_pre_delay: Duration,
}

////////////////////////////////////////////////////////////////

/// Данный провайдер запрашивает токен, выдает его наружу,
/// если токен истекает через 5 минут, тогда перезапрашивает его самостоятельно.
/// Автоматически обновляя его внутри
#[derive(Debug)]
pub struct TokenProvider {
    inner: Arc<InnerData>,
    active_token: Arc<Mutex<Option<Token>>>,
}

impl Clone for TokenProvider {
    fn clone(&self) -> Self {
        TokenProvider {
            inner: self.inner.clone(),
            active_token: self.active_token.clone(),
        }
    }
}

impl TokenProvider {
    pub fn new<D>(
        http_client: Client,
        tenant_id: D,
        client_id: String,
        client_secret: String,
    ) -> Result<TokenProvider, MicrosoftAzureError>
    where
        D: std::fmt::Display,
    {
        Self::new_custom(
            http_client,
            "https://login.microsoftonline.com",
            tenant_id,
            client_id,
            client_secret,
            Duration::from_secs(60 * 3),
        )
    }

    fn new_custom<D>(
        http_client: Client,
        api_url: &str,
        tenant_id: D,
        client_id: String,
        client_secret: String,
        token_expire_pre_delay: Duration,
    ) -> Result<TokenProvider, MicrosoftAzureError>
    where
        D: std::fmt::Display,
    {
        let token_api_url = Url::parse(api_url)?.join(&format!("{}/oauth2/token", tenant_id))?;

        // Создаем Arc на получателя токена
        let inner = Arc::new(InnerData {
            http_client,
            token_api_url,
            client_id,
            client_secret,
            token_expire_pre_delay,
        });

        // Создаем нашу структуру, изначально токен пустой
        Ok(TokenProvider {
            inner,
            active_token: Default::default(),
        })
    }

    /// Отдаваемое значение токена нужно сразу же использовать, а не сохранять где-то
    /// Так как токен короткоживущий и обновляется внутри при необходимости
    /// токен отдается в виде Arc, чтобы не делать бессмысленных копирований памяти
    pub async fn get_access_token(&self) -> Result<Arc<String>, MicrosoftAzureError> {
        // TODO: Как-то сделать лучше

        // Делаем некую машину состояний, чтобы вернуть валидный рабочий токен после всех проверок
        let mut token_guard = self.active_token.lock().await;
        loop {
            match token_guard.as_ref() {
                // Если токен не заканчивает время жизни через указанную задержку, то все норм
                // Иначе все делаем сброс
                Some(token)
                    if !token.is_will_be_expired_soon(self.inner.token_expire_pre_delay) =>
                {
                    //debug!("Token is OK");

                    // Все нормально, возвращаем склонированное значение токена
                    // Клонировать приходится, так как иначе придется долго держать блокировку
                    return Ok(token.get_token_value());
                }
                _ => {
                    debug!("Token will be expired soon or missing, refresh it");

                    // Вызываем снова запрос токена
                    let new_token = self.request_azure_token().await?;

                    // После установки идем на очередную итерацию цикла
                    token_guard.replace(new_token);
                }
            }
        }
        // unreachable!("This code is unreacheble");
    }

    async fn request_azure_token(&self) -> Result<Token, MicrosoftAzureError> {
        // https://docs.microsoft.com/en-us/azure/active-directory/azuread-dev/v1-protocols-oauth-code#refreshing-the-access-tokens
        // https://docs.microsoft.com/en-us/windows/uwp/monetize/create-and-manage-submissions-using-windows-store-services#obtain-an-azure-ad-access-token
        // https://docs.microsoft.com/en-us/windows/uwp/monetize/python-code-examples-for-the-windows-store-submission-api#create-app-submission
        // https://docs.microsoft.com/en-us/azure/active-directory/azuread-dev/v1-protocols-oauth-code#refreshing-the-access-tokens

        let form_params = serde_json::json!({
            "grant_type": "client_credentials",
            "client_id": self.inner.client_id.as_str(),
            "client_secret": self.inner.client_secret.as_str(),
            "resource": "https://manage.devcenter.microsoft.com"
        });

        // Запрос
        let result = self
            .inner
            .http_client
            .post(self.inner.token_api_url.clone())
            //.query(&query_params)
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded; charset=utf-8",
            )
            .form(&form_params)
            .send()
            .await?
            .error_for_status()?
            .json::<DataOrErrorResponse<TokenResponse>>()
            .await?
            .into_result()?;

        drop(form_params);

        debug!("Microsoft Azure: token response: {:#?}", result);

        Ok(Token::from(result))
    }
}
