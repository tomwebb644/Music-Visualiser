use std::collections::HashMap;

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
    parameters: HashMap<String, f32>,
    last_analysis: Option<AnalysisFrame>,
}

impl SceneInstance {
    pub fn new(descriptor: SceneDescriptor) -> Self {
        Self {
            descriptor,
            parameters: HashMap::new(),
            last_analysis: None,
        }
    }

    /// Applies parameter updates derived from the mapping matrix to this scene.
    pub fn apply_updates(&mut self, updates: &[ParameterUpdate]) {
        for update in updates {
            self.parameters.insert(update.target.clone(), update.value);
        }
    }

    /// Evaluates the scene against the current analysis frame.
    pub fn update(&mut self, analysis: &AnalysisFrame) {
        self.last_analysis = Some(analysis.clone());

        match &self.descriptor {
            SceneDescriptor::Stl { .. } => {
                self.parameters
                    .entry("stl.rms".to_string())
                    .or_insert(analysis.rms);
                self.parameters
                    .entry("stl.centroid".to_string())
                    .or_insert(analysis.spectral_centroid);
            }
            SceneDescriptor::Kaleidoscope { order } => {
                let base = (*order).max(1) as f32;
                self.parameters
                    .entry("kaleidoscope.rotation".to_string())
                    .or_insert(analysis.beat_confidence * base);
            }
            SceneDescriptor::Tunnel { .. } => {
                self.parameters
                    .entry("tunnel.energy".to_string())
                    .or_insert(analysis.rms);
            }
            SceneDescriptor::Particles { .. } => {
                self.parameters
                    .entry("particles.emission".to_string())
                    .or_insert(analysis.rms);
                self.parameters
                    .entry("particles.tone".to_string())
                    .or_insert(analysis.spectral_centroid);
            }
        }
    }

    pub fn parameter_value(&self, key: &str) -> Option<f32> {
        self.parameters.get(key).copied()
    }

    pub fn last_analysis(&self) -> Option<&AnalysisFrame> {
        self.last_analysis.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(rms: f32, centroid: f32, beat: f32) -> AnalysisFrame {
        AnalysisFrame {
            time: 0.0,
            rms,
            spectral_centroid: centroid,
            beat_confidence: beat,
        }
    }

    #[test]
    fn updates_store_parameter_values() {
        let mut scene = SceneInstance::new(SceneDescriptor::Kaleidoscope { order: 4 });
        scene.update(&frame(0.5, 0.25, 0.8));

        let rotation = scene.parameter_value("kaleidoscope.rotation").unwrap();
        assert!(rotation > 0.0);
        assert!(scene.last_analysis().is_some());
    }

    #[test]
    fn mapping_updates_override_defaults() {
        let mut scene = SceneInstance::new(SceneDescriptor::Tunnel { speed: 1.0 });
        scene.update(&frame(0.1, 0.5, 0.0));
        scene.apply_updates(&[ParameterUpdate {
            target: "tunnel.energy".to_string(),
            value: 0.75,
        }]);

        assert_eq!(scene.parameter_value("tunnel.energy"), Some(0.75));
    }
}
