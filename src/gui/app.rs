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
    i18n: I18n,
    label_back: String,
    label_exit: String,
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
        let lang = detect_lang();
        let i18n = I18n::new(lang);

        let label_back = i18n.t("Back");
        let label_exit = i18n.t("Exit");

        Self {
            screen: Screen::Welcome,
            config: AppConfig::default(),
            i18n,
            label_back,
            label_exit,
        }
    }
}

impl AppState {
    fn title(&self) -> String {
        let title = self.i18n.t("RSI-Launcher-Maintenance");

        let screen = match self.screen {
            Screen::Welcome => self.i18n.t("Welcome"),
            Screen::Maintenance => self.i18n.t("Maintenance"),
        };

        format!("{screen} - {title}")
    }

    pub fn theme(&self) -> iced::Theme {
        Theme::KanagawaDragon
    }

    pub fn t(&self, key: &str) -> String {
        self.i18n.t(key)
    }

    pub fn label_back(&self) -> &str {
        &self.label_back
    }

    pub fn label_exit(&self) -> &str {
        &self.label_exit
    }

    fn startup() -> (Self, Task<Message>) {
        let state = Self::default();
        let picker_title = state.t("Picker-SelectGameDir");

        (
            state,
            Task::perform(helpers::get_config_async_with_title(picker_title), Message::ConfigLoaded)
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
            Screen::Welcome => welcome::view(self).map(Message::Welcome),
            Screen::Maintenance => maintenance::view(self).map(Message::Maintenance),
        }
    }
}