use serde::Deserialize;

#[derive(Deserialize)]
pub struct KeyParams {
    pub nonce: String,
    pub tag: String,
}

#[derive(Deserialize)]
pub struct Slot {
    #[serde(rename = "type")]
    pub slot_type: u8,
    pub salt: String,
    pub n: u32,
    pub r: u32,
    pub p: u32,
    pub key: String,
    pub key_params: KeyParams,
}

#[derive(Deserialize)]
pub struct HeaderParams {
    pub nonce: String,
    pub tag: String,
}

#[derive(Deserialize)]
pub struct Header {
    pub slots: Vec<Slot>,
    pub params: HeaderParams,
}

#[derive(Deserialize)]
pub struct Vault {
    pub header: Header,
    pub db: String,
}
