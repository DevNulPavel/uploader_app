use facebook_instant_client::FacebookInstantClient;
use reqwest::Client;
use std::{env, path::PathBuf, sync::Once};
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

    // Переменные окружения
    let app_id = env::var("FACEBOOK_INSTANT_APP_ID").expect("Missing env variable");
    let app_secret = env::var("FACEBOOK_INSTANT_APP_SECRET").expect("Missing env variable");

    let zip_file_path = PathBuf::from("/Users/devnul/projects/island2/build/Emscripten_fbi/fbi-12.51.1-fb0ae4d9bd-Release-cheats.zip");

    // Создаем HTTP клиента, можно спокойно клонировать, внутри Arc
    let http_client = Client::new();

    // Создаем клиента для выгрузки
    let client = FacebookInstantClient::new(http_client, app_id, app_secret)
        .await
        .expect("Facebook client create failed");

    // Выгружаем данные
    client
        .upload(zip_file_path, "Commentary".to_owned())
        .await
        .expect("Uploading");
}
