use std::{
    sync::{
        Once
    },
    env::{
        self
    },
    path::{
        Path
    }
};
// use log::{
    // debug,
    // info
// };
use reqwest::{
    Client
};
use microsoft_azure_client::{
    MicrosoftAzureClient
};

fn setup_logs() {
    static ONCE: Once = Once::new();
    ONCE.call_once(||{
        if std::env::var("RUST_LOG").is_err(){
            // export RUST_LOG=reqwest=trace
            // unset RUST_LOG
            std::env::set_var("RUST_LOG", "microsoft_azure_client=trace,integration_test=trace,reqwest=trace");
        }
        std::env::set_var("RUST_LOG_STYLE", "auto");
        env_logger::builder()
            //.is_test(true) // Выводить логи только в случае ошибки
            .try_init() // Позволяет инициализировать много раз
            .ok();
    })
}

#[tokio::test]
async fn library_integration_test(){
    setup_logs();

    // Переменные окружения
    let tenant_id = env::var("MICROSOFT_AZURE_TENANT_ID").expect("Missing env variable");
    let client_id = env::var("MICROSOFT_AZURE_CLIENT_ID").expect("Missing env variable");
    let client_secret = env::var("MICROSOFT_AZURE_SECRET_KEY").expect("Missing env variable");
    let application_id = env::var("MICROSOFT_AZURE_STORE_ID").expect("Missing env variable");

    // Файлик выгрузки
    let upload_file_path = Path::new("/Users/devnul/Downloads/citybound-v0.1.2-824-ga076171.command.appxupload");
    // let upload_file_path = Path::new("/Users/devnul/Downloads/MHouseXGen_4.100.0.0_Win32.appxupload");

    // Создаем HTTP клиента, можно спокойно клонировать, внутри Arc
    let http_client = Client::new();

    // Создаем клиента
    let client = MicrosoftAzureClient::new(http_client, 
                                           tenant_id, 
                                           client_id, 
                                           client_secret, 
                                           application_id);

    // Делавем попытку выгрузки
    client
        .upload_production_build(upload_file_path)
        .await
        .expect("Upload failed");
}