use serde::{
    Deserialize,
    Serialize
};

// Создаем структурки, в которых будут нужные значения
#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature="sql", derive(sqlx::FromRow))]
pub struct UserInfo {
    pub id: String,
    pub name: String,
    pub real_name: Option<String>
}

impl PartialEq<UserInfo> for UserInfo {
    fn eq(&self, other: &UserInfo) -> bool {
        self.id == other.id &&
        self.name == other.name &&
        self.real_name == other.real_name
    }
}