use google_drive_client::{GoogleDriveClient, GoogleDriveUploadTask};
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
    setup_logs();

    let key = yup_oauth2::read_service_account_key("env/test_google_drive_gi_auth_new.json")
        .await
        .expect("Auth info parse failed");

    let auth = yup_oauth2::ServiceAccountAuthenticator::builder(key)
        .build()
        .await
        .expect("Authentification failed");

    let token = auth
        .token(&[
            "https://www.googleapis.com/auth/drive",
            // "https://www.googleapis.com/auth/drive/file"
        ])
        .await
        .expect("Token receive failed");

    info!("Token: {}", token.as_str());

    let client = GoogleDriveClient::new(Client::new(), token);

    info!("Google drive client created");

    let parent_folder = client
        .get_folder_for_id("18xmVr0MGGLximw6TgPeWUZp9aEf81cCc")
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
        owner_domain: None,
    };
    info!("Google drive task created");

    let res = client
        .upload(task)
        .await
        .expect("Google drive uploading failed");

    info!("Uploading success: {:#?}", res);
}
