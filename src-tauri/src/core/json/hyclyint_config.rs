use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    #[serde(rename = "fabric_version")]
    pub fabric_version: String,
    #[serde(rename = "hyclyint_version")]
    pub hyclyint_version: String,
    #[serde(rename = "required_mods")]
    pub required_mods: Vec<RequiredMod>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequiredMod {
    pub name: String,
    pub version: String,
}
