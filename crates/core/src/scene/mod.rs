use serde::{Deserialize, Serialize};

use crate::{analysis::AnalysisFrame, mapping::ParameterUpdate};

/// Enumeration of the core scene types supported by the renderer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SceneDescriptor {
    #[serde(rename = "stl")]
    Stl { asset: String, mode: StlMode },
    #[serde(rename = "kaleidoscope")]
    Kaleidoscope { order: u32 },
    #[serde(rename = "tunnel")]
    Tunnel { speed: f32 },
    #[serde(rename = "particles")]
    Particles { emit_rate: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StlMode {
    Mesh,
    Wireframe,
    PointCloud,
}

/// Runtime representation of a scene instance.
#[derive(Debug, Clone)]
pub struct SceneInstance {
    pub descriptor: SceneDescriptor,
}

impl SceneInstance {
    pub fn new(descriptor: SceneDescriptor) -> Self {
        Self { descriptor }
    }

    /// Applies parameter updates derived from the mapping matrix to this scene.
    pub fn apply_updates(&mut self, _updates: &[ParameterUpdate]) {}

    /// Evaluates the scene against the current analysis frame.
    pub fn update(&mut self, _analysis: &AnalysisFrame) {}
}
