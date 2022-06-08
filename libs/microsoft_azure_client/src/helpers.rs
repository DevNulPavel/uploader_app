use crate::error::MicrosoftAzureError;
use std::path::Path;
use log::debug;

/// Данная функция проверяет, что расширение файлика совпадает с указанным
pub fn check_file_extention(path: &Path, required_extention: &str) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| {
            if ext.eq(required_extention) {
                Some(())
            } else {
                None
            }
        })
        .is_some()
}

/// Ищем внутри архива файлики .appx / .appxupload
pub fn find_appx_filenames_in_zip(
    zip_file_path: &Path,
) -> Result<Vec<String>, MicrosoftAzureError> {
    // Проверяем расширение данного файлика
    if !check_file_extention(zip_file_path, "zip") {
        return Err(MicrosoftAzureError::InvalidUploadFileExtention);
    }

    let zip = zip::ZipArchive::new(std::fs::File::open(&zip_file_path)?)?;
    let filenames_in_zip: Vec<_> = zip
        .file_names()
        .filter(|full_path_str| {
            let file_name = std::path::Path::new(full_path_str)
                .file_name()
                .and_then(|f| f.to_str());
            if let Some(file_name) = file_name {
                !file_name.starts_with('.')
                    && (file_name.ends_with(".appx")
                        || file_name.ends_with(".appxupload")
                        || file_name.ends_with(".msixupload")
                        || file_name.ends_with(".msix"))
            } else {
                false
            }
        })
        .map(|v| v.to_owned())
        .collect();

    debug!("Microsoft Azure: filenames in zip {:?}", filenames_in_zip);

    if filenames_in_zip.is_empty() {
        return Err(MicrosoftAzureError::NoAppxFilesInZip);
    }
    Ok(filenames_in_zip)
}
