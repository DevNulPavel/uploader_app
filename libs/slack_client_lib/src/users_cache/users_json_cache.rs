use std::{
    collections::{
        hash_map::{
            HashMap
        }
    },
    path::{
        // Path,
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
    }
};
use futures::{
    FutureExt
};
use tokio::{
    fs::{
        File,
        create_dir_all
    },
    io::{
        // AsyncRead,
        AsyncReadExt,
        // AsyncWrite,
        AsyncWriteExt
    },
    task::{
        spawn_blocking
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
use super::{
    users_cache_trait::{
        UsersCache
    }
};

//////////////////////////////////////////
#[derive(Debug)]
pub enum UsersJsonCacheError{
    FileError(io::Error),
    JsonError(serde_json::Error)
}
impl Display for UsersJsonCacheError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl Error for UsersJsonCacheError{
}
impl From<io::Error> for UsersJsonCacheError{
    fn from(err: io::Error) -> Self {
        UsersJsonCacheError::FileError(err)
    }
}
impl From<serde_json::Error> for UsersJsonCacheError{
    fn from(err: serde_json::Error) -> Self {
        UsersJsonCacheError::JsonError(err)
    }
}

//////////////////////////////////////////

pub struct UsersJsonCache{
    path: PathBuf
}
impl UsersJsonCache {
    #[allow(dead_code)]
    pub async fn new(file_path: PathBuf) -> UsersJsonCache {
        // Структурка
        UsersJsonCache{
            path: file_path
        }
    }
    async fn read_file_data(&self) -> Result<HashMap<String, UserInfo>, UsersJsonCacheError> {
        File::open(&self.path)
            .then(|open_res| async move {
                if let Ok(mut file) = open_res {
                    // Читаем файлик в строку
                    let mut data = String::new();
                    file.read_to_string(&mut data).await?;

                    // Парсим асинхронно
                    let result = spawn_blocking(move ||{ serde_json::from_str::<HashMap<String, UserInfo>>(&data) })
                        .await
                        .expect("Json parsing spawn failed")?;

                    Ok(result)
                }else{
                    Err(UsersJsonCacheError::FileError(io::Error::new(io::ErrorKind::NotFound, "Cache file not found")))
                }
            })
            .await
    }

    async fn write_file_data(&self, data: HashMap<String, UserInfo>) -> Result<(), UsersJsonCacheError> {
        // Открываем файлик на запись
        File::create(&self.path)
            .then(|file_result| async move {
                // Открылся ли файл?
                let mut file = file_result?;

                // Перегоняем в Json
                let json_text = spawn_blocking(move ||{ serde_json::to_string(&data) })
                    .await
                    .expect("Convert to json spawn failed")?;
            
                // Пишем в файлик
                file
                    .write_all(json_text.as_bytes())
                    .await?;

                Ok(())
            })
            .await
    }
}
impl Clone for UsersJsonCache{
    fn clone(&self) -> Self {
        UsersJsonCache{
            path: self.path.clone()
        }
    }
}
// #[async_trait(?Send)]
#[async_trait]
impl UsersCache for UsersJsonCache {
    // Ассоциированный тип нужен для конкретного указания возвращаемого значения
    // Иначе нужно было бы использовать шаблон и руками указывать каждый раз тип
    // type ErrorType = JsonCacheError;

    async fn get(&self, key: &str) -> Result<Option<UserInfo>, Box<dyn Error>>{
    // async fn get(&self, key: &str) -> Result<UserInfo, Self::ErrorType>{        
        let res = self
            .read_file_data()
            .await?
            .get(&(key.to_lowercase()))
            .cloned();

        Ok(res)
    }

    async fn set(&self, key: &str, info: UserInfo) -> Result<(), Box<dyn Error>>{
    // async fn set(&mut self, key: &str, info: UserInfo) -> Result<(), Self::ErrorType>{        
        // Создаем папку если надо
        if let Some(folder) = self.path.parent(){
            if !folder.exists() {
                create_dir_all(folder)
                    .await?;
            }
        }

        // Читаем прошлые данные если они есть вообще
        let mut all_data = self
            .read_file_data()
            .await
            .unwrap_or_default();

        // Записываем значение
        all_data.insert(key.to_lowercase(), info);

        // Пишем снова в файлик
        self
            .write_file_data(all_data)
            .await?;

        Ok(())
    }
}


#[cfg(test)]
mod tests{
    #[tokio::test]
    async fn test_json_cache(){
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

        setup_logs();
    
        let test_users: HashMap<String, UserInfo> = generate_test_users();
    
        // Полный путь к файлику
        let cache_file_path = Path::new("users_cache.json");
    
        // Кэш
        let cache = UsersJsonCache::new(cache_file_path.into()).await;
    
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
                .expect("Cache write failed");
    
            assert_eq!(found_val, Some(v));
        }
        
        // Удаляем файлик
        let file_removed = std::fs::remove_file(cache_file_path).is_ok();
        assert!(file_removed);
    }
}