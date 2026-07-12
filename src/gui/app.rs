use iced::{time, window, Element, Subscription, Task, Theme};
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::gui::pages::*;
use crate::runner as launcher_runner;
use crate::utility::*;

pub fn run_app() -> iced::Result {
    iced::application(AppState::startup, AppState::update, AppState::view)
        .title(AppState::title)
        .theme(AppState::theme)
        .subscription(AppState::subscription)
        .run()
}

pub struct AppState {
    screen: Screen,
    config: AppConfig,
    i18n: I18n,
    label_back: String,
    label_exit: String,
    runner_state: RunnerState,
}

enum Screen {
    Welcome,
    Maintenance,
    Runner,
}

#[derive(Debug, Clone)]
enum Message {
    Welcome(welcome::Message),
    Maintenance(maintenance::Message),
    Runner(runner::Message),
    ConfigLoaded(AppConfig),
    ProgressTick,
    RunnerFinished(Result<(), String>),
}

#[derive(Debug, Clone)]
pub struct RunnerState {
    pub status: RunnerStatus,
    pub installed: bool,
    pub progress_state: Arc<Mutex<Option<f32>>>,
}

impl Default for RunnerState {
    fn default() -> Self {
        Self {
            status: RunnerStatus::default(),
            installed: false,
            progress_state: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ProgressMode {
    None,
    Determinate(f32),
    Pulse,
}

#[derive(Debug, Clone)]
pub enum RunnerStatus {
    Idle,
    FetchingVersion,
    Downloading { progress: f32 },
    Installing,
    Launching,
    Complete,
    Failed(String),
}

impl Default for RunnerStatus {
    fn default() -> Self {
        Self::Idle
    }
}

impl RunnerState {
    fn new(status: RunnerStatus, installed: bool) -> Self {
        Self {
            status,
            installed,
            progress_state: Arc::new(Mutex::new(None)),
        }
    }

    pub fn status_label(&self) -> String {
        match &self.status {
            RunnerStatus::Idle => String::new(),
            RunnerStatus::FetchingVersion => {
                "Fetching launcher version...".to_string()
            }
            RunnerStatus::Downloading { .. } => {
                "Downloading launcher installer...".to_string()
            }
            RunnerStatus::Installing => "Installing launcher...".to_string(),
            RunnerStatus::Launching => {
                "Launching the RSI Launcher...".to_string()
            }
            RunnerStatus::Complete => "Launcher is ready.".to_string(),
            RunnerStatus::Failed(err) => format!("Failed: {err}"),
        }
    }

    pub fn progress_mode(&self) -> ProgressMode {
        match &self.status {
            RunnerStatus::Downloading { progress } => {
                ProgressMode::Determinate((*progress).clamp(0.0, 1.0))
            }
            RunnerStatus::FetchingVersion
            | RunnerStatus::Installing
            | RunnerStatus::Launching => ProgressMode::Pulse,
            RunnerStatus::Idle
            | RunnerStatus::Complete
            | RunnerStatus::Failed(_) => ProgressMode::None,
        }
    }

    pub fn progress_value(&self) -> Option<f32> {
        match self.progress_mode() {
            ProgressMode::None => None,
            ProgressMode::Determinate(value) => Some(value),
            ProgressMode::Pulse => Some(0.5),
        }
    }

    pub fn error_message(&self) -> Option<String> {
        match &self.status {
            RunnerStatus::Failed(err) => Some(err.clone()),
            _ => None,
        }
    }

    pub fn can_finish(&self) -> bool {
        matches!(
            self.status,
            RunnerStatus::Complete | RunnerStatus::Failed(_)
        )
    }
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
            runner_state: RunnerState::default(),
        }
    }
}

impl AppState {
    fn title(&self) -> String {
        let title = self.i18n.t("RSI-Launcher-Maintenance");

        let screen = match self.screen {
            Screen::Welcome => self.i18n.t("Welcome"),
            Screen::Maintenance => self.i18n.t("Maintenance"),
            Screen::Runner => self.i18n.t("Run-Launcher"),
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

    pub fn launcher_installed(&self) -> bool {
        let prefix =
            launcher_runner::prefix_path_from_config(&self.config.game_path);
        launcher_runner::launcher_installed(&prefix)
    }

    fn startup() -> (Self, Task<Message>) {
        let state = Self::default();
        let picker_title = state.t("Picker-SelectGameDir");

        (
            state,
            Task::perform(
                helpers::get_config_async_with_title(picker_title),
                Message::ConfigLoaded,
            ),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        if matches!(&self.screen, Screen::Runner)
            && matches!(
                self.runner_state.status,
                RunnerStatus::FetchingVersion
                    | RunnerStatus::Downloading { .. }
                    | RunnerStatus::Installing
                    | RunnerStatus::Launching
            )
        {
            time::every(Duration::from_millis(100))
                .map(|_| Message::ProgressTick)
        } else {
            Subscription::none()
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ConfigLoaded(config) => {
                self.config = config;
                Task::none()
            }
            Message::ProgressTick => {
                if let RunnerStatus::Downloading { .. } =
                    self.runner_state.status
                {
                    let progress = self
                        .runner_state
                        .progress_state
                        .lock()
                        .unwrap_or_else(|err| err.into_inner())
                        .unwrap_or(0.0);
                    self.runner_state.status =
                        RunnerStatus::Downloading { progress };
                }
                Task::none()
            }
            Message::Welcome(welcome::Message::Maintenance) => {
                self.screen = Screen::Maintenance;
                Task::none()
            }
            Message::Welcome(welcome::Message::InstallLauncher) => {
                let prefix = launcher_runner::prefix_path_from_config(
                    &self.config.game_path,
                );
                self.screen = Screen::Runner;
                let progress_state = Arc::new(Mutex::new(None));
                self.runner_state = RunnerState::new(
                    RunnerStatus::Downloading { progress: 0.0 },
                    launcher_runner::launcher_installed(&prefix),
                );
                self.runner_state.progress_state = progress_state.clone();
                Task::perform(
                    async move {
                        launcher_runner::install_launcher_only(
                            prefix,
                            progress_state,
                        )
                        .await
                    },
                    Message::RunnerFinished,
                )
            }
            Message::Welcome(welcome::Message::RunLauncher) => {
                let prefix = launcher_runner::prefix_path_from_config(
                    &self.config.game_path,
                );
                self.screen = Screen::Runner;
                self.runner_state = RunnerState::new(
                    RunnerStatus::Launching,
                    launcher_runner::launcher_installed(&prefix),
                );
                Task::perform(
                    async move {
                        launcher_runner::run_installed_launcher(prefix).await
                    },
                    Message::RunnerFinished,
                )
            }
            Message::Welcome(welcome::Message::Exit)
            | Message::Maintenance(maintenance::Message::Exit) => {
                window::latest().and_then(window::close)
            }
            Message::Maintenance(maintenance::Message::Back) => {
                self.screen = Screen::Welcome;
                Task::none()
            }
            Message::Maintenance(maintenance::Message::Winecfg) => {
                let _ = Command::new("umu-run").arg("winecfg").spawn();
                Task::none()
            }
            Message::Maintenance(maintenance::Message::Control) => {
                let _ = Command::new("umu-run").arg("control").spawn();
                Task::none()
            }
            Message::Maintenance(maintenance::Message::Regedit) => {
                let _ = Command::new("umu-run").arg("regedit").spawn();
                Task::none()
            }
            Message::Maintenance(maintenance::Message::OpenCfg) => {
                let xdg_dirs =
                    xdg::BaseDirectories::with_prefix("starcitizen-lug");
                let config_path =
                    xdg_dirs.get_config_file(Path::new("rsi_maintenance.toml"));
                let _ = opener::open(config_path.unwrap());
                Task::none()
            }
            Message::Maintenance(maintenance::Message::OpenGameDir) => {
                let _ = opener::open(&self.config.game_path);
                Task::none()
            }
            Message::Welcome(welcome::Message::OpenWiki) => {
                let _ = opener::open("https://starcitizen-lug.github.io");
                Task::none()
            }
            Message::RunnerFinished(result) => {
                self.runner_state = match result {
                    Ok(()) => RunnerState::new(RunnerStatus::Complete, true),
                    Err(err) => {
                        RunnerState::new(RunnerStatus::Failed(err), false)
                    }
                };
                Task::none()
            }
            Message::Runner(runner::Message::InstallLauncher) => {
                self.runner_state = RunnerState::new(
                    RunnerStatus::Downloading { progress: 0.0 },
                    false,
                );
                let prefix = launcher_runner::prefix_path_from_config(
                    &self.config.game_path,
                );
                let progress_state = Arc::new(Mutex::new(None));
                self.runner_state = RunnerState::new(
                    RunnerStatus::Downloading { progress: 0.0 },
                    false,
                );
                self.runner_state.progress_state = progress_state.clone();
                Task::perform(
                    async move {
                        launcher_runner::install_launcher_only(
                            prefix,
                            progress_state,
                        )
                        .await
                    },
                    Message::RunnerFinished,
                )
            }
            Message::Runner(runner::Message::RunLauncher) => {
                self.runner_state = RunnerState::new(
                    RunnerStatus::Launching,
                    self.runner_state.installed,
                );
                let prefix = launcher_runner::prefix_path_from_config(
                    &self.config.game_path,
                );
                Task::perform(
                    async move {
                        launcher_runner::run_installed_launcher(prefix).await
                    },
                    Message::RunnerFinished,
                )
            }
            Message::Runner(runner::Message::Cancel) => {
                self.screen = Screen::Welcome;
                self.runner_state = RunnerState::default();
                Task::none()
            }
            Message::Runner(runner::Message::Done) => {
                self.screen = Screen::Welcome;
                self.runner_state = RunnerState::default();
                Task::none()
            }
            Message::Runner(runner::Message::Progress(_)) => Task::none(),
            Message::Runner(runner::Message::Error(_)) => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::Welcome => welcome::view(self).map(Message::Welcome),
            Screen::Maintenance => {
                maintenance::view(self).map(Message::Maintenance)
            }
            Screen::Runner => {
                runner::view(&self.runner_state).map(Message::Runner)
            }
        }
    }
}
