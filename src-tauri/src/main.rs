// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod auth;
mod config;
mod util;
mod core {
    pub mod api;
    pub mod directory;
    pub mod downloader;
    pub mod launcher;
    pub mod mods;
    pub mod profiles;

    pub mod json {
        pub mod backend_config;
        pub mod fabric_meta;
        pub mod fabric_version_json;
        pub mod minecraft_asset_index;
        pub mod minecraft_version_json;
        pub mod modrinth_version;
        pub mod mojang_runtime_all_json;
        pub mod mojang_runtime_gamma_meta_json;
        pub mod version_manifest;
    }
}

use core::downloader;
use core::launcher;
use core::directory;

use tauri::AppHandle;

#[tauri::command]
async fn initialize(app: AppHandle) -> Result<(), String> {
    downloader::init(&app).await;

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            initialize,
            directory::reset,
            launcher::launch_client,
            auth::login_offline,
            auth::login_microsoft,
            auth::logout,
            config::get_config,
            config::get_versions,
            core::profiles::create_profile,
            core::profiles::select_profile,
            core::profiles::rename_profile,
            core::profiles::delete_profile,
            core::mods::search_mods,
            core::mods::install_mod,
            core::mods::list_mods,
            core::mods::set_mod_enabled,
            core::mods::remove_mod
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
