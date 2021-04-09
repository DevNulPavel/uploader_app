use std::{
    path::Path
};
use reqwest::{
    // RequestBuilder,
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
use quick_error::{
    ResultExt
};
use tokio_util::{
    codec::{
        BytesCodec,
        FramedRead
    }
};
use super::{
    responses::{
        AppEditResponseOk,
        DataOrErrorResponse
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

pub struct AppEdit {
    request_builder: GooglePlayRequestBuilder
}
impl AppEdit {
    pub async fn new(request_builder: GooglePlayRequestBuilder) -> Result<AppEdit, GooglePlayError> {
        // https://developers.google.com/android-publisher/api-ref/rest/v3/edits/insert
        // https://developers.google.com/android-publisher/api-ref/rest/v3/edits#AppEdit
        let edit_info = request_builder
            .clone()
            .method(Method::POST)
            .join_path("edits")
            .build()?
            .send()
            .await
            .context("Edits request fail")?
            .json::<DataOrErrorResponse<AppEditResponseOk>>()
            .await
            .context("Edits request json parse fail")?
            .into_result()?;

        Ok(AppEdit{
            request_builder: request_builder.edit_id(edit_info.id)
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
        let response = self.request_builder
            .clone()
            .upload()
            .method(Method::POST)
            .join_path(upload_type)
            .build()?
            .query(&[
                ("uploadType", "multipart"),
                ("ackBundleInstallationWarning", "true")
            ])
            // .header("Content-Length", file_length)
            .multipart(multipart)
            .send()
            .await
            .context("Upload request fail")?
            .json::<DataOrErrorResponse<UploadResponseOk>>()
            .await
            .context("Upload request json parse failed")?
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
        let response = self.request_builder
            .clone()
            .method(Method::PUT)
            .join_path(path)
            .build()?
            .json(&body)
            .send()
            .await
            .context("Track update request failed")?
            .json::<TrackUpdateResponse>()
            .await
            .context("Track update json parse failed")?;

        Ok(response)
    }

    pub async fn validate(&self) -> Result<AppEditResponseOk, GooglePlayError>{
        // https://developers.google.com/android-publisher/api-ref/rest/v3/edits/validate
        let response = self.request_builder
            .clone()
            .method(Method::POST)
            .edit_command("validate")
            .build()?
            .send()
            .await
            .context("Validate request failed")?
            .json::<DataOrErrorResponse<AppEditResponseOk>>()
            .await
            .context("Validate request json parse failed")?
            .into_result()?;

        Ok(response)
    }

    pub async fn commit(&self) -> Result<AppEditResponseOk, GooglePlayError>{
        // https://developers.google.com/android-publisher/api-ref/rest/v3/edits/commit
        let response = self.request_builder
            .clone()
            .method(Method::POST)
            .edit_command("commit")
            .build()?
            .send()
            .await
            .context("Commit request failed")?
            .json::<DataOrErrorResponse<AppEditResponseOk>>()
            .await
            .context("Commit request json parse failed")?
            .into_result()?;

        Ok(response)
    }
}