use serde::{Deserialize, Serialize};

use crate::{AudioMode, Result};

/// Summary of analysis metadata that higher level systems can rely on without
/// knowing the internal buffer layout yet.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisSummary {
    pub sample_rate: u32,
    pub tempo_bpm: Option<f32>,
    pub duration_seconds: Option<f32>,
}

/// Thin façade that will eventually host the real DSP pipeline.
#[derive(Debug)]
pub struct AnalysisEngine {
    _mode: AudioMode,
    summary: AnalysisSummary,
}

impl AnalysisEngine {
    pub fn new(mode: AudioMode) -> Self {
        Self {
            _mode: mode,
            summary: AnalysisSummary {
                sample_rate: 48_000,
                ..Default::default()
            },
        }
    }

    /// Returns metadata collected so far about the analysed audio stream.
    pub fn summary(&self) -> &AnalysisSummary {
        &self.summary
    }

    /// Consumes audio frames and updates internal feature buffers. Right now it
    /// simply records the fact that the engine was invoked.
    pub fn process_block(&mut self, _samples: &[f32]) -> Result<()> {
        Ok(())
    }

    /// Produces a snapshot of analysis data for the requested timestamp.
    pub fn sample_at(&self, _seconds: f32) -> AnalysisFrame {
        AnalysisFrame::default()
    }
}

/// Placeholder structure representing the features available for a single
/// timestamp.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisFrame {
    pub rms: f32,
    pub spectral_centroid: f32,
    pub beat_confidence: f32,
}
