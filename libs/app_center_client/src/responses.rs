use std::{
    collections::{
        hash_map::{
            HashMap
        }
    }
};
use serde::{
    Deserialize
};
use serde_json::{
    Value
};

#[derive(Deserialize, Debug)]
pub struct ReleasesResponse{
    pub id: String,
    pub upload_domain: String,
    pub token: String,
    pub url_encoded_token: String,
    pub package_asset_id: String
}

#[derive(Deserialize, Debug)]
pub struct MetaInfoSetResponse{
    pub id: String,
    // pub status_code: i32,
    pub blob_partitions: i32,
    pub chunk_list: Vec<usize>,
    pub resume_restart: bool,
    pub chunk_size: usize
}

#[derive(Deserialize, Debug)]
pub struct UploadingFinishedResponse{
    pub error_code: String,
    pub location: String,
    pub raw_location: String,
    pub absolute_uri: String,
    pub state: String
}

#[derive(Deserialize, Debug)]
pub struct UploadingFinishedSetStatusResponse{
    pub id: String,
    pub upload_status: String
}

// https://openapi.appcenter.ms/#/distribute/releases_getReleaseUploadStatus
// Размещать поля надо от большего количества полей к меньшему для нормального парсинга
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum UploadingFinishedGetStatusResponse{
    Ready{
        id: String,
        upload_status: String,
        release_distinct_id: u64
    },
    Error{
        id: String,
        upload_status: String,
        error_details: String
    },
    Waiting{
        id: String,
        upload_status: String
    },
    Unknown(HashMap<String, Value>)
}

// https://openapi.appcenter.ms/#/distribute/releases_update
// Размещать поля надо от большего количества полей к меньшему для нормального парсинга
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ReleaseUpdateResponse{
    Failure{
        code: String,
        message: String
    },
    Success{
        destinations: Vec<HashMap<String, String>>
    }
}

// https://openapi.appcenter.ms/#/distribute/releases_getLatestByUser
#[derive(Deserialize, Debug)]
pub struct ReleaseInfoResponse{
    pub app_name: String,
    pub app_display_name: String,
    // app_os: Option<String>,
    // app_icon_url: String,
    // is_external_build: bool,
    // origin: Option<String>,
    pub id: u64,
    pub version: String,
    pub short_version: String,
    pub size: Option<u64>,
    pub min_os: Option<String>,
    pub device_family: Option<String>,
    pub bundle_identifier: Option<String>,
    // fingerprint: Option<String>,
    pub uploaded_at: String,
    pub download_url: Option<String>,
    pub install_url: Option<String>,
    pub enabled: bool,
    // provisioning_profile_type: Option<String>,
    // provisioning_profile_expiry_date: Option<String>,
    // provisioning_profile_name: Option<String>,
    // is_provisioning_profile_syncing: Option<bool>,
    
    // package_hashes: 
    //  [ '5e86dbd11db3562b7147ede0dcc8dcbd80732a4e8e76a921c2714583834bf1a0' ],
    // destinations: []
}