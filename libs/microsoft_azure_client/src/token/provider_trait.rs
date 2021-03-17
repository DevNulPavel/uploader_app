use std::{
    fmt::{
        Debug
    }, 
    sync::{
        Arc
    }
};
use async_trait::{
    async_trait
};
use crate::{
    error::{
        MicrosoftAzureError
    }
};

#[async_trait(?Send)]
pub trait TokenProvider: Debug {
    async fn get_access_token(&self) -> Result<Arc<String>, MicrosoftAzureError>;
}