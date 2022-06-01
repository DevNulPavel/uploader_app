use amazon_client::{
    request_token,
    // AmazonAccessToken,
    AmazonClient,
    AmazonUploadTask,
};
use reqwest::Client;
use std::{path::Path, sync::Once};
use tracing::debug;
use tracing_subscriber::prelude::*;

fn setup_logs() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        if std::env::var("RUST_LOG").is_err() {
            let current_package_name = env!("CARGO_PKG_NAME");
            let log_env_var_variable =
                format!("{current_package_name}=trace,integration_test=trace,reqwest=trace");
            std::env::set_var("RUST_LOG", log_env_var_variable);
        }

        // Поддержка стандартных вызовов log у других библиотек
        tracing_log::LogTracer::init().expect("Log proxy set failed");

        // Слой фильтрации сообщений
        let env_filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::default().add_directive(tracing::Level::DEBUG.into())
            });
        let env_print_layer = tracing_subscriber::fmt::layer()
            .compact()
            .with_ansi(false) // Disable colors
            .with_writer(std::io::stdout);
        let env_layer = env_filter_layer.and_then(env_print_layer);

        // Error trace capture layer
        let err_layer = tracing_error::ErrorLayer::default();

        // Собираем все слои вместе
        let reg = tracing_subscriber::registry()
            //.with(trace_layer)
            .with(env_layer)
            .with(err_layer);

        tracing::subscriber::set_global_default(reg).expect("Log subscriber set failed");
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
