use reqwest::{
    Method
};
use log::{
    debug
};
use serde_json::{
    json,
    Value
};
use super::{
    request_builder::{
        GoogleDriveRequestBuilder
    },
    responses::{
        // FilesUploadResponse,
        FilesUploadResponseOk,
        // FilesUploadResponseErr
    },
    error::{
        GoogleDriveError
    }
};

pub trait FileOwner{
    fn get_params(&self) -> Value;
}

///////////////////////////////////////////////////////////////

pub struct DomainFileOwner<'a>{
    domain: &'a str
}
impl<'a> DomainFileOwner<'a> {
    pub fn new(domain: &'a str) -> Self{
        Self{
            domain
        }
    }
}
impl<'a> FileOwner for DomainFileOwner<'a>{
    fn get_params(&self) -> Value {
        json!({
            "role": "writer",
            "type": "domain",
            "domain": self.domain
        })
    }
}

///////////////////////////////////////////////////////////////

pub struct EmailFileOwner<'a>{
    email: &'a str
}
impl<'a> EmailFileOwner<'a> {
    pub fn new(email: &'a str) -> Self{
        Self{
            email
        }
    }
}
impl<'a> FileOwner for EmailFileOwner<'a>{
    fn get_params(&self) -> Value {
        json!({
            "role": "writer",
            // "role": "owner",
            "type": "user",
            "emailAddress": self.email
        })
    }
}

///////////////////////////////////////////////////////////////

pub struct GoogleDriveFile{
    client: GoogleDriveRequestBuilder,
    info: FilesUploadResponseOk
}
impl Into<FilesUploadResponseOk> for GoogleDriveFile {
    fn into(self) -> FilesUploadResponseOk {
        self.info
    }
}
impl GoogleDriveFile {
    pub fn new(client: GoogleDriveRequestBuilder, info: FilesUploadResponseOk) -> GoogleDriveFile{
        GoogleDriveFile{
            client,
            info
        }
    }
    
    pub fn get_info(&self) -> &FilesUploadResponseOk{
        &self.info
    }

    pub async fn update_owner<O>(&self, owner: O) -> Result<(), GoogleDriveError> 
    where 
        O: FileOwner {
        // https://developers.google.com/drive/api/v3/reference/permissions/create

        let body = owner.get_params();

        let text = self.client
            .build_request(Method::POST, "drive/v3/files/fileId/permissions")?
            .query(&[
                ("fileId", self.info.id.as_str()),
                // ("transferOwnership", "true"),
                ("supportsAllDrives", "true"),
                // ("fields", "id,emailAddress,domain")
            ])
            .json(&body)
            .send()
            .await?
            .text()
            .await?;

        debug!("{}", text);
            // .json::<FilePermissionResponse>()
            // .await?;

        Ok(())
    }
}