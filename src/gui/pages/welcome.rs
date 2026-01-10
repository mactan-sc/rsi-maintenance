use iced::{
    widget::{button, text, column, row, rich_text, span, container},
    Theme, Font
};

#[derive(Debug, Clone)]
pub enum Message {
    Exit,
    Maintenance,
    OpenWiki,
}

pub fn view(theme: &Theme) -> iced::Element<'static, Message> {
    let palette = theme.extended_palette();

        container(
            column![
                text("Greetings, Space Penguin!").size(24),
                rich_text![
                    "For help with the game, refer to the LUG org's ",
                    span("wiki")
                    .color(palette.secondary.strong.color)
                                .padding([0, 8])
                                .font(Font::MONOSPACE)
                                .link(Message::OpenWiki),
                            "."
                ].on_link_click(|_: Message| Message::OpenWiki),
                row![
                button("Maintenance").on_press(Message::Maintenance),
                button("Exit").on_press(Message::Exit)
            ].spacing(8)].align_x(iced::Alignment::Center)
        )
        .padding(16)
        .center_x(iced::Length::Fill)
        .center_y(iced::Length::Fill)
    .into()
}