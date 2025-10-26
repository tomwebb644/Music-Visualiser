use crate::ParameterUpdate;

/// Placeholder for the eventual render graph.
#[derive(Debug, Default)]
pub struct RenderGraph {
    applied_updates: Vec<ParameterUpdate>,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply_updates(&mut self, updates: &[ParameterUpdate]) {
        self.applied_updates.clear();
        self.applied_updates.extend_from_slice(updates);
    }

    pub fn last_updates(&self) -> &[ParameterUpdate] {
        &self.applied_updates
    }
}
