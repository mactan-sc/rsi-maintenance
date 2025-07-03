use std::process::Command;
use iced::{
    widget::{button, text, column, row},
    Theme, window, Task
};
use iced_wgpu::Renderer;

#[derive(Debug, Default)]
struct AppState {

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
            Task::perform(
                async {
                    let _ = Command::new("wine").arg("winecfg").spawn();
                },
                |_| Message::Exit,
            )
        },
        Message::Control => {
            Task::perform(
                async {
                    let _ = Command::new("wine").arg("control").spawn();
                },
                |_| Message::Exit,
            )
        }
    }
}

fn view(state: &AppState) -> iced::Element<Message> {
    column![row![
        text("Hello, world!").size(24).width(iced::Length::Fill),
        button("Winecfg").on_press(Message::Winecfg),
        button("Control").on_press(Message::Control),
        button("Exit").on_press(Message::Exit)
    ].spacing(8)].into()
}

fn main() -> iced::Result {
    iced::application("rsi-maintenance", update, view).theme(|_s| Theme::KanagawaDragon).run()
}