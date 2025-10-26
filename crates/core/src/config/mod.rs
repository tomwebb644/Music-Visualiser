use serde::{Deserialize, Serialize};

use crate::{
    assets::StlAsset, mapping::MappingDescriptor, record::RecordingSettings, scene::SceneDescriptor,
};

/// Root configuration object deserialised from project preset files.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub audio_mode: AudioConfig,
    #[serde(default)]
    pub scenes: Vec<SceneDescriptor>,
    #[serde(default)]
    pub mappings: Vec<MappingDescriptor>,
    #[serde(default)]
    pub recording: RecordingSettings,
    #[serde(default)]
    pub assets: Vec<StlAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioConfig {
    Live,
    Precomputed { analysis_path: String },
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self::Live
    }
}
