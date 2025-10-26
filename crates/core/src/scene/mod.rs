use serde::{Deserialize, Serialize};

use crate::ParameterUpdate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SceneKind {
    Kaleidoscope,
    Tunnel,
    Stl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StlMode {
    Mesh,
    Wireframe,
    PointCloud,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneDescriptor {
    pub name: String,
    pub kind: SceneKind,
    pub stl_mode: Option<StlMode>,
}

impl SceneDescriptor {
    pub fn live_demo() -> Self {
        Self {
            name: "Live Demo".to_string(),
            kind: SceneKind::Kaleidoscope,
            stl_mode: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SceneInstance {
    pub descriptor: SceneDescriptor,
    pub intensity: f32,
    pub motion: f32,
    pub beat_emphasis: f32,
}

impl SceneInstance {
    pub fn new(descriptor: SceneDescriptor) -> Self {
        Self {
            descriptor,
            intensity: 0.0,
            motion: 0.0,
            beat_emphasis: 0.0,
        }
    }

    pub fn apply_updates(&mut self, updates: &[ParameterUpdate]) {
        for update in updates {
            match update.target.as_str() {
                "intensity" => self.intensity = update.value,
                "motion" => self.motion = update.value,
                "beat" => self.beat_emphasis = update.value,
                _ => {}
            }
        }
    }
}
