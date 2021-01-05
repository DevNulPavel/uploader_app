use log::{
    debug,
    error
};
use serde::{
    Deserialize
};
use serde_json::{
    Value
};
use reqwest::{
    multipart::{
        Form,
        Part
    }
};
use super::{
    request_builder::{
        SlackRequestBuilder
    },
    error::{
        SlackError,
        ViewOpenErrorInfo
    },
    // view_open_response::{
        // ViewOpenResponse,
        // ViewUpdateResponse,
        // ViewInfo
    // },
    message::{
        MessageInfo,
        Message
    },
    view::{
        ViewInfo,
        View
    },
    search_by_name::{
        search_by_fullname,
        iter_by_slack_users,
        UserInfo
    },
    
    UsersCache
};

//////////////////////////////////////////////////////////////////////////////////////////

// https://doc.rust-lang.org/reference/macros-by-example.html
macro_rules! slack_message_target_impl {
    ( 
        $struct_name:ident $(<$life:lifetime>)? $( ($out_type:ty) )? {
            $($var_name:ident: $var_type:ty),*
        }
        url: ($url_self:ident) 
            $url:block
        json: ($params_self:ident, $params_val:ident)
            $params:block
    ) => {
        pub struct $struct_name$(<$life>)?{
            $($var_name: $var_type),*
        }
        impl$(<$life>)? $struct_name$(<$life>)? {
            pub fn new($($var_name: $var_type),*) -> $struct_name$(<$life>)?{
                $struct_name{
                    $($var_name),*
                }
            }    
        }
        impl$(<$life>)? SlackMessageTarget for $struct_name$(<$life>)? {
            #[allow(unused_parens)]
            type Output = ( $(Option<$out_type>)? );

            fn get_url(&$url_self) -> &str{
                $url
            }
            fn update_json(&$params_self, $params_val: &mut Value) {
                $params
            }
        }
    };
}

////////////////////////////////////////////////////////

pub trait SlackMessageTargetOutput{
    fn new_valid(client: SlackRequestBuilder, info: MessageInfo, channel_id: String, thread_id: String) -> Self;
    fn new_empty() -> Self;
}
impl SlackMessageTargetOutput for () {
    fn new_valid(_: SlackRequestBuilder, _: MessageInfo, _: String, _: String) -> Self {}
    fn new_empty() -> Self {}
}
impl SlackMessageTargetOutput for Option<Message> {
    fn new_valid(client: SlackRequestBuilder, info: MessageInfo, channel_id: String, thread_id: String) -> Self {
        Some(Message::new(client, info, channel_id, thread_id))
    }
    fn new_empty() -> Self {
        None
    }
}

////////////////////////////////////////////////////////

pub trait SlackMessageTarget: Sized {
    type Output: SlackMessageTargetOutput;
    fn get_url(&self) -> &str;
    fn update_json(&self, json: &mut Value);
}

// Сообщение в канал
slack_message_target_impl!(
    SlackChannelMessageTarget<'a> (Message) {
        id: &'a str
    }
    url: (self) {
        "https://slack.com/api/chat.postMessage"
    }
    json: (self, json) {
        json["channel"] = Value::from(self.id);
    }
);
// Сообщение в тред
slack_message_target_impl!(
    SlackThreadMessageTarget<'a> (Message) {
        id: &'a str,
        thread_ts: &'a str
    }
    url: (self) {
        "https://slack.com/api/chat.postMessage"
    }
    json: (self, json) {
        json["channel"] = serde_json::Value::from(self.id);
        json["thread_ts"] = serde_json::Value::from(self.thread_ts);
    }
);
// Сообщение в личку
slack_message_target_impl!(
    SlackUserMessageTarget<'a> (Message){
        user_id: &'a str
    }
    url: (self) {
        "https://slack.com/api/chat.postMessage"
    }
    json: (self, json){
        json["channel"] = serde_json::Value::from(self.user_id);
        json["as_user"] = serde_json::Value::from(true);
    }
);
// Сообщение в канал, но видное только конкретному пользователю
slack_message_target_impl!(
    SlackEphemeralMessageTarget<'a> {
        channel_id: &'a str,
        user_id: &'a str
    }
    url: (self) {
        "https://slack.com/api/chat.postEphemeral"
    }
    json: (self, json) {
        json["channel"] = serde_json::Value::from(self.channel_id);
        json["user"] = serde_json::Value::from(self.user_id);
    }
);
// Сообщение в ответ на какое-то взаимодействие
slack_message_target_impl!(
    SlackResponseUrlMessageTarget<'a>{
        url: &'a str
    }
    url: (self) {
        self.url
    }
    json: (self, _json) {
    }
);

//////////////////////////////////////////////////////////////////////////////////////////

