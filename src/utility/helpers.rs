use std::fs;
use std::path::Path;
use std::env;
use serde::{Serialize, Deserialize};
use xdg::BaseDirectories;
use unic_langid::LanguageIdentifier;

fn parse_lang(raw: &str) -> Option<LanguageIdentifier> {
    raw.split('@').next()?
        .split('.').next()?
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

#[derive(Debug, Serialize, Deserialize, Default)]
struct ConfigFile{
    settings: AppConfig,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AppConfig {
    pub game_path: String,
}

// Variant that accepts a custom dialog title. Useful when the title should come
// from the application's i18n state.
pub async fn get_config_async_with_title(title: String) -> AppConfig {
    let xdg_dirs = BaseDirectories::with_prefix("starcitizen-lug");
    _ = xdg_dirs.create_config_directory("");

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

        let default_config = ConfigFile {
            settings: AppConfig {
                game_path,
                ..Default::default()
            },
        };

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

    config.settings
}