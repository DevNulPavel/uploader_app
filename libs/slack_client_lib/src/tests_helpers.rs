use std::{
    collections::{
        HashMap
    }
};
use crate::{
    search_by_name::{
        UserInfo
    }
};

pub fn setup_logs() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(||{
        if std::env::var("RUST_LOG").is_err(){
            std::env::set_var("RUST_LOG", "trace");
        }
        env_logger::builder().is_test(true).init();
    })
}

pub fn generate_test_users() -> HashMap<String, UserInfo>{
    let mut users_cache: HashMap<String, UserInfo> = HashMap::new();
    users_cache.insert(String::from("pershov"), UserInfo{
        id: String::from("asdasd"),
        name: String::from("pershov"),
        real_name: Some(String::from("Pavel Ershov"))
    });
    users_cache.insert(String::from("Pavel Ershov"), UserInfo{
        id: String::from("asdasd"),
        name: String::from("pershov"),
        real_name: Some(String::from("Pavel Ershov"))
    });
    users_cache.insert(String::from("Ershov Pavel"), UserInfo{
        id: String::from("asdasd"),
        name: String::from("pershov"),
        real_name: Some(String::from("Pavel Ershov"))
    });
    users_cache.insert(String::from("Pavel Ivanov"), UserInfo{
        id: String::from("ggggg"),
        name: String::from("pivanov"),
        real_name: Some(String::from("Pavel Ivanov"))
    });
    users_cache.insert(String::from("pivanov"), UserInfo{
        id: String::from("ggggg"),
        name: String::from("pivanov"),
        real_name: Some(String::from("Pavel Ivanov"))
    });
    users_cache.insert(String::from("Ivanov Pavel"), UserInfo{
        id: String::from("ggggg"),
        name: String::from("pivanov"),
        real_name: Some(String::from("Pavel Ivanov"))
    });
    users_cache.insert(String::from("Test Name"), UserInfo{
        id: String::from("gfdg"),
        name: String::from("tname"),
        real_name: Some(String::from("Test Name"))
    });
    users_cache.insert(String::from("Name Test"), UserInfo{
        id: String::from("fgdfg"),
        name: String::from("tname"),
        real_name: Some(String::from("Test Name"))
    });
    users_cache.insert(String::from("cake"), UserInfo{
        id: String::from("gdfgdfg"),
        name: String::from("cake"),
        real_name: None
    });
    
    users_cache
}