use super::{qr::create_qr_data, ResultSender};
use crate::{env_parameters::ResultSlackEnvironment, uploaders::UploadResultData};
use async_trait::async_trait;
use futures::future::{join_all, Future, FutureExt};
use reqwest::Client;
use slack_client_lib::{
    SlackChannelImageTarget,
    SlackChannelMessageTarget,
    SlackClient,
    SlackThreadImageTarget,
    SlackUserImageTarget,
    SlackUserMessageTarget,
    // UsersCache,
    // UsersJsonCache,
    UsersSqliteCache,
};
use std::{borrow::Cow, error::Error, path::PathBuf, pin::Pin, sync::Arc};
use tokio::{
    join, spawn,
    task::{
        spawn_blocking, // spawn_local
        JoinHandle,
    },
    sync::Mutex
};
use tracing::error;

fn qr_future_for_result(
    install_url: Option<String>,
) -> Pin<Box<dyn Future<Output = Option<QRInfo>> + Send>> {
    let qr_data_future = match install_url {
        Some(url) => {
            let fut = async move {
                let qr_data = url.clone();
                let res: Option<QRInfo> = spawn_blocking(move || create_qr_data(&qr_data).ok())
                    .await
                    .expect("QR code create spawn failed")
                    .map(|qr_data| {
                        /*let inner = Arc::new(QRInfoInner{
                            url: url,
                            qr_data
                        });
                        QRInfo{
                            inner
                        }*/
                        QRInfo { qr_data }
                    });
                res
            };
            fut.boxed()
        }
        None => futures::future::ready(Option::None).boxed(),
    };
    qr_data_future
}

