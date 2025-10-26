use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{MusicVizError, Result};

/// Unique identifier for assets stored in the [`AssetStore`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetHandle(pub String);

/// Minimal representation of an STL mesh.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StlAsset {
    pub name: String,
    pub path: Option<String>,
}

impl StlAsset {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: None,
        }
    }
}

/// Simple asset store that keeps placeholder metadata for STL files.
#[derive(Debug, Default)]
pub struct AssetStore {
    stl_assets: HashMap<AssetHandle, StlAsset>,
}

impl AssetStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_stl(&mut self, handle: AssetHandle, asset: StlAsset) -> Result<()> {
        if self.stl_assets.contains_key(&handle) {
            return Err(MusicVizError::InvalidInput(
                "asset handle already registered",
            ));
        }
        self.stl_assets.insert(handle, asset);
        Ok(())
    }

    pub fn get_stl(&self, handle: &AssetHandle) -> Option<&StlAsset> {
        self.stl_assets.get(handle)
    }

    pub fn list_stl(&self) -> impl Iterator<Item = (&AssetHandle, &StlAsset)> {
        self.stl_assets.iter()
    }
}
