use google_play_client::{GooglePlayClient, GooglePlayUploadTask};
use reqwest::Client;
use std::{path::PathBuf, sync::Once};
use tracing::info;
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

    let key = yup_oauth2::read_service_account_key("env/google_play_beta_credentials.json")
        .await
        .expect("Auth info parse failed");

    let auth = yup_oauth2::ServiceAccountAuthenticator::builder(key)
        .build()
        .await
        .expect("Authentification failed");

    let token = auth
        .token(&["https://www.googleapis.com/auth/androidpublisher"])
        .await
        .expect("Token receive failed");

    info!("Token: {}", token.as_str());

    let client = GooglePlayClient::new(Client::new(), token);

    info!("Google play client created");

    let file_path = PathBuf::from("/Users/devnul/Downloads/MHouseBeta-gplay-production-v5.130.1.589-b589-91d61914/MHouseXGen-gplay-production-5.130.1.589-91d61914.aab");

    let task = GooglePlayUploadTask {
        file_path: &file_path,
        package_name: "com.gameinsight.gplay.mmanorbeta2",
        target_track: Some("internal"), // target_track: None
    };
    let result = client
        .upload(task)
        .await
        .expect("Google play upload failed");

    info!("Uploaded build number: {}", result);
}
