use crate::{
    analysis::AnalysisFrame, mapping::ParameterUpdate, scene::SceneInstance, MusicVizError, Result,
};

/// Rendering backend abstraction. The current placeholder implementation keeps
/// track of registered scenes and the most recent parameter updates.
#[derive(Debug, Default)]
pub struct RenderGraph {
    scenes: Vec<SceneInstance>,
    last_updates: Vec<ParameterUpdate>,
    current_analysis: Option<AnalysisFrame>,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self {
            scenes: Vec::new(),
            last_updates: Vec::new(),
            current_analysis: None,
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

    pub fn ingest_analysis_frame(&mut self, frame: AnalysisFrame) {
        for scene in &mut self.scenes {
            scene.update(&frame);
        }
        self.current_analysis = Some(frame);
    }

    pub fn draw(&self) -> Result<()> {
        if self.scenes.is_empty() {
            return Ok(());
        }

        let all_updated = self
            .scenes
            .iter()
            .all(|scene| scene.last_analysis().is_some());

        if all_updated {
            Ok(())
        } else {
            Err(MusicVizError::msg(
                "render graph cannot draw scenes without analysis data",
            ))
        }
    }

    pub fn scenes(&self) -> &[SceneInstance] {
        &self.scenes
    }

    pub fn last_updates(&self) -> &[ParameterUpdate] {
        &self.last_updates
    }

    pub fn last_analysis(&self) -> Option<&AnalysisFrame> {
        self.current_analysis.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::SceneDescriptor;

    fn analysis_frame(time: f32) -> AnalysisFrame {
        AnalysisFrame {
            time,
            rms: 0.5,
            spectral_centroid: 0.4,
            beat_confidence: 0.6,
        }
    }

    #[test]
    fn draw_requires_analysis_for_registered_scenes() {
        let mut graph = RenderGraph::new();
        graph.register_scene(SceneInstance::new(SceneDescriptor::Tunnel { speed: 1.0 }));

        assert!(graph.draw().is_err());

        graph.ingest_analysis_frame(analysis_frame(1.0));
        graph.apply_updates(vec![ParameterUpdate {
            target: "tunnel.energy".to_string(),
            value: 0.75,
        }]);

        assert!(graph.draw().is_ok());
        assert_eq!(graph.last_updates().len(), 1);
        assert!(graph.last_analysis().is_some());
    }

    #[test]
    fn draw_succeeds_without_scenes() {
        let graph = RenderGraph::new();
        assert!(graph.draw().is_ok());
    }
}
