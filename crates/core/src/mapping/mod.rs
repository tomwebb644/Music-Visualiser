use serde::{Deserialize, Serialize};

use crate::analysis::AnalysisFrame;

/// Description of a single modulation mapping between an audio feature and a
/// visual parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingDescriptor {
    pub source: String,
    pub target: String,
    pub min: f32,
    pub max: f32,
    #[serde(default)]
    pub smoothing: f32,
}

impl MappingDescriptor {
    pub fn apply(&self, analysis: &AnalysisFrame) -> f32 {
        // TODO: evaluate expressions, look up specific features, etc.
        let normalised = analysis.rms.clamp(0.0, 1.0);
        self.min + (self.max - self.min) * normalised
    }
}

/// Collection of active mappings.
#[derive(Debug, Default, Clone)]
pub struct MappingMatrix {
    pub mappings: Vec<MappingDescriptor>,
}

impl MappingMatrix {
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }

    pub fn evaluate(&self, analysis: &AnalysisFrame) -> Vec<ParameterUpdate> {
        self.mappings
            .iter()
            .map(|mapping| ParameterUpdate {
                target: mapping.target.clone(),
                value: mapping.apply(analysis),
            })
            .collect()
    }

    pub fn add_mapping(&mut self, mapping: MappingDescriptor) {
        self.mappings.push(mapping);
    }
}

/// Result of applying a mapping to an analysis frame.
#[derive(Debug, Clone)]
pub struct ParameterUpdate {
    pub target: String,
    pub value: f32,
}
