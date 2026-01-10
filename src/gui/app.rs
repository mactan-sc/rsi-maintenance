use std::process::Command;
use std::path::Path;
use iced::{
    Theme, window, Task, Element
};

use crate::gui::pages::*;
use crate::utility::*;


pub fn run_app() -> iced::Result {
    iced::application(AppState::startup, AppState::update, AppState::view)
        .title(AppState::title)
        .theme(AppState::theme)
        .run()
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

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: Screen::Welcome,
            config: AppConfig::default(),
        }
    }
}

impl AppState {
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

    fn startup() -> (Self, Task<Message>) {
        (
            Self::default(),
            Task::perform(helpers::get_config_async(), Message::ConfigLoaded)
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
                let _ = Command::new("umu-run").arg("winecfg").spawn();
                Task::none()
            },
            Message::Maintenance(maintenance::Message::Control) => {
                let _ = Command::new("umu-run").arg("control").spawn();
                Task::none()
            },
            Message::Maintenance(maintenance::Message::Regedit) => {
                let _ = Command::new("umu-run").arg("regedit").spawn();
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

    fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::Welcome => welcome::view(&self.theme()).map(Message::Welcome),
            Screen::Maintenance => maintenance::view().map(Message::Maintenance),
        }
    }
}