use crate::error::MicrosoftAzureError;
use async_trait::async_trait;
use std::{fmt::Debug, sync::Arc};

#[async_trait(?Send)]
pub trait TokenProvider: Debug {
    async fn get_access_token(&self) -> Result<Arc<String>, MicrosoftAzureError>;
}
