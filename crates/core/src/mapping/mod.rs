use serde::{Deserialize, Serialize};

use crate::AnalysisFrame;

/// Describes how a feature should be routed to a render parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingDescriptor {
    pub source: String,
    pub target: String,
    pub gain: f32,
}

/// Runtime mapping matrix populated with [`ParameterUpdate`] values after each
/// analysis frame is processed.
#[derive(Debug, Default, Clone)]
pub struct MappingMatrix {
    updates: Vec<ParameterUpdate>,
}

impl MappingMatrix {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.updates.clear();
    }

    pub fn updates(&self) -> &[ParameterUpdate] {
        &self.updates
    }

    pub fn push(&mut self, update: ParameterUpdate) {
        self.updates.push(update);
    }

    pub fn apply_from_frame(&mut self, frame: &AnalysisFrame) {
        self.clear();
        self.push(ParameterUpdate {
            target: "intensity".to_string(),
            value: frame.rms,
        });
        self.push(ParameterUpdate {
            target: "motion".to_string(),
            value: frame.spectral_centroid,
        });
        self.push(ParameterUpdate {
            target: "beat".to_string(),
            value: frame.beat_confidence,
        });
        self.push(ParameterUpdate {
            target: "bass".to_string(),
            value: frame.low_band_energy,
        });
        self.push(ParameterUpdate {
            target: "treble".to_string(),
            value: frame.high_band_energy,
        });
        self.push(ParameterUpdate {
            target: "flux".to_string(),
            value: frame.spectral_flux,
        });
    }
}

/// Concrete value routed to a render or scene parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterUpdate {
    pub target: String,
    pub value: f32,
}
