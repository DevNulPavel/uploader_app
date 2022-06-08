use microsoft_azure_client::MicrosoftAzureClient;
use reqwest::Client;
use std::{env, path::Path, sync::Once};
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
    let tenant_id = env::var("MICROSOFT_AZURE_TENANT_ID").expect("Missing env variable");
    let client_id = env::var("MICROSOFT_AZURE_CLIENT_ID").expect("Missing env variable");
    let client_secret = env::var("MICROSOFT_AZURE_SECRET_KEY").expect("Missing env variable");

    // Файлик выгрузки
    //let application_id = "9PBPBN166FXW".to_owned(); // MM
    let application_id = "9NBLGGH2TBBG".to_owned(); // PI2
    let upload_file_path =
        Path::new("/Users/devnul/Downloads/Island2-PROD-Win-RELEASE_SERVER-PROD-49-68ba7fa563f48147d4629fe00dedf54f8ec2f8aa.zip");

    // Создаем HTTP клиента, можно спокойно клонировать, внутри Arc
    let http_client = Client::new();

    // Создаем клиента
    let client = MicrosoftAzureClient::new(
        http_client,
        tenant_id,
        client_id,
        client_secret,
        application_id,
    )
    .expect("Client create failed");

    // Делавем попытку выгрузки теста
    // {
    //     let groups = vec!["1152921504607280735".to_owned()];
    //     let test_flight_name = "Flight name test".to_owned();
    //     client
    //         .upload_flight_build(upload_file_path, groups, test_flight_name)
    //         .await
    //         .expect("Upload failed");
    // }

    // Делавем попытку выгрузки продакшена
    {
        let upload_name = "Production test".to_owned();
        client
            .upload_production_build(upload_file_path, upload_name)
            .await
            .expect("Upload failed");
    }
}
