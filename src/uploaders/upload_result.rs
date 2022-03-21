use std::{
    fmt::{
        Formatter,
        Display,
        self
    }, 
    error::{
        Error
    },
    writeln
};

#[derive(Debug)]
pub struct UploadResultData{
    pub target: &'static str,
    pub message: Option<String>,
    pub install_url: Option<String>
}

impl Display for UploadResultData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:#?}", self)
    }
}

pub type UploadResult = std::result::Result<
    UploadResultData, 
    Box<dyn Error + Send + Sync + 'static>
>;