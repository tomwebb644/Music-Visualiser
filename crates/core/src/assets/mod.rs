use serde::{Deserialize, Serialize};

use crate::{scene::SceneDescriptor, Result};

/// Descriptor for STL assets that can be loaded at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StlAsset {
    pub path: String,
    #[serde(default)]
    pub scale: f32,
}

impl Default for StlAsset {
    fn default() -> Self {
        Self {
            path: String::new(),
            scale: 1.0,
        }
    }
}

/// Registry for all assets referenced by scenes.
#[derive(Debug, Default)]
pub struct AssetStore {
    stl_assets: Vec<StlAsset>,
}

impl AssetStore {
    pub fn new() -> Self {
        Self {
            stl_assets: Vec::new(),
        }
    }

    pub fn register_stl(&mut self, asset: StlAsset) {
        self.stl_assets.push(asset);
    }

    pub fn resolve_scene_assets(&self, _scene: &SceneDescriptor) -> Result<()> {
        // TODO: perform actual loading and GPU upload.
        Ok(())
    }
}
