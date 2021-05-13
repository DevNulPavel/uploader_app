use std::{
    time::{
        Instant,
        Duration
    }
};
use reqwest::{
    Client
};
use serde_json::{
    json
};
use tracing::{
    debug
};
use super::{
    error::{
        AmazonError
    },
    responses::{
        AmazonTokenResponse
    }
};

#[derive(Debug)]
pub struct AmazonAccessToken{
    value: String,
    expire_time: Instant
}
impl AmazonAccessToken {
    pub fn new(value: String, expire_time: Instant) -> AmazonAccessToken {
        AmazonAccessToken{
            value,
            expire_time
        }
    }
    pub fn as_str_checked(&self) -> Result<&str, AmazonError>{
        if Instant::now() < self.expire_time {
            Ok(self.value.as_str())
        }else{
            Err(AmazonError::TokenIsExpired)
        }
    }
}

pub async fn request_token(http_client: &Client, client_id: &str, client_secret: &str) -> Result<AmazonAccessToken, AmazonError>{
    // https://developer.amazon.com/docs/login-with-amazon/authorization-code-grant.html#access-token-response

    let response = http_client
        .post("https://api.amazon.com/auth/o2/token")
        .json(&json!({
            "grant_type": "client_credentials",
            "client_id": client_id,
            "client_secret": client_secret,
            "scope": "appstore::apps:readwrite" // messaging:push, appstore::apps:readwrite
        }))
        .send()
        .await?
        .json::<AmazonTokenResponse>()
        .await?;

    debug!("Amazon token request response: {:#?}", response);

    let expire_time = Instant::now()
        .checked_add(Duration::from_secs(response.expires_in))
        .ok_or_else(||{
            AmazonError::InvalidTokenDuration(response.expires_in)
        })?;

    Ok(AmazonAccessToken::new(response.access_token, expire_time))
}