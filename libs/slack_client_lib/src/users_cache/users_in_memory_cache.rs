use std::{
    collections::{
        hash_map::{
            HashMap
        }
    },
    error::{
        Error
    },
    fmt::{
        Display,
        Formatter,
        self
    },
    sync::{
        Arc
    }
};
use async_trait::{
    async_trait
};
use tokio::{
    sync::{
        Mutex
    }
};
use crate::{
    search_by_name::{
        UserInfo
    }
};
use super::{
    users_cache_trait::{
        UsersCache
    }
};

//////////////////////////////////////////

#[derive(Debug)]
pub enum UsersInMemoryCacheError{
}
impl Display for UsersInMemoryCacheError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl Error for UsersInMemoryCacheError{
}

//////////////////////////////////////////

pub struct UsersInMemoryCache{
    data: Arc<Mutex<HashMap<String, UserInfo>>>
}
impl UsersInMemoryCache {
    #[allow(dead_code)]
    pub fn new() -> UsersInMemoryCache {
        // Структурка
        UsersInMemoryCache{
            data: Default::default()
        }
    }
}
impl Clone for UsersInMemoryCache{
    fn clone(&self) -> Self {
        UsersInMemoryCache{
            data: self.data.clone()
        }
    }
}
// #[async_trait(?Send)]
#[async_trait]
impl UsersCache for UsersInMemoryCache {
    // type ErrorType = InMemoryCacheError;

    // async fn get(&self, key: &str) -> Result<UserInfo, Self::ErrorType>{
    async fn get(&self, key: &str) -> Result<Option<UserInfo>, Box<dyn Error>>{
        let data = self.data.lock().await;
        let res = data
            .get(&(key.to_lowercase()))
            .cloned();
            // .ok_or(InMemoryCacheError::NotFound)
            // .ok_or(Box::new(InMemoryCacheError::NotFound))?;
        Ok(res)
    }
    // async fn set(&mut self, key: &str, info: UserInfo) -> Result<(), Self::ErrorType>{
    async fn set(&self, key: &str, info: UserInfo) -> Result<(), Box<dyn Error>>{
        let mut data = self.data.lock().await;
        data.insert(key.to_lowercase(), info);
        Ok(())
    }
}