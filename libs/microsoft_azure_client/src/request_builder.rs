use std::{
    sync::{
        Arc
    },
    ops::{
        Deref,
        // DerefMut
    }
};
// use log::{
    // debug,
    // info
// };
use tracing_error::{
    SpanTrace
};
use reqwest::{
    Client,
    Url,
    Method
};
use cow_arc::{
    CowArc
};
use super::{
    token::{
        TokenProvider
    },
    error::{
        MicrosoftAzureError
    }
};


/// Внутренняя структура билдера запросов с неизменяемыми данными
#[derive(Debug)]
struct Base { 
    http_client: Client, // Arc inside
    base_url: Url,
    application_id: String,
    token_provider: Box<dyn TokenProvider>
}

/// Непосредственно реквест билдер, который может быть легко склонирован с тем состоянием,
/// которое он имеет сейчас на момент работы
#[derive(Debug, Clone)]
pub struct RequestBuilder {
    base: Arc<Base>,
    submission_id: CowArc<Option<String>>,
    submission_command: CowArc<Option<String>>,
    path_segments: CowArc<Vec<String>>,
    method: Method,
}
impl<'a> RequestBuilder {
    pub fn new(http_client: Client,
               token_provider: Box<dyn TokenProvider>,
               application_id: String) -> RequestBuilder {

        let base_api_url = Url::parse("https://manage.devcenter.microsoft.com/")
            .expect("Microsoft Azure base api URL parse failed");
        
        RequestBuilder{
            base: Arc::new(Base{
                http_client,
                base_url: base_api_url,
                token_provider,
                application_id
            }),
            method: Default::default(),
            submission_id: Default::default(),
            submission_command: Default::default(),
            path_segments: Default::default()
            // upload: false,
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

    pub fn submission_id(mut self, sub_id: String) -> RequestBuilder {
        self.submission_id.set_val(Some(sub_id));
        self
    }

    pub fn submission_command(mut self, sub_id: String) -> RequestBuilder {
        self.submission_command.set_val(Some(sub_id));
        self
    }

    pub fn join_path(mut self, segment: String) -> RequestBuilder {
        self.path_segments.update_val(|val|{
            val.push(segment);
        });
        self
    }

    pub async fn build(self) -> Result<reqwest::RequestBuilder, MicrosoftAzureError>{
        let mut url = self.base.base_url.clone();
        {
            let mut segments = url.path_segments_mut()
                .map_err(|_|{
                    MicrosoftAzureError::UnvalidUrlSegments(SpanTrace::capture())
                })?;
            // Базовая часть
            segments.push("v1.0");
            segments.push("my");
            segments.push("applications");
            segments.push(&self.base.application_id);
            // Если есть submission id, добавляем
            if let Some(submission_id) = self.submission_id.as_ref() {
                segments.push("submissions");
                segments.push(&submission_id);
                if let Some(submission_command) = self.submission_command.as_ref() {
                    segments.push(submission_command);
                }
            }
            for segment in self.path_segments.deref() {
                let segment = segment.trim_matches('/');
                let split = segment.split("/");
                for part in split{
                    segments.push(part);
                }
            }
        }

        // Получаем токен с перезапросомм если надо
        let token = self.base.token_provider
            .get_access_token()
            .await?;

        // Создаем запрос
        let builder = self.base.http_client
            .request(self.method, url.as_str())
            .bearer_auth(token);

        Ok(builder)
    }
}

#[cfg(test)]
mod tests{
    use async_trait::{
        async_trait
    };
    use crate::{
        token::{
            TokenProvider
        }
    };
    use super::*;

    //////////////////////////////////////////////////////////////////////////////////////////

    /// Специальный провайдер токенов для тестовых целей
    #[derive(Debug)]
    struct FakeTokenProvider{
        fake_value: Arc<String>
    }
    #[async_trait(?Send)]
    impl TokenProvider for FakeTokenProvider {
        async fn get_access_token(&self) -> Result<Arc<String>, MicrosoftAzureError> {
            Ok(self.fake_value.clone())
        }
    }

    //////////////////////////////////////////////////////////////////////////////////////////

    #[tokio::test]
    async fn test_request_builder(){
        let token_provider = Box::new(FakeTokenProvider{
            fake_value: Arc::new(String::from("fake_token_value"))
        });

        let builder = RequestBuilder::new(Client::new(), 
                                                        token_provider, 
                                                        "test_application_id".to_string());

        {
            let req = builder
                .clone()
                .build()
                .await
                .expect("Builder error")
                .build()
                .expect("Builder error");
            assert_eq!(req.url().as_str(), 
                    "https://manage.devcenter.microsoft.com/v1.0/my/applications/test_application_id");
        }

        {
            let req = builder
                .clone()
                .submission_id("test_submission_id".to_owned())
                .submission_command("test_command".to_owned())
                .build()
                .await
                .expect("Builder error")
                .build()
                .expect("Builder error");
            assert_eq!(req.url().as_str(), 
                    "https://manage.devcenter.microsoft.com/v1.0/my/applications/test_application_id/submissions/test_submission_id/test_command");
        }

        {
            builder
                .clone()
                .submission_id("test_submission_id".to_owned())
                .build()
                .await
                .expect_err("Missing `submission command` with existing `submission_id` must throw an error");
        }        
    }
}