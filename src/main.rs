mod env_parameters;
mod app_parameters;
mod uploaders;
mod result_senders;

use std::{
    pin::{
        Pin
    }
};
use tokio::{
    runtime::{
        Builder
    }
};
use futures::{
    future::{
        Future,
        FutureExt,
        select_all,
        join_all
    }
};
use tracing::{
    info,
    error,
    debug,
    instrument
};
use self::{
    app_parameters::{
        AppParameters
    },
    env_parameters::{
        AppEnvValues,
        ResultSlackEnvironment
    },
    uploaders::{
        upload_in_app_center,
        upload_in_google_drive,
        upload_in_google_play,
        upload_in_amazon,
        upload_in_ios,
        upload_by_ssh,
        UploadResult
    },
    result_senders::{
        ResultSender,
        SlackResultSender,
        TerminalSender
    }
};

#[instrument(skip(active_workers, result_senders))]
async fn wait_results<W, S>(mut active_workers: Vec<W>, 
                            mut result_senders: Vec<Box<S>>)
where 
    W: Future<Output=UploadResult> + Unpin,
    S: ResultSender + ?Sized {

    // Смотрим на завершающиеся воркеры
    while active_workers.len() > 0 {
        // Выбираем успешную фьючу, получаем оставшиеся
        let (res, _, left_workers) = select_all(active_workers)
            .await;
        active_workers = left_workers;

        // Обрабатываем результат
        match res {
            Ok(res) => {
                /*let mut futures = Vec::new();
                for mut sender in result_senders{
                    let fut = sender.send_result(&res);
                    futures.push(fut);
                }*/

                // Пишем во все получатели асинхронно
                let futures_iter = result_senders
                    .iter_mut()
                    .map(|sender|{
                        sender.send_result(&res)
                    });
                join_all(futures_iter).await;
            },
            Err(err) => {
                // Пишем во все получатели асинхронно
                let futures_iter = result_senders
                    .iter_mut()
                    .map(|sender|{
                        sender.send_error(err.as_ref())
                    });
                join_all(futures_iter).await;
                error!(%err, "Uploading task failed");
            }
        }
    }
}

struct UploadersResult {
    result_slack: Option<ResultSlackEnvironment>,
    active_workers: Vec<Pin<Box<dyn Future<Output=UploadResult> + Send>>>
}

#[instrument(skip(http_client, env_params, app_parameters))]
fn build_uploaders(http_client: reqwest::Client, 
                   env_params: AppEnvValues, 
                   app_parameters: AppParameters) -> UploadersResult {

    let mut active_workers = Vec::new();

    // Создаем задачу выгрузки в AppCenter
    match (env_params.app_center, app_parameters.app_center) {
        (Some(app_center_env_params), Some(app_center_app_params)) => {
            let fut = upload_in_app_center(http_client.clone(), 
                                           app_center_env_params, 
                                           app_center_app_params,
                                           env_params.git)
                .boxed();
            info!("App center uploading task created");
            active_workers.push(fut);
        },
        _ => {}
    }

    // Создаем задачу выгрузки в Google drive
    match (env_params.google_drive, app_parameters.goolge_drive) {
        (Some(env_params), Some(app_params)) => {
            let fut = upload_in_google_drive(http_client.clone(),
                                             env_params, 
                                             app_params)
                .boxed();
            info!("Google drive uploading task created");                                             
            active_workers.push(fut);
        },
        _ => {}
    }

    // Создаем задачу выгрузки в Google Play
    match (env_params.google_play, app_parameters.goolge_play) {
        (Some(env_params), Some(app_params)) => {
            let fut = upload_in_google_play(http_client.clone(),
                                            env_params, 
                                            app_params)
                .boxed();
            info!("Google play uploading task created");                
            active_workers.push(fut);
        },
        _ => {}
    }

    // Создаем задачу выгрузки в Amazon
    match (env_params.amazon, app_parameters.amazon) {
        (Some(env_params), Some(app_params)) => {
            let fut = upload_in_amazon(http_client,
                                       env_params, 
                                       app_params)
                .boxed();
            info!("Google play uploading task created");                
            active_workers.push(fut);
        },
        _ => {}
    }

    // Создаем задачу выгрузки в IOS
    match (env_params.ios, app_parameters.ios) {
        (Some(env_params), Some(app_params)) => {
            let fut = upload_in_ios(env_params, 
                                    app_params)
                .boxed();
            info!("IOS uploading task created");
            active_workers.push(fut);
        },
        _ => {}
    }

    // Создаем задачу выгрузки на SSH сервер
    match (env_params.ssh, app_parameters.ssh) {
        (Some(env_params), Some(app_params)) => {
            let fut = upload_by_ssh(env_params, 
                                    app_params)
                .boxed();
            info!("SSH uploading task created");
            active_workers.push(fut);
        },
        _ => {}
    }

    UploadersResult{
        result_slack: env_params.result_slack,
        active_workers: active_workers
    }
    //(env_params.result_slack, active_workers)
}

