use std::{
    error::{
        Error
    }
};
use async_trait::{
    async_trait
};
use crate::{
    uploaders::{
        UploadResultData
    }
};

#[async_trait(?Send)]
pub trait ResultReceiver {
    async fn on_result_received(&mut self, result: &dyn UploadResultData);
    async fn on_error_received(&mut self, err: &dyn Error);
}