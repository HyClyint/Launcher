use std::fs;

use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::config::{self, Profile, DEFAULT_PROFILE_ID, SHARED_CONFIG};
use crate::core::directory;

fn clean_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 32 {
        return Err("A profile name must be 1-32 characters".to_string());
    }

    Ok(name.to_string())
}

fn publish(app: &AppHandle, config: &config::Config) -> Result<(), String> {
    config.save_config().map_err(|error| error.to_string())?;
    let _ = app.emit_all("config-updated", Some(config.clone()));
    Ok(())
}

#[tauri::command]
pub async fn create_profile(
    app: AppHandle,
    name: String,
    game_version: String,
) -> Result<String, String> {
    let name = clean_name(&name)?;

    let known = config::get_versions().await?;
    if !known.iter().any(|version| version.id == game_version) {
        return Err(format!("Unknown game version {}", game_version));
    }

    let mut config = SHARED_CONFIG.lock().await;
    let id = Uuid::new_v4().simple().to_string();

    config.profiles.push(Profile {
        id: id.clone(),
        name,
        game_version,
    });
    directory::get_profile_mods_directory(&id);

    publish(&app, &config)?;
    Ok(id)
}

#[tauri::command]
pub async fn select_profile(app: AppHandle, id: String) -> Result<(), String> {
    let mut config = SHARED_CONFIG.lock().await;

    if !config.profiles.iter().any(|profile| profile.id == id) {
        return Err("That profile no longer exists".to_string());
    }

    config.selected_profile = id;
    config.ensure_profiles();

    publish(&app, &config)
}

#[tauri::command]
pub async fn rename_profile(app: AppHandle, id: String, name: String) -> Result<(), String> {
    let name = clean_name(&name)?;
    let mut config = SHARED_CONFIG.lock().await;

    match config.profiles.iter_mut().find(|profile| profile.id == id) {
        Some(profile) => profile.name = name,
        None => return Err("That profile no longer exists".to_string()),
    }

    publish(&app, &config)
}

#[tauri::command]
pub async fn delete_profile(app: AppHandle, id: String) -> Result<(), String> {
    if id == DEFAULT_PROFILE_ID {
        return Err("The default profile can't be deleted".to_string());
    }

    let mut config = SHARED_CONFIG.lock().await;
    let before = config.profiles.len();
    config.profiles.retain(|profile| profile.id != id);
    if config.profiles.len() == before {
        return Err("That profile no longer exists".to_string());
    }
    config.ensure_profiles();

    // Only ever a generated UUID folder inside `<root>/profiles`.
    let _ = fs::remove_dir_all(directory::get_profile_directory(&id));

    publish(&app, &config)
}
