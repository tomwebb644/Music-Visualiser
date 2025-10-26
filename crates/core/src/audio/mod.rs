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
#[derive(Debug, Clone, Copy)]
pub struct AudioEngine {
    mode: AudioMode,
    sample_rate: u32,
}

impl AudioEngine {
    /// Creates a new audio engine instance in the requested mode.
    pub fn new(mode: AudioMode) -> Self {
        Self::with_sample_rate(mode, 48_000)
    }

    /// Creates a new audio engine instance using an explicit sample rate.
    pub fn with_sample_rate(mode: AudioMode, sample_rate: u32) -> Self {
        Self { mode, sample_rate }
    }

    /// Returns the currently configured audio mode.
    pub fn mode(&self) -> AudioMode {
        self.mode
    }

    /// Returns the sample rate the engine operates at.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Starts audio processing and returns a handle to the analysis pipeline.
    ///
    /// The current implementation is intentionally lightweight—it simply
    /// returns an [`AnalysisEngine`] that can later be wired to actual audio
    /// capture or file decoding backends.
    pub fn start(&self) -> Result<AnalysisEngine> {
        Ok(AnalysisEngine::with_sample_rate(
            self.mode,
            self.sample_rate,
        ))
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
