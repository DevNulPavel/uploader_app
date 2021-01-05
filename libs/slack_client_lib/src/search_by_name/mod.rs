mod user_info;
mod iter_users_search;
mod full_name_search;

pub use self::{
    full_name_search::{
        search_by_fullname
    },
    iter_users_search::{
        iter_by_slack_users,
    },
    user_info::{
        UserInfo
    }
};