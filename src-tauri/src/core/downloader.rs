use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::{auth, config, util};

use super::{
    api, directory,
    json::{
        fabric_meta, fabric_version_json, minecraft_asset_index, minecraft_version_json,
        modrinth_version, mojang_runtime_gamma_meta_json,
        version_manifest,
    },
};

const MINECRAFT_VERSION_MANIFEST_URL: &str =
    "https://launchermeta.mojang.com/mc/game/version_manifest.json";
const MINECRAFT_ASSET_DOWNLOAD_URL: &str = "https://resources.download.minecraft.net/";
const MINECRAFT_RUNTIME_META_DOWNLOAD_URL: &str = "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";
const FABRIC_META_DOWNLOAD_URL: &str = "https://meta.fabricmc.net/v2/versions/loader/";
const MODRINTH_API: &str = "https://api.modrinth.com/v2/";

/// Compares dotted numeric versions ("0.1.3" < "0.1.10"); ignores suffixes like "-BETA".
fn is_older(current: &str, minimum: &str) -> bool {
    fn parts(version: &str) -> Vec<u64> {
        version
            .split(|c| c == '-' || c == '+')
            .next()
            .unwrap_or("")
            .split('.')
            .map(|part| part.parse().unwrap_or(0))
            .collect()
    }

    let (mut left, mut right) = (parts(current), parts(minimum));
    let len = left.len().max(right.len());
    left.resize(len, 0);
    right.resize(len, 0);

    left < right
}

pub async fn update_hyclyint_config(app: &AppHandle) -> Result<(), String> {
    let remote_config = match api::get_global_config(true).await {
        Ok(config) => {
            println!("Loaded remote config from the HyClyint backend");
            config
        }
        Err(error) => {
            println!("Failed to download HyClyint config: {}", error);
            return Err("Failed to download HyClyint config".to_string());
        }
    };

    let mut config = config::SHARED_CONFIG.lock().await;

    let fabric_version = remote_config
        .fabric_version
        .clone()
        .filter(|version| !version.is_empty())
        .unwrap_or_else(|| config.fabric_version.clone());

    let hyclyint_version = remote_config
        .hyclyint_version
        .clone()
        .filter(|version| !version.is_empty())
        .unwrap_or_else(|| config.hyclyint_version.clone());

    if config.fabric_version != fabric_version {
        remove_directory(directory::get_versions_directory());
        remove_directory(directory::get_libraries_directory());

        config.fabric_version = fabric_version;
    }

    if config.hyclyint_version != hyclyint_version {
        // Only launcher-managed mods live here; user mods are in profiles.
        remove_directory(directory::get_hyclyint_directory().join("required_mods"));

        config.hyclyint_version = hyclyint_version;
    }

    let version_info = app.package_info().version.clone();
    let version_string = format!(
        "{}.{}.{}",
        version_info.major, version_info.minor, version_info.patch
    );

    let expected_launcher_version = remote_config
        .hyclyint_launcher_version
        .clone()
        .filter(|version| !version.is_empty())
        .unwrap_or_else(|| config.hylauncher_version.clone());

    // The backend value is a minimum: newer launchers are fine, and the Tauri
    // updater (GitHub releases) is what tells people a newer build exists.
    if is_older(&version_string, &expected_launcher_version) {
        app.emit_all("update-available", format!("Current launcher version: {} is outdated! Please update to version: {} to continue using HyClyint!", version_string, expected_launcher_version)).unwrap();
    }

    config.save_config().unwrap();

    Ok(())
}

fn remove_directory(path: PathBuf) {
    if !path.exists() {
        return;
    }

    if let Err(error) = fs::remove_dir_all(&path) {
        println!("Failed to remove {}: {}", path.display(), error);
    }
}

