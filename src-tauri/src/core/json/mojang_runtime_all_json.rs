use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Root {
    pub gamecore: Gamecore,
    pub linux: Gamecore,
    pub linux_i386: Gamecore,
    pub mac_os: Gamecore,
    pub mac_os_arm64: Gamecore,
    pub windows_arm64: Gamecore,
    pub windows_x64: Gamecore,
    pub windows_x86: Gamecore,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Gamecore {
    pub java_runtime_alpha: Vec<JavaRuntimeAlpha>,
    pub java_runtime_beta: Vec<JavaRuntimeAlpha>,
    pub java_runtime_delta: Vec<JavaRuntimeAlpha>,
    pub java_runtime_gamma: Vec<JavaRuntimeAlpha>,
    pub java_runtime_gamma_snapshot: Vec<JavaRuntimeAlpha>,
    pub jre_legacy: Vec<JavaRuntimeAlpha>,
    pub minecraft_java_exe: Vec<JavaRuntimeAlpha>,
}

#[derive(Serialize, Deserialize)]
pub struct JavaRuntimeAlpha {
    pub availability: Availability,
    pub manifest: Manifest,
    pub version: Version,
}

#[derive(Serialize, Deserialize)]
pub struct Availability {
    pub group: i64,
    pub progress: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Manifest {
    pub sha1: String,
    pub size: i64,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Version {
    pub name: String,
    pub released: String,
}
