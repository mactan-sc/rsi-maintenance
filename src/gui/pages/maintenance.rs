use iced::widget::{button, column, container, text};
use std::sync::OnceLock;
use crate::utility::*;

pub struct Maintenance {
    pub rsi_maintenance: &'static str,
    pub open_config_file: &'static str,
    pub open_game_directory: &'static str,
    pub winecfg: &'static str,
    pub control: &'static str,
    pub regedit: &'static str,
}

impl Default for Maintenance {
    fn default() -> Self {
        let lang = detect_lang();
        let i18n = I18n::new(lang);

        let rsi_maintenance = Box::leak(i18n.t("RSI-Maintenance").into_boxed_str());
        let open_config_file = Box::leak(i18n.t("Open-config-file").into_boxed_str());
        let open_game_directory = Box::leak(i18n.t("Open-game-directory").into_boxed_str());
        let winecfg = Box::leak(i18n.t("Winecfg").into_boxed_str());
        let control = Box::leak(i18n.t("Control").into_boxed_str());
        let regedit = Box::leak(i18n.t("Regedit").into_boxed_str());

        Self {
            rsi_maintenance,
            open_config_file,
            open_game_directory,
            winecfg,
            control,
            regedit,
        }
    }
}


#[derive(Debug, Clone)]
pub enum Message {
    Exit,
    Back,
    Winecfg,
    Control,
    Regedit,
    OpenCfg,
    OpenGameDir,
}

pub fn view(app: &crate::gui::app::AppState) -> iced::Element<'_, Message> {
    static MAINTENANCE: OnceLock<Maintenance> = OnceLock::new();
    let maintenance = MAINTENANCE.get_or_init(|| Maintenance::default());

    container(
    column![
            text(maintenance.rsi_maintenance).size(24).width(iced::Length::Fill),
            button(maintenance.open_config_file).on_press(Message::OpenCfg).width(iced::Length::Fill),
            button(maintenance.open_game_directory).on_press(Message::OpenGameDir).width(iced::Length::Fill),
            button(maintenance.winecfg).on_press(Message::Winecfg).width(iced::Length::Fill),
            button(maintenance.control).on_press(Message::Control).width(iced::Length::Fill),
            button(maintenance.regedit).on_press(Message::Regedit).width(iced::Length::Fill),
            button(text(app.label_back())).on_press(Message::Back).width(iced::Length::Fill),
            button(text(app.label_exit())).on_press(Message::Exit).width(iced::Length::Fill)
    ].spacing(12)
    .spacing(12)
    .align_x(iced::Alignment::Center)
    .padding(16)
    )
    .center_x(iced::Length::Fill)
    .center_y(iced::Length::Fill)
    .into()
}