pub async fn download_minecraft_version_manifest() -> Result<(), String> {
    let versions_path = directory::get_versions_directory();
    let version_manifest_path = versions_path.join("version_manifest.json");

    let response = match reqwest::get(MINECRAFT_VERSION_MANIFEST_URL).await {
        Ok(response) => response,
        Err(_) => {
            println!("Failed to download version manifest");
            return Err("Failed to download version manifest".to_string());
        }
    };

    let version_manifest = response.text().await.unwrap();
    fs::write(version_manifest_path, version_manifest).unwrap();

    Ok(())
}

pub async fn download_fabric_meta() -> Result<(), String> {
    let versions_path = directory::get_versions_directory();
    let fabric_meta_path = versions_path.join("fabric_meta.json");

    let response = match reqwest::get(FABRIC_META_DOWNLOAD_URL).await {
        Ok(response) => response,
        Err(_) => {
            println!("Failed to download fabric meta");
            return Err("Failed to download fabric meta".to_string());
        }
    };

    let fabric_meta = response.text().await.unwrap();
    fs::write(fabric_meta_path, fabric_meta).unwrap();

    Ok(())
}

pub async fn download_minecraft_version(config: &config::Config) -> Result<(), String> {
    let versions_directory = directory::get_versions_directory();
    let version_manifest_path = versions_directory.join("version_manifest.json");
    let version_manifest = fs::read_to_string(version_manifest_path).unwrap();
    let version_manifest: version_manifest::Root = serde_json::from_str(&version_manifest).unwrap();

    let selected_version = config.selected_version.clone();
    let selected_version = version_manifest
        .versions
        .iter()
        .find(|version| version.id == selected_version)
        .unwrap();

    let version_directory = directory::get_version_directory(&selected_version.id);

    let client = reqwest::Client::new();
    let response = match client.get(&selected_version.url).send().await {
        Ok(response) => response,
        Err(_) => {
            println!("Failed to download version: {}", selected_version.id);
            return Err("Failed to download version".to_string());
        }
    };

    let response = response.text().await.unwrap();
    let version_json_path = version_directory.join(format!("{}.json", selected_version.id));
    fs::write(version_json_path, response).unwrap();

    Ok(())
}

pub async fn download_runtime_meta() -> Result<(), String> {
    let runtime_directory = directory::get_runtime_directory();
    let runtime_meta_path = runtime_directory.join("runtime_meta.json");

    let response = match reqwest::get(MINECRAFT_RUNTIME_META_DOWNLOAD_URL).await {
        Ok(response) => response,
        Err(_) => {
            println!("Failed to download runtime meta");
            return Err("Failed to download runtime meta".to_string());
        }
    };

    let runtime_meta = response.text().await.unwrap();
    fs::write(runtime_meta_path, runtime_meta).unwrap();

    Ok(())
}

pub async fn download_fabric_version(config: &config::Config) -> Result<(), String> {
    let versions_directory = directory::get_versions_directory();
    let fabric_meta_path = versions_directory.join("fabric_meta.json");

    let fabric_meta = fs::read_to_string(fabric_meta_path).unwrap();
    let fabric_meta: fabric_meta::Root = serde_json::from_str(&fabric_meta).unwrap();

    let selected_version = config.selected_version.clone();
    let fabric_version = config.fabric_version.clone();

    let selected_fabric_version = fabric_meta
        .iter()
        .find(|version| version.version == fabric_version)
        .unwrap();

    let version_directory = directory::get_version_directory(&selected_version);

    let url = format!(
        "{}/{}/{}/profile/json",
        FABRIC_META_DOWNLOAD_URL, selected_version, selected_fabric_version.version,
    );

    let client = reqwest::Client::new();
    let response = match client.get(&url).send().await {
        Ok(response) => response,
        Err(_) => {
            println!(
                "Failed to download fabric version: {}",
                selected_fabric_version.version
            );
            return Err("Failed to download fabric version".to_string());
        }
    };

    let response = response.text().await.unwrap();
    let version_json_path = version_directory.join(format!("{}-fabric.json", selected_version));
    fs::write(version_json_path, response).unwrap();

    Ok(())
}

