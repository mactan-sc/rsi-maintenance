use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

pub fn launcher_exe_path(prefix_path: &Path) -> PathBuf {
    prefix_path.join("drive_c/Program Files/Roberts Space Industries/RSI Launcher/RSI Launcher.exe")
}

pub fn game_path(prefix_path: &Path) -> PathBuf {
    prefix_path
        .join("drive_c/Program Files/Roberts Space Industries/StarCitizen")
}

pub fn installer_url(version: &str) -> String {
    format!(
        "https://install.robertsspaceindustries.com/rel/2/RSI%20Launcher-Setup-{version}.exe"
    )
}

fn extract_version_from_latest_yml(raw: &str) -> Option<String> {
    raw.lines()
        .find_map(|line| line.trim().strip_prefix("version:"))
        .map(str::trim)
        .map(|value| value.trim_matches('"').to_string())
}

pub fn launcher_installed(prefix_path: &Path) -> bool {
    launcher_exe_path(prefix_path).exists()
}

pub async fn install_launcher_only(
    prefix_path: PathBuf,
    progress_state: Arc<Mutex<Option<f32>>>,
) -> Result<(), String> {
    let prefix_path = prefix_path.clone();
    let launcher_path = launcher_exe_path(&prefix_path);

    if launcher_path.exists() {
        return Ok(());
    }

    let version = fetch_latest_version().await?;
    let installer_path =
        download_installer(&version, &prefix_path, progress_state.clone())
            .await?;
    install_launcher(&installer_path).await?;
    create_live_marker(&prefix_path).await?;
    Ok(())
}

pub async fn run_installed_launcher(
    prefix_path: PathBuf,
) -> Result<(), String> {
    let launcher_path = launcher_exe_path(&prefix_path);

    if !launcher_path.exists() {
        return Err(
            "Launcher is not installed yet. Install it first.".to_string()
        );
    }

    launch_launcher(&launcher_path).await
}

pub async fn fetch_latest_version() -> Result<String, String> {
    let response = reqwest::get(
        "https://install.robertsspaceindustries.com/rel/2/latest.yml",
    )
    .await
    .map_err(|err| format!("Failed to query latest launcher version: {err}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch launcher version metadata: {}",
            response.status()
        ));
    }

    let raw = response.text().await.map_err(|err| {
        format!("Failed to read launcher version metadata: {err}")
    })?;

    extract_version_from_latest_yml(&raw).ok_or_else(|| {
        format!("Unable to parse launcher version from latest metadata: {raw}")
    })
}

pub async fn download_installer(
    version: &str,
    prefix_path: &Path,
    progress_state: Arc<Mutex<Option<f32>>>,
) -> Result<PathBuf, String> {
    let installer_name = format!("RSI-Launcher-setup-{version}.exe");
    let installer_path = prefix_path.join(&installer_name);
    let installer_url = installer_url(version);

    let response = reqwest::get(&installer_url)
        .await
        .map_err(|err| format!("Failed to start launcher download: {err}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download launcher installer from {installer_url}"
        ));
    }

    let total = response.content_length().unwrap_or_default();
    let mut downloaded = 0u64;
    let mut bytes = Vec::new();

    let mut stream = response.bytes().await.map_err(|err| {
        format!("Failed to read launcher installer bytes: {err}")
    })?;

    while !stream.is_empty() {
        let chunk = stream.split_to(std::cmp::min(stream.len(), 8192));
        downloaded += chunk.len() as u64;
        bytes.extend_from_slice(&chunk);

        if total > 0 {
            let progress =
                (downloaded as f64 / total as f64).clamp(0.0, 1.0) as f32;
            let mut guard =
                progress_state.lock().unwrap_or_else(|err| err.into_inner());
            *guard = Some(progress);
        }
    }

    fs::write(&installer_path, bytes).map_err(|err| {
        format!("Failed to write launcher installer to disk: {err}")
    })?;

    let mut guard =
        progress_state.lock().unwrap_or_else(|err| err.into_inner());
    *guard = Some(1.0);

    Ok(installer_path)
}

pub async fn install_launcher(installer_path: &Path) -> Result<(), String> {
    let installer_path = installer_path.to_string_lossy().to_string();

    let output = Command::new("umu-run")
        .arg(&installer_path)
        .arg("/S")
        .env("PROTONPATH", "GE-Proton")
        .env("WINE_NO_PRIV_ELEVATION", "1")
        .env(
            "WINEDLLOVERRIDES",
            "dxwebsetup.exe,dotNetFx45_Full_setup.exe=d",
        )
        .output()
        .map_err(|err| format!("Failed to launch installer: {err}"))?;

    if !output.status.success() {
        return Err(format!(
            "Launcher installation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

pub async fn create_live_marker(prefix_path: &Path) -> Result<(), String> {
    let game_path = game_path(prefix_path);
    let live_dir = game_path.join("LIVE");

    fs::create_dir_all(&live_dir)
        .map_err(|err| format!("Failed to create game directory: {err}"))?;

    if !live_dir.join("Data.p4k").exists() {
        fs::write(live_dir.join("Data.p4k.part"), "")
            .map_err(|err| format!("Failed to create Data.p4k.part: {err}"))?;
        fs::write(live_dir.join("Data.p4k"), "")
            .map_err(|err| format!("Failed to create Data.p4k: {err}"))?;
    }

    Ok(())
}

pub async fn launch_launcher(launcher_path: &Path) -> Result<(), String> {
    let status = Command::new("umu-run")
        .arg(launcher_path)
        .status()
        .map_err(|err| format!("Failed to start RSI Launcher: {err}"))?;

    if !status.success() {
        return Err("RSI Launcher exited with an error status.".to_string());
    }

    Ok(())
}

pub fn prefix_path_from_config(config_path: &str) -> PathBuf {
    if config_path.is_empty() {
        PathBuf::from(env::var("WINEPREFIX").unwrap_or_default())
    } else {
        PathBuf::from(config_path)
    }
}
