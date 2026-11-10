use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use futures::lock::Mutex;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use tauri::{AppHandle, Manager};

use crate::config::SHARED_CONFIG;
use crate::core::{api, directory};

const MODRINTH_API: &str = "https://api.modrinth.com/v2/";
const CDN_HOST: &str = "cdn.modrinth.com";
const USER_AGENT: &str = "HyClyint-Launcher";
const MAX_DEPENDENCIES: usize = 24;

/// Serializes installs/removals so concurrent clicks can't corrupt `mods.json`.
static LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

// ---------------------------------------------------------------- Modrinth

#[derive(Deserialize)]
struct SearchResponse {
    hits: Vec<Hit>,
    total_hits: u32,
}

#[derive(Deserialize)]
struct Hit {
    project_id: String,
    slug: String,
    title: String,
    description: String,
    author: String,
    downloads: u64,
    icon_url: Option<String>,
}

#[derive(Deserialize)]
struct Project {
    id: String,
    slug: String,
    title: String,
    icon_url: Option<String>,
}

#[derive(Deserialize)]
struct Version {
    version_number: String,
    files: Vec<VersionFile>,
    dependencies: Vec<Dependency>,
}

#[derive(Deserialize)]
struct VersionFile {
    url: String,
    filename: String,
    #[serde(default)]
    primary: bool,
    hashes: Hashes,
}

#[derive(Deserialize)]
struct Hashes {
    #[serde(default)]
    sha512: String,
}

#[derive(Deserialize)]
struct Dependency {
    project_id: Option<String>,
    dependency_type: String,
}

fn client(timeout: u64) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(timeout))
        .build()
        .map_err(|error| error.to_string())
}

async fn get_json<T: serde::de::DeserializeOwned>(
    path: &str,
    query: &[(&str, String)],
) -> Result<T, String> {
    let response = client(20)?
        .get(format!("{}{}", MODRINTH_API, path))
        .query(query)
        .send()
        .await
        .map_err(|_| "Couldn't reach Modrinth. Check your connection.".to_string())?;

    if !response.status().is_success() {
        return Err(format!("Modrinth returned HTTP {}", response.status()));
    }

    response
        .json::<T>()
        .await
        .map_err(|_| "Modrinth sent a response we couldn't read".to_string())
}

// ------------------------------------------------------------ Local state

#[derive(Serialize, Deserialize, Clone)]
struct ModMeta {
    project_id: String,
    slug: String,
    title: String,
    icon_url: Option<String>,
    version_number: String,
    /// Always the enabled name (`x.jar`), even while disabled on disk.
    filename: String,
    /// Project ids of installed mods that pulled this one in.
    #[serde(default)]
    required_by: Vec<String>,
}

#[derive(Serialize)]
pub struct ModResult {
    id: String,
    title: String,
    description: String,
    author: String,
    downloads: u64,
    icon_url: Option<String>,
    installed: bool,
}

#[derive(Serialize)]
pub struct SearchResults {
    results: Vec<ModResult>,
    total: u32,
}

#[derive(Serialize)]
pub struct InstalledMod {
    filename: String,
    enabled: bool,
    title: String,
    icon_url: Option<String>,
    version_number: Option<String>,
    project_id: Option<String>,
    /// Titles of installed mods that need this one.
    required_by: Vec<String>,
}

struct ActiveProfile {
    id: String,
    game_version: String,
}

async fn active_profile() -> ActiveProfile {
    let config = SHARED_CONFIG.lock().await;
    let profile = config.active_profile();
    ActiveProfile {
        id: profile.id,
        game_version: profile.game_version,
    }
}

fn meta_path(profile_id: &str) -> PathBuf {
    directory::get_profile_directory(profile_id).join("mods.json")
}