pub async fn download_mods(config: &config::Config, app: &AppHandle) -> Result<(), String> {
    println!("Downloading mods");

    let mods_directory = directory::get_required_mods_directory(&config.selected_version);

    // Reuses the config cached by update_hyclyint_config; the jar itself is
    // downloaded from the backend's /downloads folder.
    let remote_config = match api::get_global_config(false).await {
        Ok(config) => config,
        Err(error) => {
            println!("Failed to load the HyClyint config: {}", error);
            return Err("Failed to download HyClyint config".to_string());
        }
    };

    // latest.json names the newest published jar, so the launcher does not
    // have to guess file names from the config's hyclyint_version.
    let build = match api::fetch_latest_build().await {
        Ok(build) if !build.file.is_empty() && !build.url.is_empty() => build,
        Ok(_) => return Err("latest.json does not name a HyClyint jar".to_string()),
        Err(error) => {
            println!("Failed to load latest.json: {}", error);
            return Err("Failed to download HyClyint".to_string());
        }
    };

    // The file name becomes a path, so keep it to a plain jar name.
    let plain_name = build.file.ends_with(".jar")
        && build
            .file
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_');
    if !plain_name {
        return Err("latest.json has an invalid jar name".to_string());
    }

    println!("Downloading HyClyint {}", build.version);
    let hyclyint_url = format!("{}{}", api::BACKEND_URL, build.url.trim_start_matches('/'));
    let hyclyint_path = mods_directory.join(&build.file);

    // Drop older HyClyint jars so two versions never load together.
    if let Ok(entries) = fs::read_dir(&mods_directory) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("hyclyint-") && name.ends_with(".jar") && name != build.file {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    let response = match reqwest::get(hyclyint_url).await {
        Ok(response) => response,
        Err(_) => {
            println!("Failed to download HyClyint {}", build.version);
            return Err("Failed to download HyClyint".to_string());
        }
    };

    if !response.status().is_success() {
        println!(
            "HyClyint {} download returned HTTP {}",
            build.version,
            response.status()
        );
        return Err("Failed to download HyClyint".to_string());
    }

    let response = response.bytes().await.unwrap();
    fs::write(hyclyint_path, response).unwrap();
    println!("Downloaded HyClyint {}. Downloading required mods!", build.version);

    let mut i = 0;
    let length = remote_config.required_mods.len();

    println!("Total mods: {}", length);

    for required_mod in remote_config.required_mods {
        let modrinth_response = match reqwest::get(format!(
            "{}project/{}/version",
            MODRINTH_API, required_mod.name
        ))
        .await
        {
            Ok(response) => response,
            Err(_) => {
                println!(
                    "Failed to download mod: {}. {}/{}",
                    required_mod.name, i, length
                );
                return Err("Failed to download mod info".to_string());
            }
        };

        let modrinth_response = modrinth_response.text().await.unwrap();
        let modrinth_response: modrinth_version::Root =
            serde_json::from_str(&modrinth_response).unwrap();

        let selected_version = modrinth_response
            .iter()
            .find(|version| {
                version.version_number == required_mod.version
                    && version.loaders.contains(&"fabric".to_string())
                    && version
                        .game_versions
                        .contains(&config.selected_version.clone())
            })
            .ok_or_else(|| {
                format!(
                    "{} is not available for Minecraft {}",
                    required_mod.name, config.selected_version
                )
            })?;

        let mod_path = mods_directory.join(selected_version.files[0].filename.clone());
        let mod_url = selected_version.files[0].url.clone();

        if mod_path.exists() {
            println!("Skipping mod: {}. {}/{}", required_mod.name, i, length);
            i += 1;
            continue;
        }

        println!("Downloading mod: {}. {}/{}", required_mod.name, i, length);
        let response = match reqwest::get(mod_url).await {
            Ok(response) => response,
            Err(_) => {
                println!(
                    "Failed to download mod: {}. {}/{}",
                    required_mod.name, i, length
                );
                return Err("Failed to download mod".to_string());
            }
        };

        let response = response.bytes().await.unwrap();
        fs::write(mod_path, response).unwrap();

        println!("Downloaded mod: {}. {}/{}", required_mod.name, i, length);
        app.emit_all(
            "update-progress",
            format!("Mods: {}%", util::get_percentage(i, length as i32)),
        )
        .unwrap();
        i += 1;
    }

    println!("Downloaded {} of out {} mods", i, length);

    Ok(())
}

pub async fn download_assets_minecraft(
    config: &config::Config,
    app: &AppHandle,
) -> Result<(), String> {
    let version_manifest_path = directory::get_versions_directory().join("version_manifest.json");
    let version_manifest = fs::read_to_string(version_manifest_path).unwrap();
    let version_manifest: version_manifest::Root = serde_json::from_str(&version_manifest).unwrap();

    let selected_version = config.selected_version.clone();
    let selected_version = version_manifest
        .versions
        .iter()
        .find(|version| version.id == selected_version)
        .unwrap();

    let version_directory = directory::get_version_directory(&selected_version.id);
    let version_json_path = version_directory.join(format!("{}.json", selected_version.id));
    let version_json = fs::read_to_string(version_json_path).unwrap();
    let version_json: minecraft_version_json::Root = serde_json::from_str(&version_json).unwrap();

    println!(
        "Downloading asset index {} for version {}",
        version_json.asset_index.id, selected_version.id
    );
    let client = reqwest::Client::new();
    let response = match client.get(&version_json.asset_index.url).send().await {
        Ok(response) => response,
        Err(_) => {
            println!(
                "Failed to download asset index for version {}",
                selected_version.id
            );
            return Err("Failed to download asset index".to_string());
        }
    };

    let response = response.text().await.unwrap();
    println!("Downloaded asset index for version {}", selected_version.id);

    let asset_index_path = directory::get_assets_indexes_directory()
        .join(format!("{}.json", version_json.asset_index.id));
    fs::write(asset_index_path, response.clone()).unwrap();

    let asset_index: minecraft_asset_index::Root = serde_json::from_str(&response).unwrap();
    let length = asset_index.objects.len();
    let mut i = 0;
    let max_threads = 3;

    println!("Downloading assets for version {}", selected_version.id);
    println!("Total assets: {}", length);
    println!("Multi-threading with {} threads", max_threads);

    while i < length {
        let mut handles = vec![];

        while handles.len() < max_threads && i < length {
            let objects = asset_index.objects.clone();
            let (key, object) = objects.into_iter().nth(i).unwrap();
            let hash = &object.hash;

            let hash_path = directory::get_asset_object_directory(hash);
            let object_path = hash_path.join(hash);

            if !object_path.exists() {
                println!("Downloading asset {}/{}: {}", i, length, key);

                let url = format!("{}{}/{}", MINECRAFT_ASSET_DOWNLOAD_URL, &hash[0..2], hash);
                let path = object_path.clone();

                let handle = tokio::spawn(async move {
                    let response = match reqwest::get(url).await {
                        Ok(response) => response,
                        Err(_) => {
                            println!("Failed to download asset {}/{}: {}", i, length, key);
                            return Err("Failed to download asset".to_string());
                        }
                    };

                    let response = response.bytes().await.unwrap();
                    fs::write(path, response).unwrap();

                    Ok(())
                });

                handles.push(handle);
            } else {
                println!("Skipping asset {}/{}: {}", i, length, key);
            }

            app.emit_all(
                "update-progress",
                format!("Assets: {}%", util::get_percentage(i as i32, length as i32)),
            )
            .unwrap();
            i += 1;
        }

        for handle in handles {
            match handle.await {
                Ok(_) => {}
                Err(_) => {
                    println!("Failed to download asset");
                    return Err("Failed to download asset".to_string());
                }
            }
        }

        println!("Downloaded {}/{} assets", i, length);
    }

    Ok(())
}

pub async fn download_libraries(config: &config::Config, app: &AppHandle) -> Result<(), String> {
    let selected_version = config.selected_version.clone();

    let version_directory = directory::get_version_directory(&selected_version);
    let version_json_path = version_directory.join(format!("{}.json", selected_version));
    let version_json = fs::read_to_string(version_json_path).unwrap();
    let version_json: minecraft_version_json::Root = serde_json::from_str(&version_json).unwrap();

    let libraries_directory = directory::get_libraries_directory();
    let length = version_json.libraries.len();
    let mut i = 0;

    for library in version_json.libraries {
        let library_url = library.downloads.artifact.url;
        let mut library_path = libraries_directory.clone();

        // Split the library name into parts
        let parts = library.downloads.artifact.path.unwrap();
        let parts: Vec<&str> = parts.split('/').collect();

        for part in parts {
            library_path = library_path.join(part);

            if !library_path.exists() && !part.contains(".jar") {
                fs::create_dir(&library_path).unwrap();
            }
        }

        library_path = libraries_directory.join(library_path);

        if !library_path.exists() {
            println!("Downloading library: {}", library.name);

            let response = match reqwest::get(library_url).await {
                Ok(response) => response,
                Err(_) => {
                    println!("Failed to download library: {}", library.name);
                    return Err("Failed to download library".to_string());
                }
            };

            let response = response.bytes().await.unwrap();
            fs::write(library_path, response).unwrap();
        } else {
            println!("Skipping library: {}", library.name);
        }

        app.emit_all(
            "update-progress",
            format!("Libraries: {}%", util::get_percentage(i, length as i32)),
        )
        .unwrap();
        i += 1;
    }

    Ok(())
}

pub async fn download_libraries_fabric(
    config: &config::Config,
    app: &AppHandle,
) -> Result<(), String> {
    let selected_version = config.selected_version.clone();

    let version_file = directory::get_version_directory(&selected_version)
        .join(format!("{}-fabric.json", selected_version));

    let version_json = fs::read_to_string(version_file).unwrap();
    let version_json: fabric_version_json::Root = serde_json::from_str(&version_json).unwrap();

    let libraries_directory = directory::get_libraries_directory();
    let length = version_json.libraries.len();
    let mut i = 0;

    for library in version_json.libraries {
        let mut library_url = library.url.clone();
        let mut library_path = libraries_directory.clone();

        // Split the library name into parts
        let parts = library.name.split(':').collect::<Vec<&str>>();
        let group = parts[0].replace(".", "/");
        let artifact = parts[1];
        let version = parts[2];

        library_url = format!(
            "{}/{}/{}/{}/{}-{}.jar",
            library_url, group, artifact, version, artifact, version
        );

        let parts = library_url.split('/').collect::<Vec<&str>>();
        let file_name = parts[parts.len() - 1];

        let parts = group.split('/').collect::<Vec<&str>>();
        for part in parts {
            library_path = library_path.join(part);

            if !library_path.exists() {
                fs::create_dir(&library_path).unwrap();
            }
        }

        library_path = library_path.join(file_name);

        if !library_path.exists() {
            println!("Downloading library: {} from {}", library.name, library_url);

            let response = match reqwest::get(library_url).await {
                Ok(response) => response,
                Err(_) => {
                    println!("Failed to download library: {}", library.name);
                    return Err("Failed to download library".to_string());
                }
            };

            let response = response.bytes().await.unwrap();
            fs::write(library_path, response).unwrap();
        } else {
            println!("Skipping library: {}", library.name);
        }

        app.emit_all(
            "update-progress",
            format!("Libraries: {}%", util::get_percentage(i, length as i32)),
        )
        .unwrap();
        i += 1;
    }

    Ok(())
}

pub async fn download_minecraft_jar(
    config: &config::Config,
    app: &AppHandle,
) -> Result<(), String> {
    let versions_directory = directory::get_versions_directory();

    let selected_version = config.selected_version.clone();

    let version_file = directory::get_version_directory(&selected_version)
        .join(format!("{}.json", selected_version));

    let version_json = fs::read_to_string(version_file).unwrap();
    let version_json: minecraft_version_json::Root = serde_json::from_str(&version_json).unwrap();

    let jar_path = versions_directory
        .join(selected_version.clone())
        .join(format!("{}.jar", selected_version));

    if jar_path.exists() {
        println!(
            "Minecraft jar for version {} already exists",
            selected_version
        );
        return Ok(());
    }

    println!(
        "Downloading Minecraft jar for version {} from {}",
        selected_version, version_json.downloads.client.url
    );

    app.emit_all(
        "update-progress",
        format!("Downloading {}.jar", selected_version),
    )
    .unwrap();

    let client = reqwest::Client::new();
    let response = match client.get(&version_json.downloads.client.url).send().await {
        Ok(response) => response,
        Err(_) => {
            println!(
                "Failed to download Minecraft jar for version {}",
                selected_version
            );
            return Err("Failed to download Minecraft jar".to_string());
        }
    };

    let response = response.bytes().await.unwrap();
    fs::write(jar_path, response).unwrap();

    println!("Downloaded Minecraft jar for version {}", selected_version);
    Ok(())
}

/// The Java runtime component the game version asks for (e.g. "java-runtime-gamma"
/// for Java 17-21, "java-runtime-epsilon" for Java 25), read from its version json.
fn runtime_component(version: &str) -> String {
    let path = directory::get_version_directory(version).join(format!("{}.json", version));

    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|json| json["javaVersion"]["component"].as_str().map(str::to_string))
        .unwrap_or_else(|| "java-runtime-gamma".to_string())
}

