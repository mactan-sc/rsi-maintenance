use iced::{
    widget::{button, text, column, row}
};

#[derive(Debug, Clone)]
pub enum Message {
    Exit,
    Back,
    Winecfg,
    Control,
    OpenCfg,
}

pub fn view() -> iced::Element<'static, Message> {
    column![row![
        text("RSI Maintenance").size(24).width(iced::Length::Fill),
        button("Open config file").on_press(Message::OpenCfg),
        button("Winecfg").on_press(Message::Winecfg),
        button("Control").on_press(Message::Control),
        button("Back").on_press(Message::Back),
        button("Exit").on_press(Message::Exit)
    ].spacing(8)].into()
}