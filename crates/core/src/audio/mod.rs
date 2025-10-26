use std::sync::{Arc, Mutex, MutexGuard};

use crate::{AnalysisEngine, AnalysisFrame, AnalysisSummary, MusicVizError, PlaybackClock, Result};

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
    sample_rate: u32,
    analysis: Arc<Mutex<AnalysisEngine>>,
    clock: Arc<Mutex<Option<PlaybackClock>>>,
}

impl AudioEngine {
    /// Creates a new audio engine instance in the requested mode.
    pub fn new(mode: AudioMode) -> Self {
        Self::with_sample_rate(mode, 48_000)
    }

    /// Creates a new audio engine instance using an explicit sample rate.
    pub fn with_sample_rate(mode: AudioMode, sample_rate: u32) -> Self {
        let analysis = AnalysisEngine::with_sample_rate(mode, sample_rate);
        Self {
            mode,
            sample_rate,
            analysis: Arc::new(Mutex::new(analysis)),
            clock: Arc::new(Mutex::new(None)),
        }
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
    /// returns a handle to the [`AnalysisEngine`] that can later be wired to
    /// actual audio capture or file decoding backends.
    pub fn start(&self) -> Result<AnalysisHandle> {
        {
            let mut engine = self.lock_analysis()?;
            if engine.mode() != self.mode || engine.sample_rate() != self.sample_rate {
                *engine = AnalysisEngine::with_sample_rate(self.mode, self.sample_rate);
            } else {
                engine.reset();
            }
        }

        Ok(AnalysisHandle::new(self.analysis.clone()))
    }

    /// Feeds a block of floating point samples into the engine. Live capture
    /// will call this repeatedly. Precomputed mode can use it to inject decoded
    /// frames during deterministic playback.
    pub fn push_samples(&self, samples: &[f32]) -> Result<()> {
        if samples.is_empty() {
            return Ok(());
        }

        let mut engine = self.lock_analysis()?;
        engine.process_block(samples)
    }

    /// Associates the engine with a playback clock so that downstream systems
    /// can maintain deterministic timing.
    pub fn attach_clock(&self, clock: PlaybackClock) -> Result<()> {
        let mut slot = self.lock_clock()?;
        *slot = Some(clock);
        Ok(())
    }

    /// Returns the playback clock currently driving the audio engine, if any.
    pub fn playback_clock(&self) -> Result<Option<PlaybackClock>> {
        let slot = self.lock_clock()?;
        Ok(slot.clone())
    }

    fn lock_analysis(&self) -> Result<MutexGuard<'_, AnalysisEngine>> {
        self.analysis
            .lock()
            .map_err(|_| MusicVizError::msg("analysis pipeline has been poisoned"))
    }

    fn lock_clock(&self) -> Result<MutexGuard<'_, Option<PlaybackClock>>> {
        self.clock
            .lock()
            .map_err(|_| MusicVizError::msg("playback clock has been poisoned"))
    }
}

/// Shared, thread-safe view over the analysis engine managed by [`AudioEngine`].
#[derive(Clone)]
pub struct AnalysisHandle {
    shared: Arc<Mutex<AnalysisEngine>>,
}

impl AnalysisHandle {
    pub(crate) fn new(shared: Arc<Mutex<AnalysisEngine>>) -> Self {
        Self { shared }
    }

    /// Samples the current analysis timeline at the requested timestamp.
    pub fn sample_at(&self, seconds: f32) -> Result<AnalysisFrame> {
        let engine = self.lock()?;
        Ok(engine.sample_at(seconds))
    }

    /// Returns an up-to-date summary of the analysed stream.
    pub fn summary(&self) -> Result<AnalysisSummary> {
        let engine = self.lock()?;
        Ok(engine.summary().clone())
    }

    fn lock(&self) -> Result<MutexGuard<'_, AnalysisEngine>> {
        self.shared
            .lock()
            .map_err(|_| MusicVizError::msg("analysis pipeline has been poisoned"))
    }
}

impl std::fmt::Debug for AnalysisHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalysisHandle").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pushes_samples_into_shared_analysis() {
        let audio = AudioEngine::with_sample_rate(AudioMode::Live, 100);
        let analysis = audio.start().unwrap();

        audio
            .push_samples(&[1.0_f32; 32])
            .expect("pushing samples should succeed");

        let frame = analysis.sample_at(0.0).unwrap();
        assert!(frame.rms > 0.0);
    }

    #[test]
    fn attaches_and_reads_playback_clock() {
        let audio = AudioEngine::new(AudioMode::Precomputed);
        audio.attach_clock(PlaybackClock::start()).unwrap();

        let stored = audio.playback_clock().unwrap();
        assert!(stored.is_some());
        let stored = stored.unwrap();
        assert!(stored.elapsed() >= std::time::Duration::ZERO);
    }
}
