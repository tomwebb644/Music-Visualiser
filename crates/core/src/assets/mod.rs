use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{scene::SceneDescriptor, MusicVizError, Result};

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
    stl_assets: HashMap<String, StlAsset>,
}

impl AssetStore {
    pub fn new() -> Self {
        Self {
            stl_assets: HashMap::new(),
        }
    }

    pub fn register_stl(&mut self, asset: StlAsset) {
        self.stl_assets.insert(asset.path.clone(), asset);
    }

    pub fn resolve_scene_assets(&self, scene: &SceneDescriptor) -> Result<()> {
        match scene {
            SceneDescriptor::Stl { asset, .. } => {
                if self.stl_assets.contains_key(asset) {
                    Ok(())
                } else {
                    Err(MusicVizError::msg(format!(
                        "unknown STL asset `{asset}` referenced by scene"
                    )))
                }
            }
            _ => Ok(()),
        }
    }

    pub fn stl_asset(&self, id: &str) -> Option<&StlAsset> {
        self.stl_assets.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::StlMode;

    fn asset(path: &str) -> StlAsset {
        StlAsset {
            path: path.to_string(),
            scale: 1.0,
        }
    }

    #[test]
    fn resolves_registered_assets() {
        let mut store = AssetStore::new();
        store.register_stl(asset("model.stl"));

        let scene = SceneDescriptor::Stl {
            asset: "model.stl".to_string(),
            mode: StlMode::Mesh,
        };

        assert!(store.resolve_scene_assets(&scene).is_ok());
    }

    #[test]
    fn errors_on_missing_assets() {
        let store = AssetStore::new();
        let scene = SceneDescriptor::Stl {
            asset: "missing.stl".to_string(),
            mode: StlMode::Wireframe,
        };

        let err = store.resolve_scene_assets(&scene).unwrap_err();
        assert!(format!("{err}").contains("missing.stl"));
    }
}
