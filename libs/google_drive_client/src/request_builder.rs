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
use yup_oauth2::{
    AccessToken
};
use super::{
    error::{
        GoogleDriveError
    }
};

// https://developers.google.com/drive/api/v3/reference/files/list

//////////////////////////////////////////////////////////////////////////////////////////

struct RequestBuilderInternal{
    http_client: Client,
    token: AccessToken,
    api_url: Url,
}

//////////////////////////////////////////////////////////////////////////////////////////

pub struct GoogleDriveRequestBuilder{
    internal: Arc<RequestBuilderInternal>
}
impl Clone for GoogleDriveRequestBuilder {
    fn clone(&self) -> Self {
        GoogleDriveRequestBuilder{
            internal: self.internal.clone()
        }
    }
}
impl GoogleDriveRequestBuilder {
    pub fn new(http_client: Client, 
               token: AccessToken) -> Result<GoogleDriveRequestBuilder, GoogleDriveError> {

        const BASE_ADDR: &str = "https://www.googleapis.com/";

        if !BASE_ADDR.ends_with("/"){
            return Err(GoogleDriveError::InvalidBaseAddr("Base addr must ends with /".to_owned()));
        }
        
        let api_url = Url::parse(BASE_ADDR)?;
        debug!("Api url: {}", api_url.as_str());

        let internal = Arc::new(RequestBuilderInternal{
            http_client,
            token,
            api_url
        });

        Ok(GoogleDriveRequestBuilder{
            internal
        })
    }

    // pub fn get_http_client(&self) -> &Client {
    //     &self.internal.http_client
    // }

    // pub fn get_token(&self) -> &AccessToken {
    //     &self.internal.token
    // }

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
                         method: Method, 
                         path: &str) -> Result<RequestBuilder, GoogleDriveError> {
        let RequestBuilderInternal{
            http_client,
            token,
            api_url,
            ..
        } = self.internal.as_ref();

        let full_url = api_url.join(path)?;

        if token.is_expired(){
            return Err(GoogleDriveError::TokenIsExpired);
        }

        let token = token.as_str();

        let builder = http_client
            .request(method, full_url.as_str())
            .bearer_auth(token);

        // let builder = if json {
        //     builder
        //         .header("Content-Type", "application/json")
        //         .header("Accept", "application/json")
        // }else{
        //     builder
        // };

        Ok(builder)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////

// #[cfg(test)]
// mod tests{
    // use super::*;
    // use reqwest::{
    //     Method
    // };

    /*#[test]
    fn test_request_builder(){
        // let token: String = "asdsaddggd".to_owned();

        // let api_url = Url::parse(base_addr).unwrap();

        /*{
            let with_err = GoogleDriveRequestBuilder::new(Client::new(), token.clone());
            
            assert!(with_err.is_err(), "Google drive base url must ends with /");
        }

        
        let builder = GoogleDriveRequestBuilder::new(Client::new(), token.clone())
            .expect("Google drive request builder create failed");*/

            
        // GET
        /*{
            let req = builder
                .build_request(Method::GET, "test/path/", true)
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
        }*/

        // POST
        /*{
            let req = builder
                .build_request(AppCenterUrlTarget::API, Method::POST, "test/path/", true)
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
        }*/

        // GET APP
        /*{
            let req = builder
                .build_request(AppCenterUrlTarget::APPLICATION, Method::GET, "test/path/", true)
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
        }*/


        // POST APP
        /*{
            let req = builder
                .build_request(AppCenterUrlTarget::APPLICATION, Method::POST, "test/path/", true)
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
        }*/

        // TODO: Json
    }*/
// }