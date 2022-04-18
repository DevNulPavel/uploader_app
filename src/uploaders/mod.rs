mod amazon;
mod app_center;
mod google_drive;
mod google_play;
mod ios;
mod upload_result;

#[cfg(target_family = "unix")]
mod ssh_unix;

#[cfg(target_family = "windows")]
mod ssh_windows;

pub use self::{
    amazon::upload_in_amazon,
    app_center::upload_in_app_center,
    google_drive::upload_in_google_drive,
    google_play::upload_in_google_play,
    ios::upload_in_ios,
    upload_result::{UploadResult, UploadResultData},
};

#[cfg(target_family = "unix")]
pub use self::ssh_unix::upload_by_ssh;

#[cfg(target_family = "windows")]
pub use self::ssh_windows::upload_by_ssh;
