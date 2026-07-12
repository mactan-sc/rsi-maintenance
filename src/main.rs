use gui::app::run_app;

pub mod gui;
pub mod runner;
pub mod utility;

fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().collect();

    if utility::helpers::should_run_headless(&args) {
        if let Err(err) = utility::helpers::run_launcher_headless() {
            eprintln!("{err}");
            std::process::exit(1);
        }

        std::process::exit(0);
    }

    run_app()
}
