use std::{
    path::{
        PathBuf
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
use google_drive_client::{
    GoogleDriveClient,
    GoogleDriveUploadTask
};


fn setup_logs() {
    static ONCE: Once = Once::new();
    ONCE.call_once(||{
        if std::env::var("RUST_LOG").is_err(){
            // export RUST_LOG=reqwest=trace
            // unset RUST_LOG
            std::env::set_var("RUST_LOG", "google_drive_client=trace,integration_test=trace,reqwest=trace");
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

    let key = yup_oauth2::read_service_account_key("test_google_drive_gi_auth.json")
        .await
        .expect("Auth info parse failed");

    let auth = yup_oauth2::ServiceAccountAuthenticator::builder(key)
        .build()
        .await
        .expect("Authentification failed");

    let token = auth
        .token(&["https://www.googleapis.com/auth/drive"])
        .await
        .expect("Token receive failed");

    info!("Token: {}", token.as_str());

    let client = GoogleDriveClient::new(Client::new(), token);

    info!("Google drive client created");

    let parent_folder = client.get_folder_for_id("1L1hJLkOsmn1p9VdnbuncxE7pCdB9sdbk")
        .await
        .expect("Folder request failed")
        .expect("Folder can not be empty");

    debug!("Parent folder: {:#?}", parent_folder.get_info());

    let sub_folder = parent_folder
        .create_subfolder_if_needed("NewSubFolder")
        .await
        .expect("Subfolder create failed");

    debug!("Sub folder: {:#?}", sub_folder.get_info());

    let file_path = PathBuf::from("/Users/devnul/Downloads/jdk-15.0.1_osx-x64_bin.dmg");

    let task = GoogleDriveUploadTask {
        file_path,
        parent_folder: &sub_folder,
        owner_email: Some("devnulpavel@gmail.com"),
        owner_domain: None
    };
    info!("Google drive task created");

    let res = client
        .upload(task)
        .await
        .expect("Google drive uploading failed");

    info!("Uploading success: {:#?}", res);
}