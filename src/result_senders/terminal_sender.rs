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
// #[async_trait(?Send)]
#[async_trait]
impl ResultSender for TerminalSender {
    async fn send_result(&self, result: &UploadResultData){
        info!(%result, "Uploading task success");
    }
    async fn send_error(&self, err: &(dyn Error + Send + Sync)){
        error!(%err, "Uploading task error");
    }
}