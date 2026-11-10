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

fn safe_id(id: &str) -> String {
    let cleaned: String = id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();

    // Never resolve to the parent folder itself.
    if cleaned.is_empty() {
        "_invalid".to_string()
    } else {
        cleaned
    }
}

/// The game folder (`--gameDir`) of a profile. The default profile lives in
/// the launcher root for backwards compatibility.
pub fn get_profile_directory(id: &str) -> PathBuf {
    let root = get_hyclyint_directory();
    let path = if id == crate::config::DEFAULT_PROFILE_ID {
        root
    } else {
        root.join("profiles").join(safe_id(id))
    };
    fs::create_dir_all(&path).unwrap();

    path
}

pub fn get_profile_mods_directory(id: &str) -> PathBuf {
    let path = get_profile_directory(id).join("mods");
    fs::create_dir_all(&path).unwrap();

    path
}

/// Launcher-managed mods (HyClyint + required mods) for one game version.
/// Loaded through `-Dfabric.addMods`, so they never mix with user mods.
pub fn get_required_mods_directory(version: &str) -> PathBuf {
    let path = get_hyclyint_directory()
        .join("required_mods")
        .join(safe_id(&version.replace('.', "_")));
    fs::create_dir_all(&path).unwrap();

    path
}

#[tauri::command]
pub async fn reset(app: tauri::AppHandle) -> Result<(), String> {
    let hyclyint_path = get_hyclyint_directory();
    fs::remove_dir_all(hyclyint_path).unwrap();
    app.exit(0);
    Ok(())
}
