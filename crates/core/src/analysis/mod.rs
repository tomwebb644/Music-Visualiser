use std::{cmp::Ordering, f32::consts::PI, fmt, sync::Arc};

use realfft::{num_complex::Complex32, RealFftPlanner, RealToComplex};
use serde::{Deserialize, Serialize};

use crate::{AudioMode, MusicVizError, Result};

const BEAT_GAIN: f32 = 12.0;
const BEAT_THRESHOLD: f32 = 0.6;
const MIN_BEAT_INTERVAL: f32 = 0.2;
const MAX_BEAT_HISTORY: usize = 32;

/// Summary of the analysis metadata accumulated so far.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisSummary {
    pub sample_rate: u32,
    pub tempo_bpm: Option<f32>,
    pub duration_seconds: Option<f32>,
}

/// Representation of the feature set for a single timestamp.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisFrame {
    pub time: f32,
    pub rms: f32,
    /// Normalised [0, 1] spectral centroid where 1.0 corresponds to the
    /// Nyquist frequency of the analysed block.
    pub spectral_centroid: f32,
    pub beat_confidence: f32,
}

/// Lightweight DSP façade that focuses on a couple of simple features for the
/// live mode. The API is intentionally synchronous so it can be driven by tests
/// and by the command line demo in the application crate. The interface will be
/// preserved while the internals grow richer over time.
pub struct AnalysisEngine {
    mode: AudioMode,
    sample_rate: u32,
    summary: AnalysisSummary,
    frames: Vec<AnalysisFrame>,
    processed_samples: usize,
    last_rms: f32,
    beat_timestamps: Vec<f32>,
    fft_planner: RealFftPlanner<f32>,
    fft: Option<FftResources>,
}

impl AnalysisEngine {
    /// Creates a new engine using the default 48 kHz sample rate.
    pub fn new(mode: AudioMode) -> Self {
        Self::with_sample_rate(mode, 48_000)
    }

    /// Creates a new engine that operates at the provided sample rate.
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

    /// Returns metadata collected so far about the analysed stream.
    pub fn summary(&self) -> &AnalysisSummary {
        &self.summary
    }

    /// Returns the audio mode the engine operates in.
    pub fn mode(&self) -> AudioMode {
        self.mode
    }

    /// Returns the sample rate associated with the engine.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Clears the accumulated state while preserving configuration.
    pub fn reset(&mut self) {
        self.summary = AnalysisSummary {
            sample_rate: self.sample_rate,
            ..Default::default()
        };
        self.frames.clear();
        self.processed_samples = 0;
        self.last_rms = 0.0;
        self.beat_timestamps.clear();
    }

    /// Consumes audio samples and updates the tracked features.
    pub fn process_block(&mut self, samples: &[f32]) -> Result<AnalysisFrame> {
        if samples.is_empty() {
            return Err(MusicVizError::InvalidInput(
                "analysis requires at least one sample",
            ));
        }

        if samples.len() < 2 {
            return Err(MusicVizError::InvalidInput(
                "analysis requires blocks with at least two samples",
            ));
        }

        let block_size = samples.len();
        let sample_rate = self.sample_rate as f32;
        let start_time = self.processed_samples as f32 / sample_rate;
        let end_time = (self.processed_samples + block_size) as f32 / sample_rate;
        let timestamp = start_time + (end_time - start_time) * 0.5;

        let rms = compute_rms(samples);
        let beat_confidence = self.update_beats(timestamp, rms);
        let centroid_hz = self.compute_spectral_centroid(samples)?;
        let nyquist = (self.sample_rate as f32).max(1.0) * 0.5;
        let spectral_centroid = if nyquist > 0.0 {
            (centroid_hz / nyquist).clamp(0.0, 1.0)
        } else {
            0.0
        };

        self.processed_samples += block_size;
        self.summary.duration_seconds = Some(
            self.summary
                .duration_seconds
                .map(|d| d.max(end_time))
                .unwrap_or(end_time),
        );

        let frame = AnalysisFrame {
            time: timestamp,
            rms,
            spectral_centroid,
            beat_confidence,
        };

        self.frames.push(frame.clone());
        Ok(frame)
    }

