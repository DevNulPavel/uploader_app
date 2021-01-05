use std::{
    path::Path
};
use reqwest::{
    RequestBuilder,
    Method,
    Body
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
    request_builder: EditRequestBuilder<'a>
}
impl<'a> AppEdit<'a> {
    pub async fn new(request_builder: GooglePlayRequestBuilder<'a>) -> Result<AppEdit<'a>, GooglePlayError> {
        // https://developers.google.com/android-publisher/api-ref/rest/v3/edits/insert
        // https://developers.google.com/android-publisher/api-ref/rest/v3/edits#AppEdit
        let edit_info = request_builder
            .build_request(Method::POST, "edits")?
            .send()
            .await?
            .json::<AppEditResponse>()
            .await?;

        Ok(AppEdit{
            request_builder: EditRequestBuilder::new(request_builder, edit_info.id)
        })
    }

    pub async fn upload_build(&self, file_path: &Path) -> Result<UploadResponse, GooglePlayError>{
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
        
        // Файлик в виде стрима
        let file = File::open(file_path).await?;
        let file_length = file.metadata().await?.len();
        let reader = FramedRead::new(file, BytesCodec::new())
            /*.map(move |v| {
                if let Ok(ref v) = v{
                    total_uploaded += v.len();
                    info!("Uploaded {}: {}/{}", file_name_stream, total_uploaded, file_length);
                }
                v
            })*/;
        let body = Body::wrap_stream(reader);

        // Грузим
        let response = self.request_builder
            .build_request(Method::POST, upload_type)?
            .query(&[
                ("ackBundleInstallationWarning", "true")
            ])
            .header("Content-Length", file_length)
            .body(body)
            .send()
            .await?
            .json::<UploadResponse>()
            .await?;

        Ok(response)
    }

    pub async fn update_track_to_complete(&self, track: &str, app_version: &UploadResponse) -> Result<TrackUpdateResponse, GooglePlayError>{
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
        let response = self.request_builder
            .build_request(Method::POST, ":validate")?
            .send()
            .await?
            .json::<AppEditResponse>()
            .await?;

        Ok(response)
    }

    pub async fn commit(&self) -> Result<AppEditResponse, GooglePlayError>{
        // https://developers.google.com/android-publisher/api-ref/rest/v3/edits/commit
        let response = self.request_builder
            .build_request(Method::POST, ":commit")?
            .send()
            .await?
            .json::<AppEditResponse>()
            .await?;

        Ok(response)
    }
}