use std::{ops::Deref, sync::Arc};
// use log::{
// debug,
// info
// };
use super::{error::MicrosoftAzureError, token::TokenProvider};
use cow_arc::CowArc;
use reqwest::{Client, Method, Url};

/// Внутренняя структура билдера запросов с неизменяемыми данными
#[derive(Debug)]
struct Base {
    http_client: Client, // Arc inside
    base_url: Url,
    application_id: String,
    token_provider: TokenProvider,
}

/// Непосредственно реквест билдер, который может быть легко склонирован с тем состоянием,
/// которое он имеет сейчас на момент работы
#[derive(Debug, Clone)]
pub struct RequestBuilder {
    base: Arc<Base>,
    flight_id: CowArc<Option<String>>,
    submission_id: CowArc<Option<String>>,
    submission_command: CowArc<Option<String>>,
    path_segments: CowArc<Vec<String>>,
    method: Method,
}
impl<'a> RequestBuilder {
    pub fn new(
        http_client: Client,
        token_provider: TokenProvider,
        application_id: String,
    ) -> RequestBuilder {
        let base_api_url = Url::parse("https://manage.devcenter.microsoft.com/")
            .expect("Microsoft Azure base api URL parse failed");

        RequestBuilder {
            base: Arc::new(Base {
                http_client,
                base_url: base_api_url,
                token_provider,
                application_id,
            }),
            method: Default::default(),
            flight_id: Default::default(),
            submission_id: Default::default(),
            submission_command: Default::default(),
            path_segments: Default::default(), // upload: false,
                                               // edit_id: CowArc::new(None),
                                               // edit_command: CowArc::new(None),
        }
    }

    /// Возвращает сырой клиент без модификаций
    pub fn get_http_client(&self) -> Client {
        self.base.http_client.clone()
    }

    pub fn method(mut self, method: Method) -> RequestBuilder {
        self.method = method;
        self
    }

    pub fn flight_id(mut self, flight_id: String) -> RequestBuilder {
        self.flight_id.set_val(Some(flight_id));
        self
    }

    pub fn submission_id(mut self, sub_id: String) -> RequestBuilder {
        self.submission_id.set_val(Some(sub_id));
        self
    }

    pub fn submission_command(mut self, sub_id: String) -> RequestBuilder {
        self.submission_command.set_val(Some(sub_id));
        self
    }

    pub fn join_path(mut self, segment: String) -> RequestBuilder {
        self.path_segments.update_val(|val| {
            val.push(segment);
        });
        self
    }

    pub async fn build(self) -> Result<reqwest::RequestBuilder, MicrosoftAzureError> {
        let mut url = self.base.base_url.clone();
        {
            let mut segments = url
                .path_segments_mut()
                .map_err(|_| MicrosoftAzureError::UnvalidUrlSegments)?;
            // Базовая часть
            segments.push("v1.0");
            segments.push("my");
            segments.push("applications");
            segments.push(&self.base.application_id);
            // Если есть flight id, добавляем
            if let Some(flight_id) = self.flight_id.as_ref() {
                segments.push("flights");
                segments.push(flight_id);
            }
            // Если есть submission id, добавляем
            if let Some(submission_id) = self.submission_id.as_ref() {
                segments.push("submissions");
                segments.push(submission_id);
                // Команда сабмиссии
                if let Some(submission_command) = self.submission_command.as_ref() {
                    segments.push(submission_command);
                }
            }
            for segment in self.path_segments.deref() {
                let segment = segment.trim_matches('/');
                let split = segment.split('/');
                for part in split {
                    segments.push(part);
                }
            }
        }

        // Получаем токен с перезапросомм если надо
        let token = self.base.token_provider.get_access_token().await?;

        // Создаем запрос
        let builder = self
            .base
            .http_client
            .request(self.method, url.as_str())
            .bearer_auth(token);

        Ok(builder)
    }
}
