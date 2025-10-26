use serde::{Deserialize, Serialize};

use crate::{AnalysisFrame, SceneInstance};

#[derive(Debug, Default, Clone)]
pub struct PlaybackClock {
    pub time_seconds: f32,
}

impl PlaybackClock {
    pub fn reset(&mut self) {
        self.time_seconds = 0.0;
    }

    pub fn advance(&mut self, delta: f32) {
        self.time_seconds = (self.time_seconds + delta).max(0.0);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledEvent {
    pub time_seconds: f32,
    pub label: String,
}

impl ScheduledEvent {
    pub fn new(time_seconds: f32, label: impl Into<String>) -> Self {
        Self {
            time_seconds,
            label: label.into(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Scheduler {
    events: Vec<ScheduledEvent>,
    next_event: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_events(&mut self, events: Vec<ScheduledEvent>) {
        self.events = events;
        self.events
            .sort_by(|a, b| a.time_seconds.partial_cmp(&b.time_seconds).unwrap());
        self.next_event = 0;
    }

    pub fn tick(
        &mut self,
        clock: &PlaybackClock,
        scene: &mut SceneInstance,
        frame: &AnalysisFrame,
    ) {
        if let Some(event) = self.events.get(self.next_event) {
            if clock.time_seconds >= event.time_seconds {
                scene.beat_emphasis = frame.beat_confidence;
                self.next_event += 1;
            }
        }
    }
}