macro_rules! message_target_impl {
    ($fn_name:ident, $target_type:ident$(<$life:lifetime>)?) => {
        async fn $fn_name <'a, Q: Future<Output=Option<QRInfo>>>(sender: &SenderResolved,
                                                              qr_data_future: Q,
                                                              target: $target_type$(<$life>)?,
                                                              text: &str) {

            let (message_result, qr) = join!(
                async{
                    sender
                        .client
                        .send_message(text, target)
                        .await
                        .ok()
                        .flatten()
                },
                qr_data_future
            );

            match (message_result, qr) {
                (Some(message), Some(qr)) => {
                    let target = SlackThreadImageTarget::new(
                        message.get_channel_id(),
                        message.get_thread_id()
                    );
                    let image_res = sender
                        .client
                        .send_image(
                            qr.qr_data,
                            None,
                            target
                        )
                        .await;

                    if let Err(err) = image_res {
                        error!("Slack image uploading failed with err: {}", err);
                    }
                },
                _ => {
                }
            }
        }
    };
}
message_target_impl!(message_to_channel_target, SlackChannelMessageTarget<'a>);
message_target_impl!(message_to_user_target, SlackUserMessageTarget<'a>);

// QR код
/*struct QRInfoInner{
    url: String,
    qr_data: Vec<u8>
}
#[derive(Clone)]
struct QRInfo{
    inner: Arc<QRInfoInner>,
}*/
#[derive(Clone)]
struct QRInfo {
    qr_data: Vec<u8>,
}

enum ChannelType {
    All(String),
    ErrorsOnly(String),
    None,
}

struct SenderResolved {
    client: SlackClient,
    text_prefix: Option<String>,
    channel: ChannelType,
    user_id: Option<String>,
}

enum ResultSenderState {
    Pending(JoinHandle<SenderResolved>),
    Resolved(Arc<SenderResolved>),
}

pub struct SlackResultSender {
    inner: Arc<Mutex<ResultSenderState>>,
}
impl SlackResultSender {
    pub fn new(http_client: Client, params: ResultSlackEnvironment) -> SlackResultSender {
        let join = spawn(async move {
            let client = SlackClient::new(http_client, params.token.clone()); // TODO: Убрать клонирование
            let client_ref = &client;

            // Локальная функция для упрощения
            async fn find_user_by_name(client_ref: &SlackClient, name: &str) -> Option<String> {
                // Json cache
                /*let cache_file_path = PathBuf::new()
                    .join(dirs::home_dir().unwrap())
                    .join(".cache/uploader_app/users_cache.json");
                let cache = Some(UsersJsonCache::new(cache_file_path).await);*/

                // Sqlite cache
                let cache_file_path = PathBuf::new()
                    .join(dirs::home_dir().unwrap())
                    .join(".cache/uploader_app/users_cache.sqlite");
                let cache: Option<UsersSqliteCache> =
                    UsersSqliteCache::new(cache_file_path).await.ok();

                // TODO: Как-то сконвертировать в тип сразу?
                match cache {
                    Some(cache) => client_ref.find_user_id_by_name(name, Some(&cache)).await,
                    None => client_ref.find_user_id_by_name(name, None).await,
                }
            }

            // Пытаемся найти user_id по email или/и имени
            let user_id: Option<String> = match (params.user_email, params.user_name) {
                (Some(email), Some(name)) => {
                    // Сначала ищем по email
                    match client_ref.find_user_id_by_email(&email).await {
                        user_id @ Some(_) => user_id,
                        None => {
                            // Если нету - тогда по имени
                            find_user_by_name(client_ref, &name).await
                        }
                    }
                }
                (None, Some(name)) => {
                    // Поиск по имени
                    find_user_by_name(client_ref, &name).await
                }
                (Some(email), None) => {
                    // Поиск по email
                    client_ref.find_user_id_by_email(&email).await
                }
                (None, None) => None,
            };

            // Канал для отправки?
            let channel = match params.channel {
                Some(channel) => {
                    // Если имя канала заканчивается на "[ERRORS_ONLY]" - то канал лишь для ошибок
                    match channel.strip_suffix("[ERRORS_ONLY]") {
                        Some(real_channel) => ChannelType::ErrorsOnly(real_channel.to_owned()),
                        None => ChannelType::All(channel),
                    }
                }
                None => ChannelType::None,
            };

            SenderResolved {
                client,
                text_prefix: params.text_prefix,
                channel,
                user_id,
            }
        });
        SlackResultSender {
            inner: Arc::new(Mutex::new(ResultSenderState::Pending(join))),
        }
    }

    async fn resolve_sender(&self) -> Arc<SenderResolved> {
        let sender = loop {
            use std::ops::DerefMut;
            let mut lock = self.inner.lock().await;
            match lock.deref_mut() {
                ResultSenderState::Pending(join) => {
                    let resolved = join.await.expect("Slack sender resolve failed");
                    *lock = ResultSenderState::Resolved(Arc::new(resolved));
                    // self.inner = Arc::new(Mutex::new());
                }
                ResultSenderState::Resolved(sender) => {
                    break sender.clone();
                }
            }
        };
        sender
    }
}

// #[async_trait(?Send)]
#[async_trait]
impl ResultSender for SlackResultSender {
    async fn send_result(&self, result: &UploadResultData) {
        let sender_arc = self.resolve_sender().await;
        let sender = sender_arc.as_ref();

        // Собираем текст в кучу
        let text = {
            let mut strings = Vec::new();
            if let Some(prefix) = &sender.text_prefix {
                strings.push(Cow::from(prefix));
            }
            if let Some(message) = &result.message {
                let text = format!("```{}```", message);
                strings.push(Cow::from(text));
            }
            if !strings.is_empty() {
                Some(strings.join("\n"))
            } else {
                None
            }
        };

        // Создаем футуру с результатом QR
        let qr_data_future = qr_future_for_result(result.install_url.clone());

        // Сообщение
        if let Some(message) = &text {
            // Массив наших тасков
            let mut futures_vec = Vec::new();

            let qr_data_future = qr_data_future.shared();

            // В канал
            if let ChannelType::All(ref channel) = sender.channel {
                let qr_data_future = qr_data_future.clone();
                let fut = async move {
                    let target = SlackChannelMessageTarget::new(channel);
                    message_to_channel_target(sender, qr_data_future, target, message).await;
                };

                futures_vec.push(fut.boxed());
            }

            // Юзеру
            if let Some(user_id) = &sender.user_id {
                let fut = async move {
                    let target = SlackUserMessageTarget::new(user_id);
                    message_to_user_target(sender, qr_data_future, target, message).await;
                };

                futures_vec.push(fut.boxed());
            }

            join_all(futures_vec).await;
        } else {
            // Массив наших тасков
            let mut futures_vec = Vec::new();

            // Просто засылаем QR код если нету сообщения
            if let Some(qr_info) = qr_data_future.await {
                // let QRInfo { url, qr_data } = qr_info.deref();

                // В канал
                if let ChannelType::All(channel) = &sender.channel {
                    let target = SlackChannelImageTarget::new(channel);
                    let fut = sender
                        .client
                        .send_image(qr_info.qr_data.clone(), None, target);
                    futures_vec.push(fut.boxed());
                }

                // Юзеру
                if let Some(user_id) = &sender.user_id {
                    let target = SlackUserImageTarget::new(user_id);
                    let fut = sender.client.send_image(qr_info.qr_data, None, target);
                    futures_vec.push(fut.boxed());
                }

                join_all(futures_vec).await;
            }
        }
    }

    async fn send_error(&self, err: &(dyn Error + Send + Sync)) {
        let sender = self.resolve_sender().await;

        let message = format!("Uploading error:```{}```", err);

        let futures_vec = {
            let mut futures_vec = Vec::new();

            // Пишем в канал
            match &sender.channel {
                ChannelType::All(channel) | ChannelType::ErrorsOnly(channel) => {
                    let target = SlackChannelMessageTarget::new(channel);
                    let fut = sender.client.send_message(&message, target).boxed();
                    futures_vec.push(fut);
                }
                &ChannelType::None => {}
            }

            // Пишем пользователю
            if let Some(user_id) = &sender.user_id {
                let target = SlackUserMessageTarget::new(user_id);
                let fut = sender.client.send_message(&message, target).boxed();
                futures_vec.push(fut);
            }

            futures_vec
        };

        join_all(futures_vec).await;
    }
}