// https://doc.rust-lang.org/reference/macros-by-example.html
macro_rules! slack_image_target_impl {
    ( 
        $struct_name:ident $(<$life:lifetime>)? {
            $($var_name:ident: $var_type:ty),*
        }
        form: ($params_self:ident, $params_val:ident)
            $params:block
    ) => {
        pub struct $struct_name$(<$life>)?{
            $($var_name: $var_type),*
        }
        impl$(<$life>)? $struct_name$(<$life>)? {
            pub fn new($($var_name: $var_type),*) -> $struct_name$(<$life>)?{
                $struct_name{
                    $($var_name),*
                }
            }    
        }
        impl$(<$life>)? SlackImageTarget for $struct_name$(<$life>)? {
            fn update_form(&$params_self, $params_val: Form) -> Form {
                $params
            }
        }
    };
}

pub trait SlackImageTarget: Sized {
    fn update_form(&self, form: Form) -> Form;
}

// Сообщение в канал
slack_image_target_impl!(
    SlackChannelImageTarget<'a> {
        id: &'a str
    }
    form: (self, form) {
        form.part("channels", Part::text(self.id.to_owned()))
    }
);

// В тред
slack_image_target_impl!(
    SlackThreadImageTarget<'a> {
        id: &'a str,
        thread_ts: &'a str
    }
    form: (self, form) {
        form
            .part("channels", Part::text(self.id.to_owned()))
            .part("thread_ts", Part::text(self.thread_ts.to_owned()))
    }
);

// Таргет для сообщения в личку
slack_image_target_impl!(
    SlackUserImageTarget<'a> {
        user_id: &'a str
    }
    form: (self, form) {
        form.part("channels", Part::text(self.user_id.to_owned()))
    }
);

//////////////////////////////////////////////////////////////////////////////////////////

pub struct SlackClient{
    request_builder: SlackRequestBuilder
}

impl SlackClient {
    pub fn new(client: reqwest::Client, token: String) -> SlackClient {
        let request_builder = SlackRequestBuilder::new(client, token);
        SlackClient{
            request_builder
        }
    }

    pub async fn open_view(&self, window_json: Value) -> Result<View, SlackError>{
        // https://serde.rs/enum-representations.html
        // https://api.slack.com/methods/views.open#response
        #[derive(Deserialize, Debug)]
        #[serde(untagged)]
        pub enum ViewOpenResponse{
            Ok{ view: ViewInfo },
            Error(ViewOpenErrorInfo)
        }

        let response = self.request_builder
            .build_post_request("https://slack.com/api/views.open")
            .header("Content-type", "application/json")
            .body(serde_json::to_string(&window_json).unwrap())
            .send()
            .await
            .map_err(|err|{
                SlackError::RequestErr(err)
            })?
            .json::<ViewOpenResponse>()
            .await
            .map_err(|err|{
                SlackError::JsonParseError(err)
            })?;

        match response {
            ViewOpenResponse::Ok{view} => {
                Ok(View::new(self.request_builder.clone(), view))
            },
            ViewOpenResponse::Error(err) => {
                Err(SlackError::ViewOpenError(err))
            }
        }
    }

    pub async fn send_message<T: SlackMessageTarget>(&self, message: &str, target: T) -> Result<T::Output, SlackError> {
        // https://api.slack.com/messaging/sending
        // https://api.slack.com/methods/chat.postMessage

        // Наше сообщения
        let message_json = {
            let mut json = serde_json::json!({
                "text": message
            });
            target.update_json(&mut json);
            json
        };

        // Ответ на постинг сообщения
        #[derive(Deserialize, Debug)]
        #[serde(untagged)]
        enum MessageResponse{
            Ok{
                ok: bool,
                channel: String,
                ts: String,
                message : MessageInfo
            },
            OtherOk{
                ok: bool
            },
            Err{
                ok: bool,
                error: String
            }
        };

        // Либо можем использовать стандартный урл, 
        // либо можем использовать урл для отправки сообщения
        // https://api.slack.com/messaging/sending#sending_methods
        // https://api.slack.com/interactivity/handling#message_responses
        let url = target.get_url();

        let response = self.request_builder
            .build_post_request(url)
            .header("Content-type", "application/json")
            .body(serde_json::to_string(&message_json).unwrap())
            .send()
            .await
            .map_err(|err|{
                SlackError::RequestErr(err)
            })?;

        let response = response
            .json::<MessageResponse>()
            .await
            .map_err(|err|{
                SlackError::JsonParseError(err)
            })?;

        match response {
            MessageResponse::Ok{ok, channel, ts, message} =>{
                if ok {
                    Ok(T::Output::new_valid(self.request_builder.clone(), message, channel, ts))
                }else{
                    return Err(SlackError::Custom(format!("Slack response: {}", ok)))
                }
            },
            MessageResponse::OtherOk{ok} =>{
                if ok {
                    Ok(T::Output::new_empty())
                }else{
                    return Err(SlackError::Custom(format!("Slack response: {}", ok)))
                }
            },
            MessageResponse::Err{error, ..} => {
                return Err(SlackError::Custom(error))
            }
        }
    }
 
    pub async fn send_image<T>(&self, data: Vec<u8>, commentary: Option<String>, target: T) -> Result<(), SlackError> 
    where T: SlackImageTarget {
        // https://api.slack.com/methods/files.upload
        
        // File path
        let new_uuid = uuid::Uuid::new_v4();
        let filename = format!("{}.png", new_uuid);

        let mut form = Form::new()
            .part("filename", Part::text(filename.to_owned()))
            .part("file", Part::stream(data).file_name(filename));
    
        if let Some(commentary) = commentary{
            form = form
                .part("initial_comment", Part::text(commentary));
        }

        form = target.update_form(form); 
    
        // https://api.slack.com/methods/files.upload
        #[derive(Deserialize, Debug)]
        struct File{
            id: String,
            name: String,
            title: String,
            user: String,
            channels: Vec<String>
        }
        #[derive(Deserialize, Debug)]
        #[serde(untagged)]
        enum Response{
            Ok{
                ok: bool,
                file: File
            },
            Error{
                ok: bool,
                error: String
            }
        }

        let response = self
            .request_builder
            .build_post_request("https://slack.com/api/files.upload")
            .multipart(form)
            .send()
            .await
            .map_err(|err|{
                SlackError::RequestErr(err)
            })?
            .json::<Response>()
            .await
            .map_err(|err|{
                SlackError::JsonParseError(err)
            })?;

        match response{
            Response::Ok{ok, file} => {
                if ok {
                    debug!("File upload result: {:?}", file);
                    Ok(())
                }else{
                    Err(SlackError::Custom("File upload result: false".to_owned()))
                }
            },
            Response::Error{error, ..} => {
                Err(SlackError::Custom(error))
            }
        }
    }

    pub async fn find_user_id_by_email(&self, email: &str) -> Option<String> {
        // Проверяем наличие email
        if email.is_empty(){
            return None;
        }
    
        // Выполняем GET запрос
        let get_parameters = vec![
            //("token", self.token.to_owned()), // TODO: нужно протестировать
            ("email", email.to_owned())
        ];
        let response = self.request_builder
            .build_get_request("https://slack.com/api/users.lookupByEmail")
            .query(&get_parameters)
            .send()
            .await
            .ok()?;
        //println!("{:?}", response);
    
        // Создаем структурки, в которых будут нужные значения
        #[derive(Deserialize, Debug)]
        struct UserInfo {
            id: String,
        }
        #[derive(Deserialize, Debug)]
        struct UserResponse {
            ok: bool,
            user: UserInfo,
        }
    
        // Парсим ответ в json
        let response_json = response
            .json::<UserResponse>()
            .await
            .ok()?;
        //println!("{:?}", response_json);
        
        // Результат, если все ок
        if response_json.ok {
            return Some(response_json.user.id);
        }
    
        None
    }

    // TODO: Возвращать impl Future<>
    pub async fn find_user_id_by_name<'a>(&'a self, 
                                          user_full_name: &'a str, 
                                          cache: Option<&(dyn UsersCache + Send + Sync)>) -> Option<String> {
        // Проверяем наличие user
        if user_full_name.is_empty(){
            return None;
        }

        // Переводим имя в нижний регистр
        let user = user_full_name.to_lowercase();

        // Ищем в кеше
        if let Some(ref local_cache) = cache {
            match local_cache.get(&user).await {
                Ok(found) => {
                    if let Some(found) = found {
                        return Some(found.id);
                    }
                },
                Err(err) => {
                    error!("Find user in cache error: {}", err);
                }
            }
        }

        // Создаем новый объект результата
        let found_info: Option<UserInfo> = {
            let mut full_users_list: Vec<UserInfo> = Vec::new();

            let mut last_cursor = Option::None;
            // У цикла можно указать метку, затем с помощью break можно прервать работу именно этого цикла
            let mut found_info_local: Option<UserInfo> = 'tag: loop{
                // Получаем список юзеров итерационно
                let (new_cursor, mut users_list) = iter_by_slack_users(&self.request_builder, last_cursor).await;

                // Нет юзеров - конец
                if users_list.is_empty() {
                    break 'tag None;
                }

                // Проверяем короткое имя
                let found_info_local = users_list
                    .iter()
                    .find(|user_info|{
                        user_info.name == user
                    })
                    .map(|val| {
                        val.clone()
                    });

                // Нашли - все ок
                if found_info_local.is_some() {
                    break 'tag found_info_local;
                }

                // Если не нашлось - сохраняем для полного поиска
                full_users_list.append(&mut users_list);

                // Сохраняем курсор для новой итерации
                last_cursor = new_cursor;

                // Если нет нового курсора - заканчиваем итерации
                if last_cursor.is_none(){
                    break 'tag None;
                }
            };

            // Если поиск по короткому имени не отработал, пробуем по полному имени
            if found_info_local.is_none(){
                found_info_local = search_by_fullname(full_users_list, &user);
            }
            
            found_info_local
        };
        
        if let Some(info) = found_info{
            if let Some(users_cache) = cache {
                // Добавляем найденного пользователя в кэш
                if let Err(err) = users_cache.set(&user, info.clone()).await{
                    error!("Write cache failed with error: {}", err);
                }
            }

            //println!("{:?}", info);
            return Some(info.id);
        }

        None
    }
}