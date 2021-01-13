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
use yup_oauth2::{
    AccessToken
};
use super::{
    error::{
        GooglePlayError
    }
};

// https://developers.google.com/play/api/v3/reference/files/list

//////////////////////////////////////////////////////////////////////////////////////////

pub struct GooglePlayRequestBuilder<'a>{
    http_client: Client,
    token: &'a AccessToken,
    api_url: Url
}
impl<'a> GooglePlayRequestBuilder<'a> {
    pub fn new(http_client: Client,
               base_url: &str,
               package_name: &str,
               token: &'a AccessToken) -> Result<GooglePlayRequestBuilder<'a>, GooglePlayError> {

        let base_url = format!(
            "{}/{}/",
            base_url.trim_end_matches('/'),
            package_name
        );

        if !base_url.ends_with("/"){
            return Err(GooglePlayError::InvalidBaseAddr("Base addr must ends with /".to_owned()));
        }
        
        let api_url = Url::parse(&base_url)?;
        debug!("Api url: {}", api_url.as_str());

        Ok(GooglePlayRequestBuilder{
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
                         path: &str) -> Result<RequestBuilder, GooglePlayError> {
        let GooglePlayRequestBuilder{
            http_client,
            token,
            api_url,
            ..
        } = &self;

        let full_url = api_url.join(path)?;

        if token.is_expired(){
            return Err(GooglePlayError::TokenIsExpired);
        }

        let token = token.as_str();

        let builder = http_client
            .request(method, full_url.as_str())
            .bearer_auth(token);

        Ok(builder)
    }
}
