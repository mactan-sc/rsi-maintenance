use iced::{
    widget::{button, text, rich_text, span, column, row, container},
    Font
};
use std::sync::OnceLock;
use crate::utility::*;

pub struct Welcome {
    pub greeting: &'static str,
    pub help_intro: &'static str,
    pub wiki: &'static str,
    pub maintenance_label: &'static str,
    pub exit_label: &'static str,
}

impl Default for Welcome {
    fn default() -> Self {
        let lang = detect_lang();
        let i18n = I18n::new(lang);

        let greeting = Box::leak(i18n.t("Welcome-Greeting").into_boxed_str());
        let help_intro = Box::leak(i18n.t("Welcome-HelpIntro").into_boxed_str());
        let wiki = Box::leak(i18n.t("Welcome-Wiki").into_boxed_str());
        let maintenance_label = Box::leak(i18n.t("Maintenance").into_boxed_str());
        let exit_label = Box::leak(i18n.t("Exit").into_boxed_str());

        Self {
            greeting,
            help_intro,
            wiki,
            maintenance_label,
            exit_label,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Exit,
    Maintenance,
    OpenWiki,
}

pub fn view(app: &crate::gui::app::AppState) -> iced::Element<'_, Message> {
    let theme = app.theme();
    let palette = theme.extended_palette();

    static WELCOME: OnceLock<Welcome> = OnceLock::new();
    let welcome = WELCOME.get_or_init(|| Welcome::default());

    container(
        column![
            text(welcome.greeting).size(24),
            container(
                rich_text![
                    welcome.help_intro,
                    " ",
                    span(welcome.wiki)
                        .color(palette.secondary.strong.color)
                        .font(Font::MONOSPACE)
                        .link(Message::OpenWiki),
                    ". "
                ].on_link_click(|_: Message| Message::OpenWiki)
            ).padding([8.0, 0.0]),
            row![
                 button(text(welcome.maintenance_label)).on_press(Message::Maintenance),
                 button(text(app.label_exit())).on_press(Message::Exit)
            ].spacing(8)
        ].align_x(iced::Alignment::Center)
    )
    .padding(16)
    .center_x(iced::Length::Fill)
    .center_y(iced::Length::Fill)
    .into()
}