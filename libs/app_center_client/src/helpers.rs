use std::{
    path::{
        Path
    }
};

pub fn upload_content_type_for_file(path: &Path) -> &'static str {
    const DEFAULT_CONTENT_TYPE: &str = "application/octet-stream";

    let extention = path.extension();
    match extention {
        Some(ext) => {
            let ext = match ext.to_str() {
                Some(ext) => ext,
                None => return DEFAULT_CONTENT_TYPE
            };

            match ext {
                "apk" | "aab" => "application/vnd.android.package-archive",
                "ipa" => DEFAULT_CONTENT_TYPE,
                "appx" => "application/x-appx",
                "appxbundle" => "application/x-appxbundle",
                "appxupload" => "application/x-appxupload",
                "appxsym" => "application/x-appxupload",
                _ => DEFAULT_CONTENT_TYPE
            }
        },
        None => {
            DEFAULT_CONTENT_TYPE
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    macro_rules! assert_eq_type {
        ($path:literal, $type:literal) => {
            assert_eq!(upload_content_type_for_file(Path::new($path)), $type);    
        };
    }

    #[test]
    fn test_content_type_for_path(){
        assert_eq_type!("test.txt", "application/octet-stream");
        assert_eq_type!("test.ipa", "application/octet-stream");
        assert_eq_type!("test.apk", "application/vnd.android.package-archive");
        assert_eq_type!("test.appx", "application/x-appx");
        assert_eq_type!("test.appxupload", "application/x-appxupload");
        assert_eq_type!("test.appxsym", "application/x-appxupload");
    }
}