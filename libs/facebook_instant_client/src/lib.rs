mod error;
mod json_helpers;
mod responses;

use crate::{
    error::{convert_error, FacebookInstantError},
    json_helpers::{parse_json, ParseJson},
    responses::{ResponseWrapper, TokenResponse},
};
use backtrace::Backtrace as BacktraceNoStd;
// use error::JsonParseError;
use reqwest::Client;
use tracing::{debug, Instrument};

pub struct FacebookInstantClient {
    http_client: Client,
    app_id: String,
    token: String,
}

impl FacebookInstantClient {
    pub async fn new(
        http_client: Client,
        app_id: String,
        app_secret: &str,
    ) -> Result<Self, FacebookInstantError> {
        let token_info = http_client
            .get("https://graph.facebook.com/oauth/access_token")
            .query(&[
                ("client_id", app_id.as_str()),
                ("client_secret", app_secret),
                ("grant_type", "client_credentials"),
            ])
            .send()
            .in_current_span()
            .await
            .map_err(convert_error!(Request, "Token request"))?
            .text()
            .await
            .map_err(convert_error!(ResponseReceiving, "Token request"))?
            .parse_json_with_data_err::<ResponseWrapper<TokenResponse>>()
            .map_err(convert_error!(ResponseParsing, "Token request"))?
            .into_result()
            .map_err(convert_error!(Response, "Token request"))?;
        debug!("Received token info from Facebook: {:?}", token_info);

        Ok(FacebookInstantClient {
            app_id,
            token: token_info.access_token,
            http_client,
        })
    }

    pub async fn upload(&self) -> Result<(), FacebookInstantError> {
        todo!()
    }
}
