use std::fs;

use tauri::{AppHandle, Manager};

use crate::{auth, config, util};

use super::{
    directory,
    json::{
        fabric_meta, fabric_version_json, hyclyint_config, minecraft_asset_index,
        minecraft_version_json, modrinth_version, mojang_runtime_all_json,
        mojang_runtime_gamma_meta_json, version_manifest,
    },
};

const MINECRAFT_VERSION_MANIFEST_URL: &str =
    "https://launchermeta.mojang.com/mc/game/version_manifest.json";
const MINECRAFT_ASSET_DOWNLOAD_URL: &str = "https://resources.download.minecraft.net/";
const MINECRAFT_RUNTIME_META_DOWNLOAD_URL: &str = "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";
const FABRIC_META_DOWNLOAD_URL: &str = "https://meta.fabricmc.net/v2/versions/loader/";
const HYCLYINT_URL: &str = "https://raw.githubusercontent.com/HyClyint/launcher-data/main/";
const MODRINTH_API: &str = "https://api.modrinth.com/v2/";

pub async fn update_hyclyint_config(app: &AppHandle) -> Result<(), String> {
    let hyclyint_config_url = format!("{}config.json", HYCLYINT_URL);

    let response = match reqwest::get(hyclyint_config_url)
        .await {
        Ok(response) => response,
        Err(_) => {
            println!("Failed to download HyClyint config");
            return Err("Failed to download HyClyint config".to_string());
        }
    };

    let response = response.text().await.unwrap();
    let hyclyint_config: hyclyint_config::Root = serde_json::from_str(&response).unwrap();

    let mut config = config::SHARED_CONFIG.lock().await;

    if config.fabric_version != hyclyint_config.fabric_version {
        fs::remove_dir_all(directory::get_versions_directory()).unwrap();
        fs::remove_dir_all(directory::get_libraries_directory()).unwrap();
    }

    if config.hyclyint_version != hyclyint_config.hyclyint_version {
        fs::remove_dir_all(directory::get_mods_directory()).unwrap();
    }

    let version_info = app.package_info().version.clone();
    let version_string = format!(
        "{}.{}.{}",
        version_info.major, version_info.minor, version_info.patch
    );

    if version_string != hyclyint_config.hyclyint_launcher_version {
        app.emit_all("update-available", format!("Current launcher version: {} is outdated! Please update to version: {} to continue using HyClyint!", version_string, hyclyint_config.hyclyint_launcher_version)).unwrap();
    }

    config.fabric_version = hyclyint_config.fabric_version;
    config.hyclyint_version = hyclyint_config.hyclyint_version;

    config.save_config().unwrap();

    Ok(())
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
    let response = match client
        .get(&selected_version.url)
        .send()
        .await {
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

    let response = match reqwest::get(MINECRAFT_RUNTIME_META_DOWNLOAD_URL)
        .await {
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

pub async fn download_fabric_version(config: &config::Config) -> Result<(), String>{
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
            println!("Failed to download fabric version: {}", selected_fabric_version.version);
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

    let mods_directory = directory::get_mods_directory();

    let response = match reqwest::get(format!("{}{}", HYCLYINT_URL, "config.json"))
        .await {
        Ok(response) => response,
        Err(_) => {
            println!("Failed to download HyClyint config");
            return Err("Failed to download HyClyint config".to_string());
        }
    };

    let response = response.text().await.unwrap();
    let remote_config: hyclyint_config::Root = serde_json::from_str(&response).unwrap();

    println!("Downloading HyClyint v{}", remote_config.hyclyint_version);
    let hyclyint_url = format!(
        "{}/hyclyint-{}.jar",
        HYCLYINT_URL, remote_config.hyclyint_version
    );
    let hyclyint_path =
        mods_directory.join(format!("hyclyint-{}.jar", remote_config.hyclyint_version));

    let response = match reqwest::get(hyclyint_url).await {
        Ok(response) => response,
        Err(_) => {
            println!(
                "Failed to download HyClyint v{}",
                remote_config.hyclyint_version
            );
            return Err("Failed to download HyClyint".to_string());
        }
    };

    let response = response.bytes().await.unwrap();
    fs::write(hyclyint_path, response).unwrap();
    println!(
        "Downloaded HyClyint v{}. Downloading required mods!",
        remote_config.hyclyint_version
    );

    let mut i = 0;
    let length = remote_config.required_mods.len();

    println!("Total mods: {}", length);

    for required_mod in remote_config.required_mods {
        let modrinth_response = match reqwest::get(format!(
            "{}project/{}/version",
            MODRINTH_API, required_mod.name
        ))
        .await {
            Ok(response) => response,
            Err(_) => {
                println!("Failed to download mod: {}. {}/{}", required_mod.name, i, length);
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
            .unwrap();

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
                println!("Failed to download mod: {}. {}/{}", required_mod.name, i, length);
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

pub async fn download_runtime_gamma(app: &AppHandle) -> Result<(), String> {
    let runtime_directory = directory::get_runtime_directory();
    let runtime_meta_path = runtime_directory.join("runtime_meta.json");

    let runtime_meta = fs::read_to_string(runtime_meta_path).unwrap();
    let runtime_meta: mojang_runtime_all_json::Root = serde_json::from_str(&runtime_meta).unwrap();

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

    let selected_runtime;

    if os == "windows" && arch == "arm64" {
        selected_runtime = runtime_meta.windows_arm64;
    } else if os == "windows" && arch == "x64" {
        selected_runtime = runtime_meta.windows_x64;
    } else if os == "windows" && arch == "x86" {
        selected_runtime = runtime_meta.windows_x86;
    } else if os == "linux" && arch == "x64" {
        selected_runtime = runtime_meta.linux;
    } else if os == "mac-os" && arch == "x64" {
        selected_runtime = runtime_meta.mac_os;
    } else if os == "mac-os" && arch == "arm64" {
        selected_runtime = runtime_meta.mac_os_arm64;
    } else {
        panic!("Unsupported architecture! Using default runtime");
    }

    let selected_runtime_manifest_file =
        directory::get_runtime_directory().join("selected_runtime_manifest.json");

    if !selected_runtime_manifest_file.exists() {
        let selected_runtime_manifest_url =
            selected_runtime.java_runtime_gamma[0].manifest.url.clone();
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

    let runtime_download_dir = directory::get_runtime_directory().join("jre");

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
                    fs::write(file_path, response).unwrap();

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

    Ok(())
}

pub async fn init(app: &AppHandle) {
    // Update the config
    match update_hyclyint_config(app).await {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
            app.emit_all("update-failed", "Failed to update HyClyint config").unwrap();
            return;
        }
    };

    // check if the current token is valid
    let mut config = config::SHARED_CONFIG.lock().await;
    if config.is_logged_in && !config.minecraft_token.is_empty() {
        let profile = auth::get_minecraft_profile(&config.minecraft_token).await;

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
    directory::get_mods_directory();

    // Download the Minecraft version manifest
    println!("Updating manifests");
    match download_minecraft_version_manifest().await {
        Ok(_) => {}
        Err(_) => {
            app.emit_all("update-failed", "Failed to update Minecraft version manifest").unwrap();
            return;
        }
    };

    match download_fabric_meta().await {
        Ok(_) => {}
        Err(_) => {
            app.emit_all("update-failed", "Failed to update Fabric meta").unwrap();
            return;
        }
    };

    match download_minecraft_version(&config).await {
        Ok(_) => {}
        Err(_) => {
            app.emit_all("update-failed", "Failed to update Minecraft version").unwrap();
            return;
        }
    };

    match download_fabric_version(&config).await {
        Ok(_) => {}
        Err(_) => {
            app.emit_all("update-failed", "Failed to update Fabric version").unwrap();
            return;
        }
    };

    match download_runtime_meta().await {
        Ok(_) => {}
        Err(_) => {
            app.emit_all("update-failed", "Failed to update runtime meta").unwrap();
            return;
        }
    };

    println!("HyLauncher initialized");
}
