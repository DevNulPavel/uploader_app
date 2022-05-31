use std::{
    path::{
        PathBuf
    }, 
    sync::{
        Once
    }
};
use tracing::{
    // debug,
    info
};
use reqwest::{
    Client
};
use google_play_client::{
    GooglePlayClient,
    GooglePlayUploadTask
};


fn setup_logs() {
    static ONCE: Once = Once::new();
    ONCE.call_once(||{
        if std::env::var("RUST_LOG").is_err(){
            // export RUST_LOG=reqwest=trace
            // unset RUST_LOG
            std::env::set_var("RUST_LOG", "google_play_client=trace,integration_test=trace,reqwest=trace");
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

    let task = GooglePlayUploadTask{
        file_path: &file_path,
        package_name: "com.gameinsight.gplay.mmanorbeta2",
        target_track: Some("internal")
        // target_track: None
    };
    let result = client.upload(task)
        .await
        .expect("Google play upload failed");

    info!("Uploaded build number: {}", result);
}