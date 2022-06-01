use app_center_client::{
    AppCenterBuildGitInfo, AppCenterBuildUploadTask, AppCenterBuildVersionInfo, AppCenterClient,
};
use reqwest::Client;
use std::{path::PathBuf, sync::Once};
use tracing::{debug, info};
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
