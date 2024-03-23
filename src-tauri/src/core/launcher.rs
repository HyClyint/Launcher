use std::{fs, os};

use tauri::{AppHandle, Manager, Window};

use crate::{
    config,
    core::{directory, downloader, json::minecraft_version_json},
    util,
};

#[tauri::command]
pub async fn launch_client(app: AppHandle, window: Window) -> Result<(), String> {
    let config = config::SHARED_CONFIG.lock().await;
    let selected_version = config.selected_version.clone();

    println!("Launching version: {}", selected_version);

    println!("Downloading runtime!");
    downloader::download_runtime_gamma(&app).await;

    println!("Downloading assets!");
    downloader::download_assets_minecraft(&config, &app).await;

    println!("Downloading libraries!");
    downloader::download_libraries(&config, &app).await;
    downloader::download_libraries_fabric(&config, &app).await;

    println!("Downloading mods!");
    downloader::download_mods(&config, &app).await;

    println!("Downloading jar!");
    downloader::download_minecraft_jar(&config, &app).await;

    app.emit_all("update_progress", "Launching").unwrap();
    println!("Launching Minecraft!");
    let jre_path = directory::get_runtime_directory().join("jre");
    let java_path = jre_path.join("bin").join("java");

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
        classpath.push_str(";");
    }

    // Adds {selected_version}.jar into the classpath
    classpath.push_str(
        directory::get_version_directory(selected_version.clone().as_str())
            .join(format!("{}.jar", selected_version.clone()))
            .to_str()
            .unwrap(),
    );

    println!("{}", classpath);

    let process = match std::env::consts::OS {
        "windows" => {
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
                .arg("-cp")
                .arg(classpath)
                .arg("net.fabricmc.loader.impl.launch.knot.KnotClient")
                .arg("--username")
                .arg(config.username.clone())
                .arg("--version")
                .arg(selected_version.clone())
                .arg("--gameDir")
                .arg(directory::get_hyclyint_directory().to_str().unwrap())
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
                .current_dir(directory::get_hyclyint_directory())
                .creation_flags(0x08000000)
                .spawn()
        },

        _ => std::process::Command::new(java_path)
            .arg("-Xms2048M")
            .arg("-Xmx2048M")
            .arg("-XX:+UnlockExperimentalVMOptions")
            .arg("-XX:+UseG1GC")
            .arg("-XX:G1NewSizePercent=20")
            .arg("-XX:G1ReservePercent=20")
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
            .arg("-cp")
            .arg(classpath)
            .arg("net.fabricmc.loader.impl.launch.knot.KnotClient")
            .arg("--username")
            .arg(config.username.clone())
            .arg("--version")
            .arg(selected_version.clone())
            .arg("--gameDir")
            .arg(directory::get_hyclyint_directory().to_str().unwrap())
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
            .current_dir(directory::get_hyclyint_directory())
            .spawn(),
    };

    config.save_config().unwrap();

    if let Ok(mut process) = process {
        window.hide().unwrap();
        process.wait().unwrap();
        window.show().unwrap();
        app.emit_all("game-closed", "Game closed").unwrap();
    }

    Ok(())
}
