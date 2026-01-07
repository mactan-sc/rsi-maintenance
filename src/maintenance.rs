use iced::widget::{button, column, container, row, text};

#[derive(Debug, Clone)]
pub enum Message {
    Exit,
    Back,
    Winecfg,
    Control,
    OpenCfg,
    OpenGameDir,
}

pub fn view() -> iced::Element<'static, Message> {
    container(
        container(
            column![
                text("RSI Maintenance").size(24).width(iced::Length::Fill),
                button("Open config file").on_press(Message::OpenCfg).width(iced::Length::Fill),
                button("Open game directory").on_press(Message::OpenGameDir).width(iced::Length::Fill),
                button("Winecfg").on_press(Message::Winecfg).width(iced::Length::Fill),
                button("Control").on_press(Message::Control).width(iced::Length::Fill),
                button("Back").on_press(Message::Back).width(iced::Length::Fill),
                button("Exit").on_press(Message::Exit).width(iced::Length::Fill)
            ]
            .spacing(12)
            .spacing(12)
            .align_x(iced::Alignment::Center)
        )
        .padding(16)
        .width(iced::Length::Shrink)
    )
    .center_x(iced::Length::Fill)
    .center_y(iced::Length::Fill)
    .into()
}