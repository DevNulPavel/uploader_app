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
use app_center_client::{
    AppCenterClient,
    AppCenterBuildGitInfo,
    AppCenterBuildUploadTask
};


fn setup_logs() {
    static ONCE: Once = Once::new();
    ONCE.call_once(||{
        if std::env::var("RUST_LOG").is_err(){
            // export RUST_LOG=reqwest=trace
            // unset RUST_LOG
            std::env::set_var("RUST_LOG", "app_center_client=trace,integration_test=trace,reqwest=trace");
        }
        std::env::set_var("RUST_LOG_STYLE", "auto");
        env_logger::builder()
            //.is_test(true) // Выводить логи только в случае ошибки
            .try_init()
            .expect("Loggin setup failed");
    })
}


#[tokio::test]
async fn library_integration_test(){
    // tokio::time::delay_for(std::time::Duration::from_secs(60)).await;

    setup_logs();

    let token = std::env::var("APP_CENTER_ACCESS_TOKEN")
        .expect("APP_CENTER_ACCESS_TOKEN environment variable is missing");

    debug!("App center token: {}", token);

    let app_name = "Paradise-Island-2-Google-Play".to_owned();
    let app_owner = "Game-Insight-HQ-Organization".to_owned();

    let client = AppCenterClient::new(Client::new(), token, app_name, app_owner);

    let file_path = PathBuf::from("/Users/devnul/Downloads/\
                                  Island2-Android-qc-1130--2020.12.28_18.23-tf_12.10.0_giads_kinesis-400cd90.apk");
    let groups = vec![
        "Paradise Island 2 Team".to_owned(),
        "Collaborators".to_owned()
    ];
    let description = "Test description".to_owned();
    let git_info = AppCenterBuildGitInfo{
        branch: "test_branch_name".to_owned(),
        commit: "aaabbbcccddd1234242343234".to_owned()
    };

    let task = AppCenterBuildUploadTask{
        file_path: file_path.as_path(),
        distribution_groups: Some(groups),
        build_description: Some(description),
        git_info: Some(git_info),
        upload_threads_count: 10
    };
    let res = client
        .upload_build(&task)
        .await
        .expect("App center uploading failed");

    info!("Uploading success: {:#?}", res);
}