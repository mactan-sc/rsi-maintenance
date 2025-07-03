use std::process::Command;
use iced::{
    widget::{button, text, column, row},
    Theme, window, Task
};

struct AppState {
    screen: Screen,
}

enum Screen {
    Welcome,
    Maintenance,
}


#[derive(Debug, Clone)]
enum Message {
    Exit,
    Winecfg,
    Control
}

fn update(_state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::Exit => window::get_latest().and_then(window::close),
        Message::Winecfg => {
            let _ = Command::new("wine").arg("winecfg").spawn();
            Task::none()
        },
        Message::Control => {
            let _ = Command::new("wine").arg("control").spawn();
            Task::none()
        }
    }
}

fn view(_state: &AppState) -> iced::Element<Message> {
    column![row![
        text("Hello, world!").size(24).width(iced::Length::Fill),
        button("Winecfg").on_press(Message::Winecfg),
        button("Control").on_press(Message::Control),
        button("Exit").on_press(Message::Exit)
    ].spacing(8)].into()
}

fn main() -> iced::Result {
    iced::application("rsi-maintenance", update, view)
        .theme(theme)
        .run()
}

fn theme(_state: &AppState) -> Theme {
    Theme::KanagawaDragon
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            screen: Screen::Welcome
        }
    }
}