async fn async_main() {
    // Параметры приложения
    let app_parameters = AppParameters::parse(Some(||{
        AppEnvValues::get_possible_env_variables()    
            .into_iter()
            .fold(String::from("ENVIRONMENT VARIABLES:\n"), |mut prev, var|{
                prev.push_str("    - ");
                prev.push_str(var);
                prev.push_str("\n");
                prev
            })
    }));
    
    debug!(?app_parameters, "App params");

    // Получаем параметры окружения
    let env_params = AppEnvValues::parse();

    debug!(?env_params, "Env params");

    // Общий клиент для запросов
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .connect_timeout(std::time::Duration::from_secs(120))
        .build()
        .expect("Http client build failed");

    // Вектор с активными футурами выгрузки
    let UploadersResult{result_slack, active_workers} = build_uploaders(http_client.clone(), env_params, app_parameters);

    // Получаетели результатов выгрузки
    let result_senders = {
        let mut result_senders: Vec<Box<dyn ResultSender>> = Vec::new();

        // Создаем клиента для слака если надо отправлять результаты в слак
        if let Some(slack_params) = result_slack{
            let slack_sender = SlackResultSender::new(http_client, slack_params);
            result_senders.push(Box::new(slack_sender));    
        }

        // Результат в терминал
        result_senders.push(Box::new(TerminalSender{}));

        result_senders
    };

    wait_results(active_workers, result_senders).await;
}

fn setup_logs() {
    use tracing_subscriber::{
        prelude::{
            *
        }
    };

    // Поддержка стандартных вызовов log у других библиотек
    tracing_log::LogTracer::init()
        .expect("Log proxy set failed");

    // Слой фильтрации сообщений
    let env_filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_|{
            tracing_subscriber::EnvFilter::default()
                .add_directive(tracing::Level::INFO.into())
        });
    let env_print_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_ansi(false) // Disable colors
        .with_writer(std::io::stdout);
    let env_layer = env_filter_layer
        .and_then(env_print_layer);

    // Trace to file
    /*let (writer, guard) = tracing_appender::non_blocking(tracing_appender::rolling::never("uploading_logs/", "uploading.txt"));
    let trace_fileter_layer = tracing_subscriber::filter::LevelFilter::TRACE;
    let trace_print_layer = tracing_subscriber::fmt::layer()
        //.json()
        .with_ansi(false)
        .with_writer(writer);
    let trace_layer = trace_fileter_layer
        .and_then(trace_print_layer);*/

    // Error trace capture layer
    let err_layer = tracing_error::ErrorLayer::default();

    // Собираем все слои вместе
    let reg = tracing_subscriber::registry()
        //.with(trace_layer)
        .with(env_layer)
        .with(err_layer);

    tracing::subscriber::set_global_default(reg)
        .expect("Log subscriber set failed");
}

fn main() {
    // Активируем логирование и настраиваем уровни вывода
    let _guard = setup_logs();

    // Запускаем асинхронный рантайм
    let mut runtime = Builder::default()
        .enable_all()
        // .basic_scheduler()
        .threaded_scheduler()
        //.core_threads(1)
        //.max_threads(2)
        .build()
        .expect("Tokio runtime create failed");

    runtime.block_on(async_main());

    // Dump the report to disk
    #[cfg(feature = "flame_it")]
    flame::dump_html(&mut std::fs::File::create("flame-graph.html").unwrap()).unwrap();
}