fn load_meta(profile_id: &str) -> Vec<ModMeta> {
    fs::read_to_string(meta_path(profile_id))
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

fn save_meta(profile_id: &str, meta: &[ModMeta]) -> Result<(), String> {
    let content = serde_json::to_string_pretty(meta).map_err(|error| error.to_string())?;
    fs::write(meta_path(profile_id), content).map_err(|error| error.to_string())
}

/// A bare `something.jar` name; anything with a path in it is rejected so a
/// crafted name can't reach outside the mods folder.
fn safe_jar_name(name: &str) -> Result<String, String> {
    let bare = Path::new(name).file_name().and_then(|n| n.to_str());
    match bare {
        Some(bare) if bare == name && name.to_lowercase().ends_with(".jar") && !name.contains("..") => {
            Ok(name.to_string())
        }
        _ => Err("Invalid mod file name".to_string()),
    }
}

fn disabled_name(filename: &str) -> String {
    format!("{}.disabled", filename)
}

/// Names of mods the launcher itself installs (slugs or ids). Installing them
/// again would load the same mod twice and crash Fabric.
async fn required_names() -> HashSet<String> {
    match api::get_global_config(false).await {
        Ok(config) => config
            .required_mods
            .iter()
            .map(|m| m.name.to_lowercase())
            .collect(),
        Err(_) => HashSet::new(),
    }
}

// --------------------------------------------------------------- Commands

#[tauri::command]
pub async fn search_mods(query: String, offset: u32) -> Result<SearchResults, String> {
    let profile = active_profile().await;
    let facets = serde_json::json!([
        ["project_type:mod"],
        ["categories:fabric"],
        [format!("versions:{}", profile.game_version)]
    ])
    .to_string();

    let response: SearchResponse = get_json(
        "search",
        &[
            ("query", query.trim().to_string()),
            ("facets", facets),
            ("limit", "20".to_string()),
            ("offset", offset.to_string()),
        ],
    )
    .await?;

    let installed: HashSet<String> = load_meta(&profile.id)
        .into_iter()
        .map(|m| m.project_id)
        .collect();
    let required = required_names().await;

    Ok(SearchResults {
        total: response.total_hits,
        results: response
            .hits
            .into_iter()
            .map(|hit| ModResult {
                installed: installed.contains(&hit.project_id)
                    || required.contains(&hit.slug.to_lowercase())
                    || required.contains(&hit.project_id.to_lowercase()),
                id: hit.project_id,
                title: hit.title,
                description: hit.description,
                author: hit.author,
                downloads: hit.downloads,
                icon_url: hit.icon_url,
            })
            .collect(),
    })
}

#[tauri::command]
pub async fn install_mod(app: AppHandle, project_id: String) -> Result<(), String> {
    let _guard = LOCK.lock().await;
    let profile = active_profile().await;
    let mods_directory = directory::get_profile_mods_directory(&profile.id);
    let required = required_names().await;

    let mut meta = load_meta(&profile.id);
    let mut queue: Vec<(String, Option<String>)> = vec![(project_id, None)];
    let mut seen: HashSet<String> = HashSet::new();

    while let Some((wanted, parent)) = queue.pop() {
        if !seen.insert(wanted.to_lowercase()) {
            continue;
        }
        if seen.len() > MAX_DEPENDENCIES {
            return Err("This mod pulls in too many dependencies".to_string());
        }

        let project: Project = get_json(&format!("project/{}", wanted), &[]).await?;
        let is_root = parent.is_none();

        // Already provided by the launcher: nothing to do.
        if required.contains(&project.slug.to_lowercase()) || required.contains(&project.id.to_lowercase()) {
            if is_root {
                return Err(format!("{} is already included with HyClyint", project.title));
            }
            continue;
        }

        // Already installed: just record who else needs it.
        if let Some(existing) = meta.iter_mut().find(|m| m.project_id == project.id) {
            if let Some(parent) = &parent {
                if !existing.required_by.contains(parent) {
                    existing.required_by.push(parent.clone());
                }
                continue;
            }
            return Err(format!("{} is already installed", project.title));
        }

        let _ = app.emit_all("mod-progress", format!("Installing {}", project.title));

        let versions: Vec<Version> = get_json(
            &format!("project/{}/version", project.id),
            &[
                ("loaders", r#"["fabric"]"#.to_string()),
                ("game_versions", serde_json::json!([profile.game_version]).to_string()),
            ],
        )
        .await?;

        let version = versions.into_iter().next().ok_or_else(|| {
            if is_root {
                format!("{} has no Fabric version for Minecraft {}", project.title, profile.game_version)
            } else {
                format!(
                    "A required dependency, {}, has no Fabric version for Minecraft {}",
                    project.title, profile.game_version
                )
            }
        })?;

        let file = version
            .files
            .iter()
            .find(|f| f.primary)
            .or_else(|| version.files.first())
            .ok_or_else(|| format!("{} has no downloadable file", project.title))?;

        let filename = safe_jar_name(&file.filename)?;
        let url = reqwest::Url::parse(&file.url).map_err(|_| "Bad download link".to_string())?;
        if url.scheme() != "https" || url.host_str() != Some(CDN_HOST) {
            return Err("Refusing to download from an unexpected host".to_string());
        }

        if mods_directory.join(&filename).exists() || mods_directory.join(disabled_name(&filename)).exists() {
            return Err(format!("A file named {} already exists in this profile", filename));
        }

        let bytes = client(300)?
            .get(url)
            .send()
            .await
            .map_err(|_| format!("Failed to download {}", project.title))?
            .bytes()
            .await
            .map_err(|_| format!("Failed to download {}", project.title))?;

        if !file.hashes.sha512.is_empty() {
            let digest = format!("{:x}", Sha512::digest(&bytes));
            if !digest.eq_ignore_ascii_case(&file.hashes.sha512) {
                return Err(format!("{} failed its integrity check", project.title));
            }
        }

        let temp = mods_directory.join(format!("{}.part", filename));
        fs::write(&temp, &bytes).map_err(|error| error.to_string())?;
        fs::rename(&temp, mods_directory.join(&filename)).map_err(|error| error.to_string())?;

        meta.push(ModMeta {
            project_id: project.id.clone(),
            slug: project.slug,
            title: project.title,
            icon_url: project.icon_url,
            version_number: version.version_number,
            filename,
            required_by: parent.clone().into_iter().collect(),
        });
        // Persist after every jar so a failure halfway keeps the bookkeeping true.
        save_meta(&profile.id, &meta)?;

        for dependency in version.dependencies {
            if dependency.dependency_type == "required" {
                if let Some(dep_id) = dependency.project_id {
                    queue.push((dep_id, Some(project.id.clone())));
                }
            }
        }
    }

    let _ = app.emit_all("mod-progress", "");
    Ok(())
}

#[tauri::command]
pub async fn list_mods() -> Result<Vec<InstalledMod>, String> {
    let profile = active_profile().await;
    let mods_directory = directory::get_profile_mods_directory(&profile.id);
    let meta = load_meta(&profile.id);

    let mut mods: Vec<InstalledMod> = Vec::new();

    for entry in fs::read_dir(&mods_directory).map_err(|error| error.to_string())?.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let (filename, enabled) = if let Some(base) = name.strip_suffix(".jar.disabled") {
            (format!("{}.jar", base), false)
        } else if name.to_lowercase().ends_with(".jar") {
            (name.clone(), true)
        } else {
            continue;
        };

        let known = meta.iter().find(|m| m.filename == filename);
        mods.push(InstalledMod {
            enabled,
            title: known.map(|m| m.title.clone()).unwrap_or_else(|| filename.clone()),
            icon_url: known.and_then(|m| m.icon_url.clone()),
            version_number: known.map(|m| m.version_number.clone()),
            project_id: known.map(|m| m.project_id.clone()),
            required_by: known
                .map(|m| {
                    m.required_by
                        .iter()
                        .filter_map(|id| meta.iter().find(|o| &o.project_id == id))
                        .map(|o| o.title.clone())
                        .collect()
                })
                .unwrap_or_default(),
            filename,
        });
    }

    mods.sort_by_key(|m| m.title.to_lowercase());
    Ok(mods)
}

#[tauri::command]
pub async fn set_mod_enabled(filename: String, enabled: bool) -> Result<(), String> {
    let _guard = LOCK.lock().await;
    let filename = safe_jar_name(&filename)?;
    let profile = active_profile().await;
    let dir = directory::get_profile_mods_directory(&profile.id);

    let (from, to) = if enabled {
        (dir.join(disabled_name(&filename)), dir.join(&filename))
    } else {
        (dir.join(&filename), dir.join(disabled_name(&filename)))
    };

    if !from.exists() {
        return Ok(()); // already in the requested state
    }

    fs::rename(from, to).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn remove_mod(filename: String) -> Result<(), String> {
    let _guard = LOCK.lock().await;
    let filename = safe_jar_name(&filename)?;
    let profile = active_profile().await;
    let dir = directory::get_profile_mods_directory(&profile.id);
    let mut meta = load_meta(&profile.id);

    if let Some(target) = meta.iter().find(|m| m.filename == filename) {
        let needed_by: Vec<String> = target
            .required_by
            .iter()
            .filter_map(|id| meta.iter().find(|m| &m.project_id == id))
            .map(|m| m.title.clone())
            .collect();

        if !needed_by.is_empty() {
            return Err(format!(
                "{} is required by {}. Remove that first.",
                target.title,
                needed_by.join(", ")
            ));
        }
    }

    for path in [dir.join(&filename), dir.join(disabled_name(&filename))] {
        if path.exists() {
            fs::remove_file(path).map_err(|error| error.to_string())?;
        }
    }

    if let Some(removed) = meta.iter().find(|m| m.filename == filename).map(|m| m.project_id.clone()) {
        meta.retain(|m| m.project_id != removed);
        for other in meta.iter_mut() {
            other.required_by.retain(|id| id != &removed);
        }
        save_meta(&profile.id, &meta)?;
    }

    Ok(())
}
