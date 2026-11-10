use crate::config::Config;
use std::path::PathBuf;
use std::{fs, os};
use tauri::{AppHandle, Manager, Window};

use crate::{
    config,
    core::{directory, downloader, json::minecraft_version_json},
    util,
};

#[cfg(target_os = "windows")]
async fn get_process(
    java_path: &String,
    classpath: &String,
    temp_path: &PathBuf,
    selected_version: &String,
    config: &Config,
    version_manifest: &minecraft_version_json::Root,
) -> std::io::Result<std::process::Child> {
    use os::windows::process::CommandExt;

    std::process::Command::new(java_path)
        .arg("-Xms2048M")
        .arg("-Xmx2048M")
        .arg("-XX:+UnlockExperimentalVMOptions")
        .arg("-XX:+UseG1GC")
        .arg("-XX:G1ReservePercent=20")
        .arg("-XX:G1NewSizePercent=20")
        .arg("-XX:MaxGCPauseMillis=50")
        .arg("-XX:G1HeapRegionSize=32M")
        .arg("-Dlog4j2.formatMsgNoLookups=true")
        .arg("-DFabricMcEmu=net.minecraft.client.main.Main")
        .arg("-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump")
        .arg(format!("-Djava.library.path={}", temp_path.to_str().unwrap()))
        .arg(format!("-Djna.tmpdir={}", temp_path.to_str().unwrap()))
        .arg(format!("-Dorg.lwjgl.system.SharedLibraryExtractPath={}", temp_path.to_str().unwrap()))
        .arg(format!("-Dio.netty.native.workdir={}", temp_path.to_str().unwrap()))
        .arg("-Dminecraft.launcher.brand=hylauncher")
        .arg("-Dminecraft.launcher.version=1.0.0")
        .arg(format!(
            "-Dfabric.addMods={}",
            directory::get_required_mods_directory(selected_version).to_str().unwrap()
        ))
        .arg("-cp")
        .arg(classpath)
        .arg("net.fabricmc.loader.impl.launch.knot.KnotClient")
        .arg("--username")
        .arg(config.username.clone())
        .arg("--version")
        .arg(selected_version.clone())
        .arg("--gameDir")
        .arg(directory::get_profile_directory(&config.active_profile().id).to_str().unwrap())
        .arg("--assetsDir")
        .arg(directory::get_assets_directory().to_str().unwrap())
        .arg("--assetIndex")
        .arg(version_manifest.asset_index.id.clone())
        .arg("--uuid")
        .arg(match config.uuid.is_empty() {
            true => uuid::Uuid::new_v4().to_string(),
            false => config.uuid.clone(),
        })
        .arg("--accessToken")
        .arg(match config.minecraft_token.is_empty() {
            true => "aeef7bc935f9420eb6314dea7ad7e1e5".to_string(),
            false => config.minecraft_token.clone(),
        })
        .arg("--xuid")
        .arg(match config.xbox_token.clone().is_empty() {
            true => "0".to_string(),
            false => config.xbox_token.clone(),
        })
        .arg("--userType")
        .arg(match config.minecraft_token.is_empty() {
            true => "mojang",
            false => "microsoft",
        })
        .arg("--versionType")
        .arg("release")
        .current_dir(directory::get_profile_directory(&config.active_profile().id))
        .creation_flags(0x08000000)
        .spawn()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
async fn get_process(
    java_path: &String,
    classpath: &String,
    temp_path: &PathBuf,
    selected_version: &String,
    config: &Config,
    version_manifest: &minecraft_version_json::Root,
) -> std::io::Result<std::process::Child> {
    std::process::Command::new(java_path)
        .arg("-Xms2048M")
        .arg("-Xmx2048M")
        .arg("-XX:+UnlockExperimentalVMOptions")
        .arg("-XX:+UseG1GC")
        .arg("-XX:G1ReservePercent=20")
        .arg("-XX:G1NewSizePercent=20")
        .arg("-XX:MaxGCPauseMillis=50")
        .arg("-XX:G1HeapRegionSize=32M")
        .arg("-Dlog4j2.formatMsgNoLookups=true")
        .arg("-DFabricMcEmu=net.minecraft.client.main.Main")
        .arg("-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump")
        .arg(format!("-Djava.library.path={}", temp_path.to_str().unwrap()))
        .arg(format!("-Djna.tmpdir={}", temp_path.to_str().unwrap()))
        .arg(format!("-Dorg.lwjgl.system.SharedLibraryExtractPath={}", temp_path.to_str().unwrap()))
        .arg(format!("-Dio.netty.native.workdir={}", temp_path.to_str().unwrap()))
        .arg("-Dminecraft.launcher.brand=hylauncher")
        .arg("-Dminecraft.launcher.version=1.0.0")
        .arg(format!(
            "-Dfabric.addMods={}",
            directory::get_required_mods_directory(selected_version).to_str().unwrap()
        ))
        .arg("-cp")
        .arg(classpath)
        .arg("net.fabricmc.loader.impl.launch.knot.KnotClient")
        .arg("--username")
        .arg(config.username.clone())
        .arg("--version")
        .arg(selected_version.clone())
        .arg("--gameDir")
        .arg(directory::get_profile_directory(&config.active_profile().id).to_str().unwrap())
        .arg("--assetsDir")
        .arg(directory::get_assets_directory().to_str().unwrap())
        .arg("--assetIndex")
        .arg(version_manifest.asset_index.id.clone())
        .arg("--uuid")
        .arg(match config.uuid.is_empty() {
            true => uuid::Uuid::new_v4().to_string(),
            false => config.uuid.clone(),
        })
        .arg("--accessToken")
        .arg(match config.minecraft_token.is_empty() {
            true => "aeef7bc935f9420eb6314dea7ad7e1e5".to_string(),
            false => config.minecraft_token.clone(),
        })
        .arg("--xuid")
        .arg(match config.xbox_token.clone().is_empty() {
            true => "0".to_string(),
            false => config.xbox_token.clone(),
        })
        .arg("--userType")
        .arg(match config.minecraft_token.is_empty() {
            true => "mojang",
            false => "microsoft",
        })
        .arg("--versionType")
        .arg("release")
        .current_dir(directory::get_profile_directory(&config.active_profile().id))
        .spawn()
}

#[tauri::command]
pub async fn launch_client(app: AppHandle, window: Window) -> Result<(), String> {
    let config = config::SHARED_CONFIG.lock().await;
    let selected_version = config.selected_version.clone();
    
    println!("Launching version: {}", selected_version);

    // The version json files are only fetched at startup, so a version picked
    // afterwards would have none and the later steps would fail.
    let version_directory = directory::get_version_directory(&selected_version);
    if !version_directory.join(format!("{}.json", selected_version)).exists() {
        if downloader::download_minecraft_version(&config).await.is_err() {
            app.emit_all("update-failed", "Failed to download the Minecraft version")
                .unwrap();
            return Err("Failed to download the Minecraft version".to_string());
        }
    }
    if !version_directory.join(format!("{}-fabric.json", selected_version)).exists() {
        if downloader::download_fabric_version(&config).await.is_err() {
            app.emit_all("update-failed", "Failed to download the Fabric version")
                .unwrap();
            return Err("Failed to download the Fabric version".to_string());
        }
    }

    println!("Downloading runtime!");
    let java_path: String;

    if std::env::consts::OS == "macos" {
        println!("HyClyint won't use the downloaded runtime on MacOS, install the Java version your Minecraft version needs (Java 25 for 26.x, Java 21 for 1.20.5-1.21.x)");
        java_path = "java".to_string();
    } else {
        match downloader::download_runtime(&app, &selected_version).await {
            Ok(path) => {
                java_path = path;
            }
            Err(_) => {
                app.emit_all("update-failed", "Failed to download runtime")
                    .unwrap();
                return Err("Failed to download runtime".to_string());
            }
        };
    }

    println!("Java executable at {}", java_path);

    println!("Downloading assets!");
    match downloader::download_assets_minecraft(&config, &app).await {
        Ok(_) => {}
        Err(_) => {
            app.emit_all("update-failed", "Failed to download assets")
                .unwrap();
            return Err("Failed to download assets".to_string());
        }
    };

    println!("Downloading libraries!");
    match downloader::download_libraries(&config, &app).await {
        Ok(_) => {}
        Err(_) => {
            app.emit_all("update-failed", "Failed to download libraries")
                .unwrap();
            return Err("Failed to download libraries".to_string());
        }
    };

    match downloader::download_libraries_fabric(&config, &app).await {
        Ok(_) => {}
        Err(_) => {
            app.emit_all("update-failed", "Failed to download fabric libraries")
                .unwrap();
            return Err("Failed to download fabric libraries".to_string());
        }
    };

    println!("Downloading mods!");
    match downloader::download_mods(&config, &app).await {
        Ok(_) => {}
        Err(error) => {
            app.emit_all("update-failed", error.clone()).unwrap();
            return Err(error);
        }
    };

    println!("Downloading jar!");
    match downloader::download_minecraft_jar(&config, &app).await {
        Ok(_) => {}
        Err(_) => {
            app.emit_all("update-failed", "Failed to download jar")
                .unwrap();
            return Err("Failed to download jar".to_string());
        }
    };

    app.emit_all("update_progress", "Launching").unwrap();
    println!("Launching Minecraft!");

    let temp_path = std::env::temp_dir().join("hylauncher").join("natives");
    fs::create_dir_all(temp_path.clone()).unwrap();

    // load the version manifest
    let version_manifest = fs::read_to_string(
        directory::get_version_directory(selected_version.as_str())
            .join(format!("{}.json", selected_version)),
    )
    .unwrap();
    let version_manifest: minecraft_version_json::Root =
        serde_json::from_str(version_manifest.as_str()).unwrap();

    // get all the .jar files in the libraries folder recursively
    let libraries = util::get_jars_in_directory(&directory::get_libraries_directory());

    let mut classpath = String::new();
    for library in libraries {
        classpath.push_str(&library.to_str().unwrap());
        classpath.push_str(if cfg!(windows) { ";" } else { ":" });
    }

    // Adds {selected_version}.jar into the classpath
    classpath.push_str(
        directory::get_version_directory(selected_version.clone().as_str())
            .join(format!("{}.jar", selected_version.clone()))
            .to_str()
            .unwrap(),
    );

    println!("{}", classpath);

    let process = get_process(
        &java_path,
        &classpath,
        &temp_path,
        &selected_version,
        &config,
        &version_manifest,
    )
    .await;

    config.save_config().unwrap();

    if let Ok(mut process) = process {
        window.hide().unwrap();
        process.wait().unwrap();
        window.show().unwrap();
        app.emit_all("game-closed", "Game closed").unwrap();
    }

    Ok(())
}
