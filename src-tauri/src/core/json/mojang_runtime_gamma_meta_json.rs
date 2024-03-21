use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Root {
    pub files: HashMap<String, File>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct File {
    #[serde(rename = "type")]
    pub root_type: Option<String>,
    pub downloads: Option<Downloads>,
    pub executable: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Downloads {
    pub lzma: Option<Download>,
    pub raw: Option<Download>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Download {
    pub sha1: Option<String>,
    pub size: Option<i64>,
    pub url: Option<String>,
}
