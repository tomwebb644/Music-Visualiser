use serde::{Deserialize, Serialize};

use crate::AnalysisFrame;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSettings {
    pub enabled: bool,
}

impl Default for RecordingSettings {
    fn default() -> Self {
        Self { enabled: false }
    }
}

#[derive(Debug, Default)]
pub struct Recorder {
    settings: RecordingSettings,
    recorded_frames: usize,
}

impl Recorder {
    pub fn new(settings: RecordingSettings) -> Self {
        Self {
            settings,
            recorded_frames: 0,
        }
    }

    pub fn settings(&self) -> &RecordingSettings {
        &self.settings
    }

    pub fn recorded_frames(&self) -> usize {
        self.recorded_frames
    }

    pub fn record_frame(&mut self, frame: &AnalysisFrame) {
        if self.settings.enabled {
            let _ = frame.time;
            self.recorded_frames += 1;
        }
    }
}
