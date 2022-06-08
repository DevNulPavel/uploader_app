use app_center_client::{
    AppCenterBuildGitInfo, AppCenterBuildUploadTask, AppCenterBuildVersionInfo, AppCenterClient,
};
use reqwest::Client;
use std::{path::PathBuf, sync::Once};
use log::{debug, info};

fn setup_logs() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let logger = env_logger::builder().is_test(true).build();
        log::set_boxed_logger(Box::new(logger)).expect("Logger set failed");
    })
}

#[tokio::test]
async fn library_integration_test() {
    // tokio::time::delay_for(std::time::Duration::from_secs(60)).await;

    setup_logs();

    let token = std::env::var("APP_CENTER_ACCESS_TOKEN")
        .expect("APP_CENTER_ACCESS_TOKEN environment variable is missing");

    debug!("App center token: {}", token);

    let app_name = "Paradise-Island-2-Google-Play".to_owned();
    let app_owner = "Game-Insight-HQ-Organization".to_owned();

    let client = AppCenterClient::new(Client::new(), token, app_name, app_owner);

    let file_path = PathBuf::from(
        "/Users/devnul/Downloads/\
                                    app-release.apk",
    );
    let groups = vec![
        "Paradise Island 2 Team".to_owned(),
        "Collaborators".to_owned(),
    ];
    let description = "Test description".to_owned();
    let git_info = AppCenterBuildGitInfo {
        branch: "test_branch_name".to_owned(),
        commit: "aaabbbcccddd1234242343234".to_owned(),
    };
    let version_info = AppCenterBuildVersionInfo {
        build_code: 376,
        version: "12.9.4".to_owned(),
    };

    let task = AppCenterBuildUploadTask {
        file_path: file_path.as_path(),
        distribution_groups: Some(groups),
        build_description: Some(description),
        version_info: Some(version_info),
        git_info: Some(git_info),
        upload_threads_count: 10,
    };
    let res = client
        .upload_build(&task)
        .await
        .expect("App center uploading failed");

    info!("Uploading success: {:#?}", res);
}
