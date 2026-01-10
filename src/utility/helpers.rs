use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Default)]
struct ConfigFile{
    settings: AppConfig,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AppConfig {
    pub game_path: String,
}

pub async fn get_config_async() -> AppConfig {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("starcitizen-lug");
        let config_path = xdg_dirs
            .get_config_file(Path::new("rsi_maintenance.toml"))
            .unwrap();

        if !&config_path.exists() {
            let picked = rfd::AsyncFileDialog::new()
                .set_title("Select your Star Citizen game directory")
                .pick_folder()
                .await;

            let game_path = if let Some(folder) = picked {
                folder.path().to_string_lossy().to_string()
            } else {
                String::new()
            };

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