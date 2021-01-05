use std::{
    path::{
        PathBuf,
        Path
    }, 
    sync::{
        Once
    }
};
use log::{
    debug,
    info
};
use reqwest::{
    Client
};
use amazon_client::{
    AmazonAccessToken,
    AmazonClient,
    AmazonUploadTask,
    request_token
};


fn setup_logs() {
    static ONCE: Once = Once::new();
    ONCE.call_once(||{
        if std::env::var("RUST_LOG").is_err(){
            // export RUST_LOG=reqwest=trace
            // unset RUST_LOG
            std::env::set_var("RUST_LOG", "amazon_client=trace,integration_test=trace,reqwest=trace,serde_json=trace");
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

    let client_id = std::env::var("AMAZON_CLIENT_ID")
        .expect("AMAZON_CLIENT_ID environment variable is missing");

    let client_secret = std::env::var("AMAZON_CLIENT_SECRET")
        .expect("AMAZON_CLIENT_SECRET environment variable is missing"); 

    let app_id = std::env::var("AMAZON_APP_ID")
        .expect("AMAZON_APP_ID environment variable is missing");       

    let http_client = Client::new();

    let token = request_token(&http_client, &client_id, &client_secret)
        .await
        .expect("Access token request failed");

    let token_str = token
        .as_str_checked()
        .expect("Token string get failed");
    
    debug!("Token: {:#?}", token_str);

    let file_path = Path::new("test_file");

    let client = AmazonClient::new(http_client, token);
    let task = AmazonUploadTask{
        application_id: &app_id,
        file_path: file_path
    };
    client
        .upload(task)
        .await
        .expect("Uploading failed");
}