use crate::utility::*;
use iced::{
    widget::{button, column, container, text},
    Alignment, Element, Length,
};
use std::sync::OnceLock;

pub struct RunnerPage {
    pub title: &'static str,
    pub description: &'static str,
    pub install_label: &'static str,
    pub run_label: &'static str,
    pub cancel_label: &'static str,
    pub done_label: &'static str,
}

impl Default for RunnerPage {
    fn default() -> Self {
        let lang = detect_lang();
        let i18n = I18n::new(lang);

        Self {
            title: Box::leak(i18n.t("Run-Launcher").into_boxed_str()),
            description: Box::leak(
                i18n.t("Run-Launcher-Description").into_boxed_str(),
            ),
            install_label: Box::leak(
                i18n.t("Install-Launcher").into_boxed_str(),
            ),
            run_label: Box::leak(i18n.t("Run-Launcher").into_boxed_str()),
            cancel_label: Box::leak(i18n.t("Cancel").into_boxed_str()),
            done_label: Box::leak(i18n.t("Done").into_boxed_str()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Cancel,
    Done,
    InstallLauncher,
    RunLauncher,
    Progress(f32),
    Error(String),
}

pub fn view<'a>(
    state: &'a crate::gui::app::RunnerState,
) -> Element<'a, Message> {
    static PAGE: OnceLock<RunnerPage> = OnceLock::new();
    let page = PAGE.get_or_init(|| RunnerPage::default());

    let _progress_mode = state.progress_mode();
    let mut content = column![
        text(page.title).size(24),
        text(page.description).width(Length::Fill),
        text(state.status_label()),
    ]
    .spacing(12)
    .width(Length::Fill);

    if let Some(pb) = progress_bar_widget(state.progress_mode()) {
        content = content.push(pb);
    }

    if let Some(error) = state.error_message() {
        content = content.push(text(error));
    }

    let actions = column![
        button(text(page.install_label)).on_press(Message::InstallLauncher),
        button(text(page.run_label)).on_press(Message::RunLauncher),
    ]
    .spacing(8);

    let footer = if state.can_finish() {
        column![
            actions,
            button(text(page.done_label)).on_press(Message::Done)
        ]
        .spacing(8)
    } else {
        column![
            actions,
            button(text(page.cancel_label)).on_press(Message::Cancel)
        ]
        .spacing(8)
    };

    container(content.push(footer).align_x(Alignment::Center))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(24)
        .into()
}
