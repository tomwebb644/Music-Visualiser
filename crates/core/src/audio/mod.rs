use crate::{AnalysisEngine, PlaybackClock, Result};

/// Mode enum describes how the audio subsystem operates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioMode {
    /// Capture the host audio output with low latency.
    Live,
    /// Use pre-analysed data derived from an audio file.
    Precomputed,
}

/// High level audio engine façade.
#[derive(Debug)]
pub struct AudioEngine {
    mode: AudioMode,
}

impl AudioEngine {
    /// Creates a new audio engine instance in the requested mode.
    pub fn new(mode: AudioMode) -> Self {
        Self { mode }
    }

    /// Returns the currently configured audio mode.
    pub fn mode(&self) -> AudioMode {
        self.mode
    }

    /// Starts audio processing and returns a handle to the analysis pipeline.
    ///
    /// The current implementation is intentionally lightweight—it simply
    /// returns an [`AnalysisEngine`] that can later be wired to actual audio
    /// capture or file decoding backends.
    pub fn start(&self) -> Result<AnalysisEngine> {
        Ok(AnalysisEngine::new(self.mode))
    }

    /// Feeds a block of floating point samples into the engine. Live capture
    /// will call this repeatedly. Precomputed mode can use it to inject decoded
    /// frames during deterministic playback.
    pub fn push_samples(&self, _samples: &[f32]) -> Result<()> {
        // TODO: wire this to the real analysis pipeline once implemented.
        Ok(())
    }

    /// Associates the engine with a playback clock so that downstream systems
    /// can maintain deterministic timing.
    pub fn attach_clock(&self, _clock: PlaybackClock) -> Result<()> {
        Ok(())
    }
}
