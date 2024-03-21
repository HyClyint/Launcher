use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Root {
    pub objects: HashMap<String, Object>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Object {
    pub hash: String,
    pub size: i64,
}
