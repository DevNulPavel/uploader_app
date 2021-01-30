use std::{
    fmt::{
        // Formatter,
        // Display,
        Debug,
        // self
    }, 
    // writeln
};
use serde_json::{
    Value
};

pub trait UploadResultMessage: Debug {
    fn get_slack_blocks(&self) -> &[Value];
    fn get_plain(&self) -> &str;
}

pub trait UploadResultData: Debug {
    fn get_target(&self) -> &str;
    fn get_message(&self) -> Option<&dyn UploadResultMessage>;
    fn get_qr_data(&self) -> Option<&str>;
}

pub type UploadResult = std::result::Result<Box<dyn UploadResultData>, Box<dyn std::error::Error>>;