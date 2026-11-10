use serde::Deserialize;
use serde::Serialize;

/// Payload of `GET /api/globalConfig`.
///
/// The version fields keep the snake_case names of the static `config.json`
/// this launcher used to read from GitHub, so the backend can serve the same
/// contract and the fallback stays byte-compatible.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub whitelist: bool,
    #[serde(rename = "featuredServers", default)]
    pub featured_servers: Vec<FeaturedServer>,
    #[serde(default)]
    pub fabric_version: Option<String>,
    #[serde(default)]
    pub hyclyint_version: Option<String>,
    #[serde(default)]
    pub hyclyint_launcher_version: Option<String>,
    #[serde(default)]
    pub required_mods: Vec<RequiredMod>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeaturedServer {
    pub name: String,
    pub ip: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequiredMod {
    pub name: String,
    pub version: String,
}

/// Payload of `GET /api/whitelist`.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhitelistStatus {
    #[serde(default)]
    pub whitelisted: bool,
    #[serde(rename = "whitelistEnabled", default)]
    pub whitelist_enabled: Option<bool>,
}

/// Payload of `GET /api/isBanned`.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BanStatus {
    #[serde(default)]
    pub banned: bool,
    #[serde(default)]
    pub reason: Option<String>,
}
