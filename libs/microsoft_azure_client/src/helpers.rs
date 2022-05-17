use std::{
    path::{
        Path
    }
};

/// Данная функция проверяет, что расширение файлика совпадает с указанным
pub fn check_file_extention(path: &Path, required_extention: &str) -> bool {
    path
        .extension()
        .and_then(|ext|{
            ext.to_str()
        })
        .and_then(|ext|{
            if ext.eq(required_extention) {
                Some(())
            }else{
                None
            }
        })
        .is_some()
}
