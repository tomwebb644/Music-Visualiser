use serde::{Deserialize, Serialize};

/// Top-level configuration structure for the application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub audio: AudioConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            audio: AudioConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn live_defaults() -> Self {
        Self::default()
    }
}

/// Configuration specific to the audio subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub block_size: usize,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48_000,
            block_size: 1024,
        }
    }
}
