use iced::{
    widget::{button, text, column, row, rich_text, span},
    Theme, Font, border
};

#[derive(Debug, Clone)]
pub enum Message {
    Exit,
    Maintenance,
    OpenWiki,
}

pub fn view() -> iced::Element<'static, Message> {
    let theme = Theme::KanagawaDragon;
    let palette = theme.extended_palette();

    column![
        row![
            text("Greetings, Space Penguin!").size(24).width(iced::Length::Fill),
        ],
        rich_text![
            "For help with the game, refer to the LUG org's ",
            span("wiki")
            .color(palette.secondary.strong.color)
                        .padding([0, 8])
                        .font(Font::MONOSPACE)
                        .link(Message::OpenWiki),
                    "."
        ],
        row![
        button("Maintenance").on_press(Message::Maintenance),
        button("Exit").on_press(Message::Exit)
    ].spacing(8)].into()
}