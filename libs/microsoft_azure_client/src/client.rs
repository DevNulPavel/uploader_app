use crate::{
    error::MicrosoftAzureError, flight_submission::FlightSubmission,
    request_builder::RequestBuilder, token::TokenProvider,
};
use reqwest::Client;
use std::path::Path;
use tracing::{debug, instrument, Instrument};

pub struct MicrosoftAzureClient {
    request_builder: RequestBuilder,
}

impl MicrosoftAzureClient {
    /// Создаем нового клиента
    #[instrument(skip(http_client, tenant_id, client_id, client_secret, application_id))]
    pub fn new<D: std::fmt::Display>(
        http_client: Client,
        tenant_id: D,
        client_id: String,
        client_secret: String,
        application_id: String,
    ) -> Result<MicrosoftAzureClient, MicrosoftAzureError> {
        // Создаем провайдер токена
        let token_provider =
            TokenProvider::new(http_client.clone(), tenant_id, client_id, client_secret)?;

        // Уже с провайдером токенов создаем билдер запросов
        let request_builder = RequestBuilder::new(http_client, token_provider, application_id);

        Ok(MicrosoftAzureClient { request_builder })
    }

    /// Непосредственно выгружаем архив с билдом
    #[instrument(skip(self, zip_upload_file_path, groups, test_flight_name))]
    pub async fn upload_production_build(
        &self,
        zip_upload_file_path: &Path,
        groups: Vec<String>,
        test_flight_name: String,
    ) -> Result<(), MicrosoftAzureError> {
        // https://docs.microsoft.com/en-us/windows/uwp/monetize/manage-flights
        // https://docs.microsoft.com/en-us/windows/uwp/monetize/python-code-examples-for-the-windows-store-submission-api

        // Создаем новый Submission для данного приложения
        debug!("Microsoft Azure: flight submission create try");
        let mut submission =
            FlightSubmission::start_new(self.request_builder.clone(), groups, test_flight_name)
                .in_current_span()
                .await?;
        debug!("Microsoft Azure: flight submission created");

        // Выполняем выгрузку файлика
        debug!("Microsoft Azure: File uploading start");
        submission
            .upload_build(zip_upload_file_path)
            .in_current_span()
            .await?;
        debug!("Microsoft Azure: File uploading finished");

        Ok(())
    }
}
