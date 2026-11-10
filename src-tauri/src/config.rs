use std::fs;
use std::io;
use std::sync::Arc;

use futures::lock::Mutex;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;

use crate::core::directory;

/// Id of the profile that exists on every install. It uses the launcher root
/// as its game folder so worlds and settings from before profiles existed
/// are kept.
pub const DEFAULT_PROFILE_ID: &str = "default";

#[derive(Serialize, Deserialize, Clone)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub game_version: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub is_logged_in: bool,
    pub selected_version: String,
    pub fabric_version: String,
    pub hyclyint_version: String,
    pub hylauncher_version: String,
    pub username: String,
    pub uuid: String,
    pub xbox_token: String,
    pub minecraft_token: String,
    #[serde(default)]
    pub profiles: Vec<Profile>,
    #[serde(default)]
    pub selected_profile: String,
}

impl Config {
    pub fn new() -> Self {
        Config {
            is_logged_in: false,
            selected_version: String::from("1.20.4"),
            fabric_version: String::from("0.15.7"),
            hyclyint_version: String::from("1.0.0"),
            hylauncher_version: String::from("0.0.0"),
            username: String::new(),
            uuid: String::new(),
            xbox_token: String::new(),
            minecraft_token: String::new(),
            profiles: Vec::new(),
            selected_profile: String::new(),
        }
        .with_profiles()
    }

    fn with_profiles(mut self) -> Self {
        self.ensure_profiles();
        self
    }

    /// Makes sure there is at least one profile and that the selected one
    /// exists, keeping `selected_version` in sync with it.
    pub fn ensure_profiles(&mut self) {
        if self.profiles.is_empty() {
            self.profiles.push(Profile {
                id: DEFAULT_PROFILE_ID.to_string(),
                name: "Default".to_string(),
                game_version: self.selected_version.clone(),
            });
        }

        if !self.profiles.iter().any(|p| p.id == self.selected_profile) {
            self.selected_profile = self.profiles[0].id.clone();
        }

        self.selected_version = self.active_profile().game_version;
    }

    pub fn active_profile(&self) -> Profile {
        self.profiles
            .iter()
            .find(|p| p.id == self.selected_profile)
            .unwrap_or(&self.profiles[0])
            .clone()
    }

    // Load the configuration from file
    pub fn load_config() -> Result<Self, io::Error> {
        let hyclyint_path = directory::get_hyclyint_directory();
        let config_path = hyclyint_path.join("launcher_config.json");
        let content = fs::read_to_string(config_path)?;
        let mut parsed_config: Config = serde_json::from_str(&content)?;
        parsed_config.ensure_profiles();
        Ok(parsed_config)
    }

    // Save the configuration to file
    pub fn save_config(&self) -> Result<(), io::Error> {
        let serialized_config = serde_json::to_string(&self)?;

        let hyclyint_path = directory::get_hyclyint_directory();
        let config_path = hyclyint_path.join("launcher_config.json");
        fs::write(config_path, serialized_config)?;

        Ok(())
    }
}

pub static SHARED_CONFIG: Lazy<Arc<Mutex<Config>>> = Lazy::new(|| {
    let config = Config::load_config().unwrap_or_else(|err| {
        eprintln!(
            "Error loading config: {}, loading one from default values!",
            err
        );

        return Config::new();
    });

    Arc::new(Mutex::new(config))
});

#[tauri::command]
pub async fn get_config() -> Result<Config, String> {
    Ok(SHARED_CONFIG.lock().await.clone())
}

#[derive(Serialize)]
pub struct GameVersion {
    pub id: String,
    pub installed: bool,
}

fn is_downloaded(versions_directory: &std::path::Path, id: &str) -> bool {
    versions_directory
        .join(id)
        .join(format!("{}.jar", id))
        .exists()
}

/// Versions the launcher can offer, newest first. The backend decides which
/// are supported, since each needs matching mod builds. If it is unreachable,
/// only versions that are already installed (plus the selected one) are
/// offered, so an outage can't push people onto an untested version.
#[tauri::command]
pub async fn get_versions() -> Result<Vec<GameVersion>, String> {
    let versions_directory = directory::get_versions_directory();
    let selected = SHARED_CONFIG.lock().await.selected_version.clone();

    let mut ids = match crate::core::api::fetch_versions().await {
        Ok(ids) => ids,
        Err(error) => {
            eprintln!("Failed to load versions from the backend: {}", error);

            let mut installed: Vec<String> = fs::read_dir(&versions_directory)
                .map(|entries| {
                    entries
                        .flatten()
                        .filter_map(|entry| entry.file_name().into_string().ok())
                        .filter(|id| is_downloaded(&versions_directory, id))
                        .collect()
                })
                .unwrap_or_default();
            installed.sort_by(|a, b| version_key(b).cmp(&version_key(a)));
            installed
        }
    };

    // The newest published HyClyint build may target a version the allow-list
    // doesn't mention yet.
    if let Ok(build) = crate::core::api::fetch_latest_build().await {
        if !ids.contains(&build.minecraft_version) {
            ids.insert(0, build.minecraft_version);
        }
    }

    if !ids.contains(&selected) {
        ids.push(selected);
    }

    Ok(ids
        .into_iter()
        .map(|id| GameVersion {
            installed: is_downloaded(&versions_directory, &id),
            id,
        })
        .collect())
}

#[derive(Serialize)]
pub struct FeaturedProfile {
    pub name: String,
    pub game_version: String,
    pub hyclyint_version: String,
    pub channel: String,
    /// A profile on this game version already exists.
    pub exists: bool,
}

/// A ready-made profile for the newest HyClyint build, if the backend
/// publishes one. Errors are not fatal for the UI; it just shows nothing.
#[tauri::command]
pub async fn get_featured_profile() -> Result<FeaturedProfile, String> {
    let build = crate::core::api::fetch_latest_build().await?;
    let exists = SHARED_CONFIG
        .lock()
        .await
        .profiles
        .iter()
        .any(|profile| profile.game_version == build.minecraft_version);

    Ok(FeaturedProfile {
        name: format!("HyClyint {}", build.minecraft_version),
        game_version: build.minecraft_version,
        hyclyint_version: build.version,
        channel: build.channel,
        exists,
    })
}

fn version_key(id: &str) -> Vec<u32> {
    id.split('.').map(|part| part.parse().unwrap_or(0)).collect()
}
