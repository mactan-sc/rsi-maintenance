use std::fs;
use std::process::Command;
use std::path::Path;
use serde::Serialize;
use serde::Deserialize;
use rfd::AsyncFileDialog;
use iced::{
    Theme, window, Task, Element
};

mod welcome;
mod maintenance;

#[derive(Debug, Serialize, Deserialize, Default)]
struct ConfigFile{
    settings: AppConfig,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct AppConfig {
    game_path: String,
}

pub struct AppState {
    screen: Screen,
    config: AppConfig,
}

enum Screen {
    Welcome,
    Maintenance,
}

#[derive(Debug, Clone)]
enum Message {
    Welcome(welcome::Message),
    Maintenance(maintenance::Message),
    ConfigLoaded(AppConfig),
}

impl AppState {

    async fn get_config_async() -> AppConfig {
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

    fn title(&self) -> String {
        let screen = match self.screen {
            Screen::Welcome => "Welcome",
            Screen::Maintenance => "Maintenance",
        };

        format!("{screen} - RSI Launcher Maintenance")
    }

    fn theme(&self) -> iced::Theme {
        Theme::KanagawaDragon
    }

    fn new() -> (Self, Task<Message>) {
        (
            Self {
                screen: Screen::Welcome,
                config: AppConfig::default(),
            },
            Task::perform(AppState::get_config_async(), Message::ConfigLoaded)
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ConfigLoaded(config) => {
                self.config = config;
                Task::none()
            },
            Message::Welcome(welcome::Message::Maintenance) => {
                self.screen = Screen::Maintenance;
                Task::none()
            },
            Message::Welcome(welcome::Message::Exit) |
            Message::Maintenance(maintenance::Message::Exit) => window::latest().and_then(window::close),
            Message::Maintenance(maintenance::Message::Back) => {
                self.screen = Screen::Welcome;
                Task::none()
            },
            Message::Maintenance(maintenance::Message::Winecfg) => {
                let _ = Command::new("wine").arg("winecfg").spawn();
                Task::none()
            },
            Message::Maintenance(maintenance::Message::Control) => {
                let _ = Command::new("wine").arg("control").spawn();
                Task::none()
            },
            Message::Maintenance(maintenance::Message::OpenCfg) => {
                let xdg_dirs = xdg::BaseDirectories::with_prefix("starcitizen-lug");
                let config_path = xdg_dirs.get_config_file(Path::new("launcher.cfg"));
                let _ = opener::open(config_path.unwrap());
                Task::none()
            },
            Message::Maintenance(maintenance::Message::OpenGameDir) => {
                let _ = opener::open(&self.config.game_path);
                Task::none()
            },
            Message::Welcome(welcome::Message::OpenWiki) => {
                let _ = opener::open("https://starcitizen-lug.github.io");
                Task::none()
            },
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.screen {
            Screen::Welcome => welcome::view(&self.theme()).map(Message::Welcome),
            Screen::Maintenance => maintenance::view().map(Message::Maintenance),
        }
    }
}

fn main() -> iced::Result {
    iced::application(AppState::new, AppState::update, AppState::view)
        .title(AppState::title)
        .theme(AppState::theme)
        .run()
}