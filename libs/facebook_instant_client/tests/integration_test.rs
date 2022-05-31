use facebook_instant_client::FacebookInstantClient;
use reqwest::Client;
use std::{env, sync::Once};

fn setup_logs() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        if std::env::var("RUST_LOG").is_err() {
            // export RUST_LOG=reqwest=trace
            // unset RUST_LOG
            let current_package_name = env!("CARGO_PKG_NAME");
            let log_env_var_variable =
                format!("{current_package_name}=trace,integration_test=trace,reqwest=trace");
            std::env::set_var("RUST_LOG", log_env_var_variable);
        }
        std::env::set_var("RUST_LOG_STYLE", "auto");
        env_logger::builder()
            //.is_test(true) // Выводить логи только в случае ошибки
            .try_init() // Позволяет инициализировать много раз
            .ok();
    })
}

#[tokio::test]
async fn library_integration_test() {
    setup_logs();

    // Переменные окружения
    let app_id = env::var("FACEBOOK_INSTANT_APP_ID").expect("Missing env variable");
    let app_secret = env::var("FACEBOOK_INSTANT_APP_SECRET").expect("Missing env variable");

    // Создаем HTTP клиента, можно спокойно клонировать, внутри Arc
    let http_client = Client::new();

    // Создаем клиента для выгрузки
    let _client = FacebookInstantClient::new(http_client, app_id, &app_secret)
        .await
        .expect("Facebook client create failed");
    drop(app_secret);
}
