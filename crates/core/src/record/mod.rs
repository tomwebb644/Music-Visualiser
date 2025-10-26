use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{AnalysisFrame, MusicVizError, Result};

/// Settings controlling the behaviour of the [`Recorder`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSettings {
    pub enabled: bool,
    pub output_path: Option<PathBuf>,
}

impl Default for RecordingSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            output_path: None,
        }
    }
}

impl RecordingSettings {
    /// Creates a new settings object that enables recording.
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Self::default()
        }
    }

    /// Configures the output path that [`Recorder::finish`] will write to when
    /// recording is enabled.
    pub fn with_output_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.output_path = Some(path.into());
        self
    }
}

/// Collects analysis frames so they can be exported after a run.
#[derive(Debug, Default)]
pub struct Recorder {
    settings: RecordingSettings,
    recorded_frames: usize,
    frames: Vec<AnalysisFrame>,
}

impl Recorder {
    pub fn new(settings: RecordingSettings) -> Self {
        let capacity = if settings.enabled { 64 } else { 0 };
        Self {
            settings,
            recorded_frames: 0,
            frames: Vec::with_capacity(capacity),
        }
    }

    pub fn settings(&self) -> &RecordingSettings {
        &self.settings
    }

    pub fn recorded_frames(&self) -> usize {
        self.recorded_frames
    }

    pub fn frames(&self) -> &[AnalysisFrame] {
        &self.frames
    }

    /// Enables recording in-place using the provided settings.
    pub fn configure(&mut self, settings: RecordingSettings) {
        self.settings = settings;
        self.frames.clear();
        self.recorded_frames = 0;
        if self.settings.enabled {
            self.frames.reserve(64);
        }
    }

    pub fn set_output_path(&mut self, path: impl Into<PathBuf>) {
        self.settings.output_path = Some(path.into());
    }

    pub fn record_frame(&mut self, frame: &AnalysisFrame) {
        if !self.settings.enabled {
            return;
        }

        self.frames.push(frame.clone());
        self.recorded_frames += 1;
    }

    pub fn flush_to_path(&self, path: impl AsRef<Path>) -> Result<()> {
        if !self.settings.enabled {
            return Err(MusicVizError::InvalidInput(
                "cannot flush recording while recording is disabled",
            ));
        }

        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let payload = RecordingExport {
            settings: &self.settings,
            frames: &self.frames,
        };
        let json = serde_json::to_string_pretty(&payload)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn finish(&mut self) -> Result<Option<PathBuf>> {
        if !self.settings.enabled {
            return Ok(None);
        }

        if let Some(path) = self.settings.output_path.clone() {
            self.flush_to_path(&path)?;
            Ok(Some(path))
        } else {
            Ok(None)
        }
    }
}

#[derive(Serialize)]
struct RecordingExport<'a> {
    settings: &'a RecordingSettings,
    frames: &'a [AnalysisFrame],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_frames_when_enabled() {
        let mut recorder = Recorder::new(RecordingSettings::enabled());
        let frame = AnalysisFrame::default();
        recorder.record_frame(&frame);
        recorder.record_frame(&frame);

        assert_eq!(recorder.recorded_frames(), 2);
        assert_eq!(recorder.frames().len(), 2);
    }

    #[test]
    fn flushes_to_disk() {
        let mut recorder = Recorder::new(RecordingSettings::enabled());
        recorder.set_output_path(temporary_path("music-viz-recorder.json"));
        recorder.record_frame(&AnalysisFrame {
            time: 0.5,
            rms: 0.25,
            ..Default::default()
        });

        let path = recorder.finish().unwrap().expect("path must be returned");
        let contents = fs::read_to_string(&path).unwrap();
        assert!(contents.contains("\"frames\""));
        fs::remove_file(path).unwrap();
    }

    fn temporary_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let unique = format!(
            "{}-{}-{}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        path.push(unique);
        path
    }
}
