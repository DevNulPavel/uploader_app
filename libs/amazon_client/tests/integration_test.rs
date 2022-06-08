use amazon_client::{
    request_token,
    // AmazonAccessToken,
    AmazonClient,
    AmazonUploadTask,
};
use log::debug;
use reqwest::Client;
use std::{path::Path, sync::Once};

fn setup_logs() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "debug");
        }
        env_logger::init();
    })
}

#[tokio::test]
async fn library_integration_test() {
    setup_logs();

    let client_id = std::env::var("AMAZON_CLIENT_ID")
        .expect("AMAZON_CLIENT_ID environment variable is missing");

    let client_secret = std::env::var("AMAZON_CLIENT_SECRET")
        .expect("AMAZON_CLIENT_SECRET environment variable is missing");

    let app_id =
        std::env::var("AMAZON_APP_ID").expect("AMAZON_APP_ID environment variable is missing");

    let http_client = Client::new();

    let token = request_token(&http_client, &client_id, &client_secret)
        .await
        .expect("Access token request failed");

    let token_str = token.as_str_checked().expect("Token string get failed");

    debug!("Token: {:#?}", token_str);

    let file_path = Path::new(
        "/Users/devnul/Downloads/Island2-arm32-amazon-12.16.5-413-06092021_1919-6d8422f5.apk",
    );

    let client = AmazonClient::new(http_client, token);
    let task = AmazonUploadTask {
        application_id: &app_id,
        file_path,
    };
    client.upload(task).await.expect("Uploading failed");
}
