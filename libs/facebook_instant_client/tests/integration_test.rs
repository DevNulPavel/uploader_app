use facebook_instant_client::FacebookInstantClient;
use reqwest::Client;
use std::{env, path::PathBuf, sync::Once};

fn setup_logs() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let logger = env_logger::builder().is_test(true).build();
        log::set_boxed_logger(Box::new(logger)).expect("Logger set failed");
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
