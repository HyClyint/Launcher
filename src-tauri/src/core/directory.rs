use std::{fs, path::PathBuf};

pub fn get_hyclyint_directory() -> PathBuf {
    let home = dirs::home_dir().unwrap();
    let hyclyint_path = home.join(".hyclyint");
    if !hyclyint_path.exists() {
        fs::create_dir(hyclyint_path.clone()).unwrap();
    }

    hyclyint_path
}

pub fn get_versions_directory() -> PathBuf {
    let hyclyint_path = get_hyclyint_directory();
    let versions_path = hyclyint_path.join("versions");
    if !versions_path.exists() {
        fs::create_dir(versions_path.clone()).unwrap();
    }

    versions_path
}

pub fn get_assets_directory() -> PathBuf {
    let hyclyint_path = get_hyclyint_directory();
    let assets_path = hyclyint_path.join("assets");
    if !assets_path.exists() {
        fs::create_dir(assets_path.clone()).unwrap();
    }

    assets_path
}

pub fn get_runtime_directory() -> PathBuf {
    let hyclyint_path = get_hyclyint_directory();
    let runtime_path = hyclyint_path.join("runtime");
    if !runtime_path.exists() {
        fs::create_dir(runtime_path.clone()).unwrap();
    }

    runtime_path
}

pub fn get_libraries_directory() -> PathBuf {
    let hyclyint_path = get_hyclyint_directory();
    let libraries_path = hyclyint_path.join("libraries");
    if !libraries_path.exists() {
        fs::create_dir(libraries_path.clone()).unwrap();
    }

    libraries_path
}

pub fn get_mods_directory() -> PathBuf {
    let hyclyint_path = get_hyclyint_directory();
    let mods_path = hyclyint_path.join("mods");
    if !mods_path.exists() {
        fs::create_dir(mods_path.clone()).unwrap();
    }

    mods_path
}

pub fn get_version_directory(version: &str) -> PathBuf {
    let versions_path = get_versions_directory();
    let version_path = versions_path.join(version);
    if !version_path.exists() {
        fs::create_dir(version_path.clone()).unwrap();
    }

    version_path
}

pub fn get_assets_indexes_directory() -> PathBuf {
    let assets_path = get_assets_directory();
    let indexes_path = assets_path.join("indexes");
    if !indexes_path.exists() {
        fs::create_dir(indexes_path.clone()).unwrap();
    }

    indexes_path
}

pub fn get_asset_object_directory(hash: &str) -> PathBuf {
    let assets_path = get_assets_directory();
    let object_path = assets_path.join("objects");
    if !object_path.exists() {
        fs::create_dir(object_path.clone()).unwrap();
    }

    let hash_path = object_path.join(&hash[0..2]);
    if !hash_path.exists() {
        fs::create_dir(hash_path.clone()).unwrap();
    }

    hash_path
}

#[tauri::command]
pub async fn reset(app: tauri::AppHandle) -> Result<(), String> {
    let hyclyint_path = get_hyclyint_directory();
    fs::remove_dir_all(hyclyint_path).unwrap();
    app.exit(0);
    Ok(())
}
