use std::{
    sync::{
        Arc
    }
};
use tracing::{
    debug,
    // info
};
use reqwest::{
    Client,
    RequestBuilder,
    Url,
    Method
};
use super::{
    error::{
        AppCenterError
    }
};

//////////////////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
pub enum AppCenterUrlTarget{
    Api,
    Application
}

//////////////////////////////////////////////////////////////////////////////////////////

struct RequestBuilderInternal{
    http_client: Client,
    token: String,
    api_url: Url,
    app_url: Url,
}

//////////////////////////////////////////////////////////////////////////////////////////

pub struct AppCenterRequestBuilder{
    internal: Arc<RequestBuilderInternal>
}
impl Clone for AppCenterRequestBuilder {
    fn clone(&self) -> Self {
        AppCenterRequestBuilder{
            internal: self.internal.clone()
        }
    }
}
impl AppCenterRequestBuilder {
    pub fn new(http_client: Client, 
               token: String, 
               base_addr: &str,
               app_owner: &str,
               app_name: &str,) -> Result<AppCenterRequestBuilder, AppCenterError> {

        if !base_addr.ends_with('/'){
            return Err(AppCenterError::InvalidBaseAddr("Base addr must ends with /".to_owned()));
        }
        
        let api_url = Url::parse(base_addr)?;
        debug!("Api url: {}", api_url.as_str());

        let app_url = api_url.join(&format!("apps/{}/{}/", app_owner, app_name))?;
        debug!("Application url: {}", api_url.as_str());

        let internal = Arc::new(RequestBuilderInternal{
            http_client,
            token,
            api_url,
            app_url
        });

        Ok(AppCenterRequestBuilder{
            internal
        })
    }

    pub fn get_http_client(&self) -> &Client {
        &self.internal.http_client
    }

    /*pub fn get_token(&self) -> &str {
        &self.internal.token
    }*/

    /*pub fn build_custom<T: FnOnce(&Client)->RequestBuilder >(&self, generator: T) -> RequestBuilder {
        let RequestBuilderInternal{
            http_client,
            token,
            ..
        } = self.internal.as_ref();

        let builder = generator(http_client)
            .header("x-api-token", token);

        builder
    }*/

    pub fn build_request(&self, 
                         target: AppCenterUrlTarget, 
                         method: Method, 
                         path: &str, 
                         json: bool) -> Result<RequestBuilder, AppCenterError> {
        let RequestBuilderInternal{
            http_client,
            token,
            api_url,
            app_url,
            ..
        } = self.internal.as_ref();

        let url = match target {
            AppCenterUrlTarget::Api => api_url,
            AppCenterUrlTarget::Application => app_url,
        };

        let full_url = url.join(path)?;

        let builder = http_client
            .request(method, full_url.as_str())
            .header("x-api-token", token);

        let builder = if json {
            builder
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
        }else{
            builder
        };

        Ok(builder)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests{
    use super::*;
    use reqwest::{
        Method
    };

    #[test]
    fn test_request_builder(){
        let token: String = "asdsaddggd".to_owned();
        let base_addr: &str = "https://api.appcenter.ms/v0.1/"; // Слеш в конце обязателен для указания пути
        let owner: &str = "test_owner";
        let name: &str = "test_app_name";

        // let api_url = Url::parse(base_addr).unwrap();

        {
            let with_err = AppCenterRequestBuilder::new(Client::new(), 
                "asdasds".to_owned(), 
                &base_addr[0..(base_addr.len()-1)], 
                owner, name);
            
            assert!(with_err.is_err(), "App center base url must ends with /");
        }

        
        let builder = AppCenterRequestBuilder::new(Client::new(), 
                                                   token, 
                                                   base_addr, 
                                                   owner, name)
            .expect("App center request builder create failed");

        // GET
        {
            let req = builder
                .build_request(AppCenterUrlTarget::Api, Method::GET, "test/path/", true)
                .expect("API path failed")
                .build()
                .expect("API request build failed");
            
            // println!("{}", req.url().as_str());
            assert!(
                req.url()
                    .as_str()
                    .eq(&format!("{}test/path/", base_addr)),
                "Invalid url"
            );
            assert!(
                req.method() == Method::GET,
                "Method mest be GET"
            );
        }

        // POST
        {
            let req = builder
                .build_request(AppCenterUrlTarget::Api, Method::POST, "test/path/", true)
                .expect("API path failed")
                .build()
                .expect("API request build failed");
            
            // println!("{}", req.url().as_str());
            assert!(
                req.url()
                    .as_str()
                    .eq(&format!("{}test/path/", base_addr)),
                "Invalid url"
            );
            assert!(
                req.method() == Method::POST,
                "Method mest be GET"
            );
        }

        // GET APP
        {
            let req = builder
                .build_request(AppCenterUrlTarget::Application, Method::GET, "test/path/", true)
                .expect("API path failed")
                .build()
                .expect("API request build failed");
            
            // println!("{}", req.url().as_str());

            let need = format!("{}apps/{}/{}/test/path/", base_addr, owner, name);
            // println!("{}", need);

            assert!(
                req.url()
                    .as_str()
                    .eq(&need),
                "Invalid url"
            );
            assert!(
                req.method() == Method::GET,
                "Method mest be GET"
            );
        }


        // POST APP
        {
            let req = builder
                .build_request(AppCenterUrlTarget::Application, Method::POST, "test/path/", true)
                .expect("API path failed")
                .build()
                .expect("API request build failed");
            
            // println!("{}", req.url().as_str());

            let need = format!("{}apps/{}/{}/test/path/", base_addr, owner, name);
            // println!("{}", need);

            assert!(
                req.url()
                    .as_str()
                    .eq(&need),
                "Invalid url"
            );
            assert!(
                req.method() == Method::POST,
                "Method mest be GET"
            );
        }

        // TODO: Json
    }
}