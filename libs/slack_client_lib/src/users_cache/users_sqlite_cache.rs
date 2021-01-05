use std::{
    path::{
        PathBuf
    },
    error::{
        Error
    },
    fmt::{
        Display,
        Formatter,
        self
    },
    io::{
        self
    },
    sync::{
        Arc
    }
};
use log::{
    debug
};
// use futures::{
//     FutureExt,
//     StreamExt,
//     TryFutureExt,
//     TryStreamExt
// };
use tokio::{
    fs::{
        File,
        create_dir_all
    },
    // io::{
        // AsyncRead,
        // AsyncReadExt,
        // AsyncWrite,
        // AsyncWriteExt
    // },
    sync::{
        Mutex
    }
};
use async_trait::{
    async_trait
};
use sqlx::{
    sqlite::{
        SqliteConnection,
        // Sqlite,
        // SqliteRow,
        // SqliteColumn,
        // SqliteError
    },
    Connection,
    // Row,
    // Column,
    // query,
    // query_as
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
pub enum UsersSqliteCacheError{
    IOError(io::Error),
    PathConvertError,
    SqliteError(sqlx::Error)
}
impl Display for UsersSqliteCacheError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl Error for UsersSqliteCacheError{
}
impl From<io::Error> for UsersSqliteCacheError{
    fn from(err: io::Error) -> Self {
        UsersSqliteCacheError::IOError(err)
    }
}
impl From<sqlx::Error> for UsersSqliteCacheError{
    fn from(err: sqlx::Error) -> Self {
        UsersSqliteCacheError::SqliteError(err)
    }
}

//////////////////////////////////////////

pub struct UsersSqliteCache{
    client: Arc<Mutex<SqliteConnection>>
}
impl UsersSqliteCache {
    #[allow(dead_code)]
    pub async fn new(file_path: PathBuf) -> Result<UsersSqliteCache, UsersSqliteCacheError> {
        if !file_path.exists(){
            if let Some(folder) = file_path.parent(){
                create_dir_all(folder).await?;
            }
            File::create(&file_path).await?;
        }

        let path_str = file_path.to_str().ok_or(UsersSqliteCacheError::PathConvertError)?;

        debug!("SQLite path: {}", path_str);

        // Создаем соединение
        let mut sqlite_client: SqliteConnection = SqliteConnection::connect(&format!("file:{}", path_str)).await?;

        // Создание структуры базы если надо
        // db_id integer PRIMARY KEY AUTOINCREMENT,
        const SQL: &str = r#"
            CREATE TABLE IF NOT EXISTS found_users (
                key varchar(64) PRIMARY KEY NOT NULL,
                id varchar(64) NOT NULL,
                name varchar(64) NOT NULL,
                real_name varchar(64) NULL
            )    
        "#;
        sqlx::query(SQL)
            .execute(&mut sqlite_client)
            .await?;

        // Структурка
        Ok(UsersSqliteCache{
            client: Arc::new(Mutex::new(sqlite_client))
        })
    }
}
impl Clone for UsersSqliteCache{
    fn clone(&self) -> Self {
        UsersSqliteCache{
            client: self.client.clone()
        }
    }
}
// #[async_trait(?Send)]
#[async_trait]
impl UsersCache for UsersSqliteCache {
    // Ассоциированный тип нужен для конкретного указания возвращаемого значения
    // Иначе нужно было бы использовать шаблон и руками указывать каждый раз тип
    // type ErrorType = SqliteCacheError;

    // async fn get(&self, key: &str) -> Result<UserInfo, Self::ErrorType>{
    async fn get(&self, key: &str) -> Result<Option<UserInfo>, Box<dyn Error>>{
        let sql = format!(r#"
            SELECT *
            FROM found_users 
            WHERE key = '{}'
        "#, key.to_lowercase());

        let mut client = self.client.lock().await;

        let fetch_result = sqlx::query_as(&sql)
            .fetch_one(&mut (*client))
            .await;
        match fetch_result{
            Ok(result) => {
                return Ok(Some(result));
            },
            Err(err) => {
                return Err(Box::new(UsersSqliteCacheError::SqliteError(err)));
            }
        }
    }

    // async fn set(&mut self, key: &str, info: UserInfo) -> Result<(), Self::ErrorType>{
    async fn set(&self, key: &str, info: UserInfo) -> Result<(), Box<dyn Error>>{
        let real_name = info
            .real_name
            .map_or("NULL".to_owned(), |val|{
                format!("'{}'", val)
            });
        let sql = format!(r#"
            REPLACE INTO found_users (key, id, name, real_name)
            VALUES ('{}', '{}', '{}', {})
        "#, key.to_lowercase(), info.id, info.name, real_name);
        
        let mut client = self.client.lock().await;

        sqlx::query(&sql)
            .execute(&mut (*client))
            .await?;

        Ok(())
    }
}


#[cfg(test)]
mod tests{
    use std::{
        path::{
            Path
        },
        collections::{
            HashMap
        }
    };
    use crate::{
        tests_helpers::{
            generate_test_users,
            setup_logs
        },
        search_by_name::{
            UserInfo
        }
    };
    use super::{
        *
    };

    #[tokio::test(threaded_scheduler)]
    async fn test_sqlite_cache(){
        setup_logs();

        let test_users: HashMap<String, UserInfo> = generate_test_users();

        // Полный путь к файлику
        let cache_file_path = Path::new("users_cache.sqlite");

        // Кэш
        let cache = UsersSqliteCache::new(cache_file_path.into())
            .await
            .expect("Sqlite cache create failed");

        // Сохраняем
        for (k,v) in test_users.iter() {
            cache
                .set(k, v.clone())
                .await
                .expect("Cache write failed");
        }

        // Читаем
        for (k,v) in test_users {
            let found_val = cache
                .get(&k)
                .await
                .expect("Cache read failed");

            assert_eq!(found_val, Some(v));
        }
        
        // Удаляем файлик
        let file_removed = std::fs::remove_file(cache_file_path).is_ok();
        assert!(file_removed);
    }   
}