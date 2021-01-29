use std::{
    error::{
        Error
    }
};
use async_trait::{
    async_trait
};
use log::{
    error,
    info
};
use crate::{
    uploaders::{
        UploadResultData
    }
};
use super::{
    ResultReceiver
};

pub struct TerminalSender{
}
#[async_trait(?Send)]
impl ResultReceiver for TerminalSender {
    async fn on_result_received(&mut self, result: &dyn UploadResultData){
        if let Some(msg) = result.get_message(){
            info!("Uploading task success for {}: {}", result.get_target(), msg.get_plain());
        }else{
            info!("Uploading task success for {}", result.get_target());
        }
    }
    async fn on_error_received(&mut self, err: &dyn Error){
        error!("Uploading task error: {}", err);
    }
}