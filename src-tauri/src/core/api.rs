use std::time::Duration;

use futures::lock::Mutex;
use once_cell::sync::Lazy;

use super::json::backend_config::{BanStatus, GlobalConfig, WhitelistStatus};

/// Base URL of the HyClyint backend. The mod talks to the same host.
pub const BACKEND_URL: &str = "https://hapi.askaloe.in/";

/// Static fallback config, still hosted on GitHub. Used when the backend is
/// unreachable so an outage cannot stop people from launching the game.
pub const STATIC_CONFIG_URL: &str =
    "https://raw.githubusercontent.com/HyClyint/launcher-data/main/config.json";

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

/// The pre-backend config file. It has no whitelist and no featured servers.
pub async fn fetch_static_config() -> Result<GlobalConfig, String> {
    json::<GlobalConfig>(STATIC_CONFIG_URL).await
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

pub async fn store_global_config(config: GlobalConfig) {
    *CACHED_CONFIG.lock().await = Some(config);
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
