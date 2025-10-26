use serde::{Deserialize, Serialize};

use crate::Result;

/// Configuration options for the recording subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSettings {
    pub output_path: String,
    pub fps: u32,
    pub bitrate: Option<u32>,
}

impl Default for RecordingSettings {
    fn default() -> Self {
        Self {
            output_path: String::new(),
            fps: 60,
            bitrate: None,
        }
    }
}

/// High level abstraction responsible for piping rendered frames into an
/// encoder such as FFmpeg.
#[derive(Debug, Default)]
pub struct Recorder {
    _settings: RecordingSettings,
    is_recording: bool,
}

impl Recorder {
    pub fn new(settings: RecordingSettings) -> Self {
        Self {
            _settings: settings,
            is_recording: false,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        self.is_recording = true;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.is_recording = false;
        Ok(())
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording
    }
}
