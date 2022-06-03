use std::{
    sync::{
        Arc
    },
    ops::{
        Deref,
        // DerefMut
    }
};
// use log::{
    // debug,
    // info
// };
use reqwest::{
    Client,
    RequestBuilder,
    Url,
    Method
};
use yup_oauth2::{
    AccessToken
};
use cow_arc::{
    CowArc
};
use super::{
    error::{
        GooglePlayError
    }
};

// https://developers.google.com/play/api/v3/reference/files/list

#[derive(Debug)]
struct Base { 
    http_client: Client, // Arc inside
    base_url: Url,
    token: Arc<AccessToken>,
    package_name: String,
}

#[derive(Debug, Clone)]
pub struct GooglePlayRequestBuilder {
    base: Arc<Base>,
    upload: bool,
    edit_id: CowArc<Option<String>>,
    edit_command: CowArc<Option<String>>,
    method: Method,
    path_segments: CowArc<Vec<String>>
}
impl GooglePlayRequestBuilder {
    pub fn new(http_client: Client,
               base_url: Url,
               package_name: String,
               token: Arc<AccessToken>) -> GooglePlayRequestBuilder {
        GooglePlayRequestBuilder{
            base: Arc::new(Base{
                http_client,
                token,
                base_url,
                package_name,
            }),
            upload: false,
            edit_id: CowArc::new(None),
            edit_command: CowArc::new(None),
            method: Method::default(),
            path_segments: CowArc::new(Vec::new())
        }
    }

    // pub fn from_template(template: &'a GooglePlayRequestBuilder) -> GooglePlayRequestBuilder {
    //     template.clone()
    // }

    pub fn method(mut self, method: Method) -> GooglePlayRequestBuilder {
        self.method = method;
        self
    }

    pub fn upload(mut self) -> GooglePlayRequestBuilder {
        self.upload = true;
        self
    }

    pub fn edit_id<T: ToString>(mut self, edit_id: T) -> GooglePlayRequestBuilder {
        self.edit_id.set_val(Some(edit_id.to_string()));
        self
    }

    pub fn edit_command<T: ToString>(mut self, edit_command: T) -> GooglePlayRequestBuilder {
        self.edit_command.set_val(Some(edit_command.to_string()));
        self
    }

    pub fn join_path<T: ToString>(mut self, segment: T) -> GooglePlayRequestBuilder {
        self.path_segments.update_val(|val|{
            val.push(segment.to_string());
        });
        self
    }

    pub fn build(self) -> Result<RequestBuilder, GooglePlayError>{
        let mut url = self.base.base_url.clone();
        {
            let mut segments = url.path_segments_mut()
                .map_err(|_|{
                    GooglePlayError::EmptyUrlSegments
                })?;
            if self.upload {
                segments.push("upload");
            }
            segments.push("androidpublisher");
            segments.push("v3");
            segments.push("applications");
            segments.push(&self.base.package_name);
            if let Some(edit_id) = self.edit_id.as_ref() {
                segments.push("edits");
                if let Some(edit_command) = self.edit_command.as_ref() {
                    segments.push(&format!("{}:{}", edit_id, edit_command));
                }else{
                    segments.push(edit_id);
                }
            }
            for segment in self.path_segments.deref() {
                let segment = segment.trim_matches('/');
                let split = segment.split('/');
                for part in split{
                    segments.push(part);
                }
            }
        }

        if self.base.token.is_expired(){
            return Err(GooglePlayError::TokenIsExpired);
        }

        let token = self.base.token.as_str();
        
        let builder = self
            .base
            .http_client
            .request(self.method, url.as_str())
            .bearer_auth(token);
            // .header(reqwest::header::CONTENT_LENGTH, 0); // Лучше тут не добавлять

        Ok(builder)
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_request_builder(){
        let token = serde_json::from_str::<AccessToken>(r#"{ "value": "asdasdsfds" }"#).expect("Token parse failed");

        let base_url = reqwest::Url::parse("https://androidpublisher.googleapis.com")
            .expect("Parse url failed");

        let builder = GooglePlayRequestBuilder::new(
            Client::new(), 
            base_url, 
            "com.test.org".into(), 
            token.into()
        );

        let base_builder = builder
            .method(Method::POST)
            .upload()
            .edit_id("asdasd");

        let req = base_builder
            .clone()
            .join_path("test/custom/path")
            .build()
            .expect("Builder error")
            .build()
            .expect("Builder error");

        assert_eq!(
            req.url().as_str(), 
            "https://androidpublisher.googleapis.com/upload/androidpublisher/v3/applications/com.test.org/edits/asdasd/test/custom/path"
        );
        
        let req = base_builder
            .edit_command("commit")
            .build()
            .expect("Builder error")
            .build()
            .expect("Builder error");

        assert_eq!(
            req.url().as_str(), 
            "https://androidpublisher.googleapis.com/upload/androidpublisher/v3/applications/com.test.org/edits/asdasd:commit"
        );
    }
}