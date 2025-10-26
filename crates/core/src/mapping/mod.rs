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
    fn feature_value(&self, analysis: &AnalysisFrame) -> Option<f32> {
        match self.source.as_str() {
            "rms" => Some(analysis.rms),
            "spectral_centroid" => Some(analysis.spectral_centroid),
            "beat_confidence" => Some(analysis.beat_confidence),
            _ => None,
        }
    }

    fn map_value(&self, value: f32) -> f32 {
        let clamped = value.clamp(0.0, 1.0);
        self.min + (self.max - self.min) * clamped
    }
}

/// Collection of active mappings.
#[derive(Debug, Default, Clone)]
pub struct MappingMatrix {
    mappings: Vec<ActiveMapping>,
}

impl MappingMatrix {
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }

    pub fn evaluate(&mut self, analysis: &AnalysisFrame) -> Vec<ParameterUpdate> {
        self.mappings
            .iter_mut()
            .filter_map(|state| state.evaluate(analysis))
            .collect()
    }

    pub fn add_mapping(&mut self, mapping: MappingDescriptor) {
        self.mappings.push(ActiveMapping {
            descriptor: mapping,
            last_value: None,
        });
    }

    pub fn clear(&mut self) {
        self.mappings.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }
}

#[derive(Debug, Clone)]
struct ActiveMapping {
    descriptor: MappingDescriptor,
    last_value: Option<f32>,
}

impl ActiveMapping {
    fn evaluate(&mut self, analysis: &AnalysisFrame) -> Option<ParameterUpdate> {
        let feature = self.descriptor.feature_value(analysis)?;
        let mut mapped = self.descriptor.map_value(feature);

        if self.descriptor.smoothing > 0.0 {
            let smoothing = self.descriptor.smoothing.clamp(0.0, 0.99);
            let factor = 1.0 - smoothing;
            let previous = self.last_value.unwrap_or(mapped);
            mapped = previous + (mapped - previous) * factor;
        }

        self.last_value = Some(mapped);

        Some(ParameterUpdate {
            target: self.descriptor.target.clone(),
            value: mapped,
        })
    }
}

/// Result of applying a mapping to an analysis frame.
#[derive(Debug, Clone)]
pub struct ParameterUpdate {
    pub target: String,
    pub value: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(source: &str, smoothing: f32) -> MappingDescriptor {
        MappingDescriptor {
            source: source.to_string(),
            target: "param".to_string(),
            min: 0.0,
            max: 1.0,
            smoothing,
        }
    }

    fn frame_with_values(rms: f32, centroid: f32, beat: f32) -> AnalysisFrame {
        AnalysisFrame {
            time: 0.0,
            rms,
            spectral_centroid: centroid,
            beat_confidence: beat,
        }
    }

    #[test]
    fn applies_known_sources() {
        let mut matrix = MappingMatrix::new();
        matrix.add_mapping(descriptor("rms", 0.0));
        matrix.add_mapping(descriptor("spectral_centroid", 0.0));
        matrix.add_mapping(descriptor("beat_confidence", 0.0));

        let frame = frame_with_values(0.5, 0.25, 1.0);
        let updates = matrix.evaluate(&frame);
        assert_eq!(updates.len(), 3);
        assert!(updates.iter().any(|u| (u.value - 0.5).abs() < 1e-6));
        assert!(updates.iter().any(|u| (u.value - 0.25).abs() < 1e-6));
        assert!(updates.iter().any(|u| (u.value - 1.0).abs() < 1e-6));
    }

    #[test]
    fn smoothing_applies_progressively() {
        let mut matrix = MappingMatrix::new();
        matrix.add_mapping(descriptor("rms", 0.5));

        let first = frame_with_values(0.0, 0.0, 0.0);
        let mut updates = matrix.evaluate(&first);
        assert!((updates[0].value - 0.0).abs() < 1e-6);

        let second = frame_with_values(1.0, 0.0, 0.0);
        updates = matrix.evaluate(&second);
        assert!(updates[0].value < 1.0);
        assert!(updates[0].value > 0.0);
    }

    #[test]
    fn ignores_unknown_sources() {
        let mut matrix = MappingMatrix::new();
        matrix.add_mapping(descriptor("unknown", 0.0));

        let frame = frame_with_values(1.0, 1.0, 1.0);
        let updates = matrix.evaluate(&frame);
        assert!(updates.is_empty());
    }
}
