mod amazon;
mod app_center;
mod google_drive;
mod google_play;
mod ios;
mod upload_result;

#[cfg(target_family = "unix")]
#[path = "ssh_unix.rs"]
mod ssh;

#[cfg(target_family = "windows")]
#[path = "ssh_windows.rs"]
mod ssh;

pub use self::{
    amazon::upload_in_amazon,
    app_center::upload_in_app_center,
    google_drive::upload_in_google_drive,
    google_play::upload_in_google_play,
    ios::upload_in_ios,
    ssh::upload_by_ssh,
    upload_result::{UploadResult, UploadResultData},
};