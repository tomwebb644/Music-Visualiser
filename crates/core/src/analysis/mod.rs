use std::{cmp::Ordering, f32::consts::PI, fmt, sync::Arc};

use realfft::{num_complex::Complex32, RealFftPlanner, RealToComplex};
use serde::{Deserialize, Serialize};

use crate::{AudioMode, MusicVizError, Result};

const BEAT_GAIN: f32 = 12.0;
const BEAT_THRESHOLD: f32 = 0.6;
const MIN_BEAT_INTERVAL: f32 = 0.2;
const MAX_BEAT_HISTORY: usize = 64;

/// Summary of analysis metadata that higher level systems can rely on without
/// knowing the internal buffer layout yet.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisSummary {
    pub sample_rate: u32,
    pub tempo_bpm: Option<f32>,
    pub duration_seconds: Option<f32>,
}

/// Thin façade that will eventually host the real DSP pipeline. The current
/// implementation focuses on basic time-domain and spectral features so that
/// higher level systems have meaningful data to react to while more advanced
/// analysis is developed.
pub struct AnalysisEngine {
    mode: AudioMode,
    sample_rate: u32,
    summary: AnalysisSummary,
    frames: Vec<FrameRecord>,
    processed_samples: usize,
    last_rms: f32,
    beat_timestamps: Vec<f32>,
    fft_planner: RealFftPlanner<f32>,
    fft: Option<FftResources>,
}

impl AnalysisEngine {
    /// Creates a new engine using the default sample rate (48 kHz).
    pub fn new(mode: AudioMode) -> Self {
        Self::with_sample_rate(mode, 48_000)
    }

    /// Creates a new engine using the provided sample rate. This is primarily
    /// useful for tests or when the host audio device operates at a different
    /// rate.
    pub fn with_sample_rate(mode: AudioMode, sample_rate: u32) -> Self {
        Self {
            mode,
            sample_rate,
            summary: AnalysisSummary {
                sample_rate,
                ..Default::default()
            },
            frames: Vec::new(),
            processed_samples: 0,
            last_rms: 0.0,
            beat_timestamps: Vec::new(),
            fft_planner: RealFftPlanner::new(),
            fft: None,
        }
    }

    /// Returns metadata collected so far about the analysed audio stream.
    pub fn summary(&self) -> &AnalysisSummary {
        &self.summary
    }

    /// Returns the audio mode the engine is tracking.
    pub fn mode(&self) -> AudioMode {
        self.mode
    }

    /// Returns the sample rate associated with the analysis engine.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Consumes audio frames and updates internal feature buffers.
    pub fn process_block(&mut self, samples: &[f32]) -> Result<()> {
        if samples.is_empty() {
            return Ok(());
        }

        let block_size = samples.len();
        if block_size < 2 {
            return Err(MusicVizError::msg(
                "analysis requires blocks with at least two samples",
            ));
        }

        let sample_rate = self.sample_rate as f32;
        let start_time = self.processed_samples as f32 / sample_rate;
        let end_time = (self.processed_samples + block_size) as f32 / sample_rate;
        let timestamp = start_time + (end_time - start_time) * 0.5;

        let rms = compute_rms(samples);
        let previous_rms = self.last_rms;
        self.last_rms = rms;

        let centroid_hz = self.compute_spectral_centroid(samples)?;
        let nyquist = (self.sample_rate as f32).max(1.0) * 0.5;
        let spectral_centroid = if nyquist > 0.0 {
            (centroid_hz / nyquist).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let beat_delta = (rms - previous_rms).max(0.0);
        let beat_confidence = (beat_delta * BEAT_GAIN).clamp(0.0, 1.0);

        let frame = AnalysisFrame {
            time: timestamp,
            rms,
            spectral_centroid,
            beat_confidence,
        };

        self.frames.push(FrameRecord {
            time: timestamp,
            frame: frame.clone(),
        });

        self.processed_samples += block_size;
        self.summary.duration_seconds = Some(end_time);

        if beat_confidence > BEAT_THRESHOLD {
            let allow_push = self
                .beat_timestamps
                .last()
                .map(|last| timestamp - *last > MIN_BEAT_INTERVAL)
                .unwrap_or(true);
            if allow_push {
                self.beat_timestamps.push(timestamp);
                if self.beat_timestamps.len() > MAX_BEAT_HISTORY {
                    let excess = self.beat_timestamps.len() - MAX_BEAT_HISTORY;
                    self.beat_timestamps.drain(0..excess);
                }
                self.update_tempo();
            }
        }

        Ok(())
    }

    /// Produces a snapshot of analysis data for the requested timestamp.
    pub fn sample_at(&self, seconds: f32) -> AnalysisFrame {
        if self.frames.is_empty() {
            return AnalysisFrame::default();
        }

        let target = seconds.max(0.0);
        let idx = match self
            .frames
            .binary_search_by(|frame| frame.time.partial_cmp(&target).unwrap_or(Ordering::Equal))
        {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };

        self.frames
            .get(idx)
            .map(|record| record.frame.clone())
            .unwrap_or_else(AnalysisFrame::default)
    }

    fn ensure_fft(&mut self, len: usize) -> Result<&mut FftResources> {
        if len < 2 {
            return Err(MusicVizError::msg(
                "analysis requires blocks with at least two samples",
            ));
        }

        let recreate = match self.fft {
            Some(ref resources) if resources.size == len => false,
            _ => true,
        };

        if recreate {
            let plan = self.fft_planner.plan_fft_forward(len);
            let scratch = plan.make_scratch_vec();
            let spectrum = plan.make_output_vec();
            let input = vec![0.0; len];
            self.fft = Some(FftResources {
                size: len,
                plan,
                scratch,
                spectrum,
                input,
            });
        }

        Ok(self.fft.as_mut().expect("FFT resources must exist"))
    }

    fn compute_spectral_centroid(&mut self, samples: &[f32]) -> Result<f32> {
        let len = samples.len();
        if len < 2 {
            return Ok(0.0);
        }

        let sample_rate = self.sample_rate as f32;
        let resources = self.ensure_fft(len)?;
        for (idx, value) in samples.iter().enumerate() {
            let window = hann_value(idx, len);
            resources.input[idx] = value * window;
        }

        resources.plan.process_with_scratch(
            &mut resources.input,
            &mut resources.spectrum,
            &mut resources.scratch,
        )?;

        let mut weighted = 0.0;
        let mut sum = 0.0;
        for (bin, complex) in resources.spectrum.iter().enumerate() {
            let magnitude = complex.norm();
            if magnitude <= f32::EPSILON {
                continue;
            }
            let frequency = (bin as f32 * sample_rate) / len as f32;
            weighted += frequency * magnitude;
            sum += magnitude;
        }

        if sum > 0.0 {
            Ok(weighted / sum)
        } else {
            Ok(0.0)
        }
    }

    fn update_tempo(&mut self) {
        if self.beat_timestamps.len() < 2 {
            return;
        }

        let mut sum = 0.0;
        let mut count = 0;
        for window in self.beat_timestamps.windows(2) {
            let interval = window[1] - window[0];
            if interval > f32::EPSILON {
                sum += interval;
                count += 1;
            }
        }

        if count > 0 {
            let average_interval = sum / count as f32;
            if average_interval > 0.0 {
                self.summary.tempo_bpm = Some(60.0 / average_interval);
            }
        }
    }
}

/// Representation of the features available for a single timestamp.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisFrame {
    pub time: f32,
    pub rms: f32,
    /// Normalised [0, 1] spectral centroid where 1 represents the Nyquist
    /// frequency for the analysed block.
    pub spectral_centroid: f32,
    pub beat_confidence: f32,
}

