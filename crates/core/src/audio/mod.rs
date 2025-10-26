use crate::{AnalysisEngine, AnalysisFrame, MusicVizError, Result};

/// Operating modes supported by the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioMode {
    Live,
    Precomputed,
}

/// Lightweight audio engine façade used by the command line demo and tests.
pub struct AudioEngine {
    mode: AudioMode,
    analysis: AnalysisEngine,
}

impl AudioEngine {
    pub fn new(mode: AudioMode) -> Self {
        Self::with_sample_rate(mode, 48_000)
    }

    pub fn with_sample_rate(mode: AudioMode, sample_rate: u32) -> Self {
        let analysis = AnalysisEngine::with_sample_rate(mode, sample_rate);
        Self { mode, analysis }
    }

    pub fn new_live() -> Self {
        Self::new(AudioMode::Live)
    }

    pub fn mode(&self) -> AudioMode {
        self.mode
    }

    pub fn analysis(&self) -> &AnalysisEngine {
        &self.analysis
    }

    pub fn analysis_mut(&mut self) -> &mut AnalysisEngine {
        &mut self.analysis
    }

    pub fn process_live_block(&mut self, samples: &[f32]) -> Result<AnalysisFrame> {
        match self.mode {
            AudioMode::Live => self.analysis.process_block(samples),
            AudioMode::Precomputed => Err(MusicVizError::InvalidInput(
                "cannot process live block while in precomputed mode",
            )),
        }
    }

    pub fn prepare_precomputed(&mut self) -> Result<()> {
        match self.mode {
            AudioMode::Live => Err(MusicVizError::InvalidInput(
                "precomputed preparation is not available in live mode",
            )),
            AudioMode::Precomputed => Err(MusicVizError::Unimplemented(
                "precomputed analysis pipeline",
            )),
        }
    }

    pub fn analysis_handle(&self) -> AnalysisHandle<'_> {
        AnalysisHandle {
            engine: &self.analysis,
        }
    }
}

/// Read-only façade used by systems that only need to inspect the current
/// analysis state.
pub struct AnalysisHandle<'a> {
    engine: &'a AnalysisEngine,
}

impl<'a> AnalysisHandle<'a> {
    pub fn summary(&self) -> &crate::AnalysisSummary {
        self.engine.summary()
    }

    pub fn latest_frame(&self) -> Option<&AnalysisFrame> {
        self.engine.latest_frame()
    }
}