/// Downloads the Java runtime the selected game version needs and returns the
/// path of its java executable.
pub async fn download_runtime(app: &AppHandle, game_version: &str) -> Result<String, String> {
    let runtime_directory = directory::get_runtime_directory();
    let runtime_meta_path = runtime_directory.join("runtime_meta.json");

    let mut java_executable: String = "".to_string();

    let runtime_meta = fs::read_to_string(runtime_meta_path).unwrap();
    let runtime_meta: serde_json::Value = serde_json::from_str(&runtime_meta).unwrap();

    // os and archetecture based on the current system
    let os = match std::env::consts::OS {
        "linux" => "linux",
        "macos" => "mac-os",
        "windows" => "windows",
        _ => panic!("Unsupported OS"),
    };

    let arch = match std::env::consts::ARCH {
        "x86" => "x86",
        "x86_64" => "x64",
        "aarch64" => "arm64",
        _ => panic!("Unsupported architecture"),
    };

    let platform = match (os, arch) {
        ("windows", "arm64") => "windows-arm64",
        ("windows", "x64") => "windows-x64",
        ("windows", "x86") => "windows-x86",
        ("linux", "x64") => "linux",
        ("mac-os", "x64") => "mac-os",
        ("mac-os", "arm64") => "mac-os-arm64",
        _ => panic!("Unsupported architecture! Using default runtime"),
    };

    let component = runtime_component(game_version);
    println!("Using Java runtime {}", component);

    // Each runtime keeps its own manifest and folder so switching Minecraft
    // versions never reuses the wrong Java. gamma keeps its original paths.
    let (manifest_name, folder_name) = if component == "java-runtime-gamma" {
        ("selected_runtime_manifest.json".to_string(), "jre".to_string())
    } else {
        (
            format!("selected_runtime_manifest_{}.json", component),
            format!("jre-{}", component),
        )
    };

    let selected_runtime_manifest_file = directory::get_runtime_directory().join(manifest_name);

    if !selected_runtime_manifest_file.exists() {
        let selected_runtime_manifest_url = runtime_meta[platform][&component][0]["manifest"]["url"]
            .as_str()
            .ok_or_else(|| format!("No {} runtime is published for {}", component, platform))?
            .to_string();
        let selected_runtime_manifest = reqwest::get(selected_runtime_manifest_url)
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        fs::write(
            selected_runtime_manifest_file.clone(),
            selected_runtime_manifest,
        )
        .unwrap();
    }

    let selected_runtime_manifest = fs::read_to_string(selected_runtime_manifest_file).unwrap();
    let selected_runtime_manifest: mojang_runtime_gamma_meta_json::Root =
        serde_json::from_str(&selected_runtime_manifest).unwrap();

    let runtime_download_dir = directory::get_runtime_directory().join(folder_name);

    if !runtime_download_dir.exists() {
        fs::create_dir(&runtime_download_dir).unwrap();
    }

    let length = selected_runtime_manifest.files.len();
    let max_threads = 3;
    let mut i = 0;

    println!("Downloading runtime for {} {}", os, arch);
    println!("Total files: {}", length);
    println!("Multi-threading with {} threads", max_threads);

    while i < length {
        let mut handles = vec![];

        while handles.len() < max_threads && i < length {
            let files = selected_runtime_manifest.files.clone();
            let (path, file) = files.into_iter().nth(i).unwrap();
            let executable = file.executable.unwrap_or(false);

            if file.root_type.clone().unwrap() == "directory" {
                let mut file_path = runtime_download_dir.clone();
                let parts = path.split('/').collect::<Vec<&str>>();
                for part in parts {
                    file_path = file_path.join(part);

                    if !file_path.exists() {
                        fs::create_dir(&file_path).unwrap();
                    }
                }

                i += 1;
                continue;
            }

            if file.root_type.unwrap() == "link" {
                i += 1;
                continue;
            }

            let mut file_path = runtime_download_dir.clone();
            let mut parts = path.split('/').collect::<Vec<&str>>();
            let file_name = parts[parts.len() - 1];
            parts.pop();
            for part in parts {
                file_path = file_path.join(part);

                if !file_path.exists() {
                    fs::create_dir(&file_path).unwrap();
                }
            }

            file_path = file_path.join(file_name);

            if file_name.ends_with("java.exe") || file_name.ends_with("java") {
                java_executable = file_path.to_str().unwrap().to_string();
            }

            if !file_path.exists() {
                println!("Downloading file: {}", path);

                let file_url = file.downloads.unwrap().raw.unwrap().url.unwrap().clone();

                let handle = tokio::spawn(async move {
                    let response = match reqwest::get(file_url).await {
                        Ok(response) => response,
                        Err(_) => {
                            println!("Failed to download file: {}", path);
                            return Err("Failed to download file".to_string());
                        }
                    };

                    let response = response.bytes().await.unwrap();
                    fs::write(&file_path, response).unwrap();

                    // Downloaded files are not executable by default on Linux/macOS.
                    #[cfg(unix)]
                    if executable {
                        use std::os::unix::fs::PermissionsExt;
                        let _ = fs::set_permissions(&file_path, fs::Permissions::from_mode(0o755));
                    }
                    #[cfg(not(unix))]
                    let _ = executable;

                    println!("Downloaded file: {}", path);
                    Ok(())
                });

                handles.push(handle);
            } else {
                println!("Skipping file: {}", path);
            }

            i += 1;
            app.emit_all(
                "update-progress",
                format!("JRE: {}%", util::get_percentage(i as i32, length as i32)),
            )
            .unwrap();
        }

        for handle in handles {
            match handle.await {
                Ok(_) => {}
                Err(_) => {
                    println!("Failed to download file");
                    return Err("Failed to download file".to_string());
                }
            }
        }

        println!("Downloaded {}/{} files", i, length);
    }

    Ok(java_executable)
}