#[derive(Debug, Clone)]
struct FrameRecord {
    time: f32,
    frame: AnalysisFrame,
}

struct FftResources {
    size: usize,
    plan: Arc<dyn RealToComplex<f32>>,
    scratch: Vec<Complex32>,
    spectrum: Vec<Complex32>,
    input: Vec<f32>,
}

impl fmt::Debug for AnalysisEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnalysisEngine")
            .field("mode", &self.mode)
            .field("sample_rate", &self.sample_rate)
            .field("summary", &self.summary)
            .field("frames", &self.frames)
            .field("processed_samples", &self.processed_samples)
            .field("last_rms", &self.last_rms)
            .field("beat_timestamps", &self.beat_timestamps)
            .finish()
    }
}

impl fmt::Debug for FftResources {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FftResources")
            .field("size", &self.size)
            .finish()
    }
}

fn compute_rms(samples: &[f32]) -> f32 {
    let sum: f32 = samples.iter().map(|sample| sample * sample).sum();
    (sum / samples.len() as f32).sqrt()
}

fn hann_value(index: usize, len: usize) -> f32 {
    if len <= 1 {
        return 1.0;
    }
    0.5 - 0.5 * ((2.0 * PI * index as f32) / (len as f32 - 1.0)).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_engine(sample_rate: u32) -> AnalysisEngine {
        AnalysisEngine::with_sample_rate(AudioMode::Live, sample_rate)
    }

    #[test]
    fn computes_basic_features() {
        let mut engine = build_engine(48_000);
        let samples: Vec<f32> = vec![0.0; 1024];
        engine.process_block(&samples).unwrap();

        let frame = engine.sample_at(0.0);
        assert!((frame.rms - 0.0).abs() <= f32::EPSILON);
        assert_eq!(frame.beat_confidence, 0.0);
        assert_eq!(frame.spectral_centroid, 0.0);
    }

    #[test]
    fn updates_duration_and_tempo() {
        let mut engine = build_engine(100);
        let quiet = vec![0.0; 25];
        let loud = vec![1.0; 25];

        for _ in 0..4 {
            engine.process_block(&quiet).unwrap();
            engine.process_block(&loud).unwrap();
        }

        let summary = engine.summary();
        assert!(summary.duration_seconds.unwrap() > 0.0);
        let tempo = summary.tempo_bpm.expect("tempo should be detected");
        assert!((tempo - 120.0).abs() < 5.0);
    }

    #[test]
    fn sampling_interpolates_to_previous_frame() {
        let mut engine = build_engine(10);
        let block = vec![1.0; 10];
        engine.process_block(&block).unwrap();
        engine.process_block(&block).unwrap();

        let frame = engine.sample_at(1.5);
        assert!(frame.time <= 1.5);
    }
}
