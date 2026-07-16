use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap};
use std::env;
use std::fs;
use std::path::Path;
use unic_langid::LanguageIdentifier;
use xdg::BaseDirectories;

fn parse_lang(raw: &str) -> Option<LanguageIdentifier> {
    raw.split('@')
        .next()?
        .split('.')
        .next()?
        .replace('_', "-")
        .parse()
        .ok()
}

pub fn detect_lang() -> LanguageIdentifier {
    env::var("LANG")
        .ok()
        .and_then(|l| parse_lang(&l))
        .or_else(|| sys_locale::get_locale().and_then(|l| parse_lang(&l)))
        .unwrap_or_else(|| "en-US".parse().unwrap())
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ConfigFile {
    pub settings: AppConfig,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
}

fn default_environment() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("__GL_SHADER_DISK_CACHE".to_string(), "1".to_string()),
        (
            "__GL_SHADER_DISK_CACHE_SIZE".to_string(),
            "10737418240".to_string(),
        ),
        (
            "__GL_SHADER_DISK_CACHE_SKIP_CLEANUP".to_string(),
            "1".to_string(),
        ),
        ("MESA_SHADER_CACHE_MAX_SIZE".to_string(), "10G".to_string()),
        ("STEAM_LINUX_RUNTIME_LOG".to_string(), "0".to_string()),
        ("STEAM_LINUX_RUNTIME_VERBOSE".to_string(), "0".to_string()),
        ("PROTON_LOG".to_string(), "0".to_string()),
        ("MANGOHUD".to_string(), "0".to_string()),
        ("PROTON_ENABLE_WAYLAND".to_string(), "0".to_string()),
        ("MANGOHUD_CONFIG".to_string(), "blacklist=rsi-maintenance".to_string()),
    ])
}

fn default_config(game_path: String) -> ConfigFile {
    ConfigFile {
        settings: AppConfig { game_path },
        environment: default_environment(),
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AppConfig {
    pub game_path: String,
}

pub fn apply_config_to_environment(
    config: &AppConfig,
    environment: &BTreeMap<String, String>,
) {
    for (key, value) in environment {
        if !value.is_empty() {
            env::set_var(key, value);
        }
    }

    if !config.game_path.is_empty() {
        env::set_var("WINEPREFIX", &config.game_path);
    }
}

pub fn load_environment_from_current_process() {
    for (key, value) in env::vars() {
        if key.starts_with("RSI_")
            || key.starts_with("WINE")
            || key.starts_with("PROTON")
        {
            env::set_var(&key, &value);
        }
    }
}

// Variant that accepts a custom dialog title. Useful when the title should come
// from the application's i18n state.
pub fn load_config_without_prompt() -> AppConfig {
    let xdg_dirs = BaseDirectories::with_prefix("starcitizen-lug");
    _ = xdg_dirs.create_config_directory("");

    load_environment_from_current_process();

    let config_path = xdg_dirs
        .get_config_file(Path::new("rsi_maintenance.toml"))
        .unwrap();

    if !&config_path.exists() {
        let game_path = env::var("WINEPREFIX").unwrap_or_default();

        let default_config = default_config(game_path);

        let toml_str = toml::to_string(&default_config)
            .expect("Failed to serialize default config");
        fs::write(&config_path, toml_str)
            .expect("Failed to write default config file");
        return default_config.settings;
    }

    let toml_str = fs::read_to_string(&config_path)
        .expect("Failed to read rsi_maintenance.toml");
    let config: ConfigFile = toml::from_str(&toml_str)
        .expect("Failed to parse rsi_maintenance.toml");

    apply_config_to_environment(&config.settings, &config.environment);
    config.settings
}

// Load the full config file (settings + environment map)
pub fn load_full_config() -> ConfigFile {
    let xdg_dirs = BaseDirectories::with_prefix("starcitizen-lug");
    _ = xdg_dirs.create_config_directory("");

    let config_path = xdg_dirs
        .get_config_file(Path::new("rsi_maintenance.toml"))
        .unwrap();

    if !config_path.exists() {
        let game_path = env::var("WINEPREFIX").unwrap_or_default();
        return default_config(game_path);
    }

    let toml_str = fs::read_to_string(&config_path)
        .expect("Failed to read rsi_maintenance.toml");
    toml::from_str(&toml_str).expect("Failed to parse rsi_maintenance.toml")
}

// Save the config file
pub fn save_full_config(config: &ConfigFile) {
    let xdg_dirs = BaseDirectories::with_prefix("starcitizen-lug");
    _ = xdg_dirs.create_config_directory("");

    let config_path = xdg_dirs
        .get_config_file(Path::new("rsi_maintenance.toml"))
        .unwrap();

    let toml_str =
        toml::to_string_pretty(config).expect("Failed to serialize config");
    fs::write(&config_path, toml_str)
        .expect("Failed to write rsi_maintenance.toml");
}

pub fn should_run_headless(args: &[String]) -> bool {
    let config = load_config_without_prompt();
    let prefix = crate::runner::prefix_path_from_config(&config.game_path);
    let launcher_path = crate::runner::launcher_exe_path(&prefix);

    args.iter().any(|arg| arg == "--run") && launcher_path.exists()
}

pub fn run_launcher_headless() -> Result<(), String> {
    let config = load_config_without_prompt();
    let prefix = crate::runner::prefix_path_from_config(&config.game_path);

    let runtime = tokio::runtime::Runtime::new()
        .map_err(|err| format!("Failed to initialize runtime: {err}"))?;

    runtime.block_on(async move {
        crate::runner::run_installed_launcher(prefix).await
    })
}

pub async fn get_config_async_with_title(title: String) -> AppConfig {
    let xdg_dirs = BaseDirectories::with_prefix("starcitizen-lug");
    _ = xdg_dirs.create_config_directory("");

    load_environment_from_current_process();

    let config_path = xdg_dirs
        .get_config_file(Path::new("rsi_maintenance.toml"))
        .unwrap();

    if !&config_path.exists() {
        let mut game_path = env::var("WINEPREFIX").unwrap_or_default();

        if game_path.is_empty() {
            let picked = rfd::AsyncFileDialog::new()
                .set_title(&title)
                .pick_folder()
                .await;

            game_path = if let Some(folder) = picked {
                folder.path().to_string_lossy().to_string()
            } else {
                String::new()
            };
        }

        let default_config = default_config(game_path);

        let toml_str = toml::to_string(&default_config)
            .expect("Failed to serialize default config");
        fs::write(&config_path, toml_str)
            .expect("Failed to write default config file");
        return default_config.settings;
    }

    let toml_str = fs::read_to_string(&config_path)
        .expect("Failed to read rsi_maintenance.toml");
    let config: ConfigFile = toml::from_str(&toml_str)
        .expect("Failed to parse rsi_maintenance.toml");

    apply_config_to_environment(&config.settings, &config.environment);
    config.settings
}