pub async fn init(app: &AppHandle) {
    // Update the config
    match update_hyclyint_config(app).await {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
            app.emit_all("update-failed", "Failed to update HyClyint config")
                .unwrap();
            return;
        }
    };

    // Warn macos users
    if std::env::consts::OS == "macos" {
        app.emit_all(
            "macos-java-warning",
            "HyClyint won't use the downloaded runtime on MacOS, install the Java version your Minecraft version needs (Java 25 for 26.x, Java 21 for 1.20.5-1.21.x)",
        )
        .unwrap();
    }

    // check if the current token is valid
    let mut config = config::SHARED_CONFIG.lock().await;
    if config.is_logged_in && (!config.minecraft_token.is_empty() || !config.xbox_token.is_empty())
    {
        let profile;

        if !config.minecraft_token.is_empty() {
            let token = config.minecraft_token.clone();
            profile = auth::get_minecraft_profile(&token).await;
        } else {
            let token = config.xbox_token.clone();
            match auth::get_minecraft_token(&token).await {
                Some(token) => {
                    profile = auth::get_minecraft_profile(&token.minecraft_access_token).await;
                }
                None => {
                    config.is_logged_in = false;
                    config.username = "".to_string();
                    config.uuid = "".to_string();
                    config.minecraft_token = "".to_string();
                    config.xbox_token = "".to_string();
                    config.save_config().unwrap();
                    return;
                }
            };
        }

        if profile.is_none() {
            config.is_logged_in = false;
            config.username = "".to_string();
            config.uuid = "".to_string();
            config.minecraft_token = "".to_string();
            config.xbox_token = "".to_string();
            config.save_config().unwrap();
        } else {
            let profile = profile.unwrap();

            config.username = profile.name;
            config.uuid = profile.id.to_string();
            config.save_config().unwrap();
        }
    }

    // Create the HyClyint directories
    directory::get_hyclyint_directory();
    directory::get_versions_directory();
    directory::get_runtime_directory();
    directory::get_assets_directory();
    directory::get_libraries_directory();
    migrate_legacy_mods();

    // Download the Minecraft version manifest
    println!("Updating manifests");
    match download_minecraft_version_manifest().await {
        Ok(_) => {}
        Err(_) => {
            app.emit_all(
                "update-failed",
                "Failed to update Minecraft version manifest",
            )
            .unwrap();
            return;
        }
    };

    match download_fabric_meta().await {
        Ok(_) => {}
        Err(_) => {
            app.emit_all("update-failed", "Failed to update Fabric meta")
                .unwrap();
            return;
        }
    };

    match download_minecraft_version(&config).await {
        Ok(_) => {}
        Err(_) => {
            app.emit_all("update-failed", "Failed to update Minecraft version")
                .unwrap();
            return;
        }
    };

    match download_fabric_version(&config).await {
        Ok(_) => {}
        Err(_) => {
            app.emit_all("update-failed", "Failed to update Fabric version")
                .unwrap();
            return;
        }
    };

    match download_runtime_meta().await {
        Ok(_) => {}
        Err(_) => {
            app.emit_all("update-failed", "Failed to update runtime meta")
                .unwrap();
            return;
        }
    };

    println!("HyLauncher initialized");
}

/// Before profiles, launcher-managed mods were written to `<root>/mods`. They
/// now load from `required_mods`, so leftovers would be loaded twice and make
/// Fabric fail on duplicate mods. Runs once, guarded by a marker file, so it
/// can never touch mods users add to the default profile later.
fn migrate_legacy_mods() {
    let root = directory::get_hyclyint_directory();
    let marker = root.join(".profiles_migrated");
    if marker.exists() {
        return;
    }

    let legacy = root.join("mods");
    if let Ok(entries) = fs::read_dir(&legacy) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "jar") {
                let _ = fs::remove_file(path);
            }
        }
    }

    let _ = fs::write(marker, "");
}
