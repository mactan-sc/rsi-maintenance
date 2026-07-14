use iced::widget::progress_bar;
use iced::Element;

/// Describes the progress-bar presentation mode.
#[derive(Debug, Clone, Copy)]
pub enum ProgressMode {
    /// No progress bar should be shown.
    None,
    /// Show a determinate bar filled to the given fraction (0.0 – 1.0).
    Determinate(f32),
    /// Show an indeterminate (pulsing) bar.
    Pulse,
}

impl ProgressMode {
    /// Return the visual progress value that should be fed to a progress bar,
    /// or `None` when no bar should be rendered.
    pub fn value(self) -> Option<f32> {
        match self {
            ProgressMode::None => None,
            ProgressMode::Determinate(v) => Some(v),
            ProgressMode::Pulse => Some(0.5),
        }
    }
}

/// Build an optional progress-bar widget from a `ProgressMode`.
///
/// Returns `None` when the mode is [`ProgressMode::None`].
pub fn progress_bar_widget<'a, Message: 'a>(
    mode: ProgressMode,
) -> Option<Element<'a, Message>> {
    mode.value()
        .map(|v| progress_bar(0.0..=1.0, v).into())
}
