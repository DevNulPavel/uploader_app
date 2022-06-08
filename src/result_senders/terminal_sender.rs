use super::ResultSender;
use crate::uploaders::UploadResultData;
use async_trait::async_trait;
use log::{error, info};
use std::error::Error;

pub struct TerminalSender {}
// #[async_trait(?Send)]
#[async_trait]
impl ResultSender for TerminalSender {
    async fn send_result(&self, result: &UploadResultData) {
        info!("Uploading task success: {result}");
    }
    async fn send_error(&self, err: &(dyn Error + Send + Sync)) {
        error!("Uploading task error: {err}");
    }
}
