use std::{
    fmt::{
        // Formatter,
        Display,
        Debug,
        // self
    }, 
    // writeln
};

pub trait UploadResultMessage: Debug {
    fn get_markdown(&self) -> &str;
    fn get_plain(&self) -> &str;
}

pub trait UploadResultData: Debug {
    fn get_target(&self) -> &str;
    fn get_message(&self) -> Option<&dyn UploadResultMessage>;
    fn get_qr_data(&self) -> Option<&str>;
}

/*#[derive(Debug)]
pub struct UploadResultData{
    pub target: &'static str,
    pub message: Option<String>,
    pub install_url: Option<String>
}

impl Display for UploadResultData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:#?}", self)
    }
}*/

pub type UploadResult = std::result::Result<Box<dyn UploadResultData>, Box<dyn std::error::Error>>;