    /// Returns the latest frame emitted by the engine, if any.
    pub fn latest_frame(&self) -> Option<&AnalysisFrame> {
        self.frames.last()
    }

    /// Returns all recorded frames.
    pub fn frames(&self) -> &[AnalysisFrame] {
        &self.frames
    }

    /// Samples the feature set at (or before) the requested time. If no frame
    /// exists before the timestamp a default frame is returned.
    pub fn sample_at(&self, time: f32) -> AnalysisFrame {
        match self
            .frames
            .binary_search_by(|frame| frame.time.partial_cmp(&time).unwrap_or(Ordering::Equal))
        {
            Ok(index) => self.frames[index].clone(),
            Err(0) => AnalysisFrame {
                time,
                ..Default::default()
            },
            Err(index) => self.frames[index - 1].clone(),
        }
    }

    fn update_beats(&mut self, timestamp: f32, rms: f32) -> f32 {
        let delta = (rms - self.last_rms).max(0.0);
        self.last_rms = rms;
        let confidence = (delta * BEAT_GAIN).clamp(0.0, 1.0);

        if confidence >= BEAT_THRESHOLD {
            if self
                .beat_timestamps
                .last()
                .map(|last| timestamp - last >= MIN_BEAT_INTERVAL)
                .unwrap_or(true)
            {
                self.beat_timestamps.push(timestamp);
                if self.beat_timestamps.len() > MAX_BEAT_HISTORY {
                    let overflow = self.beat_timestamps.len() - MAX_BEAT_HISTORY;
                    self.beat_timestamps.drain(0..overflow);
                }
                self.update_tempo_estimate();
            }
        }

        confidence
    }

    fn update_tempo_estimate(&mut self) {
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

    fn compute_spectral_centroid(&mut self, samples: &[f32]) -> Result<f32> {
        let len = samples.len();
        let sample_rate = self.sample_rate as f32;
        let fft = self.prepare_fft(len)?;

        for (index, value) in samples.iter().enumerate() {
            fft.input[index] = *value * hann_value(index, len);
        }

        fft.plan
            .process_with_scratch(&mut fft.input, &mut fft.spectrum, &mut fft.scratch)?;

        let mut magnitude_sum = 0.0;
        let mut weighted_sum = 0.0;
        let bin_hz = sample_rate / len as f32;

        for (i, bin) in fft.spectrum.iter().enumerate() {
            let magnitude = bin.norm();
            magnitude_sum += magnitude;
            weighted_sum += magnitude * (i as f32 * bin_hz);
        }

        if magnitude_sum <= f32::EPSILON {
            Ok(0.0)
        } else {
            Ok(weighted_sum / magnitude_sum)
        }
    }

    fn prepare_fft(&mut self, size: usize) -> Result<&mut FftResources> {
        let rebuild = self
            .fft
            .as_ref()
            .map(|fft| fft.size != size)
            .unwrap_or(true);

        if rebuild {
            let plan = self.fft_planner.plan_fft_forward(size);
            let scratch = plan.make_scratch_vec();
            let spectrum = plan.make_output_vec();
            let input = plan.make_input_vec();
            self.fft = Some(FftResources {
                size,
                plan,
                scratch,
                spectrum,
                input,
            });
        }

        Ok(self.fft.as_mut().expect("fft resources must exist"))
    }
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
            .field("frames", &self.frames.len())
            .field("processed_samples", &self.processed_samples)
            .field("last_rms", &self.last_rms)
            .field("beat_timestamps", &self.beat_timestamps.len())
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
        let frame = engine.process_block(&samples).unwrap();

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
        assert!((tempo - 120.0).abs() < 10.0);
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
