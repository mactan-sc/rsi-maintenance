//! A self-contained download widget with state management, progress tracking,
//! and view rendering — extracted from the `download_progress` example.
use iced::futures::StreamExt;
use iced::task::{Straw, sipper};
use iced::widget::{column, container, progress_bar, text};
use iced::{Alignment, Element, Length, Task};

use crate::easing;
use crate::linear::Linear;

use std::io;
use std::path::PathBuf;
use std::sync::Arc;

pub fn download(url: impl AsRef<str>) -> impl Straw<(), Progress, Error> {
    download_inner(url.as_ref().to_string(), None)
}

// Download a URL and save to `save_path`, reporting streaming progress.
pub fn download_to_file(
    url: impl AsRef<str>,
    save_path: impl Into<PathBuf>,
) -> impl Straw<(), Progress, Error> {
    download_inner(url.as_ref().to_string(), Some(save_path.into()))
}

pub(crate) fn download_inner(
    url: String,
    save_path: Option<PathBuf>,
) -> impl Straw<(), Progress, Error> {
    sipper(async move |mut progress| {
        let response = reqwest::get(&url).await?;
        let total =
            response.content_length().ok_or(Error::NoContentLength)?;

        let _ = progress.send(Progress { percent: 0.0 }).await;

        let mut byte_stream = response.bytes_stream();
        let mut downloaded: u64 = 0;

        // Open the output file if a save path was provided.
        let mut file: Option<tokio::fs::File> = match &save_path {
            Some(path) => Some(
                tokio::fs::File::create(path)
                    .await
                    .map_err(|e| Error::IoError(Arc::new(e)))?,
            ),
            None => None,
        };

        while let Some(next_bytes) = byte_stream.next().await {
            let bytes = next_bytes?;
            downloaded += bytes.len() as u64;

            if let Some(ref mut f) = file {
                tokio::io::AsyncWriteExt::write_all(f, &bytes)
                    .await
                    .map_err(|e| Error::IoError(Arc::new(e)))?;
            }

            let _ = progress
                .send(Progress {
                    percent: 100.0 * downloaded as f32 / total as f32,
                })
                .await;
        }

        if let Some(f) = &file {
            let _ = f.sync_all().await;
        }

        Ok(())
    })
}

#[derive(Debug, Clone)]
pub struct Progress {
    pub percent: f32,
}

#[derive(Debug, Clone)]
pub enum Error {
    RequestFailed(Arc<reqwest::Error>),
    IoError(Arc<io::Error>),
    NoContentLength,
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::RequestFailed(Arc::new(error))
    }
}

#[derive(Debug, Clone)]
pub enum DownloadState {
    // No download has been started yet
    Idle,
    // Download is in progress at the given fraction (0.0 – 1.0)
    Downloading { progress: f32 },
    // Download completed successfully
    Finished,
    // Download failed with an error message
    Errored(String),
}

// Messages produced by a running download task.
#[derive(Debug, Clone)]
pub enum DownloadUpdate {
    // A progress tick (0.0 – 1.0).
    Progress(f32),
    // The download completed (or failed).
    Finished(Result<(), String>),
}

// A reusable download component that manages its own state, creates iced
// [`Task`]s for async downloads, and can render its own view.
//
// # Example
//
// ```ignore
// let mut widget = DownloadWidget::new();
// let task = widget.start();
// // ... in update:
// widget.update(update);
// // ... in view:
// widget.view().map(MyMessage::Download)
// ```
#[derive(Debug, Clone)]
pub struct DownloadWidget {
    state: DownloadState,
    url: Option<String>,
    save_path: Option<PathBuf>,
}

impl Default for DownloadWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadWidget {
    // Create a new idle [`DownloadWidget`].
    pub fn new() -> Self {
        Self {
            state: DownloadState::Idle,
            url: None,
            save_path: None,
        }
    }

    // Set the URL to download.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    // Set an optional file path to save the download to.
    pub fn save_to(mut self, path: impl Into<PathBuf>) -> Self {
        self.save_path = Some(path.into());
        self
    }

    // Start (or restart) the download. Returns a [`Task`] that produces
    // [`DownloadUpdate`] messages as progress changes.
    //
    // Does nothing if already downloading.
    pub fn start(&mut self) -> Task<DownloadUpdate> {
        let url = match &self.url {
            Some(u) => u.clone(),
            None => {
                self.state =
                    DownloadState::Errored("No URL configured".into());
                return Task::none();
            }
        };

        match &self.state {
            DownloadState::Idle
            | DownloadState::Finished
            | DownloadState::Errored(_) => {}
            DownloadState::Downloading { .. } => return Task::none(),
        }

        let (task, _handle) = Task::sip(
            download_inner(url, self.save_path.clone()),
            |p: Progress| DownloadUpdate::Progress(p.percent / 100.0),
            |result: Result<(), Error>| {
                DownloadUpdate::Finished(result.map_err(|e| match e {
                    Error::RequestFailed(arc) => {
                        format!("Request failed: {}", arc)
                    }
                    Error::IoError(arc) => {
                        format!("I/O error: {}", arc)
                    }
                    Error::NoContentLength => {
                        "Server did not report content length".into()
                    }
                }))
            },
        )
        .abortable();

        self.state = DownloadState::Downloading { progress: 0.0 };

        task
    }

    // Feed a [`DownloadUpdate`] message into the widget to advance its
    // state machine.
    pub fn update(&mut self, update: DownloadUpdate) {
        match (&mut self.state, update) {
            (
                DownloadState::Downloading { progress },
                DownloadUpdate::Progress(p),
            ) => {
                *progress = p;
            }
            (
                DownloadState::Downloading { .. },
                DownloadUpdate::Finished(result),
            ) => {
                self.state = match result {
                    Ok(()) => DownloadState::Finished,
                    Err(e) => DownloadState::Errored(e),
                };
            }
            _ => {}
        }
    }

    // The current download state.
    pub fn state(&self) -> &DownloadState {
        &self.state
    }

    // Convenience: the current progress as 0.0 – 1.0.
    pub fn progress(&self) -> f32 {
        match &self.state {
            DownloadState::Downloading { progress } => *progress,
            DownloadState::Finished => 1.0,
            _ => 0.0,
        }
    }

    // Return whether the download is currently active.
    pub fn is_downloading(&self) -> bool {
        matches!(self.state, DownloadState::Downloading { .. })
    }

    // Return whether the download has finished (success or error).
    pub fn is_done(&self) -> bool {
        matches!(
            self.state,
            DownloadState::Finished | DownloadState::Errored(_)
        )
    }

    // Render the download widget.
    //
    // During download shows a determinate progress bar that fills by
    // percentage, plus a status line.  Other states show an animated
    // [`Linear`] spinner.
    pub fn view(&self) -> Element<'_, DownloadUpdate> {
        let progress_pct = self.progress() * 100.0;

        let indicator: Element<_> = match &self.state {
            DownloadState::Downloading { .. } => container(
                progress_bar(0.0..=100.0, progress_pct),
            )
            .width(100)
            .height(4)
            .into(),
            _ => Linear::new()
                .easing(&easing::EMPHASIZED_ACCELERATE)
                .into(),
        };

        let status = match &self.state {
            DownloadState::Idle => text("Ready to download"),
            DownloadState::Downloading { .. } => {
                text!("Downloading... {:.1}%", progress_pct)
            }
            DownloadState::Finished => text("Download finished!"),
            DownloadState::Errored(err) => text!("Error: {err}"),
        };

        column![indicator, status]
            .spacing(10)
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .into()
    }
}
