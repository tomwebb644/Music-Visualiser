use crate::{mapping::ParameterUpdate, scene::SceneInstance, Result};

/// Rendering backend abstraction. The current placeholder implementation keeps
/// track of registered scenes and the most recent parameter updates.
#[derive(Debug, Default)]
pub struct RenderGraph {
    scenes: Vec<SceneInstance>,
    last_updates: Vec<ParameterUpdate>,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self {
            scenes: Vec::new(),
            last_updates: Vec::new(),
        }
    }

    pub fn register_scene(&mut self, scene: SceneInstance) {
        self.scenes.push(scene);
    }

    pub fn apply_updates(&mut self, updates: Vec<ParameterUpdate>) {
        self.last_updates = updates;
        for scene in &mut self.scenes {
            scene.apply_updates(&self.last_updates);
        }
    }

    pub fn draw(&self) -> Result<()> {
        // TODO: dispatch work to wgpu once available. For now we just confirm the
        // call path is wired end-to-end.
        Ok(())
    }
}
