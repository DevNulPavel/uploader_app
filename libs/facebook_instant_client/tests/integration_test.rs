use std::sync::Once;

fn setup_logs() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        if std::env::var("RUST_LOG").is_err() {
            // export RUST_LOG=reqwest=trace
            // unset RUST_LOG
            std::env::set_var(
                "RUST_LOG",
                "microsoft_azure_client=trace,integration_test=trace,reqwest=trace",
            );
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
}
