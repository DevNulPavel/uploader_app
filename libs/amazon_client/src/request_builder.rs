use std::{
    sync::{
        Arc
    }
};
use log::{
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
        AmazonError
    },
    token::{
        AmazonAccessToken
    }
};

// https://developers.google.com/play/api/v3/reference/files/list

//////////////////////////////////////////////////////////////////////////////////////////

pub struct AmazonAppRequestBuilder<'a>{
    http_client: Client,
    api_url: Url,
    token: &'a AmazonAccessToken
}
impl<'a> AmazonAppRequestBuilder<'a> {
    pub fn new(http_client: Client,
               token: &'a AmazonAccessToken,
               app_id: &str) -> Result<AmazonAppRequestBuilder<'a>, AmazonError> {

        let base_addr = format!("https://developer.amazon.com/api/appstore/v1/applications/{}/", app_id);

        if !base_addr.ends_with("/"){
            return Err(AmazonError::InvalidBaseAddr("Base addr must ends with /".to_owned()));
        }
        
        let api_url = Url::parse(&base_addr)?;
        debug!("Api url: {}", api_url.as_str());

        Ok(AmazonAppRequestBuilder{
            http_client,
            token,
            api_url
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
                         path: &str) -> Result<RequestBuilder, AmazonError> {
        let AmazonAppRequestBuilder{
            http_client,
            token,
            api_url,
            ..
        } = &self;

        let full_url = api_url.join(path)?;

        let token = token.as_str_checked()?;

        let builder = http_client
            .request(method, full_url.as_str())
            .bearer_auth(token);

        Ok(builder)
    }
}
