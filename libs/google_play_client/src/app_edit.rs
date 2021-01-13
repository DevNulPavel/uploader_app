use std::{
    path::Path
};
use reqwest::{
    RequestBuilder,
    Method,
    Body,
    multipart::{
        Form,
        Part
    }
};
use log::{
    debug
};
use tokio::{
    fs::{
        File
    }
};
use serde_json::{
    json
};
use tokio_util::{
    codec::{
        BytesCodec,
        FramedRead
    }
};
use into_result::{
    IntoResult
};
use super::{
    responses::{
        AppEditResponse
    },
    request_builder::{
        GooglePlayRequestBuilder
    },
    error::{
        GooglePlayError
    },
    responses::{
        *
    }
};

///////////////////////////////////////////////////////

struct EditRequestBuilder<'a>{
    request_builder: GooglePlayRequestBuilder<'a>,
    edit_id: String
}
impl<'a> EditRequestBuilder<'a> {
    fn new(request_builder: GooglePlayRequestBuilder<'a>, edit_id: String) -> EditRequestBuilder{
        EditRequestBuilder{
            request_builder,
            edit_id
        }
    }
    fn build_request(&self, method: Method, path: &str) -> Result<RequestBuilder, GooglePlayError> {
        let path = if path.starts_with(':'){
            format!("edits/{}{}", self.edit_id, path)
        }else{
            format!("edits/{}/{}", self.edit_id, path.trim_matches('/'))
        };     
        self
            .request_builder
            .build_request(method, &path)
    }
}

///////////////////////////////////////////////////////

pub struct AppEdit<'a>{
    edit_request_builder: EditRequestBuilder<'a>,
    upload_request_builder: EditRequestBuilder<'a>
}
impl<'a> AppEdit<'a> {
    pub async fn new(config_request_builder: GooglePlayRequestBuilder<'a>, upload_request_builder: GooglePlayRequestBuilder<'a>) -> Result<AppEdit<'a>, GooglePlayError> {
        // https://developers.google.com/android-publisher/api-ref/rest/v3/edits/insert
        // https://developers.google.com/android-publisher/api-ref/rest/v3/edits#AppEdit
        let edit_info = config_request_builder
            .build_request(Method::POST, "edits")?
            .send()
            .await?
            .json::<AppEditResponse>()
            .await?;

        Ok(AppEdit{
            edit_request_builder: EditRequestBuilder::new(config_request_builder, edit_info.id.clone()),
            upload_request_builder: EditRequestBuilder::new(upload_request_builder, edit_info.id)
        })
    }

    pub async fn upload_build(&self, file_path: &Path) -> Result<UploadResponseOk, GooglePlayError>{
        // https://developers.google.com/android-publisher/api-ref/rest/v3/edits.apks
        // https://developers.google.com/android-publisher/api-ref/rest/v3/edits.bundles
        // https://developers.google.com/android-publisher/upload

        // Тип выгрузки
        let extention = file_path
            .extension()
            .ok_or(GooglePlayError::WrongFilePath)?
            .to_str()
            .ok_or(GooglePlayError::WrongFilePath)?;
        let upload_type = match extention {
            "aab" => "bundles",
            "apk" => "apks",
            _ => return Err(GooglePlayError::InvalidFileExtention("Only .aab or .apk supported"))
        };

        let file_name = file_path
            .file_name()
            .ok_or(GooglePlayError::WrongFilePath)?
            .to_str()
            .ok_or(GooglePlayError::WrongFilePath)?;

        // Progress
        /*.map(move |v| {
                if let Ok(ref v) = v{
                    total_uploaded += v.len();
                    info!("Uploaded {}: {}/{}", file_name_stream, total_uploaded, file_length);
                }
                v
            })*/
        
        // Файлик в виде стрима
        let file = File::open(file_path).await?;
        let file_length = file.metadata().await?.len();
        let reader = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(reader);

        // Первой секцией идет метаинформация в формате json
        let meta = json!({
        }).to_string();

        let multipart = Form::new()
            .part("meta", Part::text(meta)
                            .mime_str("application/json; charset=UTF-8")
                            .expect("Meta set failed"))
            .part("body", Part::stream_with_length(body, file_length)
                            .file_name(file_name.to_owned())
                            .mime_str("application/octet-stream")
                            .expect("Meta set failed"));

        // Грузим
        let response = self.upload_request_builder
            .build_request(Method::POST, upload_type)?
            .query(&[
                ("uploadType", "multipart"),
                ("ackBundleInstallationWarning", "true")
            ])
            // .header("Content-Length", file_length)
            .multipart(multipart)
            .send()
            .await?
            .json::<UploadResponse>()
            .await?
            .into_result()?;
            
        debug!("Upload result: {:?}", response);

        Ok(response)
    }

    pub async fn update_track_to_complete(&self, track: &str, app_version: &UploadResponseOk) -> Result<TrackUpdateResponse, GooglePlayError>{
        // https://developers.google.com/android-publisher/api-ref/rest/v3/edits.tracks
        // https://developers.google.com/android-publisher/api-ref/rest/v3/edits.tracks#Release
        // "countryTargeting": {
        //     "countries": [],
        //     "includeRestOfWorld": true
        // },
        // "inAppUpdatePriority": 5
        let path = format!("tracks/{}", track);
        let body = json!({
            "track": track,
            "releases": [
                {
                    "status": "completed",
                    "versionCodes": [
                        app_version.version_code
                    ]
                }
            ]
        });
        let response = self.edit_request_builder
            .build_request(Method::PUT, &path)?
            .json(&body)
            .send()
            .await?
            .json::<TrackUpdateResponse>()
            .await?;

        Ok(response)
    }

    pub async fn validate(&self) -> Result<AppEditResponse, GooglePlayError>{
        // https://developers.google.com/android-publisher/api-ref/rest/v3/edits/validate
        let response = self.edit_request_builder
            .build_request(Method::POST, ":validate")?
            .send()
            .await?
            .json::<AppEditResponse>()
            .await?;

        Ok(response)
    }

    pub async fn commit(&self) -> Result<AppEditResponse, GooglePlayError>{
        // https://developers.google.com/android-publisher/api-ref/rest/v3/edits/commit
        let response = self.edit_request_builder
            .build_request(Method::POST, ":commit")?
            .send()
            .await?
            .json::<AppEditResponse>()
            .await?;

        Ok(response)
    }
}