use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub id: String,
    pub inherits_from: String,
    pub release_time: String,
    pub time: String,
    #[serde(rename = "type")]
    pub root_type: String,
    pub main_class: String,
    pub arguments: Arguments,
    pub libraries: Vec<Library>,
}

#[derive(Serialize, Deserialize)]
pub struct Arguments {
    pub game: Vec<Option<serde_json::Value>>,
    pub jvm: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Library {
    pub name: String,
    pub url: String,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub sha512: Option<String>,
    pub size: Option<i64>,
}
