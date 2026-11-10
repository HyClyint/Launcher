use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::config;
use crate::core::api;

// ---------------------------------------------------------------------------
// Microsoft / Xbox / Minecraft authentication
//
// This used to come from the `azalea` crate, which dragged a whole Minecraft
// client (~500 crates, an unstable `min_specialization` feature and a 2024
// nightly toolchain) along just for these four HTTP calls.
// ---------------------------------------------------------------------------

/// OAuth client id of the "Nintendo Switch" Minecraft app. Using this one means
/// the device-code flow also works for accounts owned by minors.
const CLIENT_ID: &str = "00000000441cc96b";
const MSA_SCOPE: &str = "service::user.auth.xboxlive.com::MBI_SSL";

const MS_DEVICE_CODE_URL: &str = "https://login.live.com/oauth20_connect.srf";
const MS_TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";
const XBL_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MC_LOGIN_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MC_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

#[derive(Debug, Deserialize)]
pub struct DeviceCodeResponse {
    pub user_code: String,
    pub device_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    #[serde(default = "default_interval")]
    pub interval: u64,
}

fn default_interval() -> u64 {
    5
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct AccessTokenResponse {
    pub token_type: String,
    pub expires_in: u64,
    pub scope: String,
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct XboxLiveAuthResponse {
    token: String,
    display_claims: HashMap<String, Vec<HashMap<String, String>>>,
}

#[derive(Debug)]
pub struct XboxLiveAuth {
    pub token: String,
    pub user_hash: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct MinecraftAuthResponse {
    pub username: String,
    pub roles: Vec<String>,
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[derive(Debug)]
pub struct MinecraftTokenResponse {
    pub minecraft_access_token: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ProfileResponse {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub skins: Vec<serde_json::Value>,
    #[serde(default)]
    pub capes: Vec<serde_json::Value>,
}

/// Decode a response body, keeping the raw text for error messages because
/// Microsoft/Xbox put their failure details in `error` / `XErr` fields.
async fn decode<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
    what: &str,
) -> Result<T, String> {
    let status = response.status();

    let body = response
        .text()
        .await
        .map_err(|e| format!("{what}: failed to read response body: {e}"))?;

    if !status.is_success() {
        return Err(format!("{what} failed ({status}): {body}"));
    }

    serde_json::from_str(&body).map_err(|e| format!("{what}: unexpected response ({e}): {body}"))
}

/// Ask Microsoft for a device code. The user opens `verification_uri` and types
/// `user_code`, then `get_ms_auth_token` completes the flow.
pub async fn get_ms_link_code(client: &reqwest::Client) -> Result<DeviceCodeResponse, String> {
    let response = client
        .post(MS_DEVICE_CODE_URL)
        .form(&[
            ("scope", MSA_SCOPE),
            ("client_id", CLIENT_ID),
            ("response_type", "device_code"),
        ])
        .send()
        .await
        .map_err(|e| format!("Microsoft device code request failed: {e}"))?;

    decode(response, "Microsoft device code request").await
}

/// Poll Microsoft until the user finished signing in with the device code.
pub async fn get_ms_auth_token(
    client: &reqwest::Client,
    res: DeviceCodeResponse,
) -> Result<AccessTokenResponse, String> {
    let url = format!("{MS_TOKEN_URL}?client_id={CLIENT_ID}");
    let deadline = Instant::now() + Duration::from_secs(res.expires_in);
    let mut interval = res.interval.max(1);

    while Instant::now() < deadline {
        tokio::time::sleep(Duration::from_secs(interval)).await;

        let response = client
            .post(&url)
            .form(&[
                ("client_id", CLIENT_ID),
                ("device_code", res.device_code.as_str()),
                (
                    "grant_type",
                    "urn:ietf:params:oauth:grant-type:device_code",
                ),
            ])
            .send()
            .await
            .map_err(|e| format!("Microsoft token request failed: {e}"))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("Microsoft token response: {e}"))?;

        if status.is_success() {
            return serde_json::from_str(&body)
                .map_err(|e| format!("Microsoft token: unexpected response ({e}): {body}"));
        }

        // The user simply hasn't finished signing in yet.
        if body.contains("authorization_pending") {
            continue;
        }

        // We're polling too fast; Microsoft asks us to back off.
        if body.contains("slow_down") {
            interval += 5;
            continue;
        }

        return Err(format!(
            "Microsoft sign-in failed ({status}): {}",
            describe_ms_error(&body)
        ));
    }

    Err("Timed out waiting for Microsoft sign-in".to_string())
}

fn describe_ms_error(body: &str) -> String {
    #[derive(Deserialize)]
    struct MsError {
        error: Option<String>,
        error_description: Option<String>,
    }

    match serde_json::from_str::<MsError>(body) {
        Ok(err) => match (err.error, err.error_description) {
            (Some(code), Some(description)) => format!("{code}: {description}"),
            (Some(code), None) => code,
            _ => body.to_string(),
        },
        Err(_) => body.to_string(),
    }
}

async fn auth_with_xbox_live(
    client: &reqwest::Client,
    msa_access_token: &str,
) -> Result<XboxLiveAuth, String> {
    let response = client
        .post(XBL_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("x-xbl-contract-version", "1")
        .json(&serde_json::json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": msa_access_token
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        }))
        .send()
        .await
        .map_err(|e| format!("Xbox Live auth request failed: {e}"))?;

    let res: XboxLiveAuthResponse = decode(response, "Xbox Live auth").await?;

    let user_hash = res
        .display_claims
        .get("xui")
        .and_then(|claims| claims.first())
        .and_then(|claim| claim.get("uhs"))
        .cloned()
        .ok_or_else(|| "Xbox Live auth: response did not contain a user hash".to_string())?;

    Ok(XboxLiveAuth {
        token: res.token,
        user_hash,
    })
}

async fn obtain_xsts_for_minecraft(
    client: &reqwest::Client,
    xbl_auth_token: &str,
) -> Result<String, String> {
    let response = client
        .post(XSTS_AUTH_URL)
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [xbl_auth_token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        }))
        .send()
        .await
        .map_err(|e| format!("XSTS auth request failed: {e}"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("XSTS auth response: {e}"))?;

    if !status.is_success() {
        return Err(format!(
            "XSTS auth failed ({status}): {}",
            describe_xsts_error(&body)
        ));
    }

    let parsed: XboxLiveAuthResponse = serde_json::from_str(&body)
        .map_err(|e| format!("XSTS auth: unexpected response ({e}): {body}"))?;

    Ok(parsed.token)
}

fn describe_xsts_error(body: &str) -> String {
    #[derive(Deserialize)]
    struct XstsError {
        #[serde(rename = "XErr")]
        x_err: Option<i64>,
        #[serde(rename = "Message")]
        message: Option<String>,
    }

    match serde_json::from_str::<XstsError>(body) {
        Ok(err) => {
            let hint = match err.x_err {
                Some(2148916233) => " (this Microsoft account has no Xbox profile)",
                Some(2148916238) => " (this is a child account and must be added to a family)",
                Some(2148916235) => " (Xbox Live is not available in this country)",
                _ => "",
            };

            match (err.x_err, err.message) {
                (Some(code), Some(message)) => format!("{message} (XErr {code}){hint}"),
                (Some(code), None) => format!("XErr {code}{hint}"),
                (None, Some(message)) => message,
                (None, None) => body.to_string(),
            }
        }
        Err(_) => body.to_string(),
    }
}

async fn auth_with_minecraft(
    client: &reqwest::Client,
    user_hash: &str,
    xsts_token: &str,
) -> Result<MinecraftAuthResponse, String> {
    let response = client
        .post(MC_LOGIN_URL)
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "identityToken": format!("XBL3.0 x={user_hash};{xsts_token}")
        }))
        .send()
        .await
        .map_err(|e| format!("Minecraft auth request failed: {e}"))?;

    decode(response, "Minecraft auth").await
}

/// Trade a Microsoft access token for a Minecraft access token.
pub async fn request_minecraft_token(
    client: &reqwest::Client,
    msa_access_token: &str,
) -> Result<MinecraftTokenResponse, String> {
    let xbl = auth_with_xbox_live(client, msa_access_token).await?;
    let xsts = obtain_xsts_for_minecraft(client, &xbl.token).await?;
    let mca = auth_with_minecraft(client, &xbl.user_hash, &xsts).await?;

    Ok(MinecraftTokenResponse {
        minecraft_access_token: mca.access_token,
    })
}

pub async fn request_profile(
    client: &reqwest::Client,
    minecraft_access_token: &str,
) -> Result<ProfileResponse, String> {
    let response = client
        .get(MC_PROFILE_URL)
        .header("Authorization", format!("Bearer {minecraft_access_token}"))
        .send()
        .await
        .map_err(|e| format!("Minecraft profile request failed: {e}"))?;

    decode(response, "Minecraft profile request").await
}

// ---------------------------------------------------------------------------
// Launcher commands
// ---------------------------------------------------------------------------

/// Blocks banned players and, while the whitelist is enabled, players that are
/// not on it.
///
/// Both checks fail open: if the backend cannot be reached the player is let in
/// rather than locked out of a launcher that otherwise works offline. A client
/// side gate is a convenience, not a security boundary - real enforcement has
/// to happen on the server.
async fn enforce_access(username: &str) -> Result<(), String> {
    match api::is_banned(username, None).await {
        Ok(status) if status.banned => {
            return Err(match status.reason.filter(|reason| !reason.is_empty()) {
                Some(reason) => format!("You are banned from HyClyint. Reason: {}", reason),
                None => "You are banned from HyClyint.".to_string(),
            });
        }
        Ok(_) => {}
        Err(error) => println!("Could not check ban status for {}: {}", username, error),
    }

    let whitelist_enabled = match api::get_global_config(false).await {
        Ok(config) => config.whitelist,
        Err(error) => {
            println!("Could not load the global config: {}", error);
            return Ok(());
        }
    };

    if !whitelist_enabled {
        return Ok(());
    }

    match api::is_whitelisted(username).await {
        Ok(status) if status.whitelisted => Ok(()),
        Ok(_) => Err(format!(
            "{} is not whitelisted on HyClyint. Ask a staff member for access.",
            username
        )),
        Err(error) => {
            println!("Could not check whitelist status for {}: {}", username, error);
            Ok(())
        }
    }
}

#[tauri::command]
pub async fn login_offline(app: AppHandle, username: &str) -> Result<(), String> {
    let username = username.trim();

    if username.is_empty() {
        return Err("A username is required".to_string());
    }

    if let Err(reason) = enforce_access(username).await {
        let _ = app.emit_all("auth-error", reason.clone());
        return Err(reason);
    }

    let mut config = config::SHARED_CONFIG.lock().await;

    config.is_logged_in = true;
    config.username = username.to_string();
    config.uuid = Uuid::new_v4().to_string();

    app.emit_all("config-updated", Some(config.clone()))
        .unwrap();

    config.save_config().unwrap();

    Ok(())
}

#[tauri::command]
pub async fn login_microsoft(app: AppHandle) -> Result<(), String> {
    let client = reqwest::Client::new();
    let res = get_ms_link_code(&client).await?;

    app.emit_all(
        "microsoft-code",
        format!("{} {}", res.verification_uri, res.user_code),
    )
    .unwrap();

    let msa = get_ms_auth_token(&client, res).await?;
    let auth_result = request_minecraft_token(&client, &msa.access_token).await?;
    let minecraft_profile = request_profile(&client, &auth_result.minecraft_access_token).await?;

    if let Err(reason) = enforce_access(&minecraft_profile.name).await {
        let _ = app.emit_all("auth-error", reason.clone());
        return Err(reason);
    }

    let mut config = config::SHARED_CONFIG.lock().await;

    config.is_logged_in = true;
    config.username = minecraft_profile.name;
    config.uuid = minecraft_profile.id.to_string();
    config.minecraft_token = auth_result.minecraft_access_token;
    config.xbox_token = msa.access_token;

    app.emit_all("config-updated", Some(config.clone()))
        .unwrap();

    config.save_config().unwrap();

    return Ok(());
}

#[tauri::command]
pub async fn logout(app: AppHandle) {
    let mut config = config::SHARED_CONFIG.lock().await;

    config.is_logged_in = false;
    config.username = "".to_string();
    config.uuid = "".to_string();
    config.minecraft_token = "".to_string();
    config.xbox_token = "".to_string();

    app.emit_all("config-updated", Some(config.clone()))
        .unwrap();

    config.save_config().unwrap();
}

// Convenience wrappers for the downloader, which only has a token at hand and
// wants an `Option` instead of a `Result`.

pub async fn get_minecraft_token(xbox_token: &String) -> Option<MinecraftTokenResponse> {
    let client = reqwest::Client::new();

    match request_minecraft_token(&client, xbox_token).await {
        Ok(token) => Some(token),
        Err(e) => {
            println!("Error getting Minecraft token: {}", e);
            None
        }
    }
}

pub async fn get_minecraft_profile(token: &String) -> Option<ProfileResponse> {
    let client = reqwest::Client::new();

    match request_profile(&client, token).await {
        Ok(profile) => Some(profile),
        Err(e) => {
            println!("Error getting profile: {}", e);
            None
        }
    }
}
