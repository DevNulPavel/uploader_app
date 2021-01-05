mod upload_result;
mod app_center;
mod google_drive;
mod google_play;
mod amazon;
mod ios;

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
    }
};