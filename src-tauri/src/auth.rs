use azalea::auth;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::config;

#[tauri::command]
pub async fn login_offline(app: AppHandle, username: &str) -> Result<(), String> {
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
    let res = auth::get_ms_link_code(&client).await.unwrap();

    app.emit_all(
        "microsoft-code",
        String::from(format!("{} {}", res.verification_uri, res.user_code)),
    )
    .unwrap();

    let msa = auth::get_ms_auth_token(&client, res).await.unwrap();
    let auth_result = auth::get_minecraft_token(&client, &msa.data.access_token)
        .await
        .unwrap();
    let minecraft_profile = get_minecraft_profile(&auth_result.minecraft_access_token).await;

    if minecraft_profile.is_none() {
        return Err("Error getting Minecraft profile".to_string());
    }

    let minecraft_profile = minecraft_profile.unwrap();

    let mut config = config::SHARED_CONFIG.lock().await;

    config.is_logged_in = true;
    config.username = minecraft_profile.name;
    config.uuid = minecraft_profile.id.to_string();
    config.minecraft_token = auth_result.minecraft_access_token;
    config.xbox_token = msa.data.access_token;

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

pub async fn get_minecraft_token(xbox_token: &String) -> Option<auth::MinecraftTokenResponse> {
    let client = reqwest::Client::new();

    let response = auth::get_minecraft_token(&client, &xbox_token).await;

    match response {
        Ok(token) => {
            return Some(token);
        }
        Err(e) => {
            println!("Error getting Minecraft token: {}", e);
            return None;
        }
    }
}

pub async fn get_minecraft_profile(token: &String) -> Option<auth::ProfileResponse> {
    let client = reqwest::Client::new();

    let response = auth::get_profile(&client, &token).await;

    match response {
        Ok(profile) => {
            return Some(profile);
        }
        Err(e) => {
            println!("Error getting profile: {}", e);
            return None;
        }
    }
}