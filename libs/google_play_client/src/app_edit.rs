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
use tap::{
    prelude::{
        *
    }
};
use reqwest_inspect_json::{
    InspectJson
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
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send()
            .await?
            .inspect_json::<DataOrErrorResponse<AppEditResponseOk>, GooglePlayError>(|v|{ 
                debug!("{:?}", v);
            })
            .await?
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
            .timeout(std::time::Duration::from_secs(600)) // В доках рекомендуется большой таймаут
            .query(&[
                ("uploadType", "multipart"),
                ("ackBundleInstallationWarning", "true")
            ])
            // .header(reqwest::header::CONTENT_LENGTH, file_length)
            // .header("Content-Length", file_length)
            // .body(body)
            .multipart(multipart)
            .send()
            .await?
            // .text()
            // .await?;
            .inspect_json::<DataOrErrorResponse<UploadResponseOk>, GooglePlayError>(|v|{ 
                debug!("{:?}", v);
            })
            .await?
            .into_result()?;
            
        debug!("Upload result: {:?}", response);

        Ok(response)
        // Err(GooglePlayError::Custom("Fail".to_owned()))
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
            .await?
            .bytes()
            .await?
            .tap(|v|{ debug!("{:?}", v) })
            .pipe(|v|{ serde_json::from_slice::<DataOrErrorResponse<TrackUpdateResponse>>(&v) })?
            .into_result()?;

        Ok(response)
    }

    pub async fn validate(&self) -> Result<AppEditResponseOk, GooglePlayError>{
        // https://developers.google.com/android-publisher/api-ref/rest/v3/edits/validate
        let response = self.request_builder
            .clone()
            .method(Method::POST)
            .edit_command("validate")
            .build()?
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send()
            .await?
            .bytes()
            .await?
            .tap(|v|{ debug!("{:?}", v) })
            .pipe(|v|{ serde_json::from_slice::<DataOrErrorResponse<AppEditResponseOk>>(&v) })?
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
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send()
            .await?
            .inspect_json::<DataOrErrorResponse<AppEditResponseOk>, GooglePlayError>(|v|{ 
                debug!("{:?}", v);
            })
            .await?
            .into_result()?;

        Ok(response)
    }
}