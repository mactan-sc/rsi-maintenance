pub use maintenance_widgets::download::{DownloadState, DownloadUpdate, DownloadWidget};
use maintenance_widgets::easing;
use maintenance_widgets::linear::Linear;
use iced::{Element, Task};

use std::path::PathBuf;

// Describes the progress-bar presentation mode for non-download operations.
#[derive(Debug, Clone, Copy)]
pub enum ProgressMode {
    // No progress bar should be shown.
    None,
    // Show a determinate bar filled to the given fraction (0.0 – 1.0).
    Determinate(f32),
    // Show an indeterminate (pulsing) bar.
    Pulse,
}

impl ProgressMode {
    // Return whether a progress indicator should be rendered.
    pub fn is_active(self) -> bool {
        !matches!(self, ProgressMode::None)
    }
}

// Build an optional animated progress-bar widget from a [`ProgressMode`].
//
// Returns an indeterminate [`Linear`] spinner with accelerate easing,
// or `None` when the mode is [`ProgressMode::None`].
pub fn progress_bar_widget<'a, Message: Clone + 'a>(
    mode: ProgressMode,
) -> Option<Element<'a, Message>> {
    if !mode.is_active() {
        return None;
    }

    Some(
        Linear::new()
            .easing(&easing::EMPHASIZED_ACCELERATE)
            .into(),
    )
}

#[derive(Debug, Clone)]
pub struct DownloadProgress {
    widget: DownloadWidget,
}

impl DownloadProgress {
    // Create a new download manager.
    //
    // * `url`       – the URL to download.
    // * `save_path` – optional path where the downloaded file will be written.
    pub fn new(url: String, save_path: Option<PathBuf>) -> Self {
        let mut w = DownloadWidget::new().url(url);
        if let Some(p) = save_path {
            w = w.save_to(p);
        }
        Self { widget: w }
    }

    // The current download state.
    pub fn state(&self) -> &DownloadState {
        self.widget.state()
    }

    // Whether the download is currently in progress.
    pub fn is_downloading(&self) -> bool {
        self.widget.is_downloading()
    }

    // Whether the download has completed (success or error).
    pub fn is_done(&self) -> bool {
        self.widget.is_done()
    }

    // Current progress as a fraction 0.0 – 1.0.
    pub fn progress(&self) -> f32 {
        self.widget.progress()
    }

    // Start (or restart) the download, returning a [`Task`] that emits
    // [`DownloadUpdate`] messages.
    pub fn start(&mut self) -> Task<DownloadUpdate> {
        self.widget.start()
    }

    // Feed a [`DownloadUpdate`] message into the state machine.
    pub fn update(&mut self, update: DownloadUpdate) {
        self.widget.update(update);
    }

    // Render the download widget (spinner + status text).
    pub fn view(&self) -> Element<'_, DownloadUpdate> {
        self.widget.view()
    }
}

