use std::time::Duration;

use futures::lock::Mutex;
use once_cell::sync::Lazy;

use super::json::backend_config::{BanStatus, GlobalConfig, WhitelistStatus};

/// Base URL of the HyClyint backend. The mod talks to the same host.
pub const BACKEND_URL: &str = "https://hapi.askaloe.in/";

const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

static CACHED_CONFIG: Lazy<Mutex<Option<GlobalConfig>>> = Lazy::new(|| Mutex::new(None));

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

async fn json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T, String> {
    let response = client()
        .get(url)
        .send()
        .await
        .map_err(|error| format!("{}: {}", url, error))?;

    if !response.status().is_success() {
        return Err(format!("{} returned HTTP {}", url, response.status()));
    }

    response
        .json::<T>()
        .await
        .map_err(|error| format!("{}: {}", url, error))
}

/// The newest published HyClyint build (`/downloads/latest.json`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LatestBuild {
    pub version: String,
    pub minecraft_version: String,
    #[serde(default)]
    pub channel: String,
    /// Jar file name, e.g. `hyclyint-0.1.2-BETA.jar`.
    #[serde(default)]
    pub file: String,
    /// Site-relative jar path, e.g. `/downloads/mods/hyclyint-0.1.2-BETA.jar`.
    #[serde(default)]
    pub url: String,
    /// Hex SHA-256 of the jar, used to skip unchanged downloads and to verify new ones.
    #[serde(default)]
    pub sha256: String,
}

pub async fn fetch_latest_build() -> Result<LatestBuild, String> {
    let build = json::<LatestBuild>(&format!("{}downloads/latest.json", BACKEND_URL)).await?;

    // Becomes a folder name and a version id, so keep it to plain version text.
    let plain = !build.minecraft_version.is_empty()
        && build
            .minecraft_version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
    if !plain {
        return Err("latest.json has an invalid minecraft_version".to_string());
    }

    Ok(build)
}

/// Minecraft versions the backend allows, newest first.
pub async fn fetch_versions() -> Result<Vec<String>, String> {
    #[derive(serde::Deserialize)]
    struct VersionsResponse {
        versions: Vec<String>,
    }

    json::<VersionsResponse>(&format!("{}api/versions", BACKEND_URL))
        .await
        .map(|response| response.versions)
}

pub async fn fetch_global_config() -> Result<GlobalConfig, String> {
    json::<GlobalConfig>(&format!("{}api/globalConfig", BACKEND_URL)).await
}

/// Returns the remote config, fetching it once and caching it afterwards.
pub async fn get_global_config(refresh: bool) -> Result<GlobalConfig, String> {
    let mut cached = CACHED_CONFIG.lock().await;

    if !refresh {
        if let Some(config) = cached.as_ref() {
            return Ok(config.clone());
        }
    }

    let config = fetch_global_config().await?;
    *cached = Some(config.clone());

    Ok(config)
}

pub async fn is_whitelisted(username: &str) -> Result<WhitelistStatus, String> {
    let response = client()
        .get(format!("{}api/whitelist", BACKEND_URL))
        .query(&[("username", username)])
        .send()
        .await
        .map_err(|error| format!("{}: {}", BACKEND_URL, error))?;

    if !response.status().is_success() {
        return Err(format!(
            "{}api/whitelist returned HTTP {}",
            BACKEND_URL,
            response.status()
        ));
    }

    response
        .json::<WhitelistStatus>()
        .await
        .map_err(|error| error.to_string())
}

pub async fn is_banned(username: &str, hwid: Option<&str>) -> Result<BanStatus, String> {
    let mut query: Vec<(&str, &str)> = vec![("username", username)];

    if let Some(hwid) = hwid {
        query.push(("hwid", hwid));
    }

    let response = client()
        .get(format!("{}api/isBanned", BACKEND_URL))
        .query(&query)
        .send()
        .await
        .map_err(|error| format!("{}: {}", BACKEND_URL, error))?;

    if !response.status().is_success() {
        return Err(format!(
            "{}api/isBanned returned HTTP {}",
            BACKEND_URL,
            response.status()
        ));
    }

    response
        .json::<BanStatus>()
        .await
        .map_err(|error| error.to_string())
}
