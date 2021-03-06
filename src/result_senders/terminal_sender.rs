use std::{
    error::{
        Error
    }
};
use async_trait::{
    async_trait
};
use tracing::{
    error,
    info
};
use crate::{
    uploaders::{
        UploadResultData
    }
};
use super::{
    ResultSender
};

pub struct TerminalSender{
}
#[async_trait(?Send)]
impl ResultSender for TerminalSender {
    async fn send_result(&mut self, result: &UploadResultData){
        info!(%result, "Uploading task success");
    }
    async fn send_error(&mut self, err: &dyn Error){
        error!(%err, "Uploading task error");
    }
}