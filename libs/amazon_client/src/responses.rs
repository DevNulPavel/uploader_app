// use std::{
//     collections::{
//         HashMap
//     }
// };
use serde::{
    Deserialize
};
// use serde_json::{
//     Value
// };


#[derive(Deserialize, Debug)]
pub struct AmazonTokenResponse{
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
    pub expires_in: u64
}

#[derive(Deserialize, Debug)]
pub struct AmazonEditRespone{
    pub id: String,
    pub status: String,
}


// https://developer.amazon.com/docs/app-submission-api/appsubapi-endpoints.html#/Edits.apks/get_2
#[derive(Deserialize, Debug)]
pub struct ApkInfoResponse{
    //pub versionCode: u64,
    pub id: String,
    pub name: String
}

// #[derive(Deserialize, Debug)]
// #[serde(untagged)]
// pub enum AmazonEditResponeDebug{
//     Ok(AmazonEditRespone),
//     Err(HashMap<String, Value>)
// }

// #[serde(rename = "versionCode")]
