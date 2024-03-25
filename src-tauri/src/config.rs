use std::fs;
use std::io;
use std::sync::Arc;

use futures::lock::Mutex;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;
use tauri::AppHandle;

use crate::core::directory;

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
}

impl Config {
    pub fn new() -> Self {
        Config {
            is_logged_in: false,
            selected_version: String::from("1.20.4"),
            fabric_version: String::from("0.15.7"),
            hyclyint_version: String::from("1.0.0"),
            hylauncher_version: String::from("0.1.1-BETA"),
            username: String::new(),
            uuid: String::new(),
            xbox_token: String::new(),
            minecraft_token: String::new(),
        }
    }

    // Load the configuration from file
    pub fn load_config() -> Result<Self, io::Error> {
        let hyclyint_path = directory::get_hyclyint_directory();
        let config_path = hyclyint_path.join("launcher_config.json");
        let content = fs::read_to_string(config_path)?;
        let parsed_config: Config = serde_json::from_str(&content)?;
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
