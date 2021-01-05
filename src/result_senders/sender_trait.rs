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
pub trait ResultSender {
    async fn send_result(&mut self, result: &UploadResultData);
    async fn send_error(&mut self, err: &dyn Error);
}