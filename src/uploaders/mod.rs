mod upload_result;
mod app_center;
mod google_drive;
mod google_play;
mod amazon;
mod ios;

// #[cfg(target_family = "unix")]
// mod ssh_unix;

// #[cfg(target_family = "windows")]
mod ssh_windows;

pub use self::{
    upload_result::{
        UploadResult,
        UploadResultData
    },
    app_center::{
        upload_in_app_center
    },
    google_drive::{
        upload_in_google_drive
    },
    google_play::{
        upload_in_google_play
    },
    amazon::{
        upload_in_amazon
    },
    ios::{
        upload_in_ios
    },

    // #[cfg(target_family = "unix")]
    // ssh_unix::{
    //     upload_by_ssh
    // }

    // #[cfg(target_family = "windows")]
    ssh_windows::{
        upload_by_ssh
    }
};