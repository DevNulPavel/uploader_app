use std::{
    error::{
        Error
    }
};
use async_trait::{
    async_trait
};
use crate::{
    search_by_name::{
        UserInfo
    }
};


// #[async_trait(?Send)]
#[async_trait]
pub trait UsersCache{
    // https://habr.com/ru/post/441444/
    // Ассоциированный тип нужен для того, чтобы тип определялся только кодом, который реализует трейт
    // Если бы использовался шаблон, тогда тип выводился бы на этапе компиляции из необходимого результата
    //
    // Для данного трейта это не хорошо, так как мы должны в точности знать, что отдаем
    // Пришлось бы каждый раз явно указывать необходимый тип ошибки
    // type ErrorType;
    // async fn get(&self, key: &str) -> Result<UserInfo, Self::ErrorType>;
    // async fn set(&mut self, key: &str, info: UserInfo) -> Result<(), Self::ErrorType>;

    async fn get(&self, key: &str) -> Result<Option<UserInfo>, Box<dyn Error>>;
    async fn set(&self, key: &str, info: UserInfo) -> Result<(), Box<dyn Error>>;
}