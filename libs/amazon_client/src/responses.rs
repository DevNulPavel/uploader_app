use serde::{
    Deserialize
};
use serde_json::{
    Value
};


#[derive(Deserialize, Debug)]
pub struct AmazonTokenResponse{
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
    pub expires_in: u64
}

#[derive(Deserialize, Debug)]
pub struct AmazonEditData{
    pub id: String,
    pub status: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum AmazonEditRespone{
    Exists(AmazonEditData),
    Empty{}
}

// https://developer.amazon.com/docs/app-submission-api/appsubapi-endpoints.html#/Edits.apks/get_2
#[derive(Deserialize, Debug)]
pub struct ApkInfoResponse{
    //pub versionCode: u64,
    pub id: String,
    pub name: String
}


// #[serde(rename = "versionCode")]

//////////////////////////////////////////////////////////////////////

/// Специальный шаблонный тип, чтобы можно было парсить возвращаемые ошибки в ответах
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum DataOrErrorResponse<D>{
    Ok(D),
    Err(ErrorResponseValue)
}
impl<D> DataOrErrorResponse<D> {
    pub fn into_result(self) -> Result<D, ErrorResponseValue> {
        match self {
            DataOrErrorResponse::Ok(ok) => Ok(ok),
            DataOrErrorResponse::Err(err) => Err(err),
        }
    }
}

/// Тип ошибки, в который мы можем парсить наши данные
#[derive(Deserialize, Debug)]
pub struct ErrorResponseValue{
    #[serde(rename = "httpCode")]
    http_code: u32,
    message: Option<String>,
    errors: Option<Vec<Value>>,
    // #[serde(flatten)]
    // other: HashMap<String, Value>